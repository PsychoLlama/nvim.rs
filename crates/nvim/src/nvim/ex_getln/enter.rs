//! Entering and leaving the command line: the outer loop.
//!
//! [`command_line_enter`] is C's `command_line_enter` — it builds the
//! `CommandLineState`, publishes a fresh `ccline`, runs the key loop through
//! `state_enter`, and unwinds all of it again on the way out.
//! [`getcmdline`] and [`getcmdline_prompt`] are the two entry points every
//! caller outside this module uses.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn init_ccline(
    mut firstc: ::core::ffi::c_int,
    mut indent: ::core::ffi::c_int,
) {
    unsafe {
        (*ccline.ptr()).overstrike = false_0;
        '_c2rust_label: {
            if indent >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"indent >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    691 as ::core::ffi::c_uint,
                    b"void init_ccline(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*ccline.ptr()).cmdfirstc = if firstc == '@' as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            firstc
        };
        (*ccline.ptr()).cmdindent = if firstc > 0 as ::core::ffi::c_int {
            indent
        } else {
            0 as ::core::ffi::c_int
        };
        alloc_cmdbuff(indent + 50 as ::core::ffi::c_int);
        (*ccline.ptr()).cmdpos = 0 as ::core::ffi::c_int;
        (*ccline.ptr()).cmdlen = (*ccline.ptr()).cmdpos;
        *(*ccline.ptr())
            .cmdbuff
            .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        (*ccline.ptr()).last_colors = ColoredCmdline {
            prompt_id: 0,
            cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            colors: CmdlineColors {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
            },
        };
        sb_text_start_cmdline();
        if firstc <= 0 as ::core::ffi::c_int {
            memset(
                (*ccline.ptr()).cmdbuff as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                indent as size_t,
            );
            *(*ccline.ptr()).cmdbuff.offset(indent as isize) = NUL as ::core::ffi::c_char;
            (*ccline.ptr()).cmdpos = indent;
            (*ccline.ptr()).cmdspos = indent;
            (*ccline.ptr()).cmdlen = indent;
        }
    }
}

pub(crate) unsafe extern "C" fn ui_ext_cmdline_hide(mut abort_0: bool) {
    unsafe {
        if ui_has(kUICmdline) {
            cmdline_was_last_drawn.set(false_0 != 0);
            (*ccline.ptr()).redraw_state = kCmdRedrawNone;
            ui_call_cmdline_hide((*ccline.ptr()).level as Integer, abort_0 as Boolean);
        }
    }
}

pub(crate) unsafe extern "C" fn command_line_enter(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut indent: ::core::ffi::c_int,
    mut clear_ccline: bool,
) -> *mut uint8_t {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeException,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut firstcbuf: [::core::ffi::c_char; 2] = [0; 2];
        static cmdline_level: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        (*cmdline_level.ptr()) += 1;
        let mut save_cmdpreview: bool = cmdpreview.get();
        cmdpreview.set(false_0 != 0);
        let mut state: CommandLineState = CommandLineState {
            state: VimState {
                check: None,
                execute: None,
            },
            firstc: firstc,
            count: count,
            indent: indent,
            c: 0,
            gotesc: false,
            do_abbr: false,
            lookfor: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            lookforlen: 0,
            hiscnt: 0,
            save_hiscnt: 0,
            histype: 0,
            is_state: incsearch_state_T {
                search_start: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                save_cursor: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                winid: 0,
                init_viewstate: viewstate_T {
                    vs_curswant: 0,
                    vs_leftcol: 0,
                    vs_skipcol: 0,
                    vs_topline: 0,
                    vs_topfill: 0,
                    vs_botline: 0,
                    vs_empty_rows: 0,
                },
                old_viewstate: viewstate_T {
                    vs_curswant: 0,
                    vs_leftcol: 0,
                    vs_skipcol: 0,
                    vs_topline: 0,
                    vs_topfill: 0,
                    vs_botline: 0,
                    vs_empty_rows: 0,
                },
                match_start: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                match_end: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                did_incsearch: false,
                incsearch_postponed: false,
                magic_overruled_save: OPTION_MAGIC_NOT_SET,
            },
            did_wild_list: false,
            wim_index: 0,
            save_msg_scroll: msg_scroll.get(),
            save_State: State.get(),
            prev_cmdpos: -1 as ::core::ffi::c_int,
            prev_cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_p_icm: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            skip_pum_redraw: false,
            some_key_typed: false,
            ignore_drag_release: true_0 != 0,
            break_ctrl_c: false,
            xpc: expand_T {
                xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_context: 0,
                xp_pattern_len: 0,
                xp_prefix: XP_PREFIX_NONE,
                xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_luaref: 0,
                xp_script_ctx: sctx_T {
                    sc_sid: 0,
                    sc_seq: 0,
                    sc_lnum: 0,
                    sc_chan: 0,
                },
                xp_backslash: 0,
                xp_shell: false,
                xp_numfiles: 0,
                xp_col: 0,
                xp_selected: 0,
                xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_buf: [0; 256],
                xp_search_dir: kDirectionNotSet,
                xp_pre_incsearch_pos: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
            },
            b_im_ptr: ::core::ptr::null_mut::<OptInt>(),
            b_im_ptr_buf: ::core::ptr::null_mut::<buf_T>(),
            cmdline_type: 0,
            event_cmdlineleavepre_triggered: false,
            did_hist_navigate: false,
        };
        let mut s: *mut CommandLineState = &raw mut state;
        (*s).save_p_icm = xstrdup(p_icm.get());
        init_incsearch_state(&raw mut (*s).is_state);
        let mut save_ccline: CmdlineInfo = CmdlineInfo {
            cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdbufflen: 0,
            cmdlen: 0,
            cmdpos: 0,
            cmdspos: 0,
            cmdfirstc: 0,
            cmdindent: 0,
            cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            hl_id: 0,
            overstrike: 0,
            xpc: ::core::ptr::null_mut::<expand_T>(),
            xp_context: 0,
            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            input_fn: 0,
            cmdbuff_replaced: false,
            prompt_id: 0,
            highlight_callback: Callback {
                data: C2Rust_Unnamed_5 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            },
            last_colors: ColoredCmdline {
                prompt_id: 0,
                cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                colors: CmdlineColors {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
                },
            },
            level: 0,
            prev_ccline: ::core::ptr::null_mut::<CmdlineInfo>(),
            special_char: 0,
            special_shift: false,
            redraw_state: kCmdRedrawNone,
            one_key: false,
            mouse_used: ::core::ptr::null_mut::<bool>(),
        };
        let mut did_save_ccline: bool = false_0 != 0;
        if !(*ccline.ptr()).cmdbuff.is_null() {
            '_c2rust_label: {
                if clear_ccline {
                } else {
                    __assert_fail(
                        b"clear_ccline\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        756 as ::core::ffi::c_uint,
                        b"uint8_t *command_line_enter(int, int, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            save_cmdline(&raw mut save_ccline);
            did_save_ccline = true_0 != 0;
        } else if clear_ccline {
            memset(
                ccline.ptr() as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<CmdlineInfo>(),
            );
        }
        if (*s).firstc == -1 as ::core::ffi::c_int {
            (*s).firstc = NUL;
            (*s).break_ctrl_c = true_0 != 0;
        }
        init_ccline((*s).firstc, (*s).indent);
        '_c2rust_label_0: {
            if !(*ccline.ptr()).cmdbuff.is_null() {
            } else {
                __assert_fail(
                    b"ccline.cmdbuff != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    771 as ::core::ffi::c_uint,
                    b"uint8_t *command_line_enter(int, int, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let c2rust_fresh1 = last_prompt_id.get();
        last_prompt_id.set((*last_prompt_id.ptr()).wrapping_add(1));
        (*ccline.ptr()).prompt_id = c2rust_fresh1;
        (*ccline.ptr()).level = cmdline_level.get();
        if cmdline_level.get() == 50 as ::core::ffi::c_int {
            emsg(gettext(
                &raw const e_command_too_recursive as *const ::core::ffi::c_char,
            ));
        } else {
            ExpandInit(&raw mut (*s).xpc);
            (*ccline.ptr()).xpc = &raw mut (*s).xpc;
            clear_cmdline_orig();
            cmdmsg_rl.set(
                (*curwin.get()).w_onebuf_opt.wo_rl != 0
                    && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int
                        == 's' as ::core::ffi::c_int
                    && ((*s).firstc == '/' as ::core::ffi::c_int
                        || (*s).firstc == '?' as ::core::ffi::c_int),
            );
            msg_grid_validate();
            redir_off.set(true_0 != 0);
            if !cmd_silent.get() {
                gotocmdline(true_0 != 0);
                redrawcmdprompt();
                (*ccline.ptr()).cmdspos = cmd_startcol();
            }
            (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            (*s).xpc.xp_backslash = XP_BS_NONE as ::core::ffi::c_int;
            (*s).xpc.xp_shell = false_0 != 0;
            if (*ccline.ptr()).input_fn != 0 {
                (*s).xpc.xp_context = (*ccline.ptr()).xp_context;
                (*s).xpc.xp_pattern = (*ccline.ptr()).cmdbuff;
                (*s).xpc.xp_arg = (*ccline.ptr()).xp_arg;
            }
            msg_scroll.set(false_0);
            State.set(MODE_CMDLINE);
            if (*s).firstc == '/' as ::core::ffi::c_int
                || (*s).firstc == '?' as ::core::ffi::c_int
                || (*s).firstc == '@' as ::core::ffi::c_int
            {
                if (*curbuf.get()).b_p_imsearch == B_IMODE_USE_INSERT as OptInt {
                    (*s).b_im_ptr = &raw mut (*curbuf.get()).b_p_iminsert;
                } else {
                    (*s).b_im_ptr = &raw mut (*curbuf.get()).b_p_imsearch;
                }
                (*s).b_im_ptr_buf = curbuf.get();
                if *(*s).b_im_ptr == B_IMODE_LMAP as OptInt {
                    (*State.ptr()) |= MODE_LANGMAP;
                }
            }
            setmouse();
            (*s).cmdline_type = if firstc > 0 as ::core::ffi::c_int {
                firstc
            } else {
                '-' as ::core::ffi::c_int
            };
            err = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            firstcbuf = [0; 2];
            firstcbuf[0 as ::core::ffi::c_int as usize] = (*s).cmdline_type as ::core::ffi::c_char;
            firstcbuf[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
            if has_event(EVENT_CMDLINEENTER) {
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
                    EVENT_CMDLINEENTER,
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
                err = Error {
                    type_0: kErrorTypeNone,
                    msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
            }
            may_trigger_modechanged();
            init_history();
            (*s).hiscnt = get_hislen();
            (*s).histype = hist_char2type((*s).firstc) as ::core::ffi::c_int;
            do_digraph(-1 as ::core::ffi::c_int);
            if did_emsg.get() != 0 {
                redrawcmd();
            }
            if !cmd_silent.get() && !exmode_active.get() {
                let mut found_one: bool = false_0 != 0;
                let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                    firstwin.get()
                } else {
                    (*curtab.get()).tp_firstwin
                };
                while !wp.is_null() {
                    if *p_stl.get() as ::core::ffi::c_int != NUL
                        || *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
                        || *p_wbr.get() as ::core::ffi::c_int != NUL
                        || *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
                    {
                        (*wp).w_redr_status = true_0 != 0;
                        found_one = true_0 != 0;
                    }
                    wp = (*wp).w_next;
                }
                if *p_tal.get() as ::core::ffi::c_int != NUL {
                    redraw_tabline.set(true_0 != 0);
                    found_one = true_0 != 0;
                }
                if redraw_custom_title_later() {
                    found_one = true_0 != 0;
                }
                if found_one {
                    redraw_statuslines();
                }
            }
            did_emsg.set(false_0);
            got_int.set(false_0 != 0);
            (*s).state.check = Some(
                command_line_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int,
            ) as state_check_callback;
            (*s).state.execute = Some(
                command_line_execute
                    as unsafe extern "C" fn(
                        *mut VimState,
                        ::core::ffi::c_int,
                    ) -> ::core::ffi::c_int,
            ) as state_execute_callback;
            state_enter(&raw mut (*s).state);
            if !(*s).event_cmdlineleavepre_triggered {
                set_vim_var_char((*s).c);
                trigger_cmd_autocmd((*s).cmdline_type, EVENT_CMDLINELEAVEPRE);
            }
            if has_event(EVENT_CMDLINELEAVE) {
                let mut save_v_event_0: save_v_event_T = save_v_event_T {
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
                let mut dict_0: *mut dict_T = get_v_event(&raw mut save_v_event_0);
                tv_dict_add_str(
                    dict_0,
                    b"cmdtype\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    &raw mut firstcbuf as *mut ::core::ffi::c_char,
                );
                tv_dict_add_nr(
                    dict_0,
                    b"cmdlevel\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    (*ccline.ptr()).level as varnumber_T,
                );
                tv_dict_set_keys_readonly(dict_0);
                tv_dict_add_bool(
                    dict_0,
                    b"abort\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    (if (*s).gotesc as ::core::ffi::c_int != 0 {
                        kBoolVarTrue as ::core::ffi::c_int
                    } else {
                        kBoolVarFalse as ::core::ffi::c_int
                    }) as BoolVarValue,
                );
                set_vim_var_char((*s).c);
                let mut tstate_0: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate_0);
                apply_autocmds(
                    EVENT_CMDLINELEAVE,
                    &raw mut firstcbuf as *mut ::core::ffi::c_char,
                    &raw mut firstcbuf as *mut ::core::ffi::c_char,
                    false,
                    curbuf.get(),
                );
                try_leave(&raw mut tstate_0, &raw mut err);
                if tv_dict_get_number(dict_0, b"abort\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as varnumber_T
                {
                    (*s).gotesc = true_0 != 0;
                }
                restore_v_event(dict_0, &raw mut save_v_event_0);
            }
            cmdmsg_rl.set(false_0 != 0);
            if cmdline_pum_active() {
                cmdline_pum_remove(false_0 != 0);
            } else {
                pum_check_clear();
            }
            wildmenu_cleanup(ccline.ptr());
            (*s).did_wild_list = false_0 != 0;
            (*s).wim_index = 0 as ::core::ffi::c_int;
            ExpandCleanup(&raw mut (*s).xpc);
            (*ccline.ptr()).xpc = ::core::ptr::null_mut::<expand_T>();
            clear_cmdline_orig();
            finish_incsearch_highlighting((*s).gotesc, &raw mut (*s).is_state, false_0 != 0);
            if !(*ccline.ptr()).cmdbuff.is_null() {
                if (*s).histype != HIST_INVALID as ::core::ffi::c_int
                    && (*ccline.ptr()).cmdlen != 0
                    && (*s).firstc != NUL
                    && ((*s).some_key_typed as ::core::ffi::c_int != 0
                        || (*s).histype == HIST_SEARCH as ::core::ffi::c_int)
                {
                    add_to_history(
                        (*s).histype,
                        ::core::slice::from_raw_parts(
                            (*ccline.ptr()).cmdbuff as *const u8,
                            (*ccline.ptr()).cmdlen as usize,
                        ),
                        true_0 != 0,
                        if (*s).histype == HIST_SEARCH as ::core::ffi::c_int {
                            (*s).firstc as u8
                        } else {
                            NUL as u8
                        },
                    );
                    if (*s).firstc == ':' as ::core::ffi::c_int {
                        xfree(new_last_cmdline.get() as *mut ::core::ffi::c_void);
                        new_last_cmdline.set(xstrnsave(
                            (*ccline.ptr()).cmdbuff,
                            (*ccline.ptr()).cmdlen as size_t,
                        ));
                    }
                }
                if (*s).gotesc {
                    abandon_cmdline();
                }
            }
            msg_check();
            if p_ch.get() == 0 as OptInt && !ui_has(kUIMessages) {
                set_must_redraw(UPD_VALID);
            }
            msg_scroll.set((*s).save_msg_scroll);
            redir_off.set(false_0 != 0);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                if !ui_has(kUIMessages) {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                emsg(err.msg);
                did_emsg.set(false_0);
                api_clear_error(&raw mut err);
            }
            if (*s).some_key_typed as ::core::ffi::c_int != 0
                && !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            {
                need_wait_return.set(false_0 != 0);
            }
            set_option_direct(
                kOptInccommand,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*s).save_p_icm),
                    },
                },
                0 as ::core::ffi::c_int,
                SID_NONE,
            );
            State.set((*s).save_State);
            if cmdpreview.get() as ::core::ffi::c_int != save_cmdpreview as ::core::ffi::c_int {
                cmdpreview.set(save_cmdpreview);
                redraw_all_later(UPD_SOME_VALID);
            }
            may_trigger_modechanged();
            setmouse();
            sb_text_end_cmdline();
        }
        xfree((*s).save_p_icm as *mut ::core::ffi::c_void);
        xfree((*ccline.ptr()).last_colors.cmdbuff as *mut ::core::ffi::c_void);
        xfree((*ccline.ptr()).last_colors.colors.items as *mut ::core::ffi::c_void);
        (*ccline.ptr()).last_colors.colors.capacity = 0 as size_t;
        (*ccline.ptr()).last_colors.colors.size = (*ccline.ptr()).last_colors.colors.capacity;
        (*ccline.ptr()).last_colors.colors.items = ::core::ptr::null_mut::<CmdlineColorChunk>();
        let mut p: *mut ::core::ffi::c_char = (*ccline.ptr()).cmdbuff;
        if ui_has(kUICmdline) {
            if exmode_active.get() as ::core::ffi::c_int != 0 && !p.is_null() {
                ui_ext_cmdline_block_append(0 as size_t, p);
            }
            ui_ext_cmdline_hide((*s).gotesc);
        }
        if !cmd_silent.get() {
            redraw_custom_title_later();
            status_redraw_all();
        }
        (*cmdline_level.ptr()) -= 1;
        if did_save_ccline {
            restore_cmdline(&raw mut save_ccline);
        } else {
            (*ccline.ptr()).cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        xfree((*s).prev_cmdbuff as *mut ::core::ffi::c_void);
        return p as *mut uint8_t;
    }
}

pub(crate) unsafe extern "C" fn command_line_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *mut CommandLineState = state as *mut CommandLineState;
        (*s).prev_cmdpos = (*ccline.ptr()).cmdpos;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*s).prev_cmdbuff as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        redir_off.set(true_0 != 0);
        quit_more.set(false_0 != 0);
        did_emsg.set(false_0);
        if ex_normal_busy.get() == 0 as ::core::ffi::c_int
            && stuff_empty() as ::core::ffi::c_int != 0
            && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        {
            (*s).some_key_typed = true_0 != 0;
        }
        may_trigger_safestate((*s).xpc.xp_numfiles <= 0 as ::core::ffi::c_int);
        if !(*ccline.ptr()).cmdbuff.is_null() {
            (*s).prev_cmdbuff = xstrdup((*ccline.ptr()).cmdbuff);
        }
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            && (*s).firstc != '@' as ::core::ffi::c_int
        {
            (*s).skip_pum_redraw = true_0 != 0;
        }
        cursorcmd();
        ui_cursor_shape();
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn abandon_cmdline() {
    unsafe {
        dealloc_cmdbuff();
        if msg_scrolled.get() == 0 as ::core::ffi::c_int {
            compute_cmdrow();
        }
        if !(*ccline.ptr()).one_key {
            msg(
                b"\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
            redraw_cmdline.set(true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn getcmdline(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return command_line_enter(firstc, count, indent, true_0 != 0) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn getcmdline_prompt(
    firstc: ::core::ffi::c_int,
    prompt: *const ::core::ffi::c_char,
    hl_id: ::core::ffi::c_int,
    xp_context: ::core::ffi::c_int,
    xp_arg: *const ::core::ffi::c_char,
    highlight_callback: Callback,
    mut one_key: bool,
    mut mouse_used: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let msg_col_save: ::core::ffi::c_int = msg_col.get();
        let mut save_ccline: CmdlineInfo = CmdlineInfo {
            cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdbufflen: 0,
            cmdlen: 0,
            cmdpos: 0,
            cmdspos: 0,
            cmdfirstc: 0,
            cmdindent: 0,
            cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            hl_id: 0,
            overstrike: 0,
            xpc: ::core::ptr::null_mut::<expand_T>(),
            xp_context: 0,
            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            input_fn: 0,
            cmdbuff_replaced: false,
            prompt_id: 0,
            highlight_callback: Callback {
                data: C2Rust_Unnamed_5 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            },
            last_colors: ColoredCmdline {
                prompt_id: 0,
                cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                colors: CmdlineColors {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
                },
            },
            level: 0,
            prev_ccline: ::core::ptr::null_mut::<CmdlineInfo>(),
            special_char: 0,
            special_shift: false,
            redraw_state: kCmdRedrawNone,
            one_key: false,
            mouse_used: ::core::ptr::null_mut::<bool>(),
        };
        let mut did_save_ccline: bool = false_0 != 0;
        if !(*ccline.ptr()).cmdbuff.is_null() {
            save_cmdline(&raw mut save_ccline);
            did_save_ccline = true_0 != 0;
        } else {
            memset(
                ccline.ptr() as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<CmdlineInfo>(),
            );
        }
        let c2rust_fresh32 = last_prompt_id.get();
        last_prompt_id.set((*last_prompt_id.ptr()).wrapping_add(1));
        (*ccline.ptr()).prompt_id = c2rust_fresh32;
        (*ccline.ptr()).cmdprompt = prompt as *mut ::core::ffi::c_char;
        (*ccline.ptr()).hl_id = hl_id;
        (*ccline.ptr()).xp_context = xp_context;
        (*ccline.ptr()).xp_arg = xp_arg as *mut ::core::ffi::c_char;
        (*ccline.ptr()).input_fn = (firstc == '@' as ::core::ffi::c_int) as ::core::ffi::c_int;
        (*ccline.ptr()).highlight_callback = highlight_callback;
        (*ccline.ptr()).one_key = one_key;
        (*ccline.ptr()).mouse_used = mouse_used;
        let cmd_silent_saved: bool = cmd_silent.get();
        let mut msg_silent_saved: ::core::ffi::c_int = msg_silent.get();
        msg_silent.set(0 as ::core::ffi::c_int);
        cmd_silent.set(false_0 != 0);
        let ret: *mut ::core::ffi::c_char = command_line_enter(
            firstc,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        ) as *mut ::core::ffi::c_char;
        (*ccline.ptr()).redraw_state = kCmdRedrawNone;
        if did_save_ccline {
            restore_cmdline(&raw mut save_ccline);
        }
        msg_silent.set(msg_silent_saved);
        cmd_silent.set(cmd_silent_saved);
        if !(*ccline.ptr()).cmdbuff.is_null() {
            msg_col.set(msg_col_save);
        }
        return ret;
    }
}
