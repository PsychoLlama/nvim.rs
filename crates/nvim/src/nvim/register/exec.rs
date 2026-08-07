//! Recording a register, and replaying one.
//!
//! `do_record` is `q`: it turns recording on, and on the second `q` moves what
//! `get_recorded` collected into the register (appending for an uppercase
//! name).  Replay is the other half -- `do_execreg` is `@`, and it does not
//! *run* anything, it stuffs the register's text into the typeahead buffer so
//! that the normal-mode loop reads it as if typed.  That is what
//! `put_in_typebuf` and `put_reedit_in_typebuf` are for, and why an
//! interrupted `@` leaves the rest of the register queued.  `insert_reg` is
//! CTRL-R in Insert mode and `cmdline_paste_reg` CTRL-R on the command
//! line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn stuff_yank(
    mut regname: ::core::ffi::c_int,
    mut p: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if regname != 0 as ::core::ffi::c_int && !valid_yank_reg(regname, true) {
            xfree(p as *mut ::core::ffi::c_void);
            return FAIL;
        }
        if regname == '_' as ::core::ffi::c_int {
            xfree(p as *mut ::core::ffi::c_void);
            return OK;
        }
        let plen: size_t = strlen(p);
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_YANK);
        if is_append_register(regname) as ::core::ffi::c_int != 0 && !(*reg).y_array.is_null() {
            let mut pp: *mut String_0 = (*reg)
                .y_array
                .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize);
            let tmplen: size_t = (*pp).size.wrapping_add(plen);
            let mut tmp: *mut ::core::ffi::c_char =
                xmalloc(tmplen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            memcpy(
                tmp as *mut ::core::ffi::c_void,
                (*pp).data as *const ::core::ffi::c_void,
                (*pp).size,
            );
            memcpy(
                tmp.offset((*pp).size as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                plen,
            );
            *tmp.offset(tmplen as isize) = NUL as ::core::ffi::c_char;
            xfree(p as *mut ::core::ffi::c_void);
            xfree((*pp).data as *mut ::core::ffi::c_void);
            *pp = String_0 {
                data: tmp,
                size: tmplen,
            };
        } else {
            free_register(reg);
            (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
            (*reg).y_array = xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
            *(*reg).y_array.offset(0 as ::core::ffi::c_int as isize) = String_0 {
                data: p,
                size: plen,
            };
            (*reg).y_size = 1 as size_t;
            (*reg).y_type = kMTCharWise;
        }
        (*reg).timestamp = os_time();
        return OK;
    }
}

pub unsafe extern "C" fn do_record(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        static regname: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        let mut retval: ::core::ffi::c_int = 0;
        if reg_recording.get() == 0 as ::core::ffi::c_int {
            if c < 0 as ::core::ffi::c_int
                || !(c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                    || ascii_isdigit(c) as ::core::ffi::c_int != 0)
                    && c != '"' as ::core::ffi::c_int
            {
                retval = FAIL;
            } else {
                reg_recording.set(c);
                showmode();
                regname.set(c);
                retval = OK;
                apply_autocmds(
                    EVENT_RECORDINGENTER,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false,
                    curbuf.get(),
                );
            }
        } else {
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
            let mut p: *mut ::core::ffi::c_char = get_recorded();
            if !p.is_null() {
                vim_unescape_ks(p);
                tv_dict_add_str(
                    dict,
                    c"regcontents".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                    p,
                );
            }
            let mut buf: [::core::ffi::c_char; 67] = [0; 67];
            buf[0 as ::core::ffi::c_int as usize] = regname.get() as ::core::ffi::c_char;
            buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            tv_dict_add_str(
                dict,
                c"regname".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut buf as *mut ::core::ffi::c_char,
            );
            tv_dict_set_keys_readonly(dict);
            apply_autocmds(
                EVENT_RECORDINGLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false,
                curbuf.get(),
            );
            restore_v_event(dict, &raw mut save_v_event);
            reg_recorded.set(reg_recording.get());
            reg_recording.set(0 as ::core::ffi::c_int);
            if p_ch.get() == 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
                showmode();
            } else {
                msg(c"".as_ptr(), 0 as ::core::ffi::c_int);
            }
            if p.is_null() {
                retval = FAIL;
            } else {
                let mut old_y_previous: *mut yankreg_T = y_previous.get();
                retval = stuff_yank(regname.get(), p);
                y_previous.set(old_y_previous);
            }
        }
        return retval;
    }
}

unsafe extern "C" fn put_in_typebuf(
    mut s: *mut ::core::ffi::c_char,
    mut esc: bool,
    mut colon: bool,
    mut silent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = OK;
        put_reedit_in_typebuf(silent);
        if colon {
            retval = ins_typebuf(
                c"\n".as_ptr() as *mut ::core::ffi::c_char,
                REMAP_NONE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true,
                silent != 0,
            );
        }
        if retval == OK {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if esc {
                p = vim_strsave_escape_ks(s);
            } else {
                p = s;
            }
            if p.is_null() {
                retval = FAIL;
            } else {
                retval = ins_typebuf(
                    p,
                    if esc as ::core::ffi::c_int != 0 {
                        REMAP_NONE as ::core::ffi::c_int
                    } else {
                        REMAP_YES as ::core::ffi::c_int
                    },
                    0 as ::core::ffi::c_int,
                    true,
                    silent != 0,
                );
            }
            if esc {
                xfree(p as *mut ::core::ffi::c_void);
            }
        }
        if colon as ::core::ffi::c_int != 0 && retval == OK {
            retval = ins_typebuf(
                c":".as_ptr() as *mut ::core::ffi::c_char,
                REMAP_NONE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true,
                silent != 0,
            );
        }
        return retval;
    }
}

unsafe extern "C" fn put_reedit_in_typebuf(mut silent: ::core::ffi::c_int) {
    unsafe {
        let mut buf: [uint8_t; 3] = [0; 3];
        if restart_edit.get() == NUL {
            return;
        }
        if restart_edit.get() == 'V' as ::core::ffi::c_int {
            buf[0 as ::core::ffi::c_int as usize] = 'g' as uint8_t;
            buf[1 as ::core::ffi::c_int as usize] = 'R' as uint8_t;
            buf[2 as ::core::ffi::c_int as usize] = NUL as uint8_t;
        } else {
            buf[0 as ::core::ffi::c_int as usize] =
                (if restart_edit.get() == 'I' as ::core::ffi::c_int {
                    'i' as ::core::ffi::c_int
                } else {
                    restart_edit.get()
                }) as uint8_t;
            buf[1 as ::core::ffi::c_int as usize] = NUL as uint8_t;
        }
        if ins_typebuf(
            &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
            REMAP_NONE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true,
            silent != 0,
        ) == OK
        {
            restart_edit.set(NUL);
        }
    }
}

unsafe extern "C" fn execreg_line_continuation(
    mut lines: *mut String_0,
    mut idx: *mut size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut cmd_start: size_t = *idx;
        '_c2rust_label: {
            if cmd_start > 0 as size_t {
            } else {
                __assert_fail(
                    c"cmd_start > 0".as_ptr(),
                    c"src/nvim/register.rs".as_ptr(),
                    575 as ::core::ffi::c_uint,
                    c"char *execreg_line_continuation(String *, size_t *)".as_ptr(),
                );
            }
        };
        let cmd_end: size_t = cmd_start;
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            400 as ::core::ffi::c_int,
        );
        loop {
            cmd_start = cmd_start.wrapping_sub(1);
            if cmd_start <= 0 as size_t {
                break;
            }
            let mut p: *mut ::core::ffi::c_char =
                skipwhite((*lines.offset(cmd_start as isize)).data);
            if *p as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
                && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '"' as ::core::ffi::c_int
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                    || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ' ' as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut tmp: *mut String_0 = lines.offset(cmd_start as isize);
        ga_concat_len(&raw mut ga, (*tmp).data, (*tmp).size);
        let mut j: size_t = cmd_start.wrapping_add(1 as size_t);
        while j <= cmd_end {
            tmp = lines.offset(j as isize);
            let mut p_0: *mut ::core::ffi::c_char = skipwhite((*tmp).data);
            if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                if ga.ga_len > 400 as ::core::ffi::c_int {
                    ga_set_growsize(
                        &raw mut ga,
                        if ga.ga_len < 8000 as ::core::ffi::c_int {
                            ga.ga_len
                        } else {
                            8000 as ::core::ffi::c_int
                        },
                    );
                }
                p_0 = p_0.offset(1);
                ga_concat_len(
                    &raw mut ga,
                    p_0,
                    (*tmp).data.offset((*tmp).size as isize).offset_from(p_0) as size_t,
                );
            }
            j = j.wrapping_add(1);
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        let mut str: *mut ::core::ffi::c_char =
            xmemdupz(ga.ga_data, ga.ga_len as size_t) as *mut ::core::ffi::c_char;
        ga_clear(&raw mut ga);
        *idx = cmd_start;
        return str;
    }
}

pub unsafe extern "C" fn do_execreg(
    mut regname: ::core::ffi::c_int,
    mut colon: ::core::ffi::c_int,
    mut addcr: ::core::ffi::c_int,
    mut silent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = OK;
        if regname == '@' as ::core::ffi::c_int {
            if execreg_lastc.get() == NUL {
                emsg(gettext(c"E748: No previously used register".as_ptr()));
                return FAIL;
            }
            regname = execreg_lastc.get();
        }
        if regname == '%' as ::core::ffi::c_int
            || regname == '#' as ::core::ffi::c_int
            || !valid_yank_reg(regname, false)
        {
            emsg_invreg(regname);
            return FAIL;
        }
        execreg_lastc.set(regname);
        if regname == '_' as ::core::ffi::c_int {
            return OK;
        }
        if regname == ':' as ::core::ffi::c_int {
            if (*last_cmdline.ptr()).is_null() {
                emsg(gettext(
                    &raw const e_nolastcmd as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                new_last_cmdline.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            let mut p: *mut ::core::ffi::c_char = vim_strsave_escaped_ext(
            last_cmdline.get(),
            c"\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F".as_ptr(),
            Ctrl_V as ::core::ffi::c_char,
            false,
        );
            if VIsual_active.get() as ::core::ffi::c_int != 0
                && strncmp(p, c"'<,'>".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int
            {
                retval = put_in_typebuf(
                    p.offset(5 as ::core::ffi::c_int as isize),
                    true,
                    true,
                    silent,
                );
            } else {
                retval = put_in_typebuf(p, true, true, silent);
            }
            xfree(p as *mut ::core::ffi::c_void);
        } else if regname == '=' as ::core::ffi::c_int {
            let mut p_0: *mut ::core::ffi::c_char = get_expr_line();
            if p_0.is_null() {
                return FAIL;
            }
            retval = put_in_typebuf(p_0, true, colon != 0, silent);
            xfree(p_0 as *mut ::core::ffi::c_void);
        } else if regname == '.' as ::core::ffi::c_int {
            let mut p_1: *mut ::core::ffi::c_char = get_last_insert_save();
            if p_1.is_null() {
                emsg(gettext(
                    &raw const e_noinstext as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
            retval = put_in_typebuf(p_1, false, colon != 0, silent);
            xfree(p_1 as *mut ::core::ffi::c_void);
        } else {
            let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE);
            if (*reg).y_array.is_null() {
                return FAIL;
            }
            let mut remap: ::core::ffi::c_int = if colon != 0 {
                REMAP_NONE as ::core::ffi::c_int
            } else {
                REMAP_YES as ::core::ffi::c_int
            };
            put_reedit_in_typebuf(silent);
            let mut i: size_t = (*reg).y_size;
            loop {
                let c2rust_fresh1 = i;
                i = i.wrapping_sub(1);
                if c2rust_fresh1 <= 0 as size_t {
                    break;
                }
                if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                    || i < (*reg).y_size.wrapping_sub(1 as size_t)
                    || addcr != 0
                {
                    if ins_typebuf(
                        c"\n".as_ptr() as *mut ::core::ffi::c_char,
                        remap,
                        0 as ::core::ffi::c_int,
                        true,
                        silent != 0,
                    ) == FAIL
                    {
                        return FAIL;
                    }
                }
                let mut str: *mut ::core::ffi::c_char = (*(*reg).y_array.offset(i as isize)).data;
                let mut free_str: bool = false;
                if colon != 0 && i > 0 as size_t {
                    let mut p_2: *mut ::core::ffi::c_char = skipwhite(str);
                    if *p_2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        || *p_2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '"' as ::core::ffi::c_int
                            && *p_2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                            && *p_2.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int
                    {
                        str = execreg_line_continuation((*reg).y_array, &raw mut i);
                        free_str = true;
                    }
                }
                let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(str);
                if free_str {
                    xfree(str as *mut ::core::ffi::c_void);
                }
                retval = ins_typebuf(escaped, remap, 0 as ::core::ffi::c_int, true, silent != 0);
                xfree(escaped as *mut ::core::ffi::c_void);
                if retval == FAIL {
                    return FAIL;
                }
                if colon != 0
                    && ins_typebuf(
                        c":".as_ptr() as *mut ::core::ffi::c_char,
                        remap,
                        0 as ::core::ffi::c_int,
                        true,
                        silent != 0,
                    ) == FAIL
                {
                    return FAIL;
                }
            }
            reg_executing.set(if regname == 0 as ::core::ffi::c_int {
                '"' as ::core::ffi::c_int
            } else {
                regname
            });
            pending_end_reg_executing.set(false);
        }
        return retval;
    }
}

pub unsafe extern "C" fn insert_reg(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut literally_arg: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = OK;
        let mut allocated: bool = false;
        let literally: bool = literally_arg as ::core::ffi::c_int != 0
            || is_literal_register(regname) as ::core::ffi::c_int != 0;
        os_breakcheck();
        if got_int.get() {
            return FAIL;
        }
        if regname != NUL && !valid_yank_reg(regname, false) {
            return FAIL;
        }
        let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if regname == '.' as ::core::ffi::c_int {
            retval = stuff_inserted(NUL, 1 as ::core::ffi::c_int, true_0);
        } else if get_spec_reg(regname, &raw mut arg, &raw mut allocated, true) {
            if arg.is_null() {
                return FAIL;
            }
            stuffescaped(arg, literally);
            if allocated {
                xfree(arg as *mut ::core::ffi::c_void);
            }
        } else {
            if reg.is_null() {
                reg = get_yank_register(regname, YREG_PASTE);
            }
            if (*reg).y_array.is_null() {
                retval = FAIL;
            } else {
                let mut i: size_t = 0 as size_t;
                while i < (*reg).y_size {
                    if regname == '-' as ::core::ffi::c_int
                        && (*reg).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                    {
                        let mut dir: Direction = BACKWARD;
                        if State.get() & REPLACE_FLAG != 0 as ::core::ffi::c_int {
                            let mut curpos: pos_T = pos_T {
                                lnum: 0,
                                col: 0,
                                coladd: 0,
                            };
                            if u_save_cursor() == FAIL {
                                return FAIL;
                            }
                            del_chars(
                                mb_charlen(
                                    (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data,
                                ),
                                true_0,
                            );
                            curpos = (*curwin.get()).w_cursor;
                            if oneright() == FAIL {
                                dir = FORWARD;
                            }
                            (*curwin.get()).w_cursor = curpos;
                        }
                        AppendCharToRedobuff(Ctrl_R);
                        AppendCharToRedobuff(regname);
                        do_put(
                            regname,
                            ::core::ptr::null_mut::<yankreg_T>(),
                            dir as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                            PUT_CURSEND as ::core::ffi::c_int,
                        );
                    } else {
                        stuffescaped((*(*reg).y_array.offset(i as isize)).data, literally);
                        if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                            || i < (*reg).y_size.wrapping_sub(1 as size_t)
                        {
                            stuffcharReadbuff('\n' as ::core::ffi::c_int);
                        }
                    }
                    i = i.wrapping_add(1);
                }
            }
        }
        return retval;
    }
}

pub unsafe extern "C" fn cmdline_paste_reg(
    mut regname: ::core::ffi::c_int,
    mut literally_arg: bool,
    mut remcr: bool,
) -> bool {
    unsafe {
        let literally: bool = literally_arg as ::core::ffi::c_int != 0
            || is_literal_register(regname) as ::core::ffi::c_int != 0;
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE);
        if (*reg).y_array.is_null() {
            return FAIL != 0;
        }
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            cmdline_paste_str((*(*reg).y_array.offset(i as isize)).data, literally);
            if i < (*reg).y_size.wrapping_sub(1 as size_t) && !remcr {
                cmdline_paste_str(c"\r".as_ptr(), literally);
            }
            os_breakcheck();
            if got_int.get() {
                return FAIL != 0;
            }
            i = i.wrapping_add(1);
        }
        return OK != 0;
    }
}
