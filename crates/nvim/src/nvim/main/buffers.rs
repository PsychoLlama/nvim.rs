//! Turning the file arguments into buffers and windows.
//!
//! The argument list is already built by the time these run; they decide which
//! buffers exist, how many windows and tab pages hold them, and which one the
//! cursor starts in.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn set_argf_var() {
    let mut list: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    let mut i: c_int = 0 as c_int;
    while i < (*global_alist.ptr()).al_ga.ga_len {
        let mut fname: *mut c_char =
            alist_name(((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(i as isize));
        if !fname.is_null() {
            vim_FullName(
                fname,
                NameBuff.ptr() as *mut c_char,
                ::core::mem::size_of::<[c_char; 4096]>(),
                false_0 != 0,
            );
            tv_list_append_string(list, NameBuff.ptr() as *mut c_char, -1 as ssize_t);
        }
        i += 1;
    }
    tv_list_set_lock(list, VAR_FIXED);
    set_vim_var_list(VV_ARGF, list);
}

pub(crate) unsafe extern "C" fn get_fname(mut _parmp: *mut mparm_T) -> *mut c_char {
    return alist_name(
        ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(0 as c_int as isize),
    );
}

pub(crate) unsafe extern "C" fn handle_quickfix(mut paramp: *mut mparm_T) {
    if (*paramp).edit_type == EDIT_QF as c_int {
        if !(*paramp).use_ef.is_null() {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*paramp).use_ef),
                    },
                },
                0 as c_int,
                SID_CARG,
            );
        }
        vim_snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE as size_t,
            b"cfile %s\0".as_ptr() as *const c_char,
            p_ef.get(),
        );
        if qf_init(
            ::core::ptr::null_mut::<win_T>(),
            p_ef.get(),
            p_efm.get(),
            true_0,
            IObuff.ptr() as *mut c_char,
            p_menc.get(),
        ) < 0 as c_int
        {
            msg_putchar('\n' as c_int);
            os_exit(3 as c_int);
        }
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"reading errorfile\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
}

pub(crate) unsafe extern "C" fn handle_tag(mut tagname: *mut c_char) {
    if !tagname.is_null() {
        swap_exists_did_quit.set(false_0 != 0);
        vim_snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE as size_t,
            b"ta %s\0".as_ptr() as *const c_char,
            tagname,
        );
        do_cmdline_cmd(IObuff.ptr() as *mut c_char);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"jumping to tag\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if swap_exists_did_quit.get() {
            ui_call_error_exit(1 as Integer);
            getout(1 as c_int);
        }
    }
}

pub(crate) unsafe extern "C" fn read_stdin() {
    swap_exists_action.set(SEA_DIALOG);
    no_wait_return.set(true_0);
    let mut save_msg_didany: bool = msg_didany.get();
    if !(*curbuf.get()).b_ffname.is_null() {
        let mut stdin_buf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            0 as linenr_T,
            BLN_LISTED as c_int,
        );
        if stdin_buf.is_null() {
            semsg(b"Failed to create buffer for stdin\0".as_ptr() as *const c_char);
            return;
        }
        let mut initial_buf_handle: handle_T = (*curbuf.get()).handle;
        set_curbuf(stdin_buf, 0 as c_int, false_0 != 0);
        readfile(
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as c_int as linenr_T,
            ::core::ptr::null_mut::<exarg_T>(),
            READ_NEW as c_int + READ_STDIN as c_int,
            true_0 != 0,
        );
        let mut stdin_buf_handle: handle_T = (*stdin_buf).handle;
        let mut stdin_buf_empty: bool = buf_is_empty(curbuf.get());
        let mut buf: [c_char; 100] = [0; 100];
        vim_snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 100]>(),
            b"silent! buffer %d\0".as_ptr() as *const c_char,
            initial_buf_handle,
        );
        do_cmdline_cmd(&raw mut buf as *mut c_char);
        if stdin_buf_empty {
            vim_snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 100]>(),
                b"silent! bwipeout! %d\0".as_ptr() as *const c_char,
                stdin_buf_handle,
            );
            do_cmdline_cmd(&raw mut buf as *mut c_char);
        }
    } else {
        set_buflisted(true_0);
        open_buffer(true_0 != 0, ::core::ptr::null_mut::<exarg_T>(), 0 as c_int);
        if buf_is_empty(curbuf.get()) as c_int != 0 && !(*curbuf.get()).b_next.is_null() {
            do_cmdline_cmd(b"silent! bnext\0".as_ptr() as *const c_char);
            do_cmdline_cmd(b"silent! bwipeout 1\0".as_ptr() as *const c_char);
        }
    }
    no_wait_return.set(false_0);
    msg_didany.set(save_msg_didany);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"reading stdin\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    check_swap_exists_action();
}

pub(crate) unsafe extern "C" fn create_windows(mut parmp: *mut mparm_T) {
    if (*parmp).window_count == -1 as c_int {
        (*parmp).window_count = 1 as c_int;
    }
    if (*parmp).window_count == 0 as c_int {
        (*parmp).window_count = (*global_alist.ptr()).al_ga.ga_len;
    }
    if (*parmp).window_count > 1 as c_int {
        if (*parmp).window_layout == 0 as c_int {
            (*parmp).window_layout = WIN_HOR as c_int;
        }
        if (*parmp).window_layout == WIN_TABS as c_int {
            (*parmp).window_count = make_tabpages((*parmp).window_count);
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"making tab pages\0".as_ptr() as *const c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else if (*firstwin.get()).w_next.is_null()
            || (*(*firstwin.get()).w_next).w_floating as c_int != 0
        {
            (*parmp).window_count = make_windows(
                (*parmp).window_count,
                (*parmp).window_layout == WIN_VER as c_int,
            );
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"making windows\0".as_ptr() as *const c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else {
            (*parmp).window_count = win_count();
        }
    } else {
        (*parmp).window_count = 1 as c_int;
    }
    if recoverymode.get() {
        msg_scroll.set(true_0);
        ml_recover(true_0 != 0);
        if (*curbuf.get()).b_ml.ml_mfp.is_null() {
            getout(1 as c_int);
        }
        do_modelines(0 as c_int);
    } else {
        let mut done: c_int = 0 as c_int;
        (*autocmd_no_enter.ptr()) += 1;
        (*autocmd_no_leave.ptr()) += 1;
        let mut dorewind: bool = true_0 != 0;
        loop {
            let c2rust_fresh0 = done;
            done = done + 1;
            if c2rust_fresh0 >= 1000 as c_int {
                break;
            }
            if dorewind {
                if (*parmp).window_layout == WIN_TABS as c_int {
                    goto_tabpage(1 as c_int);
                } else {
                    curwin.set(firstwin.get());
                }
            } else if (*parmp).window_layout == WIN_TABS as c_int {
                if (*curtab.get()).tp_next.is_null() {
                    break;
                }
                goto_tabpage(0 as c_int);
            } else {
                if (*curwin.get()).w_next.is_null() {
                    break;
                }
                curwin.set((*curwin.get()).w_next);
            }
            dorewind = false_0 != 0;
            curbuf.set((*curwin.get()).w_buffer);
            if (*curbuf.get()).b_ml.ml_mfp.is_null() {
                if p_fdls.get() >= 0 as OptInt {
                    (*curwin.get()).w_onebuf_opt.wo_fdl = p_fdls.get();
                }
                swap_exists_action.set(SEA_DIALOG);
                set_buflisted(true_0);
                open_buffer(false_0 != 0, ::core::ptr::null_mut::<exarg_T>(), 0 as c_int);
                if swap_exists_action.get() == SEA_QUIT {
                    if got_int.get() as c_int != 0 || only_one_window() as c_int != 0 {
                        did_emsg.set(false_0);
                        ui_call_error_exit(1 as Integer);
                        getout(1 as c_int);
                    }
                    setfname(
                        curbuf.get(),
                        ::core::ptr::null_mut::<c_char>(),
                        ::core::ptr::null_mut::<c_char>(),
                        false_0 != 0,
                    );
                    (*curwin.get()).w_arg_idx = -1 as c_int;
                    swap_exists_action.set(SEA_NONE);
                } else {
                    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
                }
                dorewind = true_0 != 0;
            }
            os_breakcheck();
            if !got_int.get() {
                continue;
            }
            vgetc();
            break;
        }
        if (*parmp).window_layout == WIN_TABS as c_int {
            goto_tabpage(1 as c_int);
        } else {
            curwin.set(firstwin.get());
        }
        curbuf.set((*curwin.get()).w_buffer);
        (*autocmd_no_enter.ptr()) -= 1;
        (*autocmd_no_leave.ptr()) -= 1;
    };
}

pub(crate) unsafe extern "C" fn edit_buffers(mut parmp: *mut mparm_T) {
    let mut arg_idx: c_int = 0;
    let mut advance: bool = true_0 != 0;
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut p_shm_save: *mut c_char = ::core::ptr::null_mut::<c_char>();
    (*autocmd_no_enter.ptr()) += 1;
    (*autocmd_no_leave.ptr()) += 1;
    if (*curwin.get()).w_arg_idx == -1 as c_int {
        win_close(curwin.get(), true_0 != 0, false_0 != 0);
        advance = false_0 != 0;
    }
    arg_idx = 1 as c_int;
    let mut i: c_int = 1 as c_int;
    while i < (*parmp).window_count {
        if (*curwin.get()).w_arg_idx == -1 as c_int {
            arg_idx += 1;
            win_close(curwin.get(), true_0 != 0, false_0 != 0);
            advance = false_0 != 0;
        } else {
            if advance {
                if (*parmp).window_layout == WIN_TABS as c_int {
                    if (*curtab.get()).tp_next.is_null() {
                        break;
                    }
                    goto_tabpage(0 as c_int);
                    if i == 1 as c_int {
                        let mut buf: [c_char; 100] = [0; 100];
                        p_shm_save = xstrdup(p_shm.get());
                        snprintf(
                            &raw mut buf as *mut c_char,
                            ::core::mem::size_of::<[c_char; 100]>(),
                            b"F%s\0".as_ptr() as *const c_char,
                            p_shm.get(),
                        );
                        set_option_value_give_err(
                            kOptShortmess,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: cstr_as_string(&raw mut buf as *mut c_char),
                                },
                            },
                            0 as c_int,
                        );
                    }
                } else {
                    if (*curwin.get()).w_next.is_null() {
                        break;
                    }
                    win_enter((*curwin.get()).w_next, false_0 != 0);
                }
            }
            advance = true_0 != 0;
            if curbuf.get() == (*firstwin.get()).w_buffer || (*curbuf.get()).b_ffname.is_null() {
                (*curwin.get()).w_arg_idx = arg_idx;
                swap_exists_did_quit.set(false_0 != 0);
                do_ecmd(
                    0 as c_int,
                    if arg_idx < (*global_alist.ptr()).al_ga.ga_len {
                        alist_name(
                            ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                                .offset(arg_idx as isize),
                        )
                    } else {
                        ::core::ptr::null_mut::<c_char>()
                    },
                    ::core::ptr::null_mut::<c_char>(),
                    ::core::ptr::null_mut::<exarg_T>(),
                    ECMD_LASTL as c_int as linenr_T,
                    ECMD_HIDE as c_int,
                    curwin.get(),
                );
                if swap_exists_did_quit.get() {
                    if got_int.get() as c_int != 0 || only_one_window() as c_int != 0 {
                        did_emsg.set(false_0);
                        ui_call_error_exit(1 as Integer);
                        getout(1 as c_int);
                    }
                    win_close(curwin.get(), true_0 != 0, false_0 != 0);
                    advance = false_0 != 0;
                }
                if arg_idx == (*global_alist.ptr()).al_ga.ga_len - 1 as c_int {
                    arg_had_last.set(true_0 != 0);
                }
                arg_idx += 1;
            }
            os_breakcheck();
            if got_int.get() {
                vgetc();
                break;
            }
        }
        i += 1;
    }
    if !p_shm_save.is_null() {
        set_option_value_give_err(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p_shm_save),
                },
            },
            0 as c_int,
        );
        xfree(p_shm_save as *mut c_void);
    }
    if (*parmp).window_layout == WIN_TABS as c_int {
        goto_tabpage(1 as c_int);
    }
    (*autocmd_no_enter.ptr()) -= 1;
    win = firstwin.get();
    while (*win).w_onebuf_opt.wo_pvw != 0 {
        win = (*win).w_next;
        if !win.is_null() {
            continue;
        }
        win = firstwin.get();
        break;
    }
    win_enter(win, false_0 != 0);
    (*autocmd_no_leave.ptr()) -= 1;
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"editing files in windows\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if (*parmp).window_count > 1 as c_int && (*parmp).window_layout != WIN_TABS as c_int {
        win_equal(curwin.get(), false_0 != 0, 'b' as c_int);
    }
}

pub(crate) unsafe extern "C" fn check_swap_exists_action() {
    if swap_exists_action.get() == SEA_QUIT {
        ui_call_error_exit(1 as Integer);
        getout(1 as c_int);
    }
    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
}
