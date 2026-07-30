//! `do_one_cmd` — parsing and running one command from the line,
//! and what it does with what is left over.

#[allow(unused_imports)]
use super::*;

// Ex-command callbacks and line getters are identified by address, as the C
// code did; the helpers spell the address comparison out so the intent
// survives the `unpredictable_function_pointer_comparisons` lint.
pub(crate) fn ex_func_is(
    func: Option<unsafe extern "C" fn(*mut exarg_T)>,
    f: unsafe extern "C" fn(*mut exarg_T),
) -> bool {
    func.is_some_and(|g| ::core::ptr::fn_addr_eq(g, f))
}

pub unsafe extern "C" fn is_cmd_ni(mut cmdidx: cmdidx_T) -> bool {
    return !((cmdidx as c_int) < 0 as c_int)
        && (ex_func_is((*cmdnames.ptr())[cmdidx as usize].cmd_func, ex_ni)
            || ex_func_is((*cmdnames.ptr())[cmdidx as usize].cmd_func, ex_script_ni));
}

pub(crate) unsafe extern "C" fn shift_cmd_args(mut eap: *mut exarg_T) {
    '_c2rust_label: {
        if !(*eap).args.is_null() && (*eap).argc > 0 as size_t {
        } else {
            __assert_fail(
                b"eap->args != NULL && eap->argc > 0\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                1708 as c_uint,
                b"void shift_cmd_args(exarg_T *)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut oldargs: *mut *mut c_char = (*eap).args;
    let mut oldarglens: *mut size_t = (*eap).arglens;
    (*eap).argc = (*eap).argc.wrapping_sub(1);
    (*eap).args = (if (*eap).argc > 0 as size_t {
        xcalloc((*eap).argc, ::core::mem::size_of::<*mut c_char>())
    } else {
        NULL_1
    }) as *mut *mut c_char;
    (*eap).arglens = (if (*eap).argc > 0 as size_t {
        xcalloc((*eap).argc, ::core::mem::size_of::<size_t>())
    } else {
        NULL_1
    }) as *mut size_t;
    let mut i: size_t = 0 as size_t;
    while i < (*eap).argc {
        *(*eap).args.offset(i as isize) = *oldargs.offset(i.wrapping_add(1 as size_t) as isize);
        *(*eap).arglens.offset(i as isize) =
            *oldarglens.offset(i.wrapping_add(1 as size_t) as isize);
        i = i.wrapping_add(1);
    }
    (*eap).arg = if (*eap).argc > 0 as size_t {
        *(*eap).args.offset(0 as c_int as isize)
    } else {
        (*oldargs.offset(0 as c_int as isize))
            .offset(*oldarglens.offset(0 as c_int as isize) as isize)
    };
    xfree(oldargs as *mut c_void);
    xfree(oldarglens as *mut c_void);
}

pub(crate) unsafe extern "C" fn skip_cmd(mut eap: *const exarg_T) -> bool {
    if (*eap).skip != 0 {
        match (*eap).cmdidx as c_int {
            525 | 147 | 167 | 145 | 187 | 141 | 140 | 143 | 488 | 54 | 159 | 146 | 168 | 3
            | 550 | 26 | 31 | 38 | 53 | 97 | 99 | 115 | 126 | 127 | 131 | 132 | 135 | 136 | 138
            | 139 | 149 | 151 | 157 | 176 | 181 | 183 | 188 | 189 | 198 | 199 | 209 | 207 | 206
            | 208 | 230 | 231 | 255 | 256 | 264 | 278 | 288 | 298 | 302 | 323 | 334 | 346 | 349
            | 351 | 355 | 353 | 371 | 374 | 378 | 407 | 410 | 415 | 382 | 444 | 453 | 468 | 473
            | 555 | 484 | 498 | 499 | 506 | 507 | 527 => {}
            _ => return true_0 != 0,
        }
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn do_one_cmd(
    mut cmdlinep: *mut *mut c_char,
    mut flags: c_int,
    mut cstack: *mut cstack_T,
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
) -> *mut c_char {
    let mut after_modifier: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut ni: c_int = 0;
    let mut retv: c_int = 0;
    let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
    let save_reg_executing: c_int = reg_executing.get();
    let save_pending_end_reg_executing: bool = pending_end_reg_executing.get();
    let mut ea: exarg_T = exarg {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
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
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    (*ex_nesting_level.ptr()) += 1;
    if quitmore.get() != 0
        && !getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        )
        && !getline_equal(
            fgetline,
            cookie,
            Some(getnextac as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
        )
    {
        (*quitmore.ptr()) -= 1;
    }
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    '_doend: {
        if !(*(*cmdlinep).offset(0 as c_int as isize) as c_int == '#' as c_int
            && *(*cmdlinep).offset(1 as c_int as isize) as c_int == '!' as c_int)
        {
            ea.cmd = *cmdlinep;
            ea.cmdlinep = cmdlinep;
            ea.ea_getline = fgetline;
            ea.cookie = cookie;
            ea.cstack = cstack;
            if parse_command_modifiers(&raw mut ea, &raw mut errormsg, cmdmod.ptr(), false_0 != 0)
                != FAIL
            {
                apply_cmdmod(cmdmod.ptr());
                after_modifier = ea.cmd;
                ea.skip = (did_emsg.get() != 0
                    || got_int.get() as c_int != 0
                    || did_throw.get() as c_int != 0
                    || (*cstack).cs_idx >= 0 as c_int
                        && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ACTIVE as c_int == 0)
                    as c_int;
                p = find_excmd_after_range(&raw mut ea);
                profile_cmd(&raw mut ea, cstack, fgetline, cookie);
                if !exiting.get() {
                    dbg_check_breakpoint(&raw mut ea);
                }
                if ea.skip == 0 && got_int.get() as c_int != 0 {
                    ea.skip = true_0;
                    do_intthrow(cstack);
                }
                set_cmd_addr_type(&raw mut ea, p);
                if parse_cmd_address(&raw mut ea, &raw mut errormsg, false_0 != 0) != FAIL {
                    ea.cmd = skip_colon_white(ea.cmd, true_0 != 0);
                    if *ea.cmd as c_int == NUL || *ea.cmd as c_int == '"' as c_int || {
                        ea.nextcmd = check_nextcmd(ea.cmd);
                        !ea.nextcmd.is_null()
                    } {
                        if ea.skip == 0 {
                            '_c2rust_label: {
                                if errormsg.is_null() {
                                } else {
                                    __assert_fail(
                                        b"errormsg == NULL\0".as_ptr()
                                            as *const c_char,
                                        b"src/nvim/ex_docmd.rs\0"
                                            .as_ptr() as *const c_char,
                                        2156 as c_uint,
                                        b"char *do_one_cmd(char **, int, cstack_T *, LineGetter, void *)\0"
                                            .as_ptr() as *const c_char,
                                    );
                                }
                            };
                            errormsg = ex_range_without_command(&raw mut ea);
                        }
                    } else {
                        if !p.is_null()
                            && ea.cmdidx as c_int == CMD_SIZE as c_int
                            && ea.skip == 0
                            && (*ea.cmd as c_uint >= 'A' as c_uint
                                && *ea.cmd as c_uint <= 'Z' as c_uint)
                            && has_event(EVENT_CMDUNDEFINED) as c_int != 0
                        {
                            let mut cmdname: *mut c_char = ea.cmd;
                            while *cmdname as c_uint >= 'A' as c_uint
                                && *cmdname as c_uint <= 'Z' as c_uint
                                || *cmdname as c_uint >= 'a' as c_uint
                                    && *cmdname as c_uint <= 'z' as c_uint
                                || ascii_isdigit(*cmdname as c_int) as c_int != 0
                            {
                                cmdname = cmdname.offset(1);
                            }
                            cmdname = xmemdupz(
                                ea.cmd as *const c_void,
                                cmdname.offset_from(ea.cmd) as size_t,
                            ) as *mut c_char;
                            let mut ret: c_int = apply_autocmds(
                                EVENT_CMDUNDEFINED,
                                cmdname,
                                cmdname,
                                true_0 != 0,
                                ::core::ptr::null_mut::<buf_T>(),
                            ) as c_int;
                            xfree(cmdname as *mut c_void);
                            p = if ret != 0 && !aborting() {
                                find_ex_command(&raw mut ea, ::core::ptr::null_mut::<c_int>())
                            } else {
                                ea.cmd
                            };
                        }
                        if p.is_null() {
                            if ea.skip == 0 {
                                errormsg = gettext(
                                    (e_ambiguous_use_of_user_defined_command.ptr() as *const _)
                                        as *const c_char,
                                );
                            }
                        } else if ea.cmdidx as c_int == CMD_SIZE as c_int {
                            if ea.skip == 0 {
                                xstrlcpy(
                                    IObuff.ptr() as *mut c_char,
                                    gettext(
                                        (e_not_an_editor_command.ptr() as *const _)
                                            as *const c_char,
                                    ),
                                    IOSIZE as size_t,
                                );
                                let mut cmdname_0: *mut c_char = if !after_modifier.is_null() {
                                    after_modifier
                                } else {
                                    *cmdlinep
                                };
                                if flags & DOCMD_VERBOSE as c_int == 0 {
                                    append_command(cmdname_0);
                                }
                                errormsg = IObuff.ptr() as *mut c_char;
                                did_emsg_syntax.set(true_0 != 0);
                                verify_command(cmdname_0);
                            }
                        } else {
                            ni = is_cmd_ni(ea.cmdidx) as c_int;
                            ea.forceit = parse_bang(&raw mut ea, &raw mut p) as c_int;
                            if !((ea.cmdidx as c_int) < 0 as c_int) {
                                ea.argt = (*cmdnames.ptr())[ea.cmdidx as c_int as usize].cmd_argt;
                            }
                            if ea.skip == 0 {
                                if sandbox.get() != 0 as c_int
                                    && ea.argt & EX_SBOXOK as uint32_t == 0
                                {
                                    errormsg = gettext(&raw const e_sandbox as *const c_char);
                                    break '_doend;
                                } else if (*curbuf.get()).b_p_ma == 0
                                    && ea.argt & EX_MODIFY as uint32_t != 0
                                    && !(!(*curbuf.get()).terminal.is_null()
                                        && (ea.cmdidx as c_int == CMD_put as c_int
                                            || ea.cmdidx as c_int == CMD_iput as c_int))
                                {
                                    errormsg = gettext(&raw const e_modifiable as *const c_char);
                                    break '_doend;
                                } else {
                                    if !((ea.cmdidx as c_int) < 0 as c_int) {
                                        if cmdwin_type.get() != 0 as c_int
                                            && ea.argt & EX_CMDWIN as uint32_t == 0
                                        {
                                            errormsg =
                                                gettext(&raw const e_cmdwin as *const c_char);
                                            break '_doend;
                                        } else if text_locked() as c_int != 0
                                            && ea.argt & EX_LOCK_OK as uint32_t == 0
                                        {
                                            errormsg = gettext(get_text_locked_msg());
                                            break '_doend;
                                        }
                                    }
                                    if ea.argt & EX_CMDWIN as uint32_t == 0
                                        && ea.cmdidx as c_int != CMD_checktime as c_int
                                        && ea.cmdidx as c_int != CMD_edit as c_int
                                        && ea.cmdidx as c_int != CMD_file as c_int
                                        && !((ea.cmdidx as c_int) < 0 as c_int)
                                        && curbuf_locked() as c_int != 0
                                    {
                                        break '_doend;
                                    } else if ni == 0
                                        && ea.argt & EX_RANGE as uint32_t == 0
                                        && ea.addr_count > 0 as c_int
                                    {
                                        errormsg = gettext(&raw const e_norange as *const c_char);
                                        break '_doend;
                                    }
                                }
                            }
                            if ni == 0 && ea.argt & EX_BANG as uint32_t == 0 && ea.forceit != 0 {
                                errormsg = gettext(&raw const e_nobang as *const c_char);
                            } else {
                                if ea.skip == 0 && ni == 0 && ea.argt & EX_RANGE as uint32_t != 0 {
                                    if global_busy.get() == 0 && ea.line1 > ea.line2 {
                                        if msg_silent.get() == 0 as c_int {
                                            if flags & DOCMD_VERBOSE as c_int != 0
                                                || exmode_active.get() as c_int != 0
                                            {
                                                errormsg = gettext(
                                                    b"E493: Backwards range given\0".as_ptr()
                                                        as *const c_char,
                                                );
                                                break '_doend;
                                            } else if ask_yesno(gettext(
                                                b"Backwards range given, OK to swap\0".as_ptr()
                                                    as *const c_char,
                                            )) != 'y' as c_int
                                            {
                                                break '_doend;
                                            }
                                        }
                                        let mut lnum: linenr_T = ea.line1;
                                        ea.line1 = ea.line2;
                                        ea.line2 = lnum;
                                    }
                                    errormsg = invalid_range(&raw mut ea);
                                    if !errormsg.is_null() {
                                        break '_doend;
                                    }
                                }
                                if ea.addr_type as c_uint == ADDR_OTHER as c_int as c_uint
                                    && ea.addr_count == 0 as c_int
                                {
                                    ea.line2 = 1 as c_int as linenr_T;
                                }
                                correct_range(&raw mut ea);
                                if (ea.argt & EX_WHOLEFOLD as uint32_t != 0
                                    || ea.addr_count >= 2 as c_int)
                                    && global_busy.get() == 0
                                    && ea.addr_type as c_uint == ADDR_LINES as c_int as c_uint
                                {
                                    hasFolding(
                                        curwin.get(),
                                        ea.line1,
                                        &raw mut ea.line1,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                    );
                                    hasFolding(
                                        curwin.get(),
                                        ea.line2,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                        &raw mut ea.line2,
                                    );
                                }
                                p = replace_makeprg(&raw mut ea, p, cmdlinep);
                                if !p.is_null() {
                                    ea.arg = if ea.cmdidx as c_int == CMD_bang as c_int {
                                        p
                                    } else {
                                        skipwhite(p)
                                    };
                                    if !(ea.cmdidx as c_int == CMD_file as c_int
                                        && *ea.arg as c_int != NUL
                                        && curbuf_locked() as c_int != 0)
                                    {
                                        's_449: {
                                            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                                                loop {
                                                    if !(*ea.arg.offset(0 as c_int as isize)
                                                        as c_int
                                                        == '+' as c_int
                                                        && *ea.arg.offset(1 as c_int as isize)
                                                            as c_int
                                                            == '+' as c_int)
                                                    {
                                                        break 's_449;
                                                    }
                                                    if !(getargopt(&raw mut ea) == FAIL && ni == 0)
                                                    {
                                                        continue;
                                                    }
                                                    errormsg = gettext(
                                                        &raw const e_invarg as *const c_char,
                                                    );
                                                    break '_doend;
                                                }
                                            }
                                        }
                                        if ea.cmdidx as c_int == CMD_write as c_int
                                            || ea.cmdidx as c_int == CMD_update as c_int
                                        {
                                            if *ea.arg as c_int == '>' as c_int {
                                                ea.arg = ea.arg.offset(1);
                                                if *ea.arg as c_int != '>' as c_int {
                                                    errormsg =
                                                        gettext(b"E494: Use w or w>>\0".as_ptr()
                                                            as *const c_char);
                                                    break '_doend;
                                                } else {
                                                    ea.arg = skipwhite(
                                                        ea.arg.offset(1 as c_int as isize),
                                                    );
                                                    ea.append = true_0;
                                                }
                                            } else if *ea.arg as c_int == '!' as c_int
                                                && ea.cmdidx as c_int == CMD_write as c_int
                                            {
                                                ea.arg = ea.arg.offset(1);
                                                ea.usefilter = true_0;
                                            }
                                        } else if ea.cmdidx as c_int == CMD_read as c_int {
                                            if ea.forceit != 0 {
                                                ea.usefilter = true_0;
                                                ea.forceit = false_0;
                                            } else if *ea.arg as c_int == '!' as c_int {
                                                ea.arg = ea.arg.offset(1);
                                                ea.usefilter = true_0;
                                            }
                                        } else if ea.cmdidx as c_int == CMD_lshift as c_int
                                            || ea.cmdidx as c_int == CMD_rshift as c_int
                                        {
                                            ea.amount = 1 as c_int;
                                            while *ea.arg as c_int == *ea.cmd as c_int {
                                                ea.arg = ea.arg.offset(1);
                                                ea.amount += 1;
                                            }
                                            ea.arg = skipwhite(ea.arg);
                                        }
                                        if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0
                                        {
                                            ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
                                        }
                                        if ea.argt & EX_TRLBAR as uint32_t != 0 && ea.usefilter == 0
                                        {
                                            separate_nextcmd(&raw mut ea);
                                        } else if ea.cmdidx as c_int == CMD_bang as c_int
                                            || ea.cmdidx as c_int == CMD_terminal as c_int
                                            || ea.cmdidx as c_int == CMD_global as c_int
                                            || ea.cmdidx as c_int == CMD_vglobal as c_int
                                            || ea.usefilter != 0
                                        {
                                            let mut s: *mut c_char = ea.arg;
                                            while *s != 0 {
                                                if *s as c_int == '\\' as c_int
                                                    && *s.offset(1 as c_int as isize) as c_int
                                                        == '\n' as c_int
                                                {
                                                    memmove(
                                                        s as *mut c_void,
                                                        s.offset(1 as c_int as isize)
                                                            as *const c_void,
                                                        strlen(s.offset(1 as c_int as isize))
                                                            .wrapping_add(1 as size_t),
                                                    );
                                                } else if *s as c_int == '\n' as c_int {
                                                    ea.nextcmd = s.offset(1 as c_int as isize);
                                                    *s = NUL as c_char;
                                                    break;
                                                }
                                                s = s.offset(1);
                                            }
                                        }
                                        if ea.argt & EX_DFLALL as uint32_t != 0
                                            && ea.addr_count == 0 as c_int
                                        {
                                            set_cmd_dflall_range(&raw mut ea);
                                        }
                                        parse_register(&raw mut ea);
                                        if parse_count(&raw mut ea, &raw mut errormsg, true_0 != 0)
                                            != FAIL
                                        {
                                            if ea.argt & EX_FLAGS as uint32_t != 0 {
                                                get_flags(&raw mut ea);
                                            }
                                            if ni == 0
                                                && ea.argt & EX_EXTRA as uint32_t == 0
                                                && *ea.arg as c_int != NUL
                                                && *ea.arg as c_int != '"' as c_int
                                                && (*ea.arg as c_int != '|' as c_int
                                                    || ea.argt & EX_TRLBAR as uint32_t
                                                        == 0 as uint32_t)
                                            {
                                                errormsg = ex_errmsg(
                                                    &raw const e_trailing_arg as *const c_char,
                                                    ea.arg,
                                                );
                                            } else if ni == 0
                                                && ea.argt & EX_NEEDARG as uint32_t != 0
                                                && *ea.arg as c_int == NUL
                                            {
                                                errormsg =
                                                    gettext(&raw const e_argreq as *const c_char);
                                            } else if !skip_cmd(&raw mut ea) {
                                                retv = 0 as c_int;
                                                if execute_cmd0(
                                                    &raw mut retv,
                                                    &raw mut ea,
                                                    &raw mut errormsg,
                                                    false_0 != 0,
                                                ) != FAIL
                                                {
                                                    if need_rethrow.get() {
                                                        do_throw(cstack);
                                                    } else if check_cstack.get() {
                                                        if source_finished(fgetline, cookie) {
                                                            do_finish(&raw mut ea, true_0 != 0);
                                                        } else if getline_equal(
                                                            fgetline,
                                                            cookie,
                                                            Some(
                                                                get_func_line
                                                                    as unsafe extern "C" fn(
                                                                        c_int,
                                                                        *mut c_void,
                                                                        c_int,
                                                                        bool,
                                                                    ) -> *mut c_char,
                                                            ),
                                                        ) as c_int != 0 && current_func_returned() != 0
                                                        {
                                                            do_return(&raw mut ea, true_0 != 0, false_0 != 0, NULL_1);
                                                        }
                                                    }
                                                    check_cstack.set(false_0 != 0);
                                                    need_rethrow.set(check_cstack.get());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if (*curwin.get()).w_cursor.lnum == 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
    }
    if !errormsg.is_null() && *errormsg as c_int != NUL && did_emsg.get() == 0 {
        if flags & DOCMD_VERBOSE as c_int != 0 {
            if errormsg != IObuff.ptr() as *mut c_char as *const c_char {
                xstrlcpy(IObuff.ptr() as *mut c_char, errormsg, IOSIZE as size_t);
                errormsg = IObuff.ptr() as *mut c_char;
            }
            append_command(*ea.cmdlinep);
        }
        emsg(errormsg);
    }
    do_errthrow(
        cstack,
        if ea.cmdidx as c_int != CMD_SIZE as c_int && !((ea.cmdidx as c_int) < 0 as c_int) {
            (*cmdnames.ptr())[ea.cmdidx as c_int as usize].cmd_name
        } else {
            ::core::ptr::null_mut::<c_char>()
        },
    );
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    reg_executing.set(save_reg_executing);
    pending_end_reg_executing.set(save_pending_end_reg_executing);
    if !ea.nextcmd.is_null() && *ea.nextcmd as c_int == NUL {
        ea.nextcmd = ::core::ptr::null_mut::<c_char>();
    }
    (*ex_nesting_level.ptr()) -= 1;
    xfree(ea.cmdline_tofree as *mut c_void);
    return ea.nextcmd;
}

pub(crate) unsafe extern "C" fn ex_range_without_command(mut eap: *mut exarg_T) -> *mut c_char {
    let mut errormsg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *(*eap).cmd as c_int == '|' as c_int
        || exmode_active.get() as c_int != 0
            && (*eap).cmd != (exmode_plus.ptr() as *mut c_char).offset(1 as c_int as isize)
    {
        (*eap).cmdidx = CMD_print;
        (*eap).argt = (EX_RANGE | EX_COUNT | EX_TRLBAR) as uint32_t;
        errormsg = invalid_range(eap);
        if errormsg.is_null() {
            correct_range(eap);
            ex_print(eap);
        }
    } else if (*eap).addr_count != 0 as c_int {
        (*eap).line2 = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        if (*eap).line2 < 0 as linenr_T {
            errormsg = gettext(&raw const e_invrange as *const c_char);
        } else {
            if (*eap).line2 == 0 as linenr_T {
                (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
            } else {
                (*curwin.get()).w_cursor.lnum = (*eap).line2;
            }
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
    }
    return errormsg;
}

pub(crate) unsafe extern "C" fn append_command(mut cmd: *const c_char) {
    let mut len: size_t = strlen(IObuff.ptr() as *mut c_char);
    let mut s: *const c_char = cmd;
    let mut d: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if len > (IOSIZE - 100 as c_int) as size_t {
        d = (IObuff.ptr() as *mut c_char)
            .offset(IOSIZE as isize)
            .offset(-(100 as c_int as isize));
        d = d.offset(-(utf_head_off(IObuff.ptr() as *mut c_char, d) as isize));
        strcpy(d, b"...\0".as_ptr() as *const c_char as *mut c_char);
    }
    xstrlcat(
        IObuff.ptr() as *mut c_char,
        b": \0".as_ptr() as *const c_char,
        IOSIZE as size_t,
    );
    d = (IObuff.ptr() as *mut c_char).offset(strlen(IObuff.ptr() as *mut c_char) as isize);
    while *s as c_int != NUL
        && (d.offset_from(IObuff.ptr() as *mut c_char) + 5 as isize) < IOSIZE as isize
    {
        if *s.offset(0 as c_int as isize) as uint8_t as c_int == 0xc2 as c_int
            && *s.offset(1 as c_int as isize) as uint8_t as c_int == 0xa0 as c_int
        {
            s = s.offset(2 as c_int as isize);
            strcpy(d, b"<a0>\0".as_ptr() as *const c_char as *mut c_char);
            d = d.offset(4 as c_int as isize);
        } else {
            if d.offset_from(IObuff.ptr() as *mut c_char) + utfc_ptr2len(s) as isize + 1 as isize
                >= IOSIZE as isize
            {
                break;
            }
            mb_copy_char(&raw mut s, &raw mut d);
        }
    }
    *d = NUL as c_char;
}

pub unsafe extern "C" fn ex_ni(mut eap: *mut exarg_T) {
    if (*eap).skip == 0 {
        (*eap).errmsg = gettext(
            b"E319: The command is not available in this version\0".as_ptr() as *const c_char,
        );
    }
}

pub(crate) unsafe extern "C" fn ex_script_ni(mut eap: *mut exarg_T) {
    if (*eap).skip == 0 {
        ex_ni(eap);
    } else {
        let mut len: size_t = 0;
        xfree(script_get(eap, &raw mut len) as *mut c_void);
    };
}
