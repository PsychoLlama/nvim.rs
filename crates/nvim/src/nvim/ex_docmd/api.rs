//! The API's view of a command line: `nvim_parse_cmd` and `nvim_cmd`
//! build an `exarg_T` without running anything, or run one that was built from a
//! Dict.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn parse_cmdline(
    mut cmdline: *mut *mut c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut errormsg: *mut *const c_char,
) -> bool {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut after_modifier: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut retval: bool = false_0 != 0;
    let save_ex_pressedreturn: bool = ex_pressedreturn.get();
    let save_cursor: pos_T = (*curwin.get()).w_cursor;
    save_last_search_pattern();
    memset(
        cmdinfo as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<CmdParseInfo>(),
    );
    *eap = exarg {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: *cmdline,
        cmdlinep: cmdline,
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 1 as linenr_T,
        line2: 1 as linenr_T,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: NULL_1,
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut orig_cmd: *mut c_char = (*eap).cmd;
    let mut result: c_int =
        parse_command_modifiers(eap, errormsg, &raw mut (*cmdinfo).cmdmod, false_0 != 0);
    after_modifier = (*eap).cmd;
    if !(result == FAIL && after_modifier == orig_cmd) {
        p = find_excmd_after_range(eap);
        if p.is_null() {
            *errormsg = gettext(
                (e_ambiguous_use_of_user_defined_command.ptr() as *const _) as *const c_char,
            );
        } else {
            set_cmd_addr_type(eap, p);
            if parse_cmd_address(eap, errormsg, true_0 != 0) != FAIL {
                (*eap).cmd = skip_colon_white((*eap).cmd, true_0 != 0);
                if *(*eap).cmd as c_int != '"' as c_int {
                    if !(*(*eap).cmd as c_int == NUL
                        && (*eap).addr_count == 0 as c_int
                        && after_modifier == *cmdline)
                    {
                        if *(*eap).cmd as c_int == NUL
                            && (*eap).cmdidx as c_int == CMD_SIZE as c_int
                        {
                            (*eap).arg = (*eap).cmd;
                            if (*eap).addr_count > 0 as c_int {
                                (*eap).argt = EX_RANGE as uint32_t;
                            } else {
                                (*eap).argt = 0 as uint32_t;
                                (*eap).addr_type = ADDR_NONE;
                            }
                            retval = true_0 != 0;
                        } else if (*eap).cmdidx as c_int == CMD_SIZE as c_int {
                            xstrlcpy(
                                IObuff.ptr() as *mut c_char,
                                gettext(
                                    (e_not_an_editor_command.ptr() as *const _) as *const c_char,
                                ),
                                IOSIZE as size_t,
                            );
                            let mut cmdname: *mut c_char = if !after_modifier.is_null() {
                                after_modifier
                            } else {
                                *cmdline
                            };
                            append_command(cmdname);
                            *errormsg = IObuff.ptr() as *mut c_char;
                        } else {
                            (*eap).forceit = parse_bang(eap, &raw mut p) as c_int;
                            if !(((*eap).cmdidx as c_int) < 0 as c_int) {
                                (*eap).argt =
                                    (*cmdnames.ptr())[(*eap).cmdidx as c_int as usize].cmd_argt;
                            }
                            (*eap).arg = if (*eap).cmdidx as c_int == CMD_bang as c_int {
                                p
                            } else {
                                skipwhite(p)
                            };
                            if (*eap).cmdidx as c_int == CMD_read as c_int && (*eap).forceit != 0 {
                                (*eap).forceit = false_0;
                            }
                            if (*eap).argt & EX_TRLBAR as uint32_t != 0 {
                                separate_nextcmd(eap);
                            } else if cmd_has_expr_args((*eap).cmdidx) {
                                let mut arg: *mut c_char = (*eap).arg;
                                while *arg as c_int != NUL
                                    && *arg as c_int != '|' as c_int
                                    && *arg as c_int != '\n' as c_int
                                {
                                    let mut start: *mut c_char = arg;
                                    (*emsg_skip.ptr()) += 1;
                                    skip_expr(&raw mut arg, ::core::ptr::null_mut::<evalarg_T>());
                                    (*emsg_skip.ptr()) -= 1;
                                    if arg == start {
                                        arg = arg.offset(1);
                                    }
                                }
                                if *arg as c_int == '|' as c_int || *arg as c_int == '\n' as c_int {
                                    (*eap).nextcmd = check_nextcmd(arg);
                                    *arg = NUL as c_char;
                                }
                            }
                            if (*eap).argt & EX_BANG as uint32_t == 0 && (*eap).forceit != 0 {
                                *errormsg = gettext(&raw const e_nobang as *const c_char);
                            } else if (*eap).argt & EX_RANGE as uint32_t == 0
                                && (*eap).addr_count > 0 as c_int
                            {
                                *errormsg = gettext(&raw const e_norange as *const c_char);
                            } else {
                                if (*eap).argt & EX_DFLALL as uint32_t != 0
                                    && (*eap).addr_count == 0 as c_int
                                {
                                    set_cmd_dflall_range(eap);
                                }
                                parse_register(eap);
                                if parse_count(eap, errormsg, false_0 != 0) != FAIL {
                                    if !(*eap).nextcmd.is_null() {
                                        (*eap).nextcmd =
                                            skip_colon_white((*eap).nextcmd, true_0 != 0);
                                    }
                                    if (*eap).argt & EX_XFILE as uint32_t != 0 {
                                        (*cmdinfo).magic.file = true_0 != 0;
                                    }
                                    if (*eap).argt & EX_TRLBAR as uint32_t != 0 {
                                        (*cmdinfo).magic.bar = true_0 != 0;
                                    }
                                    retval = true_0 != 0;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !retval {
        undo_cmdmod(&raw mut (*cmdinfo).cmdmod);
    }
    ex_pressedreturn.set(save_ex_pressedreturn);
    (*curwin.get()).w_cursor = save_cursor;
    restore_last_search_pattern();
    return retval;
}

pub(crate) unsafe extern "C" fn execute_cmd0(
    mut retv: *mut c_int,
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut preview: bool,
) -> c_int {
    if (*eap).argt & EX_XFILE as uint32_t != 0 {
        if expand_filename(eap, (*eap).cmdlinep, errormsg) == FAIL {
            return FAIL;
        }
    }
    if (*eap).argt & EX_BUFNAME as uint32_t != 0
        && *(*eap).arg as c_int != NUL
        && (*eap).addr_count == 0 as c_int
        && !(((*eap).cmdidx as c_int) < 0 as c_int)
    {
        if (*eap).args.is_null() {
            let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
            if (*eap).cmdidx as c_int == CMD_bdelete as c_int
                || (*eap).cmdidx as c_int == CMD_bwipeout as c_int
                || (*eap).cmdidx as c_int == CMD_bunload as c_int
            {
                p = skiptowhite_esc((*eap).arg);
            } else {
                p = (*eap).arg.offset(strlen((*eap).arg) as isize);
                while p > (*eap).arg
                    && ascii_iswhite(*p.offset(-1 as c_int as isize) as c_int) as c_int != 0
                {
                    p = p.offset(-1);
                }
            }
            (*eap).line2 = buflist_findpat(
                (*eap).arg,
                p,
                (*eap).argt & EX_BUFUNL as uint32_t != 0 as uint32_t,
                false_0 != 0,
                false_0 != 0,
            ) as linenr_T;
            (*eap).addr_count = 1 as c_int;
            (*eap).arg = skipwhite(p);
        } else {
            (*eap).line2 = buflist_findpat(
                *(*eap).args.offset(0 as c_int as isize),
                (*(*eap).args.offset(0 as c_int as isize))
                    .offset(*(*eap).arglens.offset(0 as c_int as isize) as isize),
                (*eap).argt & EX_BUFUNL as uint32_t != 0 as uint32_t,
                false_0 != 0,
                false_0 != 0,
            ) as linenr_T;
            (*eap).addr_count = 1 as c_int;
            shift_cmd_args(eap);
        }
        if (*eap).line2 < 0 as linenr_T {
            return FAIL;
        }
    }
    if (*eap).cmdidx as c_int == CMD_try as c_int && (*cmdmod.ptr()).cmod_did_esilent > 0 as c_int {
        (*emsg_silent.ptr()) -= (*cmdmod.ptr()).cmod_did_esilent;
        emsg_silent.set(if emsg_silent.get() > 0 as c_int {
            emsg_silent.get()
        } else {
            0 as c_int
        });
        (*cmdmod.ptr()).cmod_did_esilent = 0 as c_int;
    }
    if ((*eap).cmdidx as c_int) < 0 as c_int {
        *retv = do_ucmd(eap, preview);
    } else {
        (*eap).errmsg = ::core::ptr::null_mut::<c_char>();
        if preview {
            *retv = (*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_preview_func
                .expect("non-null function pointer")(
                eap,
                cmdpreview_get_ns(),
                cmdpreview_get_bufnr(),
            );
        } else {
            (*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_func
                .expect("non-null function pointer")(eap);
        }
        if !(*eap).errmsg.is_null() {
            *errormsg = (*eap).errmsg;
        }
    }
    return OK;
}

pub unsafe extern "C" fn execute_cmd(
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut preview: bool,
) -> c_int {
    let mut cstack: cstack_T = cstack_T {
        cs_flags: [0; 50],
        cs_pending: [0; 50],
        cs_pend: C2Rust_Unnamed_34 {
            csp_rv: [::core::ptr::null_mut::<c_void>(); 50],
        },
        cs_forinfo: [::core::ptr::null_mut::<c_void>(); 50],
        cs_line: [0; 50],
        cs_idx: 0,
        cs_looplevel: 0,
        cs_trylevel: 0,
        cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
        cs_lflags: 0,
    };
    let mut retv: c_int = 0 as c_int;
    if do_cmdline_start() == FAIL {
        emsg(gettext(&raw const e_command_too_recursive as *const c_char));
        return retv;
    }
    let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    cmdmod.set((*cmdinfo).cmdmod);
    apply_cmdmod(cmdmod.ptr());
    '_end: {
        if (*curbuf.get()).b_p_ma == 0
            && (*eap).argt & EX_MODIFY as uint32_t != 0
            && !(!(*curbuf.get()).terminal.is_null()
                && ((*eap).cmdidx as c_int == CMD_put as c_int
                    || (*eap).cmdidx as c_int == CMD_iput as c_int))
        {
            errormsg = gettext(&raw const e_modifiable as *const c_char);
        } else {
            if !(((*eap).cmdidx as c_int) < 0 as c_int) {
                if cmdwin_type.get() != 0 as c_int && (*eap).argt & EX_CMDWIN as uint32_t == 0 {
                    errormsg = gettext(&raw const e_cmdwin as *const c_char);
                    break '_end;
                } else if text_locked() as c_int != 0 && (*eap).argt & EX_LOCK_OK as uint32_t == 0 {
                    errormsg = gettext(get_text_locked_msg());
                    break '_end;
                }
            }
            if !((*eap).argt & EX_CMDWIN as uint32_t == 0
                && (*eap).cmdidx as c_int != CMD_checktime as c_int
                && (*eap).cmdidx as c_int != CMD_edit as c_int
                && !((*eap).cmdidx as c_int == CMD_file as c_int && *(*eap).arg as c_int == NUL)
                && !(((*eap).cmdidx as c_int) < 0 as c_int)
                && curbuf_locked() as c_int != 0)
            {
                correct_range(eap);
                if (*eap).cmdidx as c_int == CMD_SIZE as c_int && (*eap).addr_count > 0 as c_int {
                    errormsg = ex_range_without_command(eap);
                } else {
                    if ((*eap).argt & EX_WHOLEFOLD as uint32_t != 0
                        || (*eap).addr_count >= 2 as c_int)
                        && global_busy.get() == 0
                        && (*eap).addr_type as c_uint == ADDR_LINES as c_int as c_uint
                    {
                        hasFolding(
                            curwin.get(),
                            (*eap).line1,
                            &raw mut (*eap).line1,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        hasFolding(
                            curwin.get(),
                            (*eap).line2,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut (*eap).line2,
                        );
                    }
                    if parse_count(eap, &raw mut errormsg, true_0 != 0) != FAIL {
                        cstack = cstack_T {
                            cs_flags: [0; 50],
                            cs_pending: [0; 50],
                            cs_pend: C2Rust_Unnamed_34 {
                                csp_rv: [::core::ptr::null_mut::<c_void>(); 50],
                            },
                            cs_forinfo: [::core::ptr::null_mut::<c_void>(); 50],
                            cs_line: [0; 50],
                            cs_idx: -1 as c_int,
                            cs_looplevel: 0,
                            cs_trylevel: 0,
                            cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
                            cs_lflags: 0,
                        };
                        (*eap).cstack = &raw mut cstack;
                        execute_cmd0(&raw mut retv, eap, &raw mut errormsg, preview);
                    }
                }
            }
        }
    }
    if !errormsg.is_null() && *errormsg as c_int != NUL {
        emsg(errormsg);
    }
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    do_cmdline_end();
    return retv;
}

pub(crate) unsafe extern "C" fn profile_cmd(
    mut eap: *const exarg_T,
    mut cstack: *mut cstack_T,
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
) {
    if do_profiling.get() == PROF_YES
        && ((*eap).skip == 0
            || (*cstack).cs_idx == 0 as c_int
            || (*cstack).cs_idx > 0 as c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as c_int) as usize]
                    & CSF_ACTIVE as c_int
                    != 0)
    {
        let mut skip: bool =
            did_emsg.get() != 0 || got_int.get() as c_int != 0 || did_throw.get() as c_int != 0;
        if (*eap).cmdidx as c_int == CMD_catch as c_int {
            skip = !skip
                && !((*cstack).cs_idx >= 0 as c_int
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_THROWN as c_int != 0
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_CAUGHT as c_int == 0);
        } else if (*eap).cmdidx as c_int == CMD_else as c_int
            || (*eap).cmdidx as c_int == CMD_elseif as c_int
        {
            skip = skip as c_int != 0
                || !((*cstack).cs_idx >= 0 as c_int
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize]
                        & (CSF_ACTIVE as c_int | CSF_TRUE as c_int)
                        == 0);
        } else if (*eap).cmdidx as c_int == CMD_finally as c_int {
            skip = false_0 != 0;
        } else if (*eap).cmdidx as c_int != CMD_endif as c_int
            && (*eap).cmdidx as c_int != CMD_endfor as c_int
            && (*eap).cmdidx as c_int != CMD_endtry as c_int
            && (*eap).cmdidx as c_int != CMD_endwhile as c_int
        {
            skip = (*eap).skip != 0;
        }
        if !skip {
            if getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) {
                func_line_exec(getline_cookie(fgetline, cookie));
            } else if getline_equal(
                fgetline,
                cookie,
                Some(
                    getsourceline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) {
                script_line_exec();
            }
        }
    }
}
