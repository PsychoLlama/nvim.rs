//! Commands that change the buffer text or the cursor, including
//! `:normal`, which re-enters the normal-mode state machine.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ex_print(mut eap: *mut exarg_T) {
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        emsg(gettext(&raw const e_empty_buffer as *const c_char));
    } else {
        let mut line: linenr_T = (*eap).line1;
        while line <= (*eap).line2 && !got_int.get() {
            print_line(
                line,
                (*eap).cmdidx as c_int == CMD_number as c_int
                    || (*eap).cmdidx as c_int == CMD_pound as c_int
                    || (*eap).flags & EXFLAG_NR != 0,
                (*eap).cmdidx as c_int == CMD_list as c_int || (*eap).flags & EXFLAG_LIST != 0,
                line == (*eap).line1,
            );
            line += 1;
            os_breakcheck();
        }
        setpcmark();
        (*curwin.get()).w_cursor.lnum = (*eap).line2;
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    }
    ex_no_reprint.set(true_0 != 0);
}

pub(crate) unsafe extern "C" fn ex_goto(mut eap: *mut exarg_T) {
    goto_byte((*eap).line2 as c_int);
}

pub(crate) unsafe extern "C" fn ex_syncbind(mut _eap: *mut exarg_T) {
    let mut vtopline: linenr_T = 0;
    let mut old_linenr: linenr_T = (*curwin.get()).w_cursor.lnum;
    setpcmark();
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        vtopline = get_vtopline(curwin.get()) as linenr_T;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_scb != 0 && !(*wp).w_buffer.is_null() {
                let mut y: linenr_T =
                    plines_m_win_fill(wp, 1 as linenr_T, (*(*wp).w_buffer).b_ml.ml_line_count)
                        as linenr_T
                        - get_scrolloff_value(curwin.get()) as linenr_T;
                vtopline = if vtopline < y { vtopline } else { y };
            }
            wp = (*wp).w_next;
        }
        vtopline = if vtopline > 1 as linenr_T {
            vtopline
        } else {
            1 as linenr_T
        };
    } else {
        vtopline = 1 as c_int as linenr_T;
    }
    let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_0.is_null() {
        if (*wp_0).w_onebuf_opt.wo_scb != 0 {
            let mut y_0: c_int = vtopline as c_int - get_vtopline(wp_0);
            if y_0 > 0 as c_int {
                scrollup(wp_0, y_0 as linenr_T, true_0 != 0);
            } else {
                scrolldown(wp_0, -(y_0 as linenr_T), true_0);
            }
            (*wp_0).w_scbind_pos = vtopline as c_int;
            redraw_later(wp_0, UPD_VALID as c_int);
            cursor_correct(wp_0);
            (*wp_0).w_redr_status = true_0 != 0;
        }
        wp_0 = (*wp_0).w_next;
    }
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        did_syncbind.set(true_0 != 0);
        checkpcmark();
        if old_linenr != (*curwin.get()).w_cursor.lnum {
            let mut ctrl_o: [c_char; 2] = [0; 2];
            ctrl_o[0 as c_int as usize] = Ctrl_O as c_char;
            ctrl_o[1 as c_int as usize] = 0 as c_char;
            ins_typebuf(
                &raw mut ctrl_o as *mut c_char,
                REMAP_NONE as c_int,
                0 as c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn ex_equal(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int != NUL && *(*eap).arg as c_int != '|' as c_int {
        ex_lua(eap);
    } else {
        (*eap).nextcmd = find_nextcmd((*eap).arg);
        smsg(
            0 as c_int,
            b"%ld\0".as_ptr() as *const c_char,
            (*eap).line2 as int64_t,
        );
    };
}

pub(crate) unsafe extern "C" fn ex_sleep(mut eap: *mut exarg_T) {
    if cursor_valid(curwin.get()) != 0 {
        setcursor_mayforce(curwin.get(), true_0 != 0);
    }
    let mut len: int64_t = (*eap).line2 as int64_t;
    match *(*eap).arg as c_int {
        109 => {}
        NUL => {
            len *= 1000 as int64_t;
        }
        _ => {
            semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
            return;
        }
    }
    do_sleep(len, (*eap).forceit != 0);
}

pub unsafe extern "C" fn do_sleep(mut msec: int64_t, mut hide_cursor: bool) {
    if hide_cursor {
        ui_busy_start();
    }
    ui_flush();
    process_events_until(main_loop.ptr(), (*main_loop.ptr()).events, msec, || {
        got_int.get()
    });
    if got_int.get() {
        vpeekc();
    }
    if hide_cursor {
        ui_busy_stop();
    }
}

pub(crate) unsafe extern "C" fn ex_operators(mut eap: *mut exarg_T) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    clear_oparg(&raw mut oa);
    oa.regname = (*eap).regname;
    oa.start.lnum = (*eap).line1;
    oa.end.lnum = (*eap).line2;
    oa.line_count = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
    oa.motion_type = kMTLineWise;
    virtual_op.set(kFalse);
    if (*eap).cmdidx as c_int != CMD_yank as c_int {
        setpcmark();
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    }
    if VIsual_active.get() {
        end_visual_mode();
    }
    match (*eap).cmdidx as c_int {
        109 => {
            oa.op_type = OP_DELETE as c_int;
            op_delete(&raw mut oa);
        }
        546 => {
            oa.op_type = OP_YANK as c_int;
            op_yank(&raw mut oa, true_0 != 0);
        }
        _ => {
            if ((*eap).cmdidx as c_int == CMD_rshift as c_int) as c_int
                ^ (*curwin.get()).w_onebuf_opt.wo_rl
                != 0
            {
                oa.op_type = OP_RSHIFT as c_int;
            } else {
                oa.op_type = OP_LSHIFT as c_int;
            }
            op_shift(&raw mut oa, false_0 != 0, (*eap).amount);
        }
    }
    virtual_op.set(kNone);
    ex_may_print(eap);
}

pub(crate) unsafe extern "C" fn ex_put(mut eap: *mut exarg_T) {
    if (*eap).line2 == 0 as linenr_T {
        (*eap).line2 = 1 as c_int as linenr_T;
        (*eap).forceit = true_0;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    do_put(
        (*eap).regname,
        ::core::ptr::null_mut::<yankreg_T>(),
        if (*eap).forceit != 0 {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        },
        1 as c_int,
        PUT_LINE as c_int | PUT_CURSLINE as c_int,
    );
}

pub(crate) unsafe extern "C" fn ex_iput(mut eap: *mut exarg_T) {
    if (*eap).line2 == 0 as linenr_T {
        (*eap).line2 = 1 as c_int as linenr_T;
        (*eap).forceit = true_0;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    do_put(
        (*eap).regname,
        ::core::ptr::null_mut::<yankreg_T>(),
        if (*eap).forceit != 0 {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        },
        1 as c_int,
        PUT_LINE as c_int | PUT_CURSLINE as c_int | PUT_FIXINDENT as c_int,
    );
}

pub(crate) unsafe extern "C" fn ex_copymove(mut eap: *mut exarg_T) {
    let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut n: linenr_T = get_address(
        eap,
        &raw mut (*eap).arg,
        (*eap).addr_type,
        false_0 != 0,
        false_0 != 0,
        false_0,
        1 as c_int,
        &raw mut errormsg,
    );
    if (*eap).arg.is_null() {
        if !errormsg.is_null() {
            emsg(errormsg);
        }
        (*eap).nextcmd = ::core::ptr::null_mut::<c_char>();
        return;
    }
    get_flags(eap);
    if n == MAXLNUM as c_int as linenr_T
        || n < 0 as linenr_T
        || n > (*curbuf.get()).b_ml.ml_line_count
    {
        emsg(gettext(&raw const e_invrange as *const c_char));
        return;
    }
    if (*eap).cmdidx as c_int == CMD_move as c_int {
        if do_move((*eap).line1, (*eap).line2, n) == FAIL {
            return;
        }
    } else {
        ex_copy((*eap).line1, (*eap).line2, n);
    }
    u_clearline(curbuf.get());
    beginline(BL_SOL as c_int | BL_FIX as c_int);
    ex_may_print(eap);
}

pub unsafe extern "C" fn ex_may_print(mut eap: *mut exarg_T) {
    if (*eap).flags != 0 as c_int {
        print_line(
            (*curwin.get()).w_cursor.lnum,
            (*eap).flags & EXFLAG_NR != 0,
            (*eap).flags & EXFLAG_LIST != 0,
            true_0 != 0,
        );
        ex_no_reprint.set(true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn ex_submagic(mut eap: *mut exarg_T) {
    let saved: optmagic_T = magic_overruled.get();
    magic_overruled.set(
        (if (*eap).cmdidx as c_int == CMD_smagic as c_int {
            OPTION_MAGIC_ON as c_int
        } else {
            OPTION_MAGIC_OFF as c_int
        }) as optmagic_T,
    );
    ex_substitute(eap);
    magic_overruled.set(saved);
}

pub(crate) unsafe extern "C" fn ex_submagic_preview(
    mut eap: *mut exarg_T,
    mut cmdpreview_ns: c_int,
    mut cmdpreview_bufnr: handle_T,
) -> c_int {
    let saved: optmagic_T = magic_overruled.get();
    magic_overruled.set(
        (if (*eap).cmdidx as c_int == CMD_smagic as c_int {
            OPTION_MAGIC_ON as c_int
        } else {
            OPTION_MAGIC_OFF as c_int
        }) as optmagic_T,
    );
    let mut retv: c_int = ex_substitute_preview(eap, cmdpreview_ns, cmdpreview_bufnr);
    magic_overruled.set(saved);
    return retv;
}

pub(crate) unsafe extern "C" fn ex_join(mut eap: *mut exarg_T) {
    (*curwin.get()).w_cursor.lnum = (*eap).line1;
    if (*eap).line1 == (*eap).line2 {
        if (*eap).addr_count >= 2 as c_int {
            return;
        }
        if (*eap).line2 == (*curbuf.get()).b_ml.ml_line_count {
            beep_flush();
            return;
        }
        (*eap).line2 += 1;
    }
    do_join(
        ((*eap).line2 as ssize_t - (*eap).line1 as ssize_t + 1 as ssize_t) as size_t,
        (*eap).forceit == 0,
        true_0 != 0,
        true_0 != 0,
        true_0 != 0,
    );
    beginline(BL_WHITE as c_int | BL_FIX as c_int);
    ex_may_print(eap);
}

pub(crate) unsafe extern "C" fn ex_at(mut eap: *mut exarg_T) {
    let mut prev_len: c_int = (*typebuf.ptr()).tb_len;
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    let mut c: c_int = *(*eap).arg as uint8_t as c_int;
    if c == NUL {
        c = '@' as c_int;
    }
    if do_execreg(
        c,
        true_0,
        !vim_strchr(p_cpo.get(), CPO_EXECBUF).is_null() as c_int,
        true_0,
    ) == FAIL
    {
        beep_flush();
        return;
    }
    let save_efr: bool = exec_from_reg.get();
    exec_from_reg.set(true_0 != 0);
    while !stuff_empty() || (*typebuf.ptr()).tb_len > prev_len {
        do_cmdline(
            ::core::ptr::null_mut::<c_char>(),
            Some(getexline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
            NULL_1,
            DOCMD_NOWAIT as c_int | DOCMD_VERBOSE as c_int,
        );
    }
    exec_from_reg.set(save_efr);
}

pub(crate) unsafe extern "C" fn ex_undo(mut eap: *mut exarg_T) {
    if (*eap).addr_count != 1 as c_int {
        if (*eap).forceit != 0 {
            u_undo_and_forget(1 as c_int, true_0 != 0);
        } else {
            u_undo(1 as c_int);
        }
        return;
    }
    let mut step: linenr_T = (*eap).line2;
    if (*eap).forceit != 0 {
        if step >= (*curbuf.get()).b_u_seq_cur as linenr_T {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const c_char,
            ));
            return;
        }
        let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
        let mut count: c_int = 0 as c_int;
        uhp = if !(*curbuf.get()).b_u_curhead.is_null() {
            (*curbuf.get()).b_u_curhead
        } else {
            (*curbuf.get()).b_u_newhead
        };
        while !uhp.is_null() && (*uhp).uh_seq as linenr_T > step {
            uhp = (*uhp).uh_next.ptr;
            count += 1;
        }
        if step != 0 as linenr_T && (uhp.is_null() || ((*uhp).uh_seq as linenr_T) < step) {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const c_char,
            ));
            return;
        }
        u_undo_and_forget(count, true_0 != 0);
    } else {
        undo_time(step as c_int, false_0 != 0, false_0 != 0, true_0 != 0);
    };
}

pub(crate) unsafe extern "C" fn ex_redo(mut _eap: *mut exarg_T) {
    u_redo(1 as c_int);
}

pub(crate) unsafe extern "C" fn ex_later(mut eap: *mut exarg_T) {
    let mut count: c_int = 0 as c_int;
    let mut sec: bool = false_0 != 0;
    let mut file: bool = false_0 != 0;
    let mut p: *mut c_char = (*eap).arg;
    if *p as c_int == NUL {
        count = 1 as c_int;
    } else if *(*__ctype_b_loc()).offset(*p as uint8_t as c_int as isize) as c_int
        & _ISdigit as c_int as c_ushort as c_int
        != 0
    {
        count = getdigits_int(&raw mut p, false_0 != 0, 0 as c_int);
        match *p as c_int {
            115 => {
                p = p.offset(1);
                sec = true_0 != 0;
            }
            109 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 60 as c_int;
            }
            104 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 60 as c_int * 60 as c_int;
            }
            100 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 24 as c_int * 60 as c_int * 60 as c_int;
            }
            102 => {
                p = p.offset(1);
                file = true_0 != 0;
            }
            _ => {}
        }
    }
    if *p as c_int != NUL {
        semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
    } else {
        undo_time(
            if (*eap).cmdidx as c_int == CMD_earlier as c_int {
                -count
            } else {
                count
            },
            sec,
            file,
            false_0 != 0,
        );
    };
}

pub(crate) unsafe extern "C" fn ex_mark(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        emsg(gettext(&raw const e_argreq as *const c_char));
        return;
    }
    if *(*eap).arg.offset(1 as c_int as isize) as c_int != NUL {
        semsg(
            gettext(&raw const e_trailing_arg as *const c_char),
            (*eap).arg,
        );
        return;
    }
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    beginline(BL_WHITE as c_int | BL_FIX as c_int);
    if setmark(*(*eap).arg as c_int) == FAIL {
        emsg(gettext(
            b"E191: Argument must be a letter or forward/backward quote\0".as_ptr()
                as *const c_char,
        ));
    }
    (*curwin.get()).w_cursor = pos;
}

pub unsafe extern "C" fn update_topline_cursor() {
    check_cursor(curwin.get());
    update_topline(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
        validate_cursor(curwin.get());
    }
    update_curswant();
}

pub unsafe extern "C" fn save_current_state(mut sst: *mut save_state_T) -> bool {
    (*sst).save_msg_scroll = msg_scroll.get();
    (*sst).save_restart_edit = restart_edit.get();
    (*sst).save_msg_didout = msg_didout.get();
    (*sst).save_State = State.get();
    (*sst).save_finish_op = finish_op.get();
    (*sst).save_opcount = opcount.get();
    (*sst).save_reg_executing = reg_executing.get();
    (*sst).save_pending_end_reg_executing = pending_end_reg_executing.get();
    msg_scroll.set(false_0);
    restart_edit.set(0 as c_int);
    save_typeahead(&raw mut (*sst).tabuf);
    return (*sst).tabuf.typebuf_valid;
}

pub unsafe extern "C" fn restore_current_state(mut sst: *mut save_state_T) {
    restore_typeahead(&raw mut (*sst).tabuf);
    msg_scroll.set((*sst).save_msg_scroll);
    if force_restart_edit.get() {
        force_restart_edit.set(false_0 != 0);
    } else {
        restart_edit.set((*sst).save_restart_edit);
    }
    finish_op.set((*sst).save_finish_op);
    opcount.set((*sst).save_opcount);
    reg_executing.set((*sst).save_reg_executing);
    pending_end_reg_executing.set((*sst).save_pending_end_reg_executing);
    msg_didout.set(msg_didout.get() as c_int | (*sst).save_msg_didout as c_int != 0);
    State.set((*sst).save_State);
    ui_cursor_shape();
}

pub(crate) unsafe extern "C" fn ex_normal(mut eap: *mut exarg_T) {
    if !(*curbuf.get()).terminal.is_null() && State.get() & MODE_TERMINAL as c_int != 0 {
        emsg(b"Can't re-enter normal mode from terminal mode\0".as_ptr() as *const c_char);
        return;
    }
    let mut arg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if expr_map_locked() {
        emsg(gettext(&raw const e_secure as *const c_char));
        return;
    }
    if ex_normal_busy.get() as OptInt >= p_mmd.get() {
        emsg(gettext(
            b"E192: Recursive use of :normal too deep\0".as_ptr() as *const c_char,
        ));
        return;
    }
    let mut len: c_int = 0 as c_int;
    let mut l: c_int = 0;
    let mut p: *mut c_char = (*eap).arg;
    while *p as c_int != NUL {
        l = utfc_ptr2len(p) - 1 as c_int;
        while l > 0 as c_int {
            p = p.offset(1);
            if *p as c_int == K_SPECIAL as c_char as c_int {
                len += 2 as c_int;
            }
            l -= 1;
        }
        p = p.offset(1);
    }
    if len > 0 as c_int {
        arg = xmalloc(
            strlen((*eap).arg)
                .wrapping_add(len as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut c_char;
        len = 0 as c_int;
        let mut p_0: *mut c_char = (*eap).arg;
        while *p_0 as c_int != NUL {
            let c2rust_fresh17 = len;
            len = len + 1;
            *arg.offset(c2rust_fresh17 as isize) = *p_0;
            l = utfc_ptr2len(p_0) - 1 as c_int;
            while l > 0 as c_int {
                p_0 = p_0.offset(1);
                let c2rust_fresh18 = len;
                len = len + 1;
                *arg.offset(c2rust_fresh18 as isize) = *p_0;
                if *p_0 as c_int == K_SPECIAL as c_char as c_int {
                    let c2rust_fresh19 = len;
                    len = len + 1;
                    *arg.offset(c2rust_fresh19 as isize) = KS_SPECIAL as c_char;
                    let c2rust_fresh20 = len;
                    len = len + 1;
                    *arg.offset(c2rust_fresh20 as isize) = KE_FILLER as c_char;
                }
                l -= 1;
            }
            *arg.offset(len as isize) = NUL as c_char;
            p_0 = p_0.offset(1);
        }
    }
    (*ex_normal_busy.ptr()) += 1;
    let mut save_state: save_state_T = save_state_T {
        save_msg_scroll: 0,
        save_restart_edit: 0,
        save_msg_didout: false,
        save_State: 0,
        save_finish_op: false,
        save_opcount: 0,
        save_reg_executing: 0,
        save_pending_end_reg_executing: false,
        tabuf: tasave_T {
            save_typebuf: typebuf_T {
                tb_buf: ::core::ptr::null_mut::<uint8_t>(),
                tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
                tb_buflen: 0,
                tb_off: 0,
                tb_len: 0,
                tb_maplen: 0,
                tb_silent: 0,
                tb_no_abbr_cnt: 0,
                tb_change_cnt: 0,
            },
            typebuf_valid: false,
            old_char: 0,
            old_mod_mask: 0,
            save_readbuf1: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            save_readbuf2: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            save_inputbuf: String_0 {
                data: ::core::ptr::null_mut::<c_char>(),
                size: 0,
            },
        },
    };
    if save_current_state(&raw mut save_state) {
        loop {
            if (*eap).addr_count != 0 as c_int {
                let c2rust_fresh21 = (*eap).line1;
                (*eap).line1 = (*eap).line1 + 1;
                (*curwin.get()).w_cursor.lnum = c2rust_fresh21;
                (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
                check_cursor_moved(curwin.get());
            }
            exec_normal_cmd(
                if !arg.is_null() { arg } else { (*eap).arg },
                if (*eap).forceit != 0 {
                    REMAP_NONE as c_int
                } else {
                    REMAP_YES as c_int
                },
                false_0 != 0,
            );
            if !((*eap).addr_count > 0 as c_int && (*eap).line1 <= (*eap).line2 && !got_int.get()) {
                break;
            }
        }
    }
    update_topline_cursor();
    restore_current_state(&raw mut save_state);
    (*ex_normal_busy.ptr()) -= 1;
    setmouse();
    ui_cursor_shape();
    xfree(arg as *mut c_void);
}

pub(crate) unsafe extern "C" fn ex_startinsert(mut eap: *mut exarg_T) {
    if (*eap).forceit != 0 {
        if (*curwin.get()).w_cursor.lnum == 0 {
            (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        }
        set_cursor_for_append_to_line();
    }
    if State.get() & MODE_INSERT as c_int != 0 {
        return;
    }
    if (*eap).cmdidx as c_int == CMD_startinsert as c_int {
        restart_edit.set('a' as c_int);
    } else if (*eap).cmdidx as c_int == CMD_startreplace as c_int {
        restart_edit.set('R' as c_int);
    } else {
        restart_edit.set('V' as c_int);
    }
    if (*eap).forceit == 0 {
        if (*eap).cmdidx as c_int == CMD_startinsert as c_int {
            restart_edit.set('i' as c_int);
        }
        (*curwin.get()).w_curswant = 0 as c_int as colnr_T;
    }
    if VIsual_active.get() {
        showmode();
    }
}

pub(crate) unsafe extern "C" fn ex_stopinsert(mut _eap: *mut exarg_T) {
    restart_edit.set(0 as c_int);
    stop_insert_mode.set(true_0 != 0);
    clearmode();
}

pub unsafe extern "C" fn exec_normal_cmd(mut cmd: *mut c_char, mut remap: c_int, mut silent: bool) {
    ins_typebuf(cmd, remap, 0 as c_int, true_0 != 0, silent);
    exec_normal(false_0 != 0, false_0 != 0);
}

pub unsafe extern "C" fn exec_normal(mut was_typed: bool, mut use_vpeekc: bool) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    let mut c: c_int = 0;
    clear_oparg(&raw mut oa);
    finish_op.set(false_0 != 0);
    while (!stuff_empty()
        || (was_typed as c_int != 0 || typebuf_typed() == 0)
            && (*typebuf.ptr()).tb_len > 0 as c_int
        || use_vpeekc as c_int != 0
            && {
                c = vpeekc();
                c != NUL
            }
            && c != Ctrl_C)
        && !got_int.get()
    {
        update_topline_cursor();
        normal_cmd(&raw mut oa, true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn ex_fold(mut eap: *mut exarg_T) {
    if foldManualAllowed(true_0 != 0) != 0 {
        let mut start: pos_T = pos_T {
            lnum: (*eap).line1,
            col: 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        let mut end: pos_T = pos_T {
            lnum: (*eap).line2,
            col: 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        foldCreate(curwin.get(), start, end);
    }
}

pub(crate) unsafe extern "C" fn ex_foldopen(mut eap: *mut exarg_T) {
    let mut start: pos_T = pos_T {
        lnum: (*eap).line1,
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let mut end: pos_T = pos_T {
        lnum: (*eap).line2,
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    opFoldRange(
        start,
        end,
        ((*eap).cmdidx as c_int == CMD_foldopen as c_int) as c_int,
        (*eap).forceit,
        false_0 != 0,
    );
}

pub(crate) unsafe extern "C" fn ex_folddo(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= (*eap).line2 {
        if hasFolding(
            curwin.get(),
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            ::core::ptr::null_mut::<linenr_T>(),
        ) as c_int
            == ((*eap).cmdidx as c_int == CMD_folddoclosed as c_int) as c_int
        {
            ml_setmarked(lnum);
        }
        lnum += 1;
    }
    global_exe((*eap).arg);
    ml_clearmarked();
}
