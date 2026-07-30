//! What surrounds a `do_cmdline` call: the state it saves and
//! restores, the line getter it reads through, the loop line store `:while` and
//! `:for` replay from, and Ex mode.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn save_dbg_stuff(mut dsp: *mut dbg_stuff) {
    (*dsp).trylevel = trylevel.get();
    trylevel.set(0 as c_int);
    (*dsp).force_abort = force_abort.get() as c_int;
    force_abort.set(false_0 != 0);
    (*dsp).caught_stack = caught_stack.get();
    caught_stack.set(::core::ptr::null_mut::<except_T>());
    (*dsp).vv_exception = v_exception(::core::ptr::null_mut::<c_char>());
    (*dsp).vv_throwpoint = v_throwpoint(::core::ptr::null_mut::<c_char>());
    (*dsp).did_emsg = did_emsg.get();
    did_emsg.set(false_0);
    (*dsp).got_int = got_int.get() as c_int;
    got_int.set(false_0 != 0);
    (*dsp).did_throw = did_throw.get();
    did_throw.set(false_0 != 0);
    (*dsp).need_rethrow = need_rethrow.get() as c_int;
    need_rethrow.set(false_0 != 0);
    (*dsp).check_cstack = check_cstack.get() as c_int;
    check_cstack.set(false_0 != 0);
    (*dsp).current_exception = current_exception.get();
    current_exception.set(::core::ptr::null_mut::<except_T>());
}

pub(crate) unsafe extern "C" fn restore_dbg_stuff(mut dsp: *mut dbg_stuff) {
    suppress_errthrow.set(false_0 != 0);
    trylevel.set((*dsp).trylevel);
    force_abort.set((*dsp).force_abort != 0);
    caught_stack.set((*dsp).caught_stack);
    v_exception((*dsp).vv_exception);
    v_throwpoint((*dsp).vv_throwpoint);
    did_emsg.set((*dsp).did_emsg);
    got_int.set((*dsp).got_int != 0);
    did_throw.set((*dsp).did_throw);
    need_rethrow.set((*dsp).need_rethrow != 0);
    check_cstack.set((*dsp).check_cstack != 0);
    current_exception.set((*dsp).current_exception);
}

pub unsafe extern "C" fn do_exmode() {
    exmode_active.set(true_0 != 0);
    State.set(MODE_NORMAL as c_int);
    may_trigger_modechanged();
    if global_busy.get() != 0 {
        return;
    }
    let mut save_msg_scroll: c_int = msg_scroll.get();
    (*RedrawingDisabled.ptr()) += 1;
    (*no_wait_return.ptr()) += 1;
    msg(
        gettext(
            b"Entering Ex mode.  Type \"visual\" to go to Normal mode.\0".as_ptr() as *const c_char,
        ),
        0 as c_int,
    );
    while exmode_active.get() {
        if ex_normal_busy.get() > 0 as c_int && (*typebuf.ptr()).tb_len == 0 as c_int {
            exmode_active.set(false_0 != 0);
            break;
        } else {
            msg_scroll.set(true_0);
            need_wait_return.set(false_0 != 0);
            ex_pressedreturn.set(false_0 != 0);
            ex_no_reprint.set(false_0 != 0);
            let mut changedtick: varnumber_T = buf_get_changedtick(curbuf.get());
            let mut prev_msg_row: c_int = msg_row.get();
            let mut prev_line: linenr_T = (*curwin.get()).w_cursor.lnum;
            cmdline_row.set(msg_row.get());
            do_cmdline(
                ::core::ptr::null_mut::<c_char>(),
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
                NULL_1,
                0 as c_int,
            );
            lines_left.set(Rows.get() - 1 as c_int);
            if (prev_line != (*curwin.get()).w_cursor.lnum
                || changedtick != buf_get_changedtick(curbuf.get()))
                && !ex_no_reprint.get()
            {
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    emsg(gettext(&raw const e_empty_buffer as *const c_char));
                } else {
                    if ex_pressedreturn.get() {
                        msg_scroll_flush();
                        msg_row.set(prev_msg_row);
                        if prev_msg_row == Rows.get() - 1 as c_int {
                            (*msg_row.ptr()) -= 1;
                        }
                    }
                    msg_col.set(0 as c_int);
                    print_line_no_prefix((*curwin.get()).w_cursor.lnum, false_0 != 0, false_0 != 0);
                    msg_clr_eos();
                }
            } else if ex_pressedreturn.get() as c_int != 0 && !ex_no_reprint.get() {
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    emsg(gettext(&raw const e_empty_buffer as *const c_char));
                } else {
                    emsg(gettext(b"E501: At end-of-file\0".as_ptr() as *const c_char));
                }
            }
        }
    }
    (*RedrawingDisabled.ptr()) -= 1;
    (*no_wait_return.ptr()) -= 1;
    redraw_all_later(UPD_NOT_VALID as c_int);
    update_screen();
    need_wait_return.set(false_0 != 0);
    msg_scroll.set(save_msg_scroll);
}

pub(crate) unsafe extern "C" fn msg_verbose_cmd(mut lnum: linenr_T, mut cmd: *mut c_char) {
    (*no_wait_return.ptr()) += 1;
    verbose_enter_scroll();
    if lnum == 0 as linenr_T {
        smsg(
            0 as c_int,
            gettext(b"Executing: %s\0".as_ptr() as *const c_char),
            cmd,
        );
    } else {
        smsg(
            0 as c_int,
            gettext(b"line %d: %s\0".as_ptr() as *const c_char),
            lnum,
            cmd,
        );
    }
    if msg_silent.get() == 0 as c_int {
        msg_puts(b"\n\0".as_ptr() as *const c_char);
    }
    verbose_leave_scroll();
    (*no_wait_return.ptr()) -= 1;
}

pub(crate) unsafe extern "C" fn do_cmdline_start() -> c_int {
    '_c2rust_label: {
        if cmdline_call_depth.get() >= 0 as c_int {
        } else {
            __assert_fail(
                b"cmdline_call_depth >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                364 as c_uint,
                b"int do_cmdline_start(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    if cmdline_call_depth.get() >= 200 as c_int && cmdline_call_depth.get() as OptInt >= p_mfd.get()
    {
        return FAIL;
    }
    (*cmdline_call_depth.ptr()) += 1;
    crate::src::nvim::clipboard::start_batch_changes();
    return OK;
}

pub(crate) unsafe extern "C" fn do_cmdline_end() {
    (*cmdline_call_depth.ptr()) -= 1;
    '_c2rust_label: {
        if cmdline_call_depth.get() >= 0 as c_int {
        } else {
            __assert_fail(
                b"cmdline_call_depth >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                380 as c_uint,
                b"void do_cmdline_end(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    crate::src::nvim::clipboard::end_batch_changes();
}

pub unsafe extern "C" fn handle_did_throw() {
    '_c2rust_label: {
        if !(*current_exception.ptr()).is_null() {
        } else {
            __assert_fail(
                b"current_exception != NULL\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                974 as c_uint,
                b"void handle_did_throw(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut messages: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    match (*current_exception.get()).type_0 as c_uint {
        0 => {
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                gettext(b"E605: Exception not caught: %s\0".as_ptr() as *const c_char),
                (*current_exception.get()).value,
            );
            p = xstrdup(IObuff.ptr() as *mut c_char);
        }
        1 => {
            messages = (*current_exception.get()).messages;
            (*current_exception.get()).messages = ::core::ptr::null_mut::<msglist_T>();
        }
        2 | _ => {}
    }
    estack_push(
        ETYPE_EXCEPT,
        (*current_exception.get()).throw_name,
        (*current_exception.get()).throw_lnum,
    );
    (*current_exception.get()).throw_name = ::core::ptr::null_mut::<c_char>();
    discard_current_exception();
    if emsg_silent.get() == 0 as c_int {
        suppress_errthrow.set(true_0 != 0);
        force_abort.set(true_0 != 0);
    }
    if !messages.is_null() {
        loop {
            let mut next: *mut msglist_T = (*messages).next;
            emsg_multiline(
                (*messages).msg,
                b"emsg\0".as_ptr() as *const c_char,
                HLF_E as c_int,
                (*messages).multiline,
            );
            xfree((*messages).msg as *mut c_void);
            xfree((*messages).sfile as *mut c_void);
            xfree(messages as *mut c_void);
            messages = next;
            if messages.is_null() {
                break;
            }
        }
    } else if !p.is_null() {
        emsg(p);
        xfree(p as *mut c_void);
    }
    xfree(
        (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
        .es_name as *mut c_void,
    );
    estack_pop();
}

pub(crate) unsafe extern "C" fn get_loop_line(
    mut c: c_int,
    mut cookie: *mut c_void,
    mut indent: c_int,
    mut do_concat: bool,
) -> *mut c_char {
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    if (*cp).current_line + 1 as c_int >= (*(*cp).lines_gap).ga_len {
        if (*cp).repeating != 0 {
            return ::core::ptr::null_mut::<c_char>();
        }
        let mut line: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if (*cp).lc_getline.is_none() {
            line = getcmdline(c, 0 as c_int, indent, do_concat);
        } else {
            line = (*cp).lc_getline.expect("non-null function pointer")(
                c,
                (*cp).cookie,
                indent,
                do_concat,
            );
        }
        if !line.is_null() {
            store_loop_line((*cp).lines_gap, line);
            (*cp).current_line += 1;
        }
        return line;
    }
    KeyTyped.set(false_0 != 0);
    (*cp).current_line += 1;
    let mut wp: *mut wcmd_T =
        ((*(*cp).lines_gap).ga_data as *mut wcmd_T).offset((*cp).current_line as isize);
    (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
    .es_lnum = (*wp).lnum;
    return xstrdup((*wp).line);
}

pub(crate) unsafe extern "C" fn store_loop_line(mut gap: *mut garray_T, mut line: *mut c_char) {
    let mut p: *mut wcmd_T =
        ga_append_via_ptr(gap, ::core::mem::size_of::<wcmd_T>()) as *mut wcmd_T;
    (*p).line = xstrdup(line);
    (*p).lnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
    .es_lnum;
}

pub(crate) fn line_getter_eq(a: LineGetter, b: LineGetter) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => ::core::ptr::fn_addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

pub unsafe extern "C" fn getline_equal(
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
    mut func: LineGetter,
) -> bool {
    let mut gp: LineGetter = fgetline;
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    while line_getter_eq(gp, Some(get_loop_line)) {
        gp = (*cp).lc_getline;
        cp = (*cp).cookie as *mut loop_cookie;
    }
    return line_getter_eq(gp, func);
}

pub unsafe extern "C" fn getline_cookie(
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
) -> *mut c_void {
    let mut gp: LineGetter = fgetline;
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    while line_getter_eq(gp, Some(get_loop_line)) {
        gp = (*cp).lc_getline;
        cp = (*cp).cookie as *mut loop_cookie;
    }
    return cp as *mut c_void;
}

pub unsafe extern "C" fn ex_errmsg(msg_0: *const c_char, arg: *const c_char) -> *mut c_char {
    vim_snprintf(
        ex_error_buf.ptr() as *mut c_char,
        MSG_BUF_LEN as size_t,
        gettext(msg_0),
        arg,
    );
    return ex_error_buf.ptr() as *mut c_char;
}

pub unsafe extern "C" fn not_exiting(mut save_exiting: bool) {
    exiting.set(save_exiting);
    set_vim_var_string(
        VV_EXITREASON,
        ::core::ptr::null::<c_char>(),
        -1 as ptrdiff_t,
    );
}
