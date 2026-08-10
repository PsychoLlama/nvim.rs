//! Handing buffer text to a shell -- `:!cmd`, `:range!cmd` and `:shell`.
//!
//! `do_bang` is the command-line half: it expands `!` to the previous command
//! (`prevcmd`), `%`/`#` to file names, and decides whether this is a filter (a
//! range was given) or a plain `:!`.  `do_filter` is the buffer half: write the
//! range to a temp file, run the command with the file redirected in and its
//! output redirected out, read the output back over the range, and fix the
//! cursor.  `make_filter_cmd` and `append_redir` build that shell line from
//! 'shell', 'shellredir' and 'shellpipe'; `print_line` is `:print`'s and
//! `:number`'s output, shared with `:global`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

static prevcmd: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

unsafe extern "C" fn prevcmd_is_set() -> ::core::ffi::c_int {
    unsafe {
        if (*prevcmd.ptr()).is_null() {
            emsg(gettext(&raw const e_noprev as *const ::core::ffi::c_char));
            return false_0;
        }
        return true_0;
    }
}

pub unsafe extern "C" fn do_bang(
    mut addr_count: ::core::ffi::c_int,
    mut eap: *mut exarg_T,
    mut forceit: bool,
    mut do_in: bool,
    mut do_out: bool,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut line1: linenr_T = (*eap).line1;
        let mut line2: linenr_T = (*eap).line2;
        let mut newcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut free_newcmd: bool = false_0 != 0;
        let mut scroll_save: ::core::ffi::c_int = msg_scroll.get();
        if check_secure() {
            return;
        }
        if addr_count == 0 as ::core::ffi::c_int {
            msg_scroll.set(false_0);
            autowrite_all();
            msg_scroll.set(scroll_save);
        }
        let mut ins_prevcmd: bool = forceit;
        let mut trailarg: *mut ::core::ffi::c_char = skipwhite(arg);
        loop {
            let mut len: size_t = strlen(trailarg).wrapping_add(1 as size_t);
            if !newcmd.is_null() {
                len = len.wrapping_add(strlen(newcmd));
            }
            if ins_prevcmd {
                if prevcmd_is_set() == 0 {
                    xfree(newcmd as *mut ::core::ffi::c_void);
                    return;
                }
                len = len.wrapping_add(strlen(prevcmd.get()));
            }
            let mut t: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
            *t = NUL as ::core::ffi::c_char;
            if !newcmd.is_null() {
                strcat(t, newcmd);
            }
            if ins_prevcmd {
                strcat(t, prevcmd.get());
            }
            let mut p: *mut ::core::ffi::c_char = t.add(strlen(t));
            strcat(t, trailarg);
            xfree(newcmd as *mut ::core::ffi::c_void);
            newcmd = t;
            trailarg = ::core::ptr::null_mut::<::core::ffi::c_char>();
            while *p != 0 {
                if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                    if p > newcmd
                        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                    {
                        memmove(
                            p.offset(-(1 as ::core::ffi::c_int as isize))
                                as *mut ::core::ffi::c_void,
                            p as *const ::core::ffi::c_void,
                            strlen(p).wrapping_add(1 as size_t),
                        );
                    } else {
                        trailarg = p;
                        let c2rust_fresh4 = trailarg;
                        trailarg = trailarg.offset(1);
                        *c2rust_fresh4 = NUL as ::core::ffi::c_char;
                        ins_prevcmd = true_0 != 0;
                        break;
                    }
                }
                p = p.offset(1);
            }
            if trailarg.is_null() {
                break;
            }
        }
        if strlen(newcmd) > 0 as size_t {
            xfree(prevcmd.get() as *mut ::core::ffi::c_void);
            prevcmd.set(newcmd);
        } else {
            free_newcmd = true_0 != 0;
        }
        '_theend: {
            if bangredo.get() {
                if prevcmd_is_set() == 0 {
                    break '_theend;
                } else {
                    let mut cmd: *mut ::core::ffi::c_char =
                        vim_strsave_escaped(prevcmd.get(), c"%#".as_ptr());
                    AppendToRedobuffLit(cmd, -1 as ::core::ffi::c_int);
                    xfree(cmd as *mut ::core::ffi::c_void);
                    AppendToRedobuff(c"\n".as_ptr());
                    bangredo.set(false_0 != 0);
                }
            }
            if *p_shq.get() as ::core::ffi::c_int != NUL {
                if free_newcmd {
                    xfree(newcmd as *mut ::core::ffi::c_void);
                }
                newcmd = xmalloc(
                    strlen(prevcmd.get())
                        .wrapping_add((2 as size_t).wrapping_mul(strlen(p_shq.get())))
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                strcpy(newcmd, p_shq.get());
                strcat(newcmd, prevcmd.get());
                strcat(newcmd, p_shq.get());
                free_newcmd = true_0 != 0;
            }
            if addr_count == 0 as ::core::ffi::c_int {
                msg_start();
                msg_ext_set_kind(c"shell_cmd".as_ptr());
                msg_putchar(':' as ::core::ffi::c_int);
                msg_putchar('!' as ::core::ffi::c_int);
                msg_outtrans(newcmd, 0 as ::core::ffi::c_int, false_0 != 0);
                msg_clr_eos();
                ui_cursor_goto(msg_row.get(), msg_col.get());
                do_shell(newcmd, 0 as ::core::ffi::c_int);
            } else {
                do_filter(line1, line2, eap, newcmd, do_in, do_out);
                apply_autocmds(
                    EVENT_SHELLFILTERPOST,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
            }
        }
        if free_newcmd {
            xfree(newcmd as *mut ::core::ffi::c_void);
        }
    }
}

unsafe extern "C" fn do_filter(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut eap: *mut exarg_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut do_in: bool,
    mut do_out: bool,
) {
    unsafe {
        let mut read_linecount: linenr_T = 0;
        let mut cmd_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut itmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut otmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut old_curbuf: *mut buf_T = curbuf.get();
        let mut shell_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let orig_start: pos_T = (*curbuf.get()).b_op_start;
        let orig_end: pos_T = (*curbuf.get()).b_op_end;
        let stmp: ::core::ffi::c_int = p_stmp.get();
        if *cmd as ::core::ffi::c_int == NUL {
            return;
        }
        let save_cmod_flags: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_flags;
        (*cmdmod.ptr()).cmod_flags &= !(CMOD_LOCKMARKS as ::core::ffi::c_int);
        let mut cursor_save: pos_T = (*curwin.get()).w_cursor;
        let mut linecount: linenr_T = line2 - line1 + 1 as linenr_T;
        (*curwin.get()).w_cursor.lnum = line1;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        changed_line_abv_curs();
        invalidate_botline_win(curwin.get());
        if do_out {
            shell_flags |= kShellOptDoOut as ::core::ffi::c_int;
        }
        '_filterend: {
            if !do_in && do_out as ::core::ffi::c_int != 0 && stmp == 0 {
                shell_flags |= kShellOptRead as ::core::ffi::c_int;
                (*curwin.get()).w_cursor.lnum = line2;
            } else if do_in as ::core::ffi::c_int != 0 && !do_out && stmp == 0 {
                shell_flags |= kShellOptWrite as ::core::ffi::c_int;
                (*curbuf.get()).b_op_start.lnum = line1;
                (*curbuf.get()).b_op_end.lnum = line2;
            } else if do_in as ::core::ffi::c_int != 0
                && do_out as ::core::ffi::c_int != 0
                && stmp == 0
            {
                shell_flags |=
                    kShellOptRead as ::core::ffi::c_int | kShellOptWrite as ::core::ffi::c_int;
                (*curbuf.get()).b_op_start.lnum = line1;
                (*curbuf.get()).b_op_end.lnum = line2;
                (*curwin.get()).w_cursor.lnum = line2;
            } else if do_in as ::core::ffi::c_int != 0 && {
                itmp = vim_tempname();
                itmp.is_null()
            } || do_out as ::core::ffi::c_int != 0 && {
                otmp = vim_tempname();
                otmp.is_null()
            } {
                emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
                break '_filterend;
            }
            (*no_wait_return.ptr()) += 1;
            if !itmp.is_null()
                && buf_write(
                    curbuf.get(),
                    itmp,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    line1,
                    line2,
                    eap,
                    WriteRequest::filter(),
                ) == FAIL
            {
                if !ui_has(kUIMessages) {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                (*no_wait_return.ptr()) -= 1;
                if !aborting() {
                    semsg_c!(gettext(c"E482: Can't create file %s".as_ptr()), itmp,);
                }
            } else if curbuf.get() == old_curbuf {
                if !do_out {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                cmd_buf = make_filter_cmd(cmd, itmp, otmp, do_in);
                ui_cursor_goto(
                    Rows.get() - 1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                '_error: {
                    if do_out {
                        if u_save(line2, line2 + 1 as linenr_T) == FAIL {
                            xfree(cmd_buf as *mut ::core::ffi::c_void);
                            break '_error;
                        } else {
                            redraw_curbuf_later(UPD_VALID);
                        }
                    }
                    read_linecount = (*curbuf.get()).b_ml.ml_line_count;
                    call_shell(
                        cmd_buf,
                        kShellOptFilter as ::core::ffi::c_int | shell_flags,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    xfree(cmd_buf as *mut ::core::ffi::c_void);
                    did_check_timestamps.set(false_0 != 0);
                    need_check_timestamps.set(true_0 != 0);
                    os_breakcheck();
                    got_int.set(false_0 != 0);
                    if do_out {
                        if !otmp.is_null() {
                            if readfile(
                                otmp,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                line2,
                                0 as linenr_T,
                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                eap,
                                READ_FILTER as ::core::ffi::c_int,
                                false_0 != 0,
                            ) != OK
                            {
                                if !aborting() {
                                    msg_putchar('\n' as ::core::ffi::c_int);
                                    semsg_c!(
                                        gettext(
                                            &raw const e_cant_read_file_str
                                                as *const ::core::ffi::c_char,
                                        ),
                                        otmp,
                                    );
                                }
                                break '_error;
                            } else if curbuf.get() != old_curbuf {
                                break '_filterend;
                            }
                        }
                        read_linecount = (*curbuf.get()).b_ml.ml_line_count - read_linecount;
                        if shell_flags & kShellOptRead as ::core::ffi::c_int != 0 {
                            (*curbuf.get()).b_op_start.lnum = line2 + 1 as linenr_T;
                            (*curbuf.get()).b_op_end.lnum = (*curwin.get()).w_cursor.lnum;
                            appended_lines_mark(line2, read_linecount as ::core::ffi::c_int);
                        }
                        if do_in {
                            if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int
                                != 0
                                || vim_strchr(p_cpo.get(), CPO_REMMARK).is_null()
                            {
                                if read_linecount >= linecount {
                                    mark_adjust(
                                        line1,
                                        line2,
                                        linecount,
                                        0 as linenr_T,
                                        kExtmarkNOOP,
                                    );
                                } else {
                                    mark_adjust(
                                        line1,
                                        line1 + read_linecount - 1 as linenr_T,
                                        linecount,
                                        0 as linenr_T,
                                        kExtmarkNOOP,
                                    );
                                    mark_adjust(
                                        line1 + read_linecount,
                                        line2,
                                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                                        0 as linenr_T,
                                        kExtmarkNOOP,
                                    );
                                }
                            }
                            (*curwin.get()).w_cursor.lnum = line1;
                            del_lines(linecount, true_0 != 0);
                            (*curbuf.get()).b_op_start.lnum -= linecount;
                            (*curbuf.get()).b_op_end.lnum -= linecount;
                            write_lnum_adjust(-linecount);
                            foldUpdate(
                                curwin.get(),
                                (*curbuf.get()).b_op_start.lnum,
                                (*curbuf.get()).b_op_end.lnum,
                            );
                        } else {
                            linecount = (*curbuf.get()).b_op_end.lnum
                                - (*curbuf.get()).b_op_start.lnum
                                + 1 as linenr_T;
                            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_op_end.lnum;
                        }
                        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                        (*no_wait_return.ptr()) -= 1;
                        if linecount as OptInt > p_report.get() {
                            if do_in {
                                vim_snprintf(
                                    msg_buf.ptr() as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 480]>(),
                                    gettext(c"%ld lines filtered".as_ptr()),
                                    linecount as int64_t,
                                );
                                if msg(
                                    msg_buf.ptr() as *mut ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0
                                    && msg_scroll.get() == 0
                                {
                                    set_keep_msg(
                                        msg_buf.ptr() as *mut ::core::ffi::c_char,
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                            } else {
                                msgmore(linecount as ::core::ffi::c_int);
                            }
                        }
                        break '_filterend;
                    }
                }
                (*curwin.get()).w_cursor = cursor_save;
                (*no_wait_return.ptr()) -= 1;
                wait_return(false_0);
            }
        }
        (*cmdmod.ptr()).cmod_flags = save_cmod_flags;
        if curbuf.get() != old_curbuf {
            (*no_wait_return.ptr()) -= 1;
            emsg(gettext(
                c"E135: *Filter* Autocommands must not change current buffer".as_ptr(),
            ));
        } else if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
            (*curbuf.get()).b_op_start = orig_start;
            (*curbuf.get()).b_op_end = orig_end;
        }
        if !itmp.is_null() {
            os_remove(itmp);
        }
        if !otmp.is_null() {
            os_remove(otmp);
        }
        xfree(itmp as *mut ::core::ffi::c_void);
        xfree(otmp as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn do_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        if check_secure() {
            msg_end();
            return;
        }
        msg_putchar('\r' as ::core::ffi::c_int);
        msg_putchar('\n' as ::core::ffi::c_int);
        if p_warn.get() != 0 && !autocmd_busy.get() && msg_silent.get() == 0 as ::core::ffi::c_int {
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                if bufIsChanged(buf) {
                    msg_puts(gettext(c"[No write since last change]\n".as_ptr()));
                    break;
                } else {
                    buf = (*buf).b_next;
                }
            }
        }
        ui_cursor_goto(msg_row.get(), msg_col.get());
        call_shell(cmd, flags, ::core::ptr::null_mut::<::core::ffi::c_char>());
        if msg_silent.get() == 0 as ::core::ffi::c_int {
            msg_didout.set(true_0 != 0);
        }
        did_check_timestamps.set(false_0 != 0);
        need_check_timestamps.set(true_0 != 0);
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        msg_col.set(0 as ::core::ffi::c_int);
        apply_autocmds(
            EVENT_SHELLCMDPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
}

pub unsafe extern "C" fn make_filter_cmd(
    mut cmd: *mut ::core::ffi::c_char,
    mut itmp: *mut ::core::ffi::c_char,
    mut otmp: *mut ::core::ffi::c_char,
    mut do_in: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut is_fish_shell: bool = strncmp(
            invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
            c"fish".as_ptr(),
            4 as size_t,
        ) == 0 as ::core::ffi::c_int;
        let mut is_pwsh: bool = strncmp(
            invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
            c"pwsh".as_ptr(),
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
                c"powershell".as_ptr(),
                10 as size_t,
            ) == 0 as ::core::ffi::c_int;
        let mut len: size_t = strlen(cmd).wrapping_add(1 as size_t);
        len = (len as ::core::ffi::c_ulong).wrapping_add(
            (if is_fish_shell as ::core::ffi::c_int != 0 {
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1_usize)
            } else if !is_pwsh {
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1_usize)
            } else {
                0_usize
            }) as ::core::ffi::c_ulong,
        ) as size_t;
        if !itmp.is_null() {
            len = (len as ::core::ffi::c_ulong).wrapping_add(
                (if is_pwsh as ::core::ffi::c_int != 0 {
                    strlen(itmp)
                        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 24]>())
                        .wrapping_sub(1 as size_t)
                        .wrapping_add(6 as size_t)
                } else {
                    strlen(itmp)
                        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 10]>())
                        .wrapping_sub(1 as size_t)
                }) as ::core::ffi::c_ulong,
            ) as size_t;
        }
        if do_in as ::core::ffi::c_int != 0 && is_pwsh as ::core::ffi::c_int != 0 {
            len = (len as ::core::ffi::c_ulong)
                .wrapping_add(
                    ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as ::core::ffi::c_ulong
                ) as size_t;
        }
        if !otmp.is_null() {
            len = len.wrapping_add(
                strlen(otmp)
                    .wrapping_add(strlen(p_srr.get()))
                    .wrapping_add(2 as size_t),
            );
        }
        let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        if is_pwsh {
            if !itmp.is_null() {
                xstrlcpy(
                    buf,
                    c"& { Get-Content ".as_ptr(),
                    len.wrapping_sub(1 as size_t),
                );
                xstrlcat(buf, itmp, len.wrapping_sub(1 as size_t));
                xstrlcat(buf, c" | & ".as_ptr(), len.wrapping_sub(1 as size_t));
                xstrlcat(buf, cmd, len.wrapping_sub(1 as size_t));
                xstrlcat(buf, c" }".as_ptr(), len.wrapping_sub(1 as size_t));
            } else if do_in {
                xstrlcpy(buf, c" $input | ".as_ptr(), len.wrapping_sub(1 as size_t));
                xstrlcat(buf, cmd, len);
            } else {
                xstrlcpy(buf, cmd, len);
            }
        } else {
            if !itmp.is_null() || !otmp.is_null() {
                let mut fmt: *mut ::core::ffi::c_char =
                    (if is_fish_shell as ::core::ffi::c_int != 0 {
                        c"begin; %s; end".as_ptr()
                    } else {
                        c"(%s)".as_ptr()
                    }) as *mut ::core::ffi::c_char;
                vim_snprintf(buf, len, fmt, cmd);
            } else {
                xstrlcpy(buf, cmd, len);
            }
            if !itmp.is_null() {
                xstrlcat(buf, c" < ".as_ptr(), len.wrapping_sub(1 as size_t));
                xstrlcat(buf, itmp, len.wrapping_sub(1 as size_t));
            }
        }
        if !otmp.is_null() {
            append_redir(buf, len, p_srr.get(), otmp);
        }
        return buf;
    }
}

pub unsafe extern "C" fn append_redir(
    buf: *mut ::core::ffi::c_char,
    buflen: size_t,
    opt: *const ::core::ffi::c_char,
    fname: *const ::core::ffi::c_char,
) {
    unsafe {
        let end: *mut ::core::ffi::c_char = buf.add(strlen(buf));
        let mut p: *const ::core::ffi::c_char = opt;
        loop {
            p = strchr(p, '%' as ::core::ffi::c_int);
            if p.is_null() {
                break;
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
            {
                break;
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '%' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            p = p.offset(1);
        }
        if !p.is_null() {
            *end = ' ' as ::core::ffi::c_char;
            vim_snprintf(
                end.offset(1 as ::core::ffi::c_int as isize),
                (buflen as ptrdiff_t
                    - end
                        .offset(1 as ::core::ffi::c_int as isize)
                        .offset_from(buf)) as size_t,
                opt,
                fname,
            );
        } else {
            vim_snprintf(
                end,
                (buflen as ptrdiff_t - end.offset_from(buf)) as size_t,
                c" %s %s".as_ptr(),
                opt,
                fname,
            );
        };
    }
}

pub unsafe extern "C" fn print_line_no_prefix(
    mut lnum: linenr_T,
    mut use_number: bool,
    mut list: bool,
) {
    unsafe {
        let mut numbuf: [::core::ffi::c_char; 30] = [0; 30];
        if (*curwin.get()).w_onebuf_opt.wo_nu != 0 || use_number as ::core::ffi::c_int != 0 {
            vim_snprintf(
                &raw mut numbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                c"%*d ".as_ptr(),
                number_width(curwin.get()),
                lnum,
            );
            msg_puts_hl(
                &raw mut numbuf as *mut ::core::ffi::c_char,
                HLF_N + 1 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_prt_line(ml_get(lnum), list);
    }
}

pub(crate) static global_need_msg_kind: GlobalCell<bool> = GlobalCell::new(false_0 != 0);

pub unsafe extern "C" fn print_line(
    mut lnum: linenr_T,
    mut use_number: bool,
    mut list: bool,
    mut first: bool,
) {
    unsafe {
        let mut save_silent: bool = silent_mode.get();
        if message_filtered(ml_get(lnum)) {
            return;
        }
        silent_mode.set(false_0 != 0);
        info_message.set(true_0 != 0);
        if (global_busy.get() == 0 || global_need_msg_kind.get() as ::core::ffi::c_int != 0)
            && first as ::core::ffi::c_int != 0
        {
            msg_start();
            msg_ext_set_kind(c"list_cmd".as_ptr());
            global_need_msg_kind.set(false_0 != 0);
        } else if !save_silent {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        print_line_no_prefix(lnum, use_number, list);
        if save_silent {
            msg_putchar('\n' as ::core::ffi::c_int);
            silent_mode.set(save_silent);
        }
        info_message.set(false_0 != 0);
    }
}
