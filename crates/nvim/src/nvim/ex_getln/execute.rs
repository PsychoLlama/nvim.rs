//! One key, dispatched: the key loop's per-key state machine.
//!
//! [`command_line_execute`] is the `state_execute` callback — it reads a key,
//! gives the wildmenu and `<C-\>` their chance at it, and hands the rest to
//! [`super::handlekey::command_line_handle_key`].  [`command_line_changed`]
//! is the other half: what runs after a key that edited the line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn command_line_handle_ctrl_bsl(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        if (*s).c != Ctrl_N
            && (*s).c != Ctrl_G
            && ((*s).c != 'e' as ::core::ffi::c_int
                || (*ccline.ptr()).cmdfirstc == '=' as ::core::ffi::c_int
                    && KeyTyped.get() as ::core::ffi::c_int != 0
                || cmdline_star.get() > 0 as ::core::ffi::c_int)
        {
            vungetc((*s).c);
            return PROCESS_NEXT_KEY;
        }
        if (*s).c == 'e' as ::core::ffi::c_int {
            if (*ccline.ptr()).cmdpos == (*ccline.ptr()).cmdlen {
                new_cmdpos.set(99999 as ::core::ffi::c_int);
            } else {
                new_cmdpos.set((*ccline.ptr()).cmdpos);
            }
            (*s).c = get_expr_register();
            if (*s).c == '=' as ::core::ffi::c_int {
                (*textlock.ptr()) += 1;
                let mut p: *mut ::core::ffi::c_char = get_expr_line();
                (*textlock.ptr()) -= 1;
                if !p.is_null() {
                    let mut len: ::core::ffi::c_int = strlen(p) as ::core::ffi::c_int;
                    realloc_cmdbuff(len + 1 as ::core::ffi::c_int);
                    (*ccline.ptr()).cmdlen = len;
                    strcpy((*ccline.ptr()).cmdbuff, p);
                    xfree(p as *mut ::core::ffi::c_void);
                    (*ccline.ptr()).cmdpos = if (*ccline.ptr()).cmdlen < new_cmdpos.get() {
                        (*ccline.ptr()).cmdlen
                    } else {
                        new_cmdpos.get()
                    };
                    KeyTyped.set(false_0 != 0);
                    redrawcmd();
                    return CMDLINE_CHANGED;
                }
            }
            beep_flush();
            got_int.set(false_0 != 0);
            did_emsg.set(false_0);
            emsg_on_display.set(false_0 != 0);
            redrawcmd();
            return CMDLINE_NOT_CHANGED;
        }
        (*s).gotesc = true_0 != 0;
        return GOTO_NORMAL_MODE;
    }
}

pub(crate) unsafe extern "C" fn command_line_end_wildmenu(
    mut s: *mut CommandLineState,
    mut key_is_wc: bool,
    mut c: ::core::ffi::c_int,
) {
    unsafe {
        if cmdline_pum_active() {
            if c != -1 as ::core::ffi::c_int {
                (*s).skip_pum_redraw = (*s).skip_pum_redraw as ::core::ffi::c_int != 0
                    && !key_is_wc
                    && !ascii_iswhite(c)
                    && (vim_isprintc(c) as ::core::ffi::c_int != 0
                        || c == K_BS
                        || c == Ctrl_H
                        || c == K_DEL
                        || c == K_KDEL
                        || c == Ctrl_W
                        || c == Ctrl_U);
            }
            cmdline_pum_remove(
                c != -1 as ::core::ffi::c_int && (*s).skip_pum_redraw as ::core::ffi::c_int != 0,
            );
        }
        if (*s).xpc.xp_numfiles != -1 as ::core::ffi::c_int {
            ExpandOne(
                &raw mut (*s).xpc,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                WILD_FREE,
            );
        }
        (*s).did_wild_list = false_0 != 0;
        if p_wmnu.get() == 0 || c != K_UP && c != K_DOWN {
            (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
        (*s).wim_index = 0 as ::core::ffi::c_int;
        wildmenu_cleanup(ccline.ptr());
    }
}

pub(crate) unsafe extern "C" fn command_line_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if key == K_IGNORE || key == K_NOP {
            return -1 as ::core::ffi::c_int;
        }
        let mut display_tick_saved: disptick_T = (*curwin.get()).w_display_tick;
        let mut s: *mut CommandLineState = state as *mut CommandLineState;
        (*s).c = key;
        if (*ccline.ptr()).cmdbuff_replaced as ::core::ffi::c_int != 0
            && (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
        {
            command_line_end_wildmenu(s, false_0 != 0, -1 as ::core::ffi::c_int);
        }
        (*ccline.ptr()).cmdbuff_replaced = false_0 != 0;
        if (*s).c == K_WILD && (*s).did_hist_navigate as ::core::ffi::c_int != 0 {
            (*s).did_hist_navigate = false_0 != 0;
            return 1 as ::core::ffi::c_int;
        }
        if (*s).c == K_EVENT || (*s).c == K_COMMAND || (*s).c == K_LUA {
            if (*s).c == K_EVENT {
                state_handle_k_event();
            } else if (*s).c == K_COMMAND {
                do_cmdline(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    Some(
                        getcmdkeycmd
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                    NULL_0,
                    DOCMD_NOWAIT,
                );
            } else {
                map_execute_lua(false_0 != 0, false_0 != 0);
            }
            if (*s).is_state.winid != (*curwin.get()).handle {
                init_incsearch_state(&raw mut (*s).is_state);
            }
            if (*curwin.get()).w_display_tick > display_tick_saved
                && (*s).is_state.did_incsearch as ::core::ffi::c_int != 0
            {
                may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
            }
            if (*ccline.ptr()).cmdbuff_replaced {
                command_line_changed(s);
            }
            if (*pum_want.ptr()).active {
                if cmdline_pum_active() {
                    nextwild(
                        &raw mut (*s).xpc,
                        WILD_PUM_WANT,
                        0 as ::core::ffi::c_int,
                        (*s).firstc != '@' as ::core::ffi::c_int,
                    );
                    if (*pum_want.ptr()).finish {
                        nextwild(
                            &raw mut (*s).xpc,
                            WILD_APPLY,
                            WILD_NO_BEEP,
                            (*s).firstc != '@' as ::core::ffi::c_int,
                        );
                        command_line_end_wildmenu(s, false_0 != 0, (*s).c);
                    }
                }
                (*pum_want.ptr()).active = false_0 != 0;
            }
            if !cmdline_was_last_drawn.get() {
                redrawcmdline();
            }
            return 1 as ::core::ffi::c_int;
        }
        if KeyTyped.get() {
            (*s).some_key_typed = true_0 != 0;
            if cmdmsg_rl.get() as ::core::ffi::c_int != 0 && KeyStuffed.get() == 0 {
                match (*s).c {
                    K_RIGHT => {
                        (*s).c = K_LEFT;
                    }
                    K_S_RIGHT => {
                        (*s).c = K_S_LEFT;
                    }
                    -22269 => {
                        (*s).c = K_C_LEFT;
                    }
                    K_LEFT => {
                        (*s).c = K_RIGHT;
                    }
                    K_S_LEFT => {
                        (*s).c = K_S_RIGHT;
                    }
                    -22013 => {
                        (*s).c = K_C_RIGHT;
                    }
                    _ => {}
                }
            }
        }
        if (*s).c == Ctrl_C
            && (*s).firstc != '@' as ::core::ffi::c_int
            && (!(*s).break_ctrl_c || exmode_active.get() as ::core::ffi::c_int != 0)
            && global_busy.get() == 0
        {
            got_int.set(false_0 != 0);
        }
        if !(*s).lookfor.is_null()
            && (*s).c != K_S_DOWN
            && (*s).c != K_S_UP
            && (*s).c != K_DOWN
            && (*s).c != K_UP
            && (*s).c != K_PAGEDOWN
            && (*s).c != K_PAGEUP
            && (*s).c != K_KPAGEDOWN
            && (*s).c != K_KPAGEUP
            && (*s).c != K_LEFT
            && (*s).c != K_RIGHT
            && ((*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
                || (*s).c != Ctrl_P && (*s).c != Ctrl_N)
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*s).lookfor as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            (*s).lookforlen = 0 as ::core::ffi::c_int;
        }
        if (*s).c as OptInt != p_wc.get()
            && (*s).c == K_S_TAB
            && (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
        {
            (*s).c = Ctrl_P;
        }
        if p_wmnu.get() != 0 {
            (*s).c =
                wildmenu_translate_key(ccline.ptr(), (*s).c, &raw mut (*s).xpc, (*s).did_wild_list);
        }
        let mut wild_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let key_is_wc: bool = (*s).c as OptInt == p_wc.get()
            && KeyTyped.get() as ::core::ffi::c_int != 0
            || (*s).c as OptInt == p_wcm.get();
        if (cmdline_pum_active() as ::core::ffi::c_int != 0
            || wild_menu_showing.get() != 0
            || (*s).did_wild_list as ::core::ffi::c_int != 0)
            && !key_is_wc
            && (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
        {
            if (*s).c == Ctrl_E || (*s).c == Ctrl_Y {
                wild_type = if (*s).c == Ctrl_E {
                    WILD_CANCEL
                } else {
                    WILD_APPLY
                };
                nextwild(
                    &raw mut (*s).xpc,
                    wild_type,
                    WILD_NO_BEEP,
                    (*s).firstc != '@' as ::core::ffi::c_int,
                );
            }
        }
        if KeyTyped.get() as ::core::ffi::c_int != 0
            && ((*s).c == '\n' as ::core::ffi::c_int
                || (*s).c == '\r' as ::core::ffi::c_int
                || (*s).c == K_KENTER
                || (*s).c == ESC)
            || (*s).c == Ctrl_C
        {
            set_vim_var_char((*s).c);
            trigger_cmd_autocmd((*s).cmdline_type, EVENT_CMDLINELEAVEPRE);
            (*s).event_cmdlineleavepre_triggered = true_0 != 0;
            if ((*s).c == ESC || (*s).c == Ctrl_C)
                && (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    & kOptWimFlagList as ::core::ffi::c_int
                    != 0
            {
                set_no_hlsearch(true_0 != 0);
            }
        }
        let mut end_wildmenu: bool = !key_is_wc
            && (*s).c != Ctrl_Z
            && (*s).c != Ctrl_N
            && (*s).c != Ctrl_P
            && (*s).c != Ctrl_A
            && (*s).c != Ctrl_L;
        end_wildmenu = end_wildmenu as ::core::ffi::c_int != 0
            && (!cmdline_pum_active()
                || (*s).c != K_PAGEDOWN
                    && (*s).c != K_PAGEUP
                    && (*s).c != K_KPAGEDOWN
                    && (*s).c != K_KPAGEUP);
        if end_wildmenu {
            command_line_end_wildmenu(s, key_is_wc, (*s).c);
        }
        if p_wmnu.get() != 0 {
            (*s).c = wildmenu_process_key(ccline.ptr(), (*s).c, &raw mut (*s).xpc);
        }
        if (*s).c == Ctrl_BSL {
            match command_line_handle_ctrl_bsl(s) {
                2 => return command_line_changed(s),
                1 => return command_line_not_changed(s),
                3 => return 0 as ::core::ffi::c_int,
                _ => {
                    (*s).c = Ctrl_BSL;
                }
            }
        }
        if (*s).c == cedit_key.get() || (*s).c == K_CMDWIN {
            if ((*s).c == K_CMDWIN || ex_normal_busy.get() == 0 as ::core::ffi::c_int)
                && got_int.get() as ::core::ffi::c_int == false_0
            {
                (*s).c = open_cmdwin();
                (*s).some_key_typed = true_0 != 0;
            }
        } else {
            (*s).c = do_digraph((*s).c);
        }
        if (*s).c == '\n' as ::core::ffi::c_int
            || (*s).c == '\r' as ::core::ffi::c_int
            || (*s).c == K_KENTER
            || (*s).c == ESC && (!KeyTyped.get() || !vim_strchr(p_cpo.get(), CPO_ESC).is_null())
        {
            if exmode_active.get() as ::core::ffi::c_int != 0
                && (*s).c != ESC
                && (*ccline.ptr()).cmdpos == (*ccline.ptr()).cmdlen
                && (*ccline.ptr()).cmdpos > 0 as ::core::ffi::c_int
                && *(*ccline.ptr())
                    .cmdbuff
                    .offset(((*ccline.ptr()).cmdpos - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                if (*s).c == K_KENTER {
                    (*s).c = '\n' as ::core::ffi::c_int;
                }
            } else {
                (*s).gotesc = false_0 != 0;
                if ccheck_abbr((*s).c + ABBR_OFF) {
                    return command_line_changed(s);
                }
                if !cmd_silent.get() {
                    if !ui_has(kUICmdline) {
                        msg_cursor_goto(msg_row.get(), 0 as ::core::ffi::c_int);
                    }
                    ui_flush();
                }
                return 0 as ::core::ffi::c_int;
            }
        }
        if (*s).c as OptInt == p_wc.get()
            && !(*s).gotesc
            && KeyTyped.get() as ::core::ffi::c_int != 0
            || (*s).c as OptInt == p_wcm.get()
            || (*s).c == K_WILD
            || (*s).c == Ctrl_Z
        {
            if (*s).c == K_WILD {
                (*emsg_silent.ptr()) += 1;
            }
            let mut res: ::core::ffi::c_int = command_line_wildchar_complete(s);
            if (*s).c == K_WILD {
                (*emsg_silent.ptr()) -= 1;
            }
            if res == CMDLINE_CHANGED {
                return command_line_changed(s);
            }
            if (*s).c == K_WILD {
                return command_line_not_changed(s);
            }
        }
        (*s).gotesc = false_0 != 0;
        if (*s).c == K_S_TAB && KeyTyped.get() as ::core::ffi::c_int != 0 {
            if nextwild(
                &raw mut (*s).xpc,
                WILD_EXPAND_KEEP,
                0 as ::core::ffi::c_int,
                (*s).firstc != '@' as ::core::ffi::c_int,
            ) == OK
            {
                if (*s).xpc.xp_numfiles > 1 as ::core::ffi::c_int
                    && (!(*s).did_wild_list
                        && (*wim_flags.ptr())[(*s).wim_index as usize] as ::core::ffi::c_int
                            & kOptWimFlagList as ::core::ffi::c_int
                            != 0
                        || p_wmnu.get() != 0)
                {
                    showmatches(
                        &raw mut (*s).xpc,
                        p_wmnu.get() != 0,
                        (*wim_flags.ptr())[(*s).wim_index as usize] as ::core::ffi::c_int
                            & kOptWimFlagList as ::core::ffi::c_int
                            != 0,
                        (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                            & kOptWimFlagNoselect as ::core::ffi::c_int
                            != 0,
                    );
                }
                nextwild(
                    &raw mut (*s).xpc,
                    WILD_PREV,
                    0 as ::core::ffi::c_int,
                    (*s).firstc != '@' as ::core::ffi::c_int,
                );
                nextwild(
                    &raw mut (*s).xpc,
                    WILD_PREV,
                    0 as ::core::ffi::c_int,
                    (*s).firstc != '@' as ::core::ffi::c_int,
                );
                return command_line_changed(s);
            }
        }
        if (*s).c == NUL || (*s).c == K_ZERO {
            (*s).c = NL;
        }
        (*s).do_abbr = true_0 != 0;
        if wild_type == WILD_CANCEL || wild_type == WILD_APPLY {
            if (*s).is_state.winid != (*curwin.get()).handle {
                init_incsearch_state(&raw mut (*s).is_state);
            }
            if KeyTyped.get() as ::core::ffi::c_int != 0 || vpeekc() == NUL {
                may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
            }
            return command_line_not_changed(s);
        }
        return command_line_handle_key(s);
    }
}

pub(crate) unsafe extern "C" fn may_trigger_cursormovedc(mut s: *mut CommandLineState) {
    unsafe {
        if (*ccline.ptr()).cmdpos != (*s).prev_cmdpos {
            trigger_cmd_autocmd((*s).cmdline_type, EVENT_CURSORMOVEDC);
            (*ccline.ptr()).redraw_state = (if (*ccline.ptr()).redraw_state as ::core::ffi::c_uint
                > kCmdRedrawPos as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*ccline.ptr()).redraw_state as ::core::ffi::c_uint
            } else {
                kCmdRedrawPos as ::core::ffi::c_int as ::core::ffi::c_uint
            }) as CmdRedraw;
        }
    }
}

pub(crate) unsafe extern "C" fn command_line_not_changed(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        may_trigger_cursormovedc(s);
        (*s).prev_cmdpos = (*ccline.ptr()).cmdpos;
        if !(*s).is_state.incsearch_postponed {
            return 1 as ::core::ffi::c_int;
        }
        return command_line_changed(s);
    }
}

pub(crate) unsafe extern "C" fn do_autocmd_cmdlinechanged(mut firstc: ::core::ffi::c_int) {
    unsafe {
        if has_event(EVENT_CMDLINECHANGED) {
            let mut err: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let mut save_v_event: save_v_event_T = save_v_event_T {
                sve_did_save: false,
                sve_hashtab: hashtab_T {
                    ht_mask: 0,
                    ht_used: 0,
                    ht_filled: 0,
                    ht_changed: 0,
                    ht_locked: 0,
                    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                    ht_smallarray: [hashitem_T {
                        hi_hash: 0,
                        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    }; 16],
                },
            };
            let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
            let mut firstcbuf: [::core::ffi::c_char; 2] = [0; 2];
            firstcbuf[0 as ::core::ffi::c_int as usize] = firstc as ::core::ffi::c_char;
            firstcbuf[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
            tv_dict_add_str(
                dict,
                b"cmdtype\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
            );
            tv_dict_add_nr(
                dict,
                b"cmdlevel\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                (*ccline.ptr()).level as varnumber_T,
            );
            tv_dict_set_keys_readonly(dict);
            let mut tstate: TryState = TryState {
                current_exception: ::core::ptr::null_mut::<except_T>(),
                private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                msg_list: ::core::ptr::null::<*const msglist_T>(),
                got_int: 0,
                did_throw: false,
                need_rethrow: 0,
                did_emsg: 0,
            };
            try_enter(&raw mut tstate);
            apply_autocmds(
                EVENT_CMDLINECHANGED,
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
                false,
                curbuf.get(),
            );
            restore_v_event(dict, &raw mut save_v_event);
            try_leave(&raw mut tstate, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                if !ui_has(kUIMessages) {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                msg_scroll.set(true_0);
                msg_puts_hl(err.msg, HLF_E, true_0 != 0);
                api_clear_error(&raw mut err);
                redrawcmd();
            }
        }
    }
}

pub(crate) unsafe extern "C" fn command_line_changed(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        let prev_cmdpreview: bool = cmdpreview.get();
        if !((*s).firstc == ':' as ::core::ffi::c_int
            && (*current_sctx.ptr()).sc_sid == 0 as ::core::ffi::c_int
            && *p_icm.get() as ::core::ffi::c_int != NUL
            && !exmode_active.get()
            && cmdline_star.get() == 0 as ::core::ffi::c_int
            && vpeekc_any() == 0
            && cmdpreview_may_show(s) as ::core::ffi::c_int != 0)
        {
            cmdpreview.set(false_0 != 0);
            if prev_cmdpreview {
                update_screen();
            }
            if (*s).xpc.xp_context == EXPAND_NOTHING as ::core::ffi::c_int
                && (KeyTyped.get() as ::core::ffi::c_int != 0 || vpeekc() == NUL)
            {
                may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
            }
        }
        if !(*ccline.ptr()).cmdbuff_replaced
            && ((*ccline.ptr()).cmdpos != (*s).prev_cmdpos
                || !(*s).prev_cmdbuff.is_null()
                    && strcmp((*s).prev_cmdbuff, (*ccline.ptr()).cmdbuff)
                        != 0 as ::core::ffi::c_int)
        {
            do_autocmd_cmdlinechanged(if (*s).firstc > 0 as ::core::ffi::c_int {
                (*s).firstc
            } else {
                '-' as ::core::ffi::c_int
            });
        }
        may_trigger_cursormovedc(s);
        if p_arshape.get() != 0 && p_tbidi.get() == 0 {
            if !ui_has(kUICmdline) && vpeekc() == NUL {
                redrawcmd();
            }
        }
        return 1 as ::core::ffi::c_int;
    }
}
