//! One key, dispatched: the key loop's per-key state machine.
//!
//! [`command_line_execute`] is the `state_execute` callback — it reads a key,
//! gives the wildmenu and `<C-\>` their chance at it, and hands the rest to
//! [`super::handlekey::command_line_handle_key`].  [`command_line_changed`]
//! is the other half: what runs after a key that edited the line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

/// What `CTRL-\` did with the key typed after it.
enum CtrlBsl {
    /// `CTRL-\ e` replaced the line with an expression's value.
    Changed,
    /// `CTRL-\ e` was refused, or its expression failed.
    NotChanged,
    /// `CTRL-\ CTRL-N` / `CTRL-\ CTRL-G`: leave the command line.
    GotoNormalMode,
    /// Not a `CTRL-\` sequence at all; the key was pushed back.
    ProcessNextKey,
}

/// Handle CTRL-\ pressed in Command-line mode:
///
/// - `CTRL-\ CTRL-N` or `CTRL-\ CTRL-G` goes to Normal mode.
/// - `CTRL-\ e` prompts for an expression.
unsafe fn command_line_handle_ctrl_bsl(s: *mut CommandLineState) -> CtrlBsl {
    unsafe {
        let cc = ccline.ptr();
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;

        // CTRL-\ e doesn't work when obtaining an expression, unless it is
        // in a mapping.
        if (*s).c != Ctrl_N
            && (*s).c != Ctrl_G
            && ((*s).c != 'e' as ::core::ffi::c_int
                || ((*cc).cmdfirstc == '=' as ::core::ffi::c_int && KeyTyped.get())
                || cmdline_star.get() > 0)
        {
            vungetc((*s).c);
            return CtrlBsl::ProcessNextKey;
        }

        if (*s).c != 'e' as ::core::ffi::c_int {
            // Will free ccline.cmdbuff after putting it in the history.
            (*s).gotesc = true;
            return CtrlBsl::GotoNormalMode;
        }

        // Replace the command line with the result of an expression. This
        // calls getcmdline() recursively, from get_expr_register().
        new_cmdpos.set(if (*cc).cmdpos == (*cc).cmdlen {
            99999 // keep it at the end
        } else {
            (*cc).cmdpos
        });

        (*s).c = get_expr_register();
        if (*s).c == '=' as ::core::ffi::c_int {
            // Evaluate the expression. "textlock" avoids nasty things like
            // going to another buffer.
            (*textlock.ptr()) += 1;
            let p = get_expr_line();
            (*textlock.ptr()) -= 1;

            if !p.is_null() {
                let len = strlen(p) as ::core::ffi::c_int;
                realloc_cmdbuff(len + 1);
                (*cc).cmdlen = len;
                strcpy((*cc).cmdbuff, p);
                xfree(p as *mut ::core::ffi::c_void);

                // Restore the cursor, or use the position set with
                // set_cmdline_pos().
                (*cc).cmdpos = (*cc).cmdlen.min(new_cmdpos.get());

                KeyTyped.set(false); // don't do 'wildchar' completion
                redrawcmd();
                return CtrlBsl::Changed;
            }
        }
        beep_flush();
        got_int.set(false); // don't abandon the command line
        did_emsg.set(0);
        emsg_on_display.set(false);
        redrawcmd();
        CtrlBsl::NotChanged
    }
}

/// Free the expanded names and take the wildmenu down.  `c` is the key that
/// ended it, or -1 when no key did.
pub(crate) unsafe fn command_line_end_wildmenu(
    s: *mut CommandLineState,
    key_is_wc: bool,
    c: ::core::ffi::c_int,
) {
    unsafe {
        if cmdline_pum_active() {
            if c != -1 {
                (*s).skip_pum_redraw = (*s).skip_pum_redraw
                    && !key_is_wc
                    && !ascii_iswhite(c)
                    && (vim_isprintc(c)
                        || c == K_BS
                        || c == Ctrl_H
                        || c == K_DEL
                        || c == K_KDEL
                        || c == Ctrl_W
                        || c == Ctrl_U);
            }
            cmdline_pum_remove(c != -1 && (*s).skip_pum_redraw);
        }
        if (*s).xpc.xp_numfiles != -1 {
            ExpandOne(
                &raw mut (*s).xpc,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0,
                WILD_FREE,
            );
        }
        (*s).did_wild_list = false;
        if p_wmnu.get() == 0 || (c != K_UP && c != K_DOWN) {
            (*s).xpc.xp_context = EXPAND_NOTHING;
        }
        (*s).wim_index = 0;
        wildmenu_cleanup(ccline.ptr());
    }
}

/// The key loop's `state_execute` callback: one key, dispatched.  Installed
/// in a `VimState`, so this one keeps its C ABI.  Answers -1 to fetch
/// another key, 0 to leave the command line and 1 to keep going.
pub(crate) unsafe extern "C" fn command_line_execute(
    state: *mut VimState,
    key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if key == K_IGNORE || key == K_NOP {
            return -1; // get another key
        }

        let display_tick_saved: disptick_T = (*curwin.get()).w_display_tick;
        let s: *mut CommandLineState = state as *mut CommandLineState;
        let cc = ccline.ptr();
        (*s).c = key;

        // If the cmdline was replaced externally (e.g. by setcmdline() during
        // an <expr> mapping), clean up the wildmenu completion state so that
        // stale completion data is not used.
        if (*cc).cmdbuff_replaced && (*s).xpc.xp_numfiles > 0 {
            command_line_end_wildmenu(s, false, -1);
        }
        (*cc).cmdbuff_replaced = false;

        // Skip the wildmenu during history navigation with Up/Down.
        if (*s).c == K_WILD && (*s).did_hist_navigate {
            (*s).did_hist_navigate = false;
            return 1;
        }

        if (*s).c == K_EVENT || (*s).c == K_COMMAND || (*s).c == K_LUA {
            if (*s).c == K_EVENT {
                state_handle_k_event();
            } else if (*s).c == K_COMMAND {
                do_cmdline(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    Some(getcmdkeycmd),
                    ::core::ptr::null_mut(),
                    DOCMD_NOWAIT,
                );
            } else {
                map_execute_lua(false, false);
            }
            // If the window changed, the incremental search state is invalid.
            if (*s).is_state.winid != (*curwin.get()).handle {
                init_incsearch_state(&raw mut (*s).is_state);
            }
            // Re-apply 'incsearch' highlighting in case it was cleared.
            if (*curwin.get()).w_display_tick > display_tick_saved && (*s).is_state.did_incsearch {
                may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
            }
            // If f_setcmdline() changed the cmdline, treat it as such.
            if (*cc).cmdbuff_replaced {
                command_line_changed(s);
            }

            // nvim_select_popupmenu_item() can be called from the handling of
            // K_EVENT, K_COMMAND or K_LUA.
            if (*pum_want.ptr()).active {
                if cmdline_pum_active() {
                    nextwild(
                        &raw mut (*s).xpc,
                        WILD_PUM_WANT,
                        0,
                        (*s).firstc != '@' as ::core::ffi::c_int,
                    );
                    if (*pum_want.ptr()).finish {
                        nextwild(
                            &raw mut (*s).xpc,
                            WILD_APPLY,
                            WILD_NO_BEEP,
                            (*s).firstc != '@' as ::core::ffi::c_int,
                        );
                        command_line_end_wildmenu(s, false, (*s).c);
                    }
                }
                (*pum_want.ptr()).active = false;
            }

            if !cmdline_was_last_drawn.get() {
                redrawcmdline();
            }
            return 1;
        }

        if KeyTyped.get() {
            (*s).some_key_typed = true;

            if cmdmsg_rl.get() && KeyStuffed.get() == 0 {
                // Invert horizontal movements and operations. Only when typed
                // by the user directly, not as the result of a mapping.
                match (*s).c {
                    K_RIGHT => (*s).c = K_LEFT,
                    K_S_RIGHT => (*s).c = K_S_LEFT,
                    K_C_RIGHT => (*s).c = K_C_LEFT,
                    K_LEFT => (*s).c = K_RIGHT,
                    K_S_LEFT => (*s).c = K_S_RIGHT,
                    K_C_LEFT => (*s).c = K_C_RIGHT,
                    _ => {}
                }
            }
        }

        // Ignore got_int when CTRL-C was typed here. Don't ignore it in
        // :global, we really need to break then, e.g. for ":g/pat/normal /pat"
        // (without the <CR>). Don't ignore it for the input() function.
        if (*s).c == Ctrl_C
            && (*s).firstc != '@' as ::core::ffi::c_int
            // do clear got_int in Ex mode, to avoid an infinite Ctrl-C loop
            && (!(*s).break_ctrl_c || exmode_active.get())
            && global_busy.get() == 0
        {
            got_int.set(false);
        }

        // Free the old command line when finished moving around in the
        // history list.
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
            && ((*s).xpc.xp_numfiles > 0 || ((*s).c != Ctrl_P && (*s).c != Ctrl_N))
        {
            xfree((*s).lookfor as *mut ::core::ffi::c_void);
            (*s).lookfor = ::core::ptr::null_mut();
            (*s).lookforlen = 0;
        }

        // When there are matching completions to select, <S-Tab> works like
        // CTRL-P (unless 'wildchar' is <S-Tab>).
        if (*s).c as OptInt != p_wc.get() && (*s).c == K_S_TAB && (*s).xpc.xp_numfiles > 0 {
            (*s).c = Ctrl_P;
        }

        if p_wmnu.get() != 0 {
            (*s).c = wildmenu_translate_key(cc, (*s).c, &raw mut (*s).xpc, (*s).did_wild_list);
        }

        let mut wild_type = 0;
        let key_is_wc =
            ((*s).c as OptInt == p_wc.get() && KeyTyped.get()) || (*s).c as OptInt == p_wcm.get();
        if (cmdline_pum_active() || wild_menu_showing.get() != 0 || (*s).did_wild_list)
            && !key_is_wc
            && (*s).xpc.xp_numfiles > 0
        {
            // Ctrl-Y: accept the current selection and close the popup menu.
            // Ctrl-E: cancel the cmdline popup menu and return the original
            // text.
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

        // Trigger the CmdlineLeavePre autocommand.
        if (KeyTyped.get()
            && ((*s).c == '\n' as ::core::ffi::c_int
                || (*s).c == '\r' as ::core::ffi::c_int
                || (*s).c == K_KENTER
                || (*s).c == ESC))
            || (*s).c == Ctrl_C
        {
            set_vim_var_char((*s).c); // set v:char
            trigger_cmd_autocmd((*s).cmdline_type, EVENT_CMDLINELEAVEPRE);
            (*s).event_cmdlineleavepre_triggered = true;
            if ((*s).c == ESC || (*s).c == Ctrl_C) && wim_has(0, kOptWimFlagList) {
                set_no_hlsearch(true);
            }
        }

        // The wildmenu is cleared if the pressed key is not used for
        // navigating it (i.e. is not 'wildchar' or 'wildcharm' or Ctrl-N or
        // Ctrl-P or Ctrl-A or Ctrl-L). If the popup menu is displayed then
        // PageDown and PageUp navigate it too.
        let end_wildmenu = !key_is_wc
            && (*s).c != Ctrl_Z
            && (*s).c != Ctrl_N
            && (*s).c != Ctrl_P
            && (*s).c != Ctrl_A
            && (*s).c != Ctrl_L
            && (!cmdline_pum_active()
                || ((*s).c != K_PAGEDOWN
                    && (*s).c != K_PAGEUP
                    && (*s).c != K_KPAGEDOWN
                    && (*s).c != K_KPAGEUP));

        // Free the expanded names when finished walking through the matches.
        if end_wildmenu {
            command_line_end_wildmenu(s, key_is_wc, (*s).c);
        }

        if p_wmnu.get() != 0 {
            (*s).c = wildmenu_process_key(cc, (*s).c, &raw mut (*s).xpc);
        }

        // CTRL-\ CTRL-N or CTRL-\ CTRL-G goes to Normal mode, CTRL-\ e
        // prompts for an expression.
        if (*s).c == Ctrl_BSL {
            match command_line_handle_ctrl_bsl(s) {
                CtrlBsl::Changed => return command_line_changed(s),
                CtrlBsl::NotChanged => return command_line_not_changed(s),
                CtrlBsl::GotoNormalMode => return 0, // back to cmd mode
                // The backslash key was not processed by
                // command_line_handle_ctrl_bsl().
                CtrlBsl::ProcessNextKey => (*s).c = Ctrl_BSL,
            }
        }

        if (*s).c == cedit_key.get() || (*s).c == K_CMDWIN {
            // TODO(vim): why is ex_normal_busy checked here?
            if ((*s).c == K_CMDWIN || ex_normal_busy.get() == 0) && !got_int.get() {
                // Open a window to edit the command line (and history).
                (*s).c = open_cmdwin();
                (*s).some_key_typed = true;
            }
        } else {
            (*s).c = do_digraph((*s).c);
        }

        if (*s).c == '\n' as ::core::ffi::c_int
            || (*s).c == '\r' as ::core::ffi::c_int
            || (*s).c == K_KENTER
            || ((*s).c == ESC && (!KeyTyped.get() || !vim_strchr(p_cpo.get(), CPO_ESC).is_null()))
        {
            // In Ex mode a backslash escapes a newline.
            if exmode_active.get()
                && (*s).c != ESC
                && (*cc).cmdpos == (*cc).cmdlen
                && (*cc).cmdpos > 0
                && *(*cc).cmdbuff.offset(((*cc).cmdpos - 1) as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                if (*s).c == K_KENTER {
                    (*s).c = '\n' as ::core::ffi::c_int;
                }
            } else {
                // Might have typed ESC previously; don't truncate the cmdline
                // now.
                (*s).gotesc = false;
                if ccheck_abbr((*s).c + ABBR_OFF) {
                    return command_line_changed(s);
                }

                if !cmd_silent.get() {
                    if !ui_has(kUICmdline) {
                        msg_cursor_goto(msg_row.get(), 0);
                    }
                    ui_flush();
                }
                return 0;
            }
        }

        // Completion for 'wildchar', 'wildcharm' and wildtrigger().
        if ((*s).c as OptInt == p_wc.get() && !(*s).gotesc && KeyTyped.get())
            || (*s).c as OptInt == p_wcm.get()
            || (*s).c == K_WILD
            || (*s).c == Ctrl_Z
        {
            if (*s).c == K_WILD {
                (*emsg_silent.ptr()) += 1; // silence the bell
            }
            let res = command_line_wildchar_complete(s);
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

        (*s).gotesc = false;

        // <S-Tab> goes to the last match, in a clumsy way.
        if (*s).c == K_S_TAB
            && KeyTyped.get()
            && nextwild(
                &raw mut (*s).xpc,
                WILD_EXPAND_KEEP,
                0,
                (*s).firstc != '@' as ::core::ffi::c_int,
            ) == OK
        {
            if (*s).xpc.xp_numfiles > 1
                && ((!(*s).did_wild_list && wim_has((*s).wim_index, kOptWimFlagList))
                    || p_wmnu.get() != 0)
            {
                // Trigger the popup menu when wildoptions=pum.
                showmatches(
                    &raw mut (*s).xpc,
                    p_wmnu.get() != 0,
                    wim_has((*s).wim_index, kOptWimFlagList),
                    wim_has(0, kOptWimFlagNoselect),
                );
            }
            nextwild(
                &raw mut (*s).xpc,
                WILD_PREV,
                0,
                (*s).firstc != '@' as ::core::ffi::c_int,
            );
            nextwild(
                &raw mut (*s).xpc,
                WILD_PREV,
                0,
                (*s).firstc != '@' as ::core::ffi::c_int,
            );
            return command_line_changed(s);
        }

        if (*s).c == NUL || (*s).c == K_ZERO {
            (*s).c = NL; // NUL is stored as NL
        }

        (*s).do_abbr = true; // default: check for an abbreviation

        // If the key was already used to cancel or accept the wildmenu, don't
        // process it any further.
        if wild_type == WILD_CANCEL || wild_type == WILD_APPLY {
            // Apply search highlighting.
            if (*s).is_state.winid != (*curwin.get()).handle {
                init_incsearch_state(&raw mut (*s).is_state);
            }
            if KeyTyped.get() || vpeekc() == NUL {
                may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
            }
            return command_line_not_changed(s);
        }

        command_line_handle_key(s)
    }
}

pub(crate) unsafe fn may_trigger_cursormovedc(s: *mut CommandLineState) {
    unsafe {
        let cc = ccline.ptr();
        if (*cc).cmdpos != (*s).prev_cmdpos {
            trigger_cmd_autocmd((*s).cmdline_type, EVENT_CURSORMOVEDC);
            (*cc).redraw_state = (*cc).redraw_state.max(kCmdRedrawPos);
        }
    }
}

/// A key was read but the command line did not change.
///
/// Incremental searches for `/` and `?` only search and redraw here if
/// something changed in the past; [`command_line_changed`] is what runs when
/// the line itself did change.
pub(crate) unsafe fn command_line_not_changed(s: *mut CommandLineState) -> ::core::ffi::c_int {
    unsafe {
        may_trigger_cursormovedc(s);
        (*s).prev_cmdpos = (*ccline.ptr()).cmdpos;
        if !(*s).is_state.incsearch_postponed {
            return 1;
        }
        command_line_changed(s)
    }
}

/// Trigger the `CmdlineChanged` autocommands.
pub(crate) unsafe fn do_autocmd_cmdlinechanged(firstc: ::core::ffi::c_int) {
    unsafe {
        if !has_event(EVENT_CMDLINECHANGED) {
            return;
        }
        let mut err: Error = ERROR_INIT;
        let mut save_v_event: save_v_event_T = SAVE_V_EVENT_INIT;
        let mut firstcbuf: [::core::ffi::c_char; 2] = [firstc as ::core::ffi::c_char, 0];
        let dict = cmdline_event_dict(&raw mut save_v_event, firstcbuf.as_ptr());

        // C's TRY_WRAP, with restore_v_event() inside it.
        let mut tstate: TryState = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        apply_autocmds(
            EVENT_CMDLINECHANGED,
            firstcbuf.as_mut_ptr(),
            firstcbuf.as_mut_ptr(),
            false,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);
        try_leave(&raw mut tstate, &raw mut err);

        if err.type_0 != kErrorTypeNone {
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            msg_scroll.set(1);
            msg_puts_hl(err.msg, HLF_E, true);
            api_clear_error(&raw mut err);
            redrawcmd();
        }
    }
}

/// A key changed the command line: show the `'inccommand'` preview or the
/// `'incsearch'` highlighting, and fire `CmdlineChanged`.
pub(crate) unsafe fn command_line_changed(s: *mut CommandLineState) -> ::core::ffi::c_int {
    unsafe {
        let cc = ccline.ptr();
        let prev_cmdpreview = cmdpreview.get();
        let preview_shown = (*s).firstc == ':' as ::core::ffi::c_int
            && (*current_sctx.ptr()).sc_sid == 0 // only if interactive
            && *p_icm.get() as ::core::ffi::c_int != NUL // 'inccommand' is set
            && !exmode_active.get() // not in ex mode
            && cmdline_star.get() == 0 // not typing a password
            && vpeekc_any() == 0
            && cmdpreview_may_show(s);
        if !preview_shown {
            cmdpreview.set(false);
            if prev_cmdpreview {
                // TODO(bfredl): add an immediate redraw flag for cmdline mode
                // which will trigger at the next wait-for-input.
                update_screen(); // clear the 'inccommand' preview
            }
            if (*s).xpc.xp_context == EXPAND_NOTHING && (KeyTyped.get() || vpeekc() == NUL) {
                may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
            }
        }

        if !(*cc).cmdbuff_replaced
            && ((*cc).cmdpos != (*s).prev_cmdpos
                || (!(*s).prev_cmdbuff.is_null() && strcmp((*s).prev_cmdbuff, (*cc).cmdbuff) != 0))
        {
            do_autocmd_cmdlinechanged(if (*s).firstc > 0 {
                (*s).firstc
            } else {
                '-' as ::core::ffi::c_int
            });
        }

        may_trigger_cursormovedc(s);

        if p_arshape.get() != 0 && p_tbidi.get() == 0 {
            // Always redraw the whole command line, to fix shaping and
            // right-left typing. Not efficient, but it works. Only do it when
            // there are no characters left to read, to avoid useless
            // intermediate redraws. If the cmdline is external the UI handles
            // shaping and no redraw is needed.
            if !ui_has(kUICmdline) && vpeekc() == NUL {
                redrawcmd();
            }
        }

        1
    }
}
