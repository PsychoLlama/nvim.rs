//! Splitting, resizing, moving between and listing windows and tab
//! pages.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn current_win_nr(mut win: *const win_T) -> c_int {
    let mut nr: c_int = 0 as c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        nr += 1;
        if wp == win as *mut win_T {
            break;
        }
        wp = (*wp).w_next;
    }
    return nr;
}

pub(crate) unsafe extern "C" fn current_tab_nr(mut tab: *mut tabpage_T) -> c_int {
    let mut nr: c_int = 0 as c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        nr += 1;
        if tp == tab {
            break;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return nr;
}

pub(crate) unsafe extern "C" fn ex_wrongmodifier(mut eap: *mut exarg_T) {
    (*eap).errmsg = gettext(&raw const e_invcmd as *const c_char);
}

pub unsafe extern "C" fn ex_splitview(mut eap: *mut exarg_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let use_tab: bool = (*eap).cmdidx as c_int == CMD_tabedit as c_int
        || (*eap).cmdidx as c_int == CMD_tabfind as c_int
        || (*eap).cmdidx as c_int == CMD_tabnew as c_int;
    if bt_quickfix(curbuf.get()) as c_int != 0 && (*cmdmod.ptr()).cmod_tab == 0 as c_int {
        if (*eap).cmdidx as c_int == CMD_split as c_int {
            (*eap).cmdidx = CMD_new;
        }
        if (*eap).cmdidx as c_int == CMD_vsplit as c_int {
            (*eap).cmdidx = CMD_vnew;
        }
    }
    '_theend: {
        if (*eap).cmdidx as c_int == CMD_sfind as c_int
            || (*eap).cmdidx as c_int == CMD_tabfind as c_int
        {
            if *get_findfunc() as c_int != NUL {
                fname = findfunc_find_file(
                    (*eap).arg,
                    strlen((*eap).arg),
                    if (*eap).addr_count > 0 as c_int {
                        (*eap).line2 as c_int
                    } else {
                        1 as c_int
                    },
                );
            } else {
                let mut file_to_find: *mut c_char = ::core::ptr::null_mut::<c_char>();
                let mut search_ctx: *mut c_char = ::core::ptr::null_mut::<c_char>();
                fname = find_file_in_path(
                    (*eap).arg,
                    strlen((*eap).arg),
                    FNAME_MESS as c_int,
                    true_0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
                xfree(file_to_find as *mut c_void);
                vim_findfile_cleanup(search_ctx as *mut c_void);
            }
            if fname.is_null() {
                break '_theend;
            } else {
                (*eap).arg = fname;
            }
        }
        if use_tab {
            if !win_new_tabpage(
                if (*cmdmod.ptr()).cmod_tab != 0 as c_int {
                    (*cmdmod.ptr()).cmod_tab
                } else if (*eap).addr_count == 0 as c_int {
                    0 as c_int
                } else {
                    (*eap).line2 as c_int + 1 as c_int
                },
                (*eap).arg,
                true_0 != 0,
                ::core::ptr::null_mut::<*mut win_T>(),
            )
            .is_null()
            {
                do_exedit(eap, old_curwin);
                apply_autocmds(
                    EVENT_TABNEWENTERED,
                    ::core::ptr::null_mut::<c_char>(),
                    ::core::ptr::null_mut::<c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
                if curwin.get() != old_curwin
                    && win_valid(old_curwin) as c_int != 0
                    && (*old_curwin).w_buffer != curbuf.get()
                    && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as c_int == 0 as c_int
                {
                    (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as c_int;
                }
            }
        } else if win_split(
            if (*eap).addr_count > 0 as c_int {
                (*eap).line2 as c_int
            } else {
                0 as c_int
            },
            if *(*eap).cmd as c_int == 'v' as c_int {
                WSP_VERT as c_int
            } else {
                0 as c_int
            },
        ) != FAIL
        {
            if *(*eap).arg as c_int != NUL {
                (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
            } else {
                do_check_scrollbind(false_0 != 0);
            }
            do_exedit(eap, old_curwin);
        }
    }
    xfree(fname as *mut c_void);
}

pub unsafe extern "C" fn tabpage_new() {
    let mut ea: exarg_T = exarg {
        arg: b"\0".as_ptr() as *const c_char as *mut c_char,
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: b"tabn\0".as_ptr() as *const c_char as *mut c_char,
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_tabnew,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
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
    ex_splitview(&raw mut ea);
}

pub(crate) unsafe extern "C" fn ex_tabnext(mut eap: *mut exarg_T) {
    let mut tab_number: c_int = 0;
    match (*eap).cmdidx as c_int {
        458 | 466 => {
            goto_tabpage(1 as c_int);
        }
        460 => {
            goto_tabpage(9999 as c_int);
        }
        464 | 465 => {
            if !(*eap).arg.is_null() && *(*eap).arg as c_int != NUL {
                let mut p: *mut c_char = (*eap).arg;
                let mut p_save: *mut c_char = p;
                tab_number = getdigits(&raw mut p, false_0 != 0, 0 as intmax_t) as c_int;
                if p == p_save
                    || *p_save as c_int == '-' as c_int
                    || *p_save as c_int == '+' as c_int
                    || *p as c_int != NUL
                    || tab_number == 0 as c_int
                {
                    (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
                    return;
                }
            } else if (*eap).addr_count == 0 as c_int {
                tab_number = 1 as c_int;
            } else {
                tab_number = (*eap).line2 as c_int;
                if tab_number < 1 as c_int {
                    (*eap).errmsg = gettext(&raw const e_invrange as *const c_char);
                    return;
                }
            }
            goto_tabpage(-tab_number);
        }
        _ => {
            tab_number = get_tabpage_arg(eap);
            if (*eap).errmsg.is_null() {
                goto_tabpage(tab_number);
            }
        }
    };
}

pub(crate) unsafe extern "C" fn ex_tabmove(mut eap: *mut exarg_T) {
    let mut tab_number: c_int = get_tabpage_arg(eap);
    if (*eap).errmsg.is_null() {
        tabpage_move(tab_number);
    }
}

pub(crate) unsafe extern "C" fn ex_tabs(mut _eap: *mut exarg_T) {
    let mut tabcount: c_int = 1 as c_int;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
    msg_start();
    msg_scroll.set(true_0);
    let mut lastused_win: *mut win_T = if valid_tabpage(lastused_tabpage.get()) as c_int != 0 {
        (*lastused_tabpage.get()).tp_curwin
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if got_int.get() {
            break;
        }
        if msg_col.get() > 0 as c_int {
            msg_putchar('\n' as c_int);
        }
        let c2rust_fresh1 = tabcount;
        tabcount = tabcount + 1;
        vim_snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE as size_t,
            gettext(b"Tab page %d\0".as_ptr() as *const c_char),
            c2rust_fresh1,
        );
        msg_outtrans(IObuff.ptr() as *mut c_char, HLF_T as c_int, false_0 != 0);
        os_breakcheck();
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if got_int.get() {
                break;
            }
            if !(!(*wp).w_config.focusable || (*wp).w_config.hide as c_int != 0) {
                msg_putchar('\n' as c_int);
                msg_putchar(if wp == curwin.get() {
                    '>' as c_int
                } else if wp == lastused_win {
                    '#' as c_int
                } else {
                    ' ' as c_int
                });
                msg_putchar(' ' as c_int);
                msg_putchar(if bufIsChanged((*wp).w_buffer) as c_int != 0 {
                    '+' as c_int
                } else {
                    ' ' as c_int
                });
                msg_putchar(' ' as c_int);
                if !buf_spname((*wp).w_buffer).is_null() {
                    xstrlcpy(
                        IObuff.ptr() as *mut c_char,
                        buf_spname((*wp).w_buffer),
                        IOSIZE as size_t,
                    );
                } else {
                    home_replace(
                        (*wp).w_buffer,
                        (*(*wp).w_buffer).b_fname,
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        true_0 != 0,
                    );
                }
                msg_outtrans(IObuff.ptr() as *mut c_char, 0 as c_int, false_0 != 0);
                os_breakcheck();
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}

pub(crate) unsafe extern "C" fn ex_mode(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        must_redraw.set(UPD_CLEAR as c_int);
        ex_redraw(eap);
    } else {
        emsg(gettext(&raw const e_screenmode as *const c_char));
    };
}

pub(crate) unsafe extern "C" fn ex_resize(mut eap: *mut exarg_T) {
    let mut wp: *mut win_T = curwin.get();
    if (*eap).addr_count > 0 as c_int {
        let mut n: c_int = (*eap).line2 as c_int;
        wp = firstwin.get();
        while !(*wp).w_next.is_null() && {
            n -= 1;
            n > 0 as c_int
        } {
            wp = (*wp).w_next;
        }
    }
    let mut n_0: c_int = atol((*eap).arg) as c_int;
    if (*cmdmod.ptr()).cmod_split & WSP_VERT as c_int != 0 {
        if *(*eap).arg as c_int == '-' as c_int || *(*eap).arg as c_int == '+' as c_int {
            n_0 += (*wp).w_width;
        } else if n_0 == 0 as c_int && *(*eap).arg.offset(0 as c_int as isize) as c_int == NUL {
            n_0 = Columns.get();
        }
        win_setwidth_win(n_0, wp);
    } else {
        if *(*eap).arg as c_int == '-' as c_int || *(*eap).arg as c_int == '+' as c_int {
            n_0 += (*wp).w_height;
        } else if n_0 == 0 as c_int && *(*eap).arg.offset(0 as c_int as isize) as c_int == NUL {
            n_0 = Rows.get() - 1 as c_int;
        }
        win_setheight_win(n_0, wp);
    };
}

pub(crate) unsafe extern "C" fn ex_nogui(mut eap: *mut exarg_T) {
    (*eap).errmsg = gettext(b"E25: Nvim does not have a built-in GUI\0".as_ptr() as *const c_char);
}

pub(crate) unsafe extern "C" fn ex_popup(mut eap: *mut exarg_T) {
    pum_make_popup((*eap).arg, (*eap).forceit);
}

pub(crate) unsafe extern "C" fn ex_winsize(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    if !ascii_isdigit(*arg as c_int) {
        semsg(gettext(&raw const e_invarg2 as *const c_char), arg);
        return;
    }
    let mut w: c_int = getdigits_int(&raw mut arg, false_0 != 0, 10 as c_int);
    arg = skipwhite(arg);
    let mut p: *mut c_char = arg;
    let mut h: c_int = getdigits_int(&raw mut arg, false_0 != 0, 10 as c_int);
    if *p as c_int != NUL && *arg as c_int == NUL {
        screen_resize(w, h);
    } else {
        emsg(gettext(
            b"E465: :winsize requires two number arguments\0".as_ptr() as *const c_char,
        ));
    };
}

pub(crate) unsafe extern "C" fn ex_wincmd(mut eap: *mut exarg_T) {
    let mut xchar: c_int = NUL;
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *(*eap).arg as c_int == 'g' as c_int || *(*eap).arg as c_int == Ctrl_G {
        if *(*eap).arg.offset(1 as c_int as isize) as c_int == NUL {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }
        xchar = *(*eap).arg.offset(1 as c_int as isize) as uint8_t as c_int;
        p = (*eap).arg.offset(2 as c_int as isize);
    } else {
        p = (*eap).arg.offset(1 as c_int as isize);
    }
    (*eap).nextcmd = check_nextcmd(p);
    p = skipwhite(p);
    if *p as c_int != NUL && *p as c_int != '"' as c_int && (*eap).nextcmd.is_null() {
        emsg(gettext(&raw const e_invarg as *const c_char));
    } else if (*eap).skip == 0 {
        postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
        postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
        do_window(
            *(*eap).arg as c_int,
            if (*eap).addr_count > 0 as c_int {
                (*eap).line2 as c_int
            } else {
                0 as c_int
            },
            xchar,
        );
        postponed_split_flags.set(0 as c_int);
        postponed_split_tab.set(0 as c_int);
    }
}

pub(crate) unsafe extern "C" fn ex_psearch(mut eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    ex_findpat(eap);
    g_do_tagpreview.set(0 as c_int);
}

pub(crate) unsafe extern "C" fn ex_pedit(mut eap: *mut exarg_T) {
    let mut curwin_save: *mut win_T = curwin.get();
    prepare_preview_window();
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
    back_to_current_window(curwin_save);
}

pub(crate) unsafe extern "C" fn ex_pbuffer(mut eap: *mut exarg_T) {
    let mut curwin_save: *mut win_T = curwin.get();
    prepare_preview_window();
    do_exbuffer(eap);
    back_to_current_window(curwin_save);
}

pub(crate) unsafe extern "C" fn prepare_preview_window() {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    prepare_tagpreview(true_0 != 0);
}

pub(crate) unsafe extern "C" fn back_to_current_window(mut curwin_save: *mut win_T) {
    if curwin.get() != curwin_save && win_valid(curwin_save) as c_int != 0 {
        validate_cursor(curwin.get());
        redraw_later(curwin.get(), UPD_VALID as c_int);
        win_enter(curwin_save, true_0 != 0);
    }
    g_do_tagpreview.set(0 as c_int);
}
