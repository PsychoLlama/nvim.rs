//! Printing a list to the message area.
//!
//! [`qf_list`] is `:clist`, one line per entry via [`qf_list_entry`];
//! [`qf_age`] is `:colder`/`:cnewer`, [`qf_history`] is `:chistory` and
//! [`qf_view_result`] is what `:cc` prints when there is nothing to jump
//! to. `qfga` is the shared growable buffer these build their text in.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn qfga_get() -> *mut garray_T {
    unsafe {
        static initialized: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if !initialized.get() {
            initialized.set(true_0 != 0);
            ga_init(
                qfga.ptr(),
                1 as ::core::ffi::c_int,
                256 as ::core::ffi::c_int,
            );
        }
        (*qfga.ptr()).ga_len = 0 as ::core::ffi::c_int;
        return qfga.ptr();
    }
}

pub(crate) unsafe extern "C" fn qfga_clear() {
    unsafe {
        if (*qfga.ptr()).ga_maxlen > 1000 as ::core::ffi::c_int {
            ga_clear(qfga.ptr());
        } else {
            (*qfga.ptr()).ga_len = 0 as ::core::ffi::c_int;
        };
    }
}

pub(crate) unsafe extern "C" fn qf_list_entry(
    mut qfp: *mut qfline_T,
    mut qf_idx: ::core::ffi::c_int,
    mut cursel: bool,
) {
    unsafe {
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !(*qfp).qf_module.is_null() && *(*qfp).qf_module as ::core::ffi::c_int != NUL {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%2d %s\0".as_ptr() as *const ::core::ffi::c_char,
                qf_idx,
                (*qfp).qf_module,
            );
        } else {
            let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
            if (*qfp).qf_fnum != 0 as ::core::ffi::c_int && {
                buf = buflist_findnr((*qfp).qf_fnum);
                !buf.is_null()
            } {
                fname = if (*qfp).qf_fname.is_null() {
                    (*buf).b_fname
                } else {
                    (*qfp).qf_fname
                };
                if (*qfp).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                    fname = path_tail(fname);
                }
            }
            if fname.is_null() {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%2d\0".as_ptr() as *const ::core::ffi::c_char,
                    qf_idx,
                );
            } else {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%2d %s\0".as_ptr() as *const ::core::ffi::c_char,
                    qf_idx,
                    fname,
                );
            }
        }
        let mut filter_entry: bool = true_0 != 0;
        if !(*qfp).qf_module.is_null() && *(*qfp).qf_module as ::core::ffi::c_int != NUL {
            filter_entry = filter_entry as ::core::ffi::c_int
                & message_filtered((*qfp).qf_module) as ::core::ffi::c_int
                != 0;
        }
        if filter_entry as ::core::ffi::c_int != 0 && !fname.is_null() {
            filter_entry = filter_entry as ::core::ffi::c_int
                & message_filtered(fname) as ::core::ffi::c_int
                != 0;
        }
        if filter_entry as ::core::ffi::c_int != 0 && !(*qfp).qf_pattern.is_null() {
            filter_entry = filter_entry as ::core::ffi::c_int
                & message_filtered((*qfp).qf_pattern) as ::core::ffi::c_int
                != 0;
        }
        if filter_entry {
            filter_entry = filter_entry as ::core::ffi::c_int
                & message_filtered((*qfp).qf_text) as ::core::ffi::c_int
                != 0;
        }
        if filter_entry {
            return;
        }
        if msg_col.get() > 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        msg_outtrans(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            if cursel as ::core::ffi::c_int != 0 {
                HLF_QFL as ::core::ffi::c_int
            } else {
                qfFile_hl_id.get()
            },
            false_0 != 0,
        );
        if (*qfp).qf_lnum != 0 as linenr_T {
            msg_puts_hl(
                b":\0".as_ptr() as *const ::core::ffi::c_char,
                qfSep_hl_id.get(),
                false_0 != 0,
            );
        }
        let mut gap: *mut garray_T = qfga_get();
        if (*qfp).qf_lnum != 0 as linenr_T {
            qf_range_text(gap, qfp);
        }
        ga_concat(
            gap,
            qf_types((*qfp).qf_type as ::core::ffi::c_int, (*qfp).qf_nr),
        );
        ga_append(gap, NUL as uint8_t);
        if *((*gap).ga_data as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL {
            msg_puts_hl(
                (*gap).ga_data as *const ::core::ffi::c_char,
                qfLine_hl_id.get(),
                false_0 != 0,
            );
        }
        msg_puts_hl(
            b":\0".as_ptr() as *const ::core::ffi::c_char,
            qfSep_hl_id.get(),
            false_0 != 0,
        );
        if !(*qfp).qf_pattern.is_null() {
            gap = qfga_get();
            qf_fmt_text(gap, (*qfp).qf_pattern);
            ga_append(gap, NUL as uint8_t);
            msg_puts((*gap).ga_data as *const ::core::ffi::c_char);
            msg_puts_hl(
                b":\0".as_ptr() as *const ::core::ffi::c_char,
                qfSep_hl_id.get(),
                false_0 != 0,
            );
        }
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        gap = qfga_get();
        qf_fmt_text(
            gap,
            if !fname.is_null() || (*qfp).qf_lnum != 0 as linenr_T {
                skipwhite((*qfp).qf_text)
            } else {
                (*qfp).qf_text
            },
        );
        ga_append(gap, NUL as uint8_t);
        msg_prt_line((*gap).ga_data as *const ::core::ffi::c_char, false_0 != 0);
    }
}

pub unsafe fn qf_list(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut all: ::core::ffi::c_int = (*eap).forceit;
        let mut qi: *mut qf_info_T = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        if qf_stack_empty(qi) as ::core::ffi::c_int != 0
            || qf_list_empty(qf_get_curlist(qi)) as ::core::ffi::c_int != 0
        {
            emsg(gettext(
                &raw const e_no_errors as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut plus: bool = false_0 != 0;
        if *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
            arg = arg.offset(1);
            plus = true_0 != 0;
        }
        let mut idx1: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut idx2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if get_list_range(&raw mut arg, &raw mut idx1, &raw mut idx2) == 0
            || *arg as ::core::ffi::c_int != NUL
        {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                arg,
            );
            return;
        }
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        let mut i: ::core::ffi::c_int = 0;
        if plus {
            i = (*qfl).qf_index;
            idx2 = i + idx1;
            idx1 = i;
        } else {
            i = (*qfl).qf_count;
            if idx1 < 0 as ::core::ffi::c_int {
                idx1 = if -idx1 > i {
                    0 as ::core::ffi::c_int
                } else {
                    idx1 + i + 1 as ::core::ffi::c_int
                };
            }
            if idx2 < 0 as ::core::ffi::c_int {
                idx2 = if -idx2 > i {
                    0 as ::core::ffi::c_int
                } else {
                    idx2 + i + 1 as ::core::ffi::c_int
                };
            }
        }
        shorten_fnames(false_0);
        qfFile_hl_id.set(syn_name2id(
            b"qfFileName\0".as_ptr() as *const ::core::ffi::c_char
        ));
        if qfFile_hl_id.get() == 0 as ::core::ffi::c_int {
            qfFile_hl_id.set(HLF_D as ::core::ffi::c_int);
        }
        qfSep_hl_id.set(syn_name2id(
            b"qfSeparator\0".as_ptr() as *const ::core::ffi::c_char
        ));
        if qfSep_hl_id.get() == 0 as ::core::ffi::c_int {
            qfSep_hl_id.set(HLF_D as ::core::ffi::c_int);
        }
        qfLine_hl_id.set(syn_name2id(
            b"qfLineNr\0".as_ptr() as *const ::core::ffi::c_char
        ));
        if qfLine_hl_id.get() == 0 as ::core::ffi::c_int {
            qfLine_hl_id.set(HLF_N as ::core::ffi::c_int);
        }
        if (*qfl).qf_nonevalid {
            all = true_0;
        }
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        i = 1 as ::core::ffi::c_int;
        qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if ((*qfp).qf_valid as ::core::ffi::c_int != 0 || all != 0) && idx1 <= i && i <= idx2 {
                qf_list_entry(qfp, i, i == (*qfl).qf_index);
            }
            os_breakcheck();
            i += 1;
            qfp = (*qfp).qf_next;
        }
        qfga_clear();
    }
}

pub(crate) unsafe extern "C" fn qf_fmt_text(
    mut gap: *mut garray_T,
    mut text: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut p: *const ::core::ffi::c_char = text;
        while *p as ::core::ffi::c_int != NUL {
            if *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                ga_append(gap, ' ' as uint8_t);
                loop {
                    p = p.offset(1);
                    if *p as ::core::ffi::c_int == NUL {
                        break;
                    }
                    if !ascii_iswhite(*p as ::core::ffi::c_int)
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        break;
                    }
                }
            } else {
                let c2rust_fresh0 = p;
                p = p.offset(1);
                ga_append(gap, *c2rust_fresh0 as uint8_t);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_range_text(mut gap: *mut garray_T, mut qfp: *const qfline_T) {
    unsafe {
        let mut buf: String_0 = String_0 {
            data: IObuff.ptr() as *mut ::core::ffi::c_char,
            size: 0 as size_t,
        };
        buf.size = vim_snprintf_safelen(
            buf.data,
            IOSIZE as size_t,
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*qfp).qf_lnum,
        );
        if (*qfp).qf_end_lnum > 0 as linenr_T && (*qfp).qf_lnum != (*qfp).qf_end_lnum {
            buf.size = buf.size.wrapping_add(vim_snprintf_safelen(
                buf.data.offset(buf.size as isize),
                (IOSIZE as size_t).wrapping_sub(buf.size),
                b"-%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*qfp).qf_end_lnum,
            ));
        }
        if (*qfp).qf_col > 0 as ::core::ffi::c_int {
            buf.size = buf.size.wrapping_add(vim_snprintf_safelen(
                buf.data.offset(buf.size as isize),
                (IOSIZE as size_t).wrapping_sub(buf.size),
                b" col %d\0".as_ptr() as *const ::core::ffi::c_char,
                (*qfp).qf_col,
            ));
            if (*qfp).qf_end_col > 0 as ::core::ffi::c_int && (*qfp).qf_col != (*qfp).qf_end_col {
                buf.size = buf.size.wrapping_add(vim_snprintf_safelen(
                    buf.data.offset(buf.size as isize),
                    (IOSIZE as size_t).wrapping_sub(buf.size),
                    b"-%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*qfp).qf_end_col,
                ));
            }
        }
        ga_concat_len(gap, buf.data, buf.size);
    }
}

pub(crate) unsafe extern "C" fn qf_msg(
    mut qi: *mut qf_info_T,
    mut which: ::core::ffi::c_int,
    mut lead: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut title: *mut ::core::ffi::c_char = (*(*qi).qf_lists.offset(which as isize)).qf_title;
        let mut count: ::core::ffi::c_int = (*(*qi).qf_lists.offset(which as isize)).qf_count;
        let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            gettext(b"%serror list %d of %d; %d errors \0".as_ptr() as *const ::core::ffi::c_char),
            lead,
            which + 1 as ::core::ffi::c_int,
            (*qi).qf_listcount,
            count,
        );
        if !title.is_null() {
            let mut len: size_t = strlen(&raw mut buf as *mut ::core::ffi::c_char);
            if len < 34 as size_t {
                memset(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                        as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    (34 as size_t).wrapping_sub(len),
                );
                buf[34 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            }
            xstrlcat(
                &raw mut buf as *mut ::core::ffi::c_char,
                title,
                IOSIZE as size_t,
            );
        }
        trunc_string(
            &raw mut buf as *mut ::core::ffi::c_char,
            &raw mut buf as *mut ::core::ffi::c_char,
            Columns.get() - 1 as ::core::ffi::c_int,
            IOSIZE,
        );
        msg(
            &raw mut buf as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
}

pub unsafe fn qf_age(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut count: ::core::ffi::c_int = if (*eap).addr_count != 0 as ::core::ffi::c_int {
            (*eap).line2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
        loop {
            let c2rust_fresh23 = count;
            count = count - 1;
            if c2rust_fresh23 == 0 {
                break;
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_colder as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lolder as ::core::ffi::c_int
            {
                if (*qi).qf_curlist == 0 as ::core::ffi::c_int {
                    emsg(gettext(b"E380: At bottom of quickfix stack\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    break;
                } else {
                    (*qi).qf_curlist -= 1;
                }
            } else if (*qi).qf_curlist >= (*qi).qf_listcount - 1 as ::core::ffi::c_int {
                emsg(gettext(
                    b"E381: At top of quickfix stack\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break;
            } else {
                (*qi).qf_curlist += 1;
            }
        }
        qf_msg(
            qi,
            (*qi).qf_curlist,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
    }
}

pub unsafe fn qf_history(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = qf_cmd_get_stack(eap, false_0 != 0);
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            if qi.is_null() {
                emsg(gettext(&raw const e_loclist as *const ::core::ffi::c_char));
                return;
            }
            if (*eap).line2 > 0 as linenr_T && (*eap).line2 <= (*qi).qf_listcount as linenr_T {
                (*qi).qf_curlist = ((*eap).line2 - 1 as linenr_T) as ::core::ffi::c_int;
                qf_msg(
                    qi,
                    (*qi).qf_curlist,
                    b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
            } else {
                emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            }
            return;
        }
        if qf_stack_empty(qi) {
            msg(
                gettext(b"No entries\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*qi).qf_listcount {
                qf_msg(
                    qi,
                    i,
                    (if i == (*qi).qf_curlist {
                        b"> \0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"  \0".as_ptr() as *const ::core::ffi::c_char
                    }) as *mut ::core::ffi::c_char,
                );
                i += 1;
            }
        };
    }
}

pub(crate) unsafe extern "C" fn qf_types(
    mut c: ::core::ffi::c_int,
    mut nr: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static cc: GlobalCell<[::core::ffi::c_char; 3]> = GlobalCell::new([0; 3]);
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if c == 'W' as ::core::ffi::c_int || c == 'w' as ::core::ffi::c_int {
            p = b" warning\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if c == 'I' as ::core::ffi::c_int || c == 'i' as ::core::ffi::c_int {
            p = b" info\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if c == 'N' as ::core::ffi::c_int || c == 'n' as ::core::ffi::c_int {
            p = b" note\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if c == 'E' as ::core::ffi::c_int
            || c == 'e' as ::core::ffi::c_int
            || c == 0 as ::core::ffi::c_int && nr > 0 as ::core::ffi::c_int
        {
            p = b" error\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if c == 0 as ::core::ffi::c_int || c == 1 as ::core::ffi::c_int {
            p = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            (*cc.ptr())[0 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
            (*cc.ptr())[1 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
            (*cc.ptr())[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            p = cc.ptr() as *mut ::core::ffi::c_char;
        }
        if nr <= 0 as ::core::ffi::c_int {
            return p;
        }
        static buf: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new([0; 20]);
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
            b"%s %3d\0".as_ptr() as *const ::core::ffi::c_char,
            p,
            nr,
        );
        return buf.ptr() as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn qf_view_result(mut split: bool) {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3776 as ::core::ffi::c_uint,
                    b"void qf_view_result(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
            && !(*curwin.get()).w_llist_ref.is_null()
        {
            qi = if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
                && !(*curwin.get()).w_llist_ref.is_null()
            {
                (*curwin.get()).w_llist_ref
            } else {
                (*curwin.get()).w_llist
            };
        }
        if qf_list_empty(qf_get_curlist(qi)) {
            emsg(gettext(
                &raw const e_no_errors as *const ::core::ffi::c_char,
            ));
            return;
        }
        if split {
            qf_jump_newwin(
                qi,
                0 as ::core::ffi::c_int,
                (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int,
                false_0,
                true_0 != 0,
            );
            do_cmdline_cmd(b"clearjumps\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        do_cmdline_cmd(
            if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
                && !(*curwin.get()).w_llist_ref.is_null()
            {
                b".ll\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b".cc\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
    }
}
