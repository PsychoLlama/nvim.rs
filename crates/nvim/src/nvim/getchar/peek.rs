//! `vgetorpeek` and `inchar`: the bottom of the input stack.
//!
//! [`vgetorpeek`] is the loop that turns "something is in the typeahead" into
//! "a byte to hand out": it consults the stuff buffers, runs the mapping
//! match, and blocks in [`inchar`] when neither has anything.  [`inchar`] is
//! the only place that reads the script stack or the OS input buffer.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn vgetorpeek(mut advance: bool) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        let mut timedout: bool = false_0 != 0;
        let mut mapdepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut mode_deleted: bool = false_0 != 0;
        if vgetc_busy.get() > 0 as ::core::ffi::c_int
            && ex_normal_busy.get() == 0 as ::core::ffi::c_int
        {
            return NUL;
        }
        (*vgetc_busy.ptr()) += 1;
        if advance {
            KeyStuffed.set(false_0);
            typebuf_was_empty.set(false_0 != 0);
        }
        init_typebuf();
        start_stuff();
        check_end_reg_executing(advance);
        loop {
            if typeahead_char.get() != 0 as ::core::ffi::c_int {
                c = typeahead_char.get();
                if advance {
                    typeahead_char.set(0 as ::core::ffi::c_int);
                }
            } else {
                c = read_readbuffers(advance);
            }
            if c != NUL && !got_int.get() {
                if advance {
                    KeyStuffed.set(true_0);
                }
                if (*typebuf.ptr()).tb_no_abbr_cnt == 0 as ::core::ffi::c_int {
                    (*typebuf.ptr()).tb_no_abbr_cnt = 1 as ::core::ffi::c_int;
                }
            } else {
                loop {
                    check_end_reg_executing(advance);
                    if (*typebuf.ptr()).tb_maplen != 0 {
                        line_breakcheck();
                    } else {
                        if (mapped_ctrl_c.get() | (*curbuf.get()).b_mapped_ctrl_c)
                            & get_real_state()
                            != 0
                        {
                            ctrl_c_interrupts.set(false_0 != 0);
                        }
                        os_breakcheck();
                        ctrl_c_interrupts.set(true_0 != 0);
                    }
                    let mut keylen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if got_int.get() {
                        c = inchar(
                            (*typebuf.ptr()).tb_buf,
                            (*typebuf.ptr()).tb_buflen - 1 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_long,
                        );
                        if (c != 0 || (*typebuf.ptr()).tb_maplen != 0)
                            && State.get() & (MODE_INSERT | MODE_CMDLINE) != 0
                        {
                            c = ESC;
                        } else {
                            c = Ctrl_C;
                        }
                        flush_buffers(FLUSH_INPUT);
                        if advance {
                            *(*typebuf.ptr()).tb_buf = c as uint8_t;
                            gotchars((*typebuf.ptr()).tb_buf, 1 as size_t);
                        }
                        cmd_silent.set(false_0 != 0);
                        break;
                    } else {
                        if (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
                            let mut result: map_result_T = handle_mapping(
                                &raw mut keylen,
                                &raw mut timedout,
                                &raw mut mapdepth,
                            )
                                as map_result_T;
                            if result as ::core::ffi::c_uint
                                == map_result_retry as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                continue;
                            }
                            if result as ::core::ffi::c_uint
                                == map_result_fail as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                c = -1 as ::core::ffi::c_int;
                                break;
                            } else if result as ::core::ffi::c_uint
                                == map_result_get as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                c = *(*typebuf.ptr())
                                    .tb_buf
                                    .offset((*typebuf.ptr()).tb_off as isize)
                                    as ::core::ffi::c_int;
                                if advance {
                                    cmd_silent
                                        .set((*typebuf.ptr()).tb_silent > 0 as ::core::ffi::c_int);
                                    if (*typebuf.ptr()).tb_maplen > 0 as ::core::ffi::c_int {
                                        KeyTyped.set(false_0 != 0);
                                    } else {
                                        KeyTyped.set(true_0 != 0);
                                        gotchars(
                                            (*typebuf.ptr())
                                                .tb_buf
                                                .offset((*typebuf.ptr()).tb_off as isize),
                                            1 as size_t,
                                        );
                                    }
                                    KeyNoremap.set(
                                        *(*typebuf.ptr())
                                            .tb_noremap
                                            .offset((*typebuf.ptr()).tb_off as isize)
                                            as ::core::ffi::c_uchar
                                            as ::core::ffi::c_int,
                                    );
                                    del_typebuf(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                                }
                                break;
                            }
                        }
                        c = 0 as ::core::ffi::c_int;
                        let mut new_wcol: ::core::ffi::c_int = (*curwin.get()).w_wcol;
                        let mut new_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
                        if advance as ::core::ffi::c_int != 0
                            && (*typebuf.ptr()).tb_len == 1 as ::core::ffi::c_int
                            && *(*typebuf.ptr())
                                .tb_buf
                                .offset((*typebuf.ptr()).tb_off as isize)
                                as ::core::ffi::c_int
                                == ESC
                            && no_mapping.get() == 0
                            && ex_normal_busy.get() == 0 as ::core::ffi::c_int
                            && (*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int
                            && State.get() & MODE_INSERT != 0
                            && (p_timeout.get() != 0
                                || keylen == KEYLEN_PART_KEY as ::core::ffi::c_int
                                    && p_ttimeout.get() != 0)
                            && {
                                c = inchar(
                                    (*typebuf.ptr())
                                        .tb_buf
                                        .offset((*typebuf.ptr()).tb_off as isize)
                                        .offset((*typebuf.ptr()).tb_len as isize),
                                    3 as ::core::ffi::c_int,
                                    25 as ::core::ffi::c_long,
                                );
                                c == 0 as ::core::ffi::c_int
                            }
                        {
                            if mode_displayed.get() {
                                unshowmode(true_0 != 0);
                                mode_deleted = true_0 != 0;
                            }
                            validate_cursor(curwin.get());
                            let mut old_wcol: ::core::ffi::c_int = (*curwin.get()).w_wcol;
                            let mut old_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
                            if (*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int {
                                let mut col: colnr_T = 0 as colnr_T;
                                let mut ptr: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                if (*curwin.get()).w_wcol > 0 as ::core::ffi::c_int {
                                    if did_ai.get() as ::core::ffi::c_int != 0
                                        && *skipwhite(
                                            get_cursor_line_ptr()
                                                .offset((*curwin.get()).w_cursor.col as isize),
                                        )
                                            as ::core::ffi::c_int
                                            == NUL
                                    {
                                        (*curwin.get()).w_wcol = 0 as ::core::ffi::c_int;
                                        ptr = get_cursor_line_ptr();
                                        let mut endptr: *mut ::core::ffi::c_char =
                                            ptr.offset((*curwin.get()).w_cursor.col as isize);
                                        let mut csarg: CharsizeArg = CharsizeArg::default();
                                        let mut cstype: CharsizeKind = init_charsize_arg(
                                            &mut csarg,
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ptr,
                                        );
                                        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(ptr);
                                        let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while ci.ptr < endptr {
                                            if !ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
                                                (*curwin.get()).w_wcol = vcol;
                                            }
                                            vcol += win_charsize(
                                                cstype,
                                                vcol,
                                                ci.ptr,
                                                ci.chr.value,
                                                &mut csarg,
                                            )
                                            .width;
                                            ci = utfc_next(ci);
                                        }
                                        (*curwin.get()).w_wrow = (*curwin.get()).w_cline_row
                                            + (*curwin.get()).w_wcol / (*curwin.get()).w_view_width;
                                        (*curwin.get()).w_wcol %= (*curwin.get()).w_view_width;
                                        (*curwin.get()).w_wcol += win_col_off(curwin.get());
                                        col = 0 as ::core::ffi::c_int as colnr_T;
                                    } else {
                                        (*curwin.get()).w_wcol -= 1;
                                        col = ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int)
                                            as colnr_T;
                                    }
                                } else if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
                                    && (*curwin.get()).w_wrow != 0
                                {
                                    (*curwin.get()).w_wrow -= 1;
                                    (*curwin.get()).w_wcol =
                                        (*curwin.get()).w_view_width - 1 as ::core::ffi::c_int;
                                    col = ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as colnr_T;
                                }
                                if col > 0 as ::core::ffi::c_int
                                    && (*curwin.get()).w_wcol > 0 as ::core::ffi::c_int
                                {
                                    ptr = get_cursor_line_ptr();
                                    col -= utf_head_off(ptr, ptr.offset(col as isize));
                                    if utf_ptr2cells(ptr.offset(col as isize))
                                        > 1 as ::core::ffi::c_int
                                    {
                                        (*curwin.get()).w_wcol -= 1;
                                    }
                                }
                            }
                            setcursor();
                            ui_flush();
                            new_wcol = (*curwin.get()).w_wcol;
                            new_wrow = (*curwin.get()).w_wrow;
                            (*curwin.get()).w_wcol = old_wcol;
                            (*curwin.get()).w_wrow = old_wrow;
                        }
                        if c < 0 as ::core::ffi::c_int {
                            continue;
                        }
                        let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        while n <= c {
                            *(*typebuf.ptr())
                                .tb_noremap
                                .offset(((*typebuf.ptr()).tb_off + n) as isize) =
                                RM_YES as ::core::ffi::c_int as uint8_t;
                            n += 1;
                        }
                        (*typebuf.ptr()).tb_len += c;
                        if (*typebuf.ptr()).tb_len
                            >= (*typebuf.ptr()).tb_maplen + MAXMAPLEN as ::core::ffi::c_int
                        {
                            timedout = true_0 != 0;
                        } else if ex_normal_busy.get() > 0 as ::core::ffi::c_int {
                            static tc: GlobalCell<::core::ffi::c_int> =
                                GlobalCell::new(0 as ::core::ffi::c_int);
                            if (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
                                timedout = true_0 != 0;
                            } else {
                                c = if State.get() & MODE_CMDLINE != 0
                                    || cmdwin_type.get() > 0 as ::core::ffi::c_int
                                        && tc.get() == ESC
                                {
                                    Ctrl_C
                                } else {
                                    ESC
                                };
                                tc.set(c);
                                if advance {
                                    typebuf_was_empty.set(true_0 != 0);
                                }
                                if pending_exmode_active.get() {
                                    exmode_active.set(true_0 != 0);
                                }
                                (*typebuf.ptr()).tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
                                break;
                            }
                        } else {
                            if (State.get() & MODE_INSERT != 0 as ::core::ffi::c_int
                                || p_lz.get() != 0)
                                && State.get() & MODE_CMDLINE == 0 as ::core::ffi::c_int
                                && advance as ::core::ffi::c_int != 0
                                && must_redraw.get() != 0 as ::core::ffi::c_int
                                && !need_wait_return.get()
                            {
                                update_screen();
                                setcursor();
                            }
                            let mut showcmd_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut showing_partial: bool = false_0 != 0;
                            if (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int
                                && advance as ::core::ffi::c_int != 0
                                && !exmode_active.get()
                            {
                                if (State.get() & (MODE_NORMAL | MODE_INSERT) != 0
                                    || State.get() == MODE_LANGMAP)
                                    && State.get() != MODE_HITRETURN
                                {
                                    if State.get() & MODE_INSERT != 0
                                        && ptr2cells(
                                            ((*typebuf.ptr()).tb_buf as *mut ::core::ffi::c_char)
                                                .offset((*typebuf.ptr()).tb_off as isize)
                                                .offset((*typebuf.ptr()).tb_len as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize)),
                                        ) == 1 as ::core::ffi::c_int
                                    {
                                        edit_putchar(
                                            *(*typebuf.ptr()).tb_buf.offset(
                                                ((*typebuf.ptr()).tb_off + (*typebuf.ptr()).tb_len
                                                    - 1 as ::core::ffi::c_int)
                                                    as isize,
                                            )
                                                as ::core::ffi::c_int,
                                            false_0 != 0,
                                        );
                                        setcursor();
                                        showing_partial = true_0 != 0;
                                    }
                                    let mut old_wcol_0: ::core::ffi::c_int = (*curwin.get()).w_wcol;
                                    let mut old_wrow_0: ::core::ffi::c_int = (*curwin.get()).w_wrow;
                                    (*curwin.get()).w_wcol = new_wcol;
                                    (*curwin.get()).w_wrow = new_wrow;
                                    push_showcmd();
                                    if (*typebuf.ptr()).tb_len > SHOWCMD_COLS as ::core::ffi::c_int
                                    {
                                        showcmd_idx = (*typebuf.ptr()).tb_len
                                            - SHOWCMD_COLS as ::core::ffi::c_int;
                                    }
                                    while showcmd_idx < (*typebuf.ptr()).tb_len {
                                        let c2rust_fresh5 = showcmd_idx;
                                        showcmd_idx = showcmd_idx + 1;
                                        add_byte_to_showcmd(*(*typebuf.ptr()).tb_buf.offset(
                                            ((*typebuf.ptr()).tb_off + c2rust_fresh5) as isize,
                                        ));
                                    }
                                    (*curwin.get()).w_wcol = old_wcol_0;
                                    (*curwin.get()).w_wrow = old_wrow_0;
                                }
                                if State.get() & MODE_CMDLINE != 0
                                    && !(*get_cmdline_info()).cmdbuff.is_null()
                                    && cmdline_star.get() == 0 as ::core::ffi::c_int
                                {
                                    let mut p: *mut ::core::ffi::c_char = ((*typebuf.ptr()).tb_buf
                                        as *mut ::core::ffi::c_char)
                                        .offset((*typebuf.ptr()).tb_off as isize)
                                        .offset((*typebuf.ptr()).tb_len as isize)
                                        .offset(-(1 as ::core::ffi::c_int as isize));
                                    if ptr2cells(p) == 1 as ::core::ffi::c_int
                                        && (*p as uint8_t as ::core::ffi::c_int)
                                            < 128 as ::core::ffi::c_int
                                    {
                                        putcmdline(*p, false_0 != 0);
                                        showing_partial = true_0 != 0;
                                    }
                                }
                            }
                            if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int {
                                timedout = false_0 != 0;
                            }
                            let mut wait_time: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if advance {
                                if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
                                    || !(p_timeout.get() != 0
                                        || p_ttimeout.get() != 0
                                            && keylen == KEYLEN_PART_KEY as ::core::ffi::c_int)
                                {
                                    wait_time = -1 as ::core::ffi::c_int;
                                } else if keylen == KEYLEN_PART_KEY as ::core::ffi::c_int
                                    && p_ttm.get() >= 0 as OptInt
                                {
                                    wait_time = p_ttm.get() as ::core::ffi::c_int;
                                } else {
                                    wait_time = p_tm.get() as ::core::ffi::c_int;
                                }
                            }
                            let mut wait_tb_len: ::core::ffi::c_int = (*typebuf.ptr()).tb_len;
                            c = inchar(
                                (*typebuf.ptr())
                                    .tb_buf
                                    .offset((*typebuf.ptr()).tb_off as isize)
                                    .offset((*typebuf.ptr()).tb_len as isize),
                                (*typebuf.ptr()).tb_buflen
                                    - (*typebuf.ptr()).tb_off
                                    - (*typebuf.ptr()).tb_len
                                    - 1 as ::core::ffi::c_int,
                                wait_time as ::core::ffi::c_long,
                            );
                            if showcmd_idx != 0 as ::core::ffi::c_int {
                                pop_showcmd();
                            }
                            if showing_partial as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                                if State.get() & MODE_INSERT != 0 {
                                    edit_unputchar();
                                }
                                if State.get() & MODE_CMDLINE != 0
                                    && !(*get_cmdline_info()).cmdbuff.is_null()
                                {
                                    unputcmdline();
                                } else {
                                    setcursor();
                                }
                            }
                            if c < 0 as ::core::ffi::c_int {
                                continue;
                            }
                            if c == NUL {
                                if !advance {
                                    break;
                                }
                                if wait_tb_len <= 0 as ::core::ffi::c_int {
                                    continue;
                                }
                                timedout = true_0 != 0;
                            } else {
                                while *(*typebuf.ptr()).tb_buf.offset(
                                    ((*typebuf.ptr()).tb_off + (*typebuf.ptr()).tb_len) as isize,
                                ) as ::core::ffi::c_int
                                    != NUL
                                {
                                    let c2rust_fresh6 = (*typebuf.ptr()).tb_len;
                                    (*typebuf.ptr()).tb_len = (*typebuf.ptr()).tb_len + 1;
                                    *(*typebuf.ptr()).tb_noremap.offset(
                                        ((*typebuf.ptr()).tb_off + c2rust_fresh6) as isize,
                                    ) = RM_YES as ::core::ffi::c_int as uint8_t;
                                }
                            }
                        }
                    }
                }
            }
            if !(c < 0 as ::core::ffi::c_int || advance as ::core::ffi::c_int != 0 && c == NUL) {
                break;
            }
        }
        if advance as ::core::ffi::c_int != 0
            && p_smd.get() != 0
            && msg_silent.get() == 0 as ::core::ffi::c_int
            && State.get() & MODE_INSERT != 0
        {
            if c == ESC
                && !mode_deleted
                && no_mapping.get() == 0
                && mode_displayed.get() as ::core::ffi::c_int != 0
            {
                if (*typebuf.ptr()).tb_len != 0 && !KeyTyped.get() {
                    redraw_cmdline.set(true_0 != 0);
                } else {
                    unshowmode(false_0 != 0);
                }
            } else if c != ESC && mode_deleted as ::core::ffi::c_int != 0 {
                if (*typebuf.ptr()).tb_len != 0 && !KeyTyped.get() {
                    redraw_cmdline.set(true_0 != 0);
                } else {
                    showmode();
                }
            }
        }
        if timedout as ::core::ffi::c_int != 0 && c == ESC {
            gotchars_ignore();
        }
        (*vgetc_busy.ptr()) -= 1;
        return c;
    }
}

pub(crate) unsafe extern "C" fn inchar(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
    mut wait_time: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut retesc: ::core::ffi::c_int = false_0;
        let tb_change_cnt: ::core::ffi::c_int = (*typebuf.ptr()).tb_change_cnt;
        if wait_time == -1 as ::core::ffi::c_long || wait_time > 100 as ::core::ffi::c_long {
            ui_flush();
        }
        if State.get() != MODE_HITRETURN {
            did_outofmem_msg.set(false_0 != 0);
            did_swapwrite_msg.set(false_0 != 0);
        }
        let mut read_size: ptrdiff_t = -1 as ptrdiff_t;
        while curscript.get() >= 0 as ::core::ffi::c_int
            && read_size <= 0 as ptrdiff_t
            && !ignore_script.get()
        {
            let mut script_char: ::core::ffi::c_char = 0;
            if got_int.get() as ::core::ffi::c_int != 0 || {
                read_size = file_read(
                    (scriptin.ptr() as *mut FileDescriptor).offset(curscript.get() as isize),
                    &raw mut script_char,
                    1 as size_t,
                );
                read_size != 1 as ptrdiff_t
            } {
                closescript();
                if got_int.get() {
                    retesc = true_0;
                } else {
                    return -1 as ::core::ffi::c_int;
                }
            } else {
                *buf.offset(0 as ::core::ffi::c_int as isize) = script_char as uint8_t;
                len = 1 as ::core::ffi::c_int;
            }
        }
        if read_size <= 0 as ptrdiff_t {
            if got_int.get() {
                let mut dum: [uint8_t; 154] = [0; 154];
                loop {
                    len = input_get(
                        &raw mut dum as *mut uint8_t,
                        MAXMAPLEN as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                            + 3 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<MultiQueue>(),
                    );
                    if len == 0 as ::core::ffi::c_int
                        || len == 1 as ::core::ffi::c_int
                            && dum[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == Ctrl_C
                    {
                        break;
                    }
                }
                return retesc;
            }
            if wait_time == -1 as ::core::ffi::c_long || wait_time > 10 as ::core::ffi::c_long {
                ui_flush();
            }
            len = input_get(
                buf,
                maxlen / 3 as ::core::ffi::c_int,
                wait_time as ::core::ffi::c_int,
                tb_change_cnt,
                ::core::ptr::null_mut::<MultiQueue>(),
            );
        }
        if typebuf_changed(tb_change_cnt) {
            return 0 as ::core::ffi::c_int;
        }
        if len > 0 as ::core::ffi::c_int && {
            (*typebuf.ptr()).tb_change_cnt += 1;
            (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int
        } {
            (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
        }
        return fix_input_buffer(buf, len);
    }
}

pub unsafe extern "C" fn fix_input_buffer(
    mut buf: *mut uint8_t,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if using_script() == 0 {
            *buf.offset(len as isize) = NUL as uint8_t;
            return len;
        }
        let mut p: *mut uint8_t = buf;
        let mut i: ::core::ffi::c_int = len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
                    && (i < 2 as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != KS_EXTRA)
            {
                memmove(
                    p.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    i as size_t,
                );
                *p.offset(2 as ::core::ffi::c_int as isize) =
                    (if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == K_SPECIAL
                        || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    {
                        KE_FILLER as ::core::ffi::c_uint
                    } else {
                        -(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_uint
                            >> 8 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_uint
                    }) as uint8_t;
                *p.offset(1 as ::core::ffi::c_int as isize) = (if *p
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == K_SPECIAL
                {
                    KS_SPECIAL
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                    KS_ZERO
                } else {
                    -(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        & 0xff as ::core::ffi::c_int
                }) as uint8_t;
                *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
                p = p.offset(2 as ::core::ffi::c_int as isize);
                len += 2 as ::core::ffi::c_int;
            }
            p = p.offset(1);
        }
        *p = NUL as uint8_t;
        return len;
    }
}
