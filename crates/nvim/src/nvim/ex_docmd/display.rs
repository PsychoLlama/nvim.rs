//! Commands about what is on the screen rather than in the buffer:
//! redrawing, `:redir`, highlighting and the digraph table.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ex_colorscheme(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        let mut expr: *mut c_char = xstrdup(b"g:colors_name\0".as_ptr() as *const c_char);
        (*emsg_off.ptr()) += 1;
        let mut p: *mut c_char = eval_to_string(expr, false_0 != 0, false_0 != 0);
        (*emsg_off.ptr()) -= 1;
        xfree(expr as *mut c_void);
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
        if !p.is_null() {
            msg(p, 0 as c_int);
            xfree(p as *mut c_void);
        } else {
            msg(b"default\0".as_ptr() as *const c_char, 0 as c_int);
        }
    } else if load_colors((*eap).arg) == FAIL {
        semsg(
            gettext(b"E185: Cannot find color scheme '%s'\0".as_ptr() as *const c_char),
            (*eap).arg,
        );
    }
}

pub(crate) unsafe extern "C" fn ex_highlight(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL
        && *(*eap).cmd.offset(2 as c_int as isize) as c_int == '!' as c_int
    {
        msg(
            gettext(b"Greetings, Vim user!\0".as_ptr() as *const c_char),
            0 as c_int,
        );
    }
    do_highlight((*eap).arg, (*eap).forceit != 0, false_0 != 0);
}

pub(crate) unsafe extern "C" fn ex_redir(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    if strcasecmp(
        (*eap).arg,
        b"END\0".as_ptr() as *const c_char as *mut c_char,
    ) == 0 as c_int
    {
        close_redir();
    } else if *arg as c_int == '>' as c_int {
        arg = arg.offset(1);
        let mut mode: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if *arg as c_int == '>' as c_int {
            arg = arg.offset(1);
            mode = b"a\0".as_ptr() as *const c_char as *mut c_char;
        } else {
            mode = b"w\0".as_ptr() as *const c_char as *mut c_char;
        }
        arg = skipwhite(arg);
        close_redir();
        let mut fname: *mut c_char = expand_env_save(arg);
        if fname.is_null() {
            return;
        }
        redir_fd.set(open_exfile(fname, (*eap).forceit, mode));
        xfree(fname as *mut c_void);
    } else if *arg as c_int == '@' as c_int {
        close_redir();
        arg = arg.offset(1);
        if valid_yank_reg(*arg as c_int, true_0 != 0) as c_int != 0 && *arg as c_int != '_' as c_int
        {
            let c2rust_fresh15 = arg;
            arg = arg.offset(1);
            redir_reg.set(*c2rust_fresh15 as uint8_t as c_int);
            if *arg as c_int == '>' as c_int
                && *arg.offset(1 as c_int as isize) as c_int == '>' as c_int
            {
                arg = arg.offset(2 as c_int as isize);
            } else {
                if *arg as c_int == '>' as c_int {
                    arg = arg.offset(1);
                }
                if *arg as c_int == NUL
                    && *(*__ctype_b_loc()).offset(redir_reg.get() as isize) as c_int
                        & _ISupper as c_int as c_ushort as c_int
                        == 0
                {
                    write_reg_contents(
                        redir_reg.get(),
                        b"\0".as_ptr() as *const c_char,
                        0 as ssize_t,
                        false_0,
                    );
                }
            }
        }
        if *arg as c_int != NUL {
            redir_reg.set(0 as c_int);
            semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
        }
    } else if *arg as c_int == '=' as c_int
        && *arg.offset(1 as c_int as isize) as c_int == '>' as c_int
    {
        let mut append: bool = false;
        close_redir();
        arg = arg.offset(2 as c_int as isize);
        if *arg as c_int == '>' as c_int {
            arg = arg.offset(1);
            append = true_0 != 0;
        } else {
            append = false_0 != 0;
        }
        if var_redir_start(skipwhite(arg), append) == OK {
            redir_vname.set(true_0 != 0);
        }
    } else {
        semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
    }
    if !(*redir_fd.ptr()).is_null() || redir_reg.get() != 0 || redir_vname.get() as c_int != 0 {
        redir_off.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn ex_redraw(mut eap: *mut exarg_T) {
    if cmdpreview.get() {
        return;
    }
    let mut r: c_int = RedrawingDisabled.get();
    let mut p: c_int = p_lz.get();
    RedrawingDisabled.set(0 as c_int);
    p_lz.set(false_0);
    validate_cursor(curwin.get());
    update_topline(curwin.get());
    if (*eap).forceit != 0 {
        redraw_all_later(UPD_NOT_VALID as c_int);
        redraw_cmdline.set(true_0 != 0);
    } else if VIsual_active.get() {
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    update_screen();
    if need_maketitle.get() {
        maketitle();
    }
    RedrawingDisabled.set(r);
    p_lz.set(p);
    msg_didout.set(false_0 != 0);
    msg_col.set(0 as c_int);
    need_wait_return.set(false_0 != 0);
    ui_flush();
}

pub(crate) unsafe extern "C" fn ex_redrawstatus(mut eap: *mut exarg_T) {
    if cmdpreview.get() {
        return;
    }
    let mut r: c_int = RedrawingDisabled.get();
    let mut p: c_int = p_lz.get();
    if (*eap).forceit != 0 {
        status_redraw_all();
    } else {
        status_redraw_curbuf();
    }
    RedrawingDisabled.set(0 as c_int);
    p_lz.set(false_0);
    if State.get() & MODE_CMDLINE as c_int != 0 {
        redraw_statuslines();
    } else {
        if VIsual_active.get() {
            redraw_curbuf_later(UPD_INVERTED as c_int);
        }
        update_screen();
    }
    RedrawingDisabled.set(r);
    p_lz.set(p);
    ui_flush();
}

pub(crate) unsafe extern "C" fn ex_redrawtabline(mut _eap: *mut exarg_T) {
    let r: c_int = RedrawingDisabled.get();
    let p: c_int = p_lz.get();
    RedrawingDisabled.set(0 as c_int);
    p_lz.set(false_0);
    draw_tabline();
    RedrawingDisabled.set(r);
    p_lz.set(p);
    ui_flush();
}

pub(crate) unsafe extern "C" fn close_redir() {
    if !(*redir_fd.ptr()).is_null() {
        fclose(redir_fd.get());
        redir_fd.set(::core::ptr::null_mut::<FILE>());
    }
    redir_reg.set(0 as c_int);
    if redir_vname.get() {
        var_redir_stop();
        redir_vname.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn ex_digraphs(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int != NUL {
        putdigraph(::core::ffi::CStr::from_ptr((*eap).arg).to_bytes());
    } else {
        listdigraphs((*eap).forceit != 0);
    };
}

pub unsafe extern "C" fn set_no_hlsearch(mut flag: bool) {
    no_hlsearch.set(flag);
    set_vim_var_nr(
        VV_HLSEARCH,
        (!no_hlsearch.get() && p_hls.get() != 0) as c_int as varnumber_T,
    );
}

pub(crate) unsafe extern "C" fn ex_nohlsearch(mut _eap: *mut exarg_T) {
    set_no_hlsearch(true_0 != 0);
    redraw_all_later(UPD_SOME_VALID as c_int);
}

pub unsafe extern "C" fn get_pressedreturn() -> bool {
    return ex_pressedreturn.get();
}

pub unsafe extern "C" fn set_pressedreturn(mut val: bool) {
    ex_pressedreturn.set(val);
}
