//! `do_cmdline` — the loop that runs a sequence of Ex command
//! lines, and the one-line shortcut into it.
//!
//! This is the re-entrant heart of the editor: a sourced file, `:execute`, a
//! `:global` body, an autocommand and a mapping all arrive here, nested inside
//! one another, with the conditional stack, the exception state and the loop
//! line store threaded through.  Ordering is load-bearing throughout.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_cmdline_cmd(mut cmd: *const c_char) -> c_int {
    return do_cmdline(
        cmd as *mut c_char,
        None,
        NULL_1,
        DOCMD_VERBOSE as c_int | DOCMD_NOWAIT as c_int | DOCMD_KEYTYPED as c_int,
    );
}

pub unsafe extern "C" fn do_cmdline(
    mut cmdline: *mut c_char,
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
    mut flags: c_int,
) -> c_int {
    let mut next_cmdline: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut cmdline_copy: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut used_getline: bool = false_0 != 0;
    static recursive: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    let mut msg_didout_before_start: bool = false_0 != 0;
    let mut count: c_int = 0 as c_int;
    let mut did_inc: bool = false_0 != 0;
    let mut did_block: bool = false_0 != 0;
    let mut retval: c_int = OK;
    let mut cstack: cstack_T = cstack_T {
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
    let mut lines_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    let mut current_line: c_int = 0 as c_int;
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut breakpoint: *mut linenr_T = ::core::ptr::null_mut::<linenr_T>();
    let mut dbg_tick: *mut c_int = ::core::ptr::null_mut::<c_int>();
    let mut debug_saved: dbg_stuff = dbg_stuff {
        trylevel: 0,
        force_abort: 0,
        caught_stack: ::core::ptr::null_mut::<except_T>(),
        vv_exception: ::core::ptr::null_mut::<c_char>(),
        vv_throwpoint: ::core::ptr::null_mut::<c_char>(),
        did_emsg: 0,
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        check_cstack: 0,
        current_exception: ::core::ptr::null_mut::<except_T>(),
    };
    let mut private_msg_list: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    let mut cmd_getline: Option<
        unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
    > = None;
    let mut cmd_cookie: *mut c_void = ::core::ptr::null_mut::<c_void>();
    let mut cmd_loop_cookie: loop_cookie = loop_cookie {
        lines_gap: ::core::ptr::null_mut::<garray_T>(),
        current_line: 0,
        repeating: 0,
        lc_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
    };
    let mut saved_msg_list: *mut *mut msglist_T = msg_list.get();
    msg_list.set(&raw mut private_msg_list);
    private_msg_list = ::core::ptr::null_mut::<msglist_T>();
    if do_cmdline_start() == FAIL {
        emsg(gettext(&raw const e_command_too_recursive as *const c_char));
        do_errthrow(NULL_1 as *mut cstack_T, ::core::ptr::null_mut::<c_char>());
        msg_list.set(saved_msg_list);
        return FAIL;
    }
    ga_init(
        &raw mut lines_ga,
        ::core::mem::size_of::<wcmd_T>() as c_int,
        10 as c_int,
    );
    let mut real_cookie: *mut c_void = getline_cookie(fgetline, cookie);
    let mut getline_is_func: bool = getline_equal(
        fgetline,
        cookie,
        Some(get_func_line as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    );
    if getline_is_func as c_int != 0 && ex_nesting_level.get() == func_level(real_cookie) {
        (*ex_nesting_level.ptr()) += 1;
    }
    if getline_is_func {
        fname = func_name(real_cookie);
        breakpoint = func_breakpoint(real_cookie);
        dbg_tick = func_dbg_tick(real_cookie);
    } else if getline_equal(
        fgetline,
        cookie,
        Some(getsourceline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    ) {
        fname = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
        .es_name;
        breakpoint = source_breakpoint(real_cookie);
        dbg_tick = source_dbg_tick(real_cookie);
    }
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
    }
    if flags & DOCMD_EXCRESET as c_int != 0 {
        save_dbg_stuff(&raw mut debug_saved);
    } else {
        memset(
            &raw mut debug_saved as *mut c_void,
            0 as c_int,
            ::core::mem::size_of::<dbg_stuff>(),
        );
    }
    let mut initial_trylevel: c_int = trylevel.get();
    did_throw.set(false_0 != 0);
    did_emsg.set(false_0);
    if flags & DOCMD_KEYTYPED as c_int == 0
        && !getline_equal(
            fgetline,
            cookie,
            Some(getexline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
        )
    {
        KeyTyped.set(false_0 != 0);
    }
    next_cmdline = cmdline;
    loop {
        getline_is_func = getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        );
        if next_cmdline.is_null()
            && !force_abort.get()
            && cstack.cs_idx < 0 as c_int
            && !(getline_is_func as c_int != 0 && func_has_abort(real_cookie) != 0)
        {
            did_emsg.set(false_0);
        }
        if cstack.cs_looplevel > 0 as c_int && current_line < lines_ga.ga_len {
            let mut ptr_: *mut *mut c_void = &raw mut cmdline_copy as *mut *mut c_void;
            xfree(*ptr_);
            *ptr_ = NULL_1;
            let _ = *ptr_;
            if getline_is_func {
                if do_profiling.get() == PROF_YES {
                    func_line_end(real_cookie);
                }
                if func_has_ended(real_cookie) != 0 {
                    retval = FAIL;
                    break;
                }
            } else if do_profiling.get() == PROF_YES
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getsourceline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
            {
                script_line_end();
            }
            if source_finished(fgetline, cookie) {
                retval = FAIL;
                break;
            } else {
                if !breakpoint.is_null() && !dbg_tick.is_null() && *dbg_tick != debug_tick.get() {
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(
                            fgetline,
                            cookie,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        c_int,
                                        *mut c_void,
                                        c_int,
                                        bool,
                                    )
                                        -> *mut c_char,
                            ),
                        ),
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                    );
                    *dbg_tick = debug_tick.get();
                }
                next_cmdline =
                    (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).line;
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_lnum = (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).lnum;
                if !breakpoint.is_null()
                    && *breakpoint != 0 as linenr_T
                    && *breakpoint
                        <= (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum
                {
                    dbg_breakpoint(
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                    );
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(
                            fgetline,
                            cookie,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        c_int,
                                        *mut c_void,
                                        c_int,
                                        bool,
                                    )
                                        -> *mut c_char,
                            ),
                        ),
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                    );
                    *dbg_tick = debug_tick.get();
                }
                if do_profiling.get() == PROF_YES {
                    if getline_is_func {
                        func_line_start(real_cookie);
                    } else if getline_equal(
                        fgetline,
                        cookie,
                        Some(
                            getsourceline
                                as unsafe extern "C" fn(
                                    c_int,
                                    *mut c_void,
                                    c_int,
                                    bool,
                                )
                                    -> *mut c_char,
                        ),
                    ) {
                        script_line_start();
                    }
                }
            }
        }
        if next_cmdline.is_null() {
            let mut indent: c_int = if cstack.cs_idx < 0 as c_int {
                0 as c_int
            } else {
                (cstack.cs_idx + 1 as c_int) * 2 as c_int
            };
            if count == 1 as c_int
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getexline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
            {
                if ui_has(kUICmdline) {
                    ui_ext_cmdline_block_append(0 as size_t, last_cmdline.get());
                    did_block = true_0 != 0;
                }
                msg_didout.set(true_0 != 0);
            }
            if fgetline.is_none() || {
                next_cmdline = fgetline.expect("non-null function pointer")(
                    ':' as c_int,
                    cookie,
                    indent,
                    true_0 != 0,
                );
                next_cmdline.is_null()
            } {
                if KeyTyped.get() as c_int != 0 && flags & DOCMD_REPEAT as c_int == 0 {
                    need_wait_return.set(false_0 != 0);
                }
                retval = FAIL;
                break;
            } else {
                used_getline = true_0 != 0;
                if ui_has(kUICmdline) as c_int != 0
                    && count > 0 as c_int
                    && getline_equal(
                        fgetline,
                        cookie,
                        Some(
                            getexline
                                as unsafe extern "C" fn(
                                    c_int,
                                    *mut c_void,
                                    c_int,
                                    bool,
                                )
                                    -> *mut c_char,
                        ),
                    ) as c_int
                        != 0
                {
                    ui_ext_cmdline_block_append(indent as size_t, next_cmdline);
                }
                if flags & DOCMD_KEEPLINE as c_int != 0 {
                    xfree(repeat_cmdline.get() as *mut c_void);
                    if count == 0 as c_int {
                        repeat_cmdline.set(xstrdup(next_cmdline));
                    } else {
                        repeat_cmdline.set(::core::ptr::null_mut::<c_char>());
                    }
                }
            }
        } else if cmdline_copy.is_null() {
            next_cmdline = xstrdup(next_cmdline);
        }
        cmdline_copy = next_cmdline;
        let mut current_line_before: c_int = 0 as c_int;
        if cstack.cs_looplevel > 0 as c_int || has_loop_cmd(next_cmdline) as c_int != 0 {
            cmd_getline = Some(
                get_loop_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            )
                as Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>;
            cmd_cookie = &raw mut cmd_loop_cookie as *mut c_void;
            cmd_loop_cookie.lines_gap = &raw mut lines_ga;
            cmd_loop_cookie.current_line = current_line;
            cmd_loop_cookie.lc_getline = fgetline;
            cmd_loop_cookie.cookie = cookie;
            cmd_loop_cookie.repeating = (current_line < lines_ga.ga_len) as c_int;
            if current_line == lines_ga.ga_len {
                store_loop_line(&raw mut lines_ga, next_cmdline);
            }
            current_line_before = current_line;
        } else {
            cmd_getline = fgetline
                as Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>;
            cmd_cookie = cookie;
        }
        did_endif.set(false_0 != 0);
        let c2rust_fresh0 = count;
        count = count + 1;
        if c2rust_fresh0 == 0 as c_int {
            if flags & DOCMD_NOWAIT as c_int == 0 && recursive.get() == 0 {
                msg_didout_before_start = msg_didout.get();
                msg_didany.set(false_0 != 0);
                msg_start();
                msg_scroll.set(true_0);
                (*no_wait_return.ptr()) += 1;
                (*RedrawingDisabled.ptr()) += 1;
                did_inc = true_0 != 0;
            }
        }
        if p_verbose.get() >= 15 as OptInt
            && !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
            .es_name
            .is_null()
            || p_verbose.get() >= 16 as OptInt
        {
            msg_verbose_cmd(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_lnum,
                cmdline_copy,
            );
        }
        (*recursive.ptr()) += 1;
        next_cmdline = do_one_cmd(
            &raw mut cmdline_copy,
            flags,
            &raw mut cstack,
            cmd_getline as LineGetter,
            cmd_cookie,
        );
        (*recursive.ptr()) -= 1;
        if cmd_cookie == &raw mut cmd_loop_cookie as *mut c_void {
            current_line = cmd_loop_cookie.current_line;
        }
        if next_cmdline.is_null() {
            let mut ptr__0: *mut *mut c_void = &raw mut cmdline_copy as *mut *mut c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_1;
            let _ = *ptr__0;
            if getline_equal(
                fgetline,
                cookie,
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
                && !(*new_last_cmdline.ptr()).is_null()
            {
                xfree(last_cmdline.get() as *mut c_void);
                last_cmdline.set(new_last_cmdline.get());
                new_last_cmdline.set(::core::ptr::null_mut::<c_char>());
            }
        } else {
            memmove(
                cmdline_copy as *mut c_void,
                next_cmdline as *const c_void,
                strlen(next_cmdline).wrapping_add(1 as size_t),
            );
            next_cmdline = cmdline_copy;
        }
        if did_emsg.get() != 0
            && !force_abort.get()
            && getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
            && func_has_abort(real_cookie) == 0
        {
            did_emsg.set(false_0);
        }
        if cstack.cs_looplevel > 0 as c_int {
            current_line += 1;
            if cstack.cs_lflags & (CSL_HAD_CONT as c_int | CSL_HAD_ENDLOOP as c_int) != 0 {
                cstack.cs_lflags &= !(CSL_HAD_CONT as c_int | CSL_HAD_ENDLOOP as c_int);
                if did_emsg.get() == 0
                    && !got_int.get()
                    && !did_throw.get()
                    && cstack.cs_idx >= 0 as c_int
                    && cstack.cs_flags[cstack.cs_idx as usize]
                        & (CSF_WHILE as c_int | CSF_FOR as c_int)
                        != 0
                    && cstack.cs_line[cstack.cs_idx as usize] >= 0 as c_int
                    && cstack.cs_flags[cstack.cs_idx as usize] & CSF_ACTIVE as c_int != 0
                {
                    current_line = cstack.cs_line[cstack.cs_idx as usize];
                    cstack.cs_lflags |= CSL_HAD_LOOP as c_int;
                    line_breakcheck();
                    if !breakpoint.is_null() && lines_ga.ga_len > current_line {
                        *breakpoint = dbg_find_breakpoint(
                            getline_equal(
                                fgetline,
                                cookie,
                                Some(
                                    getsourceline
                                        as unsafe extern "C" fn(
                                            c_int,
                                            *mut c_void,
                                            c_int,
                                            bool,
                                        )
                                            -> *mut c_char,
                                ),
                            ),
                            fname,
                            (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).lnum
                                - 1 as linenr_T,
                        );
                        *dbg_tick = debug_tick.get();
                    }
                } else if cstack.cs_idx >= 0 as c_int {
                    rewind_conditionals(
                        &raw mut cstack,
                        cstack.cs_idx - 1 as c_int,
                        CSF_WHILE as c_int | CSF_FOR as c_int,
                        &raw mut cstack.cs_looplevel,
                    );
                }
            } else if cstack.cs_lflags & CSL_HAD_LOOP as c_int != 0 {
                cstack.cs_lflags &= !(CSL_HAD_LOOP as c_int);
                cstack.cs_line[cstack.cs_idx as usize] = current_line_before;
            }
        }
        if cstack.cs_looplevel == 0 as c_int {
            if !(lines_ga.ga_len <= 0 as c_int) {
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_lnum = (*(lines_ga.ga_data as *mut wcmd_T)
                    .offset((lines_ga.ga_len - 1 as c_int) as isize))
                .lnum;
                let mut _gap: *mut garray_T = &raw mut lines_ga;
                if !(*_gap).ga_data.is_null() {
                    let mut i: c_int = 0 as c_int;
                    while i < (*_gap).ga_len {
                        let mut _item: *mut wcmd_T =
                            ((*_gap).ga_data as *mut wcmd_T).offset(i as isize);
                        xfree((*_item).line as *mut c_void);
                        i += 1;
                    }
                }
                ga_clear(_gap);
            }
            current_line = 0 as c_int;
        }
        if cstack.cs_lflags & CSL_HAD_FINA as c_int != 0 {
            cstack.cs_lflags &= !(CSL_HAD_FINA as c_int);
            report_make_pending(
                cstack.cs_pending[cstack.cs_idx as usize] as c_int
                    & (CSTP_ERROR as c_int | CSTP_INTERRUPT as c_int | CSTP_THROW as c_int),
                (if did_throw.get() as c_int != 0 {
                    current_exception.get()
                } else {
                    ::core::ptr::null_mut::<except_T>()
                }) as *mut c_void,
            );
            did_throw.set(false_0 != 0);
            got_int.set(did_throw.get());
            did_emsg.set(got_int.get() as c_int);
            cstack.cs_flags[cstack.cs_idx as usize] |= CSF_ACTIVE as c_int | CSF_FINALLY as c_int;
        }
        trylevel.set(initial_trylevel + cstack.cs_trylevel);
        if trylevel.get() == 0 as c_int && did_emsg.get() == 0 && !got_int.get() && !did_throw.get()
        {
            force_abort.set(false_0 != 0);
        }
        do_intthrow(&raw mut cstack);
        if !(!((got_int.get() as c_int != 0
            || did_emsg.get() != 0 && force_abort.get() as c_int != 0
            || did_throw.get() as c_int != 0)
            && cstack.cs_trylevel == 0 as c_int)
            && !(did_emsg.get() != 0
                && (cstack.cs_trylevel == 0 as c_int || did_emsg_syntax.get() as c_int != 0)
                && used_getline as c_int != 0
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getexline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0)
            && (!next_cmdline.is_null()
                || cstack.cs_idx >= 0 as c_int
                || flags & DOCMD_REPEAT as c_int != 0))
        {
            break;
        }
    }
    xfree(cmdline_copy as *mut c_void);
    did_emsg_syntax.set(false_0 != 0);
    let mut _gap_0: *mut garray_T = &raw mut lines_ga;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: c_int = 0 as c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut wcmd_T = ((*_gap_0).ga_data as *mut wcmd_T).offset(i_0 as isize);
            xfree((*_item_0).line as *mut c_void);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    if cstack.cs_idx >= 0 as c_int {
        if !got_int.get()
            && !did_throw.get()
            && !aborting()
            && (getline_equal(
                fgetline,
                cookie,
                Some(
                    getsourceline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
                && !source_finished(fgetline, cookie)
                || getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        get_func_line
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
                    && func_has_ended(real_cookie) == 0)
        {
            if cstack.cs_flags[cstack.cs_idx as usize] & CSF_TRY as c_int != 0 {
                emsg(gettext(&raw const e_endtry as *const c_char));
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_WHILE as c_int != 0 {
                emsg(gettext(&raw const e_endwhile as *const c_char));
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_FOR as c_int != 0 {
                emsg(gettext(&raw const e_endfor as *const c_char));
            } else {
                emsg(gettext(&raw const e_endif as *const c_char));
            }
        }
        loop {
            let mut idx: c_int = cleanup_conditionals(&raw mut cstack, 0 as c_int, true_0);
            if idx >= 0 as c_int {
                idx -= 1;
            }
            rewind_conditionals(
                &raw mut cstack,
                idx,
                CSF_WHILE as c_int | CSF_FOR as c_int,
                &raw mut cstack.cs_looplevel,
            );
            if cstack.cs_idx < 0 as c_int {
                break;
            }
        }
        trylevel.set(initial_trylevel);
    }
    do_errthrow(
        &raw mut cstack,
        (if getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) as c_int
            != 0
        {
            b"endfunction\0".as_ptr() as *const c_char
        } else {
            ::core::ptr::null::<c_char>()
        }) as *mut c_char,
    );
    if trylevel.get() == 0 as c_int {
        if did_throw.get() {
            handle_did_throw();
        } else if got_int.get() as c_int != 0
            || did_emsg.get() != 0 && force_abort.get() as c_int != 0
        {
            suppress_errthrow.set(true_0 != 0);
        }
    }
    if did_throw.get() {
        need_rethrow.set(true_0 != 0);
    }
    if getline_equal(
        fgetline,
        cookie,
        Some(getsourceline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    ) as c_int
        != 0
        && ex_nesting_level.get() > source_level(real_cookie)
        || getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) as c_int
            != 0
            && ex_nesting_level.get() > func_level(real_cookie) + 1 as c_int
    {
        if !did_throw.get() {
            check_cstack.set(true_0 != 0);
        }
    } else {
        if getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) {
            (*ex_nesting_level.ptr()) -= 1;
        }
        if (getline_equal(
            fgetline,
            cookie,
            Some(
                getsourceline
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) as c_int
            != 0
            || getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0)
            && ex_nesting_level.get() + 1 as c_int <= debug_break_level.get()
        {
            do_debug(
                if getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getsourceline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
                {
                    gettext(b"End of sourced file\0".as_ptr() as *const c_char)
                } else {
                    gettext(b"End of function\0".as_ptr() as *const c_char)
                },
            );
        }
    }
    if flags & DOCMD_EXCRESET as c_int != 0 {
        restore_dbg_stuff(&raw mut debug_saved);
    }
    msg_list.set(saved_msg_list);
    if !cstack.cs_emsg_silent_list.is_null() {
        let mut temp: *mut eslist_T = ::core::ptr::null_mut::<eslist_T>();
        let mut elem: *mut eslist_T = cstack.cs_emsg_silent_list;
        while !elem.is_null() {
            temp = (*elem).next;
            xfree(elem as *mut c_void);
            elem = temp;
        }
    }
    if did_inc {
        (*RedrawingDisabled.ptr()) -= 1;
        (*no_wait_return.ptr()) -= 1;
        msg_scroll.set(false_0);
        if retval == FAIL
            || did_endif.get() as c_int != 0 && KeyTyped.get() as c_int != 0 && did_emsg.get() == 0
        {
            need_wait_return.set(false_0 != 0);
            msg_didany.set(false_0 != 0);
        } else if need_wait_return.get() {
            msg_didout.set(msg_didout.get() as c_int | msg_didout_before_start as c_int != 0);
            wait_return(false_0);
        }
    }
    if did_block {
        ui_ext_cmdline_block_leave();
    }
    did_endif.set(false_0 != 0);
    do_cmdline_end();
    return retval;
}
