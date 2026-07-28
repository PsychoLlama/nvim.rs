//! `:echo`, `:echohl`, `:execute` and where a variable was last set.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ex_echo(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut atstart: bool = true_0 != 0;
    let mut need_clear: bool = true_0 != 0;
    let did_emsg_before: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    while *arg as c_int != NUL
        && *arg as c_int != '|' as c_int
        && *arg as c_int != '\n' as c_int
        && !got_int.get()
    {
        need_clr_eos.set(true_0 != 0);
        let mut p: *mut c_char = arg;
        if eval1(&raw mut arg, &raw mut rettv, &raw mut evalarg) == FAIL {
            if !aborting()
                && did_emsg.get() == did_emsg_before
                && called_emsg.get() == called_emsg_before
            {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), p);
            }
            need_clr_eos.set(false_0 != 0);
            break;
        } else {
            need_clr_eos.set(false_0 != 0);
            if (*eap).skip == 0 {
                if atstart {
                    atstart = false_0 != 0;
                    msg_ext_set_append((*eap).cmdidx as c_int == CMD_echon as c_int);
                    msg_ext_set_kind(b"echo\0".as_ptr() as *const c_char);
                    if (*eap).cmdidx as c_int == CMD_echo as c_int {
                        if !msg_didout.get() {
                            msg_sb_eol();
                        }
                        msg_start();
                    }
                } else if (*eap).cmdidx as c_int == CMD_echo as c_int {
                    msg_puts_hl(
                        b" \0".as_ptr() as *const c_char,
                        echo_hl_id.get(),
                        false_0 != 0,
                    );
                }
                let mut tofree: *mut c_char =
                    encode_tv2echo(&raw mut rettv, ::core::ptr::null_mut::<size_t>());
                msg_multiline(
                    cstr_as_string(tofree),
                    echo_hl_id.get(),
                    true_0 != 0,
                    false_0 != 0,
                    &raw mut need_clear,
                );
                xfree(tofree as *mut c_void);
            }
            tv_clear(&raw mut rettv);
            arg = skipwhite(arg);
        }
    }
    (*eap).nextcmd = check_nextcmd(arg);
    clear_evalarg(&raw mut evalarg, eap);
    msg_ext_set_append(false_0 != 0);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    } else {
        if ui_has(kUIMessages) as c_int != 0
            && (*(*eap).arg as c_int == NUL
                || *(*eap).arg as c_int == '|' as c_int
                || *(*eap).arg as c_int == '\n' as c_int)
        {
            msg_puts_len(
                b"\0".as_ptr() as *const c_char,
                0 as ptrdiff_t,
                0 as c_int,
                false_0 != 0,
            );
        } else if need_clear {
            msg_clr_eos();
        }
        if (*eap).cmdidx as c_int == CMD_echo as c_int {
            msg_end();
        }
    };
}

pub unsafe extern "C" fn ex_echohl(mut eap: *mut exarg_T) {
    echo_hl_id.set(syn_name2id((*eap).arg));
}

pub unsafe extern "C" fn get_echo_hl_id() -> c_int {
    return echo_hl_id.get();
}

pub unsafe extern "C" fn ex_execute(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut ret: c_int = OK;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    ga_init(&raw mut ga, 1 as c_int, 80 as c_int);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    while *arg as c_int != NUL && *arg as c_int != '|' as c_int && *arg as c_int != '\n' as c_int {
        ret = eval1_emsg(&raw mut arg, &raw mut rettv, eap);
        if ret == FAIL {
            break;
        }
        if (*eap).skip == 0 {
            let argstr: *const c_char = if (*eap).cmdidx as c_int == CMD_execute as c_int {
                tv_get_string(&raw mut rettv)
            } else {
                (if rettv.v_type as c_uint == VAR_STRING as c_int as c_uint {
                    encode_tv2echo(&raw mut rettv, ::core::ptr::null_mut::<size_t>())
                } else {
                    encode_tv2string(&raw mut rettv, ::core::ptr::null_mut::<size_t>())
                }) as *const c_char
            };
            let len: size_t = strlen(argstr);
            ga_grow(&raw mut ga, len as c_int + 2 as c_int);
            if !(ga.ga_len <= 0 as c_int) {
                let c2rust_fresh21 = ga.ga_len;
                ga.ga_len = ga.ga_len + 1;
                *(ga.ga_data as *mut c_char).offset(c2rust_fresh21 as isize) = ' ' as c_char;
            }
            memcpy(
                (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                argstr as *const c_void,
                len.wrapping_add(1 as size_t),
            );
            if (*eap).cmdidx as c_int != CMD_execute as c_int {
                xfree(argstr as *mut c_void);
            }
            ga.ga_len += len as c_int;
        }
        tv_clear(&raw mut rettv);
        arg = skipwhite(arg);
    }
    if ret != FAIL && !ga.ga_data.is_null() {
        if (*eap).cmdidx as c_int == CMD_echomsg as c_int {
            msg_ext_set_kind(b"echomsg\0".as_ptr() as *const c_char);
            msg(ga.ga_data as *const c_char, echo_hl_id.get());
        } else if (*eap).cmdidx as c_int == CMD_echoerr as c_int {
            let mut save_did_emsg: c_int = did_emsg.get();
            emsg_multiline(
                ga.ga_data as *const c_char,
                b"echoerr\0".as_ptr() as *const c_char,
                HLF_E as c_int,
                true_0 != 0,
            );
            if !force_abort.get() {
                did_emsg.set(save_did_emsg);
            }
        } else if (*eap).cmdidx as c_int == CMD_execute as c_int {
            do_cmdline(
                ga.ga_data as *mut c_char,
                (*eap).ea_getline,
                (*eap).cookie,
                DOCMD_NOWAIT as c_int | DOCMD_VERBOSE as c_int,
            );
        }
    }
    ga_clear(&raw mut ga);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    }
    (*eap).nextcmd = check_nextcmd(arg);
}

pub unsafe extern "C" fn var_flavour(mut varname: *mut c_char) -> var_flavour_T {
    let mut p: *mut c_char = varname;
    if *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint {
        loop {
            p = p.offset(1);
            if *p == 0 {
                break;
            }
            if *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint {
                return VAR_FLAVOUR_SESSION;
            }
        }
        return VAR_FLAVOUR_SHADA;
    }
    return VAR_FLAVOUR_DEFAULT;
}

pub unsafe extern "C" fn var_set_global(name: *const c_char, mut vartv: typval_T) {
    let mut funccall_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccall_entry);
    set_var(name, strlen(name), &raw mut vartv, false_0 != 0);
    restore_funccal();
}

pub unsafe extern "C" fn last_set_msg(mut script_ctx: sctx_T) {
    if script_ctx.sc_sid == 0 as c_int {
        return;
    }
    let mut should_free: bool = false;
    let mut p: *mut c_char = get_scriptname(script_ctx, &raw mut should_free);
    msg_ext_skip_verbose.set(true_0 != 0);
    verbose_enter();
    msg_puts(gettext(b"\n\tLast set from \0".as_ptr() as *const c_char));
    msg_puts(p);
    if script_ctx.sc_lnum > 0 as linenr_T {
        msg_puts(gettext(&raw const line_msg as *const c_char));
        msg_outnum(script_ctx.sc_lnum as c_int);
    } else if script_is_lua(script_ctx.sc_sid) {
        msg_puts(gettext(
            b" (run Nvim with -V1 for more details)\0".as_ptr() as *const c_char
        ));
    }
    if should_free {
        xfree(p as *mut c_void);
    }
    verbose_leave();
}
