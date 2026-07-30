//! Ex ranges: one address, the pair around a `,` or `;`, and the
//! defaults a command supplies when it was given none.
//!
//! An address is not always a line number — `cmd_addr_type` decides whether it
//! counts lines, windows, buffers, arguments, tab pages or quickfix entries, and
//! the same syntax means different things for each.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn compute_buffer_local_count(
    mut addr_type: cmd_addr_T,
    mut lnum: linenr_T,
    mut offset: c_int,
) -> c_int {
    let mut count: c_int = offset;
    let mut buf: *mut buf_T = firstbuf.get();
    while !(*buf).b_next.is_null() && ((*buf).handle as linenr_T) < lnum {
        buf = (*buf).b_next;
    }
    while count != 0 as c_int {
        count += if count < 0 as c_int {
            1 as c_int
        } else {
            -1 as c_int
        };
        let mut nextbuf: *mut buf_T = if offset < 0 as c_int {
            (*buf).b_prev
        } else {
            (*buf).b_next
        };
        if nextbuf.is_null() {
            break;
        }
        buf = nextbuf;
        if addr_type as c_uint == ADDR_LOADED_BUFFERS as c_int as c_uint {
            while (*buf).b_ml.ml_mfp.is_null() {
                nextbuf = if offset < 0 as c_int {
                    (*buf).b_prev
                } else {
                    (*buf).b_next
                };
                if nextbuf.is_null() {
                    break;
                }
                buf = nextbuf;
            }
        }
    }
    if addr_type as c_uint == ADDR_LOADED_BUFFERS as c_int as c_uint {
        while (*buf).b_ml.ml_mfp.is_null() {
            let mut nextbuf_0: *mut buf_T = if offset >= 0 as c_int {
                (*buf).b_prev
            } else {
                (*buf).b_next
            };
            if nextbuf_0.is_null() {
                break;
            }
            buf = nextbuf_0;
        }
    }
    return (*buf).handle as c_int;
}

pub(crate) unsafe extern "C" fn get_wincmd_addr_type(
    mut arg: *const c_char,
    mut eap: *mut exarg_T,
) {
    match *arg as c_int {
        83 | Ctrl_S | 115 | Ctrl_N | 110 | 106 | Ctrl_J | 107 | Ctrl_K | 84 | Ctrl_R | 114 | 82
        | 75 | 74 | 43 | 45 | Ctrl__ | 95 | 124 | 93 | Ctrl_RSB | 103 | Ctrl_G | Ctrl_V | 118
        | 104 | Ctrl_H | 108 | Ctrl_L | 72 | 76 | 62 | 60 | 125 | 102 | 70 | Ctrl_F | 105
        | Ctrl_I | 100 | Ctrl_D => {
            (*eap).addr_type = ADDR_OTHER;
        }
        Ctrl_HAT | 94 => {
            (*eap).addr_type = ADDR_BUFFERS;
        }
        Ctrl_Q | 113 | Ctrl_C | 99 | Ctrl_O | 111 | Ctrl_W | 119 | 87 | 120 | Ctrl_X => {
            (*eap).addr_type = ADDR_WINDOWS;
        }
        Ctrl_Z | 122 | 80 | 116 | Ctrl_T | 98 | Ctrl_B | 112 | Ctrl_P | 61 | CAR => {
            (*eap).addr_type = ADDR_NONE;
        }
        _ => {}
    };
}

pub unsafe extern "C" fn set_cmd_addr_type(mut eap: *mut exarg_T, mut p: *mut c_char) {
    if ((*eap).cmdidx as c_int) < 0 as c_int {
        return;
    }
    if (*eap).cmdidx as c_int != CMD_SIZE as c_int {
        (*eap).addr_type = (*cmdnames.ptr())[(*eap).cmdidx as c_int as usize].cmd_addr_type;
    } else {
        (*eap).addr_type = ADDR_LINES;
    }
    if (*eap).cmdidx as c_int == CMD_wincmd as c_int && !p.is_null() {
        get_wincmd_addr_type(skipwhite(p), eap);
    }
    if ((*eap).cmdidx as c_int == CMD_cc as c_int || (*eap).cmdidx as c_int == CMD_ll as c_int)
        && bt_quickfix(curbuf.get()) as c_int != 0
    {
        (*eap).addr_type = ADDR_OTHER;
    }
}

pub unsafe extern "C" fn get_cmd_default_range(mut eap: *mut exarg_T) -> linenr_T {
    match (*eap).addr_type as c_uint {
        0 | 10 => {
            return if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
        }
        1 => return current_win_nr(curwin.get()) as linenr_T,
        2 => {
            return if ((*curwin.get()).w_arg_idx + 1 as c_int)
                < (*(*curwin.get()).w_alist).al_ga.ga_len
            {
                (*curwin.get()).w_arg_idx as linenr_T + 1 as linenr_T
            } else {
                (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T
            };
        }
        3 | 4 => return (*curbuf.get()).handle as linenr_T,
        5 => return current_tab_nr(curtab.get()) as linenr_T,
        6 | 9 => return 1 as linenr_T,
        8 => return qf_get_cur_idx(eap) as linenr_T,
        7 => return qf_get_cur_valid_idx(eap) as linenr_T,
        _ => return 0 as linenr_T,
    };
}

pub unsafe extern "C" fn set_cmd_dflall_range(mut eap: *mut exarg_T) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    (*eap).line1 = 1 as c_int as linenr_T;
    match (*eap).addr_type as c_uint {
        0 | 10 => {
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
        3 => {
            buf = firstbuf.get();
            while !(*buf).b_next.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = (*buf).b_next;
            }
            (*eap).line1 = (*buf).handle as linenr_T;
            buf = lastbuf.get();
            while !(*buf).b_prev.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = (*buf).b_prev;
            }
            (*eap).line2 = (*buf).handle as linenr_T;
        }
        4 => {
            (*eap).line1 = (*firstbuf.get()).handle as linenr_T;
            (*eap).line2 = (*lastbuf.get()).handle as linenr_T;
        }
        1 => {
            (*eap).line2 = current_win_nr(::core::ptr::null::<win_T>()) as linenr_T;
        }
        5 => {
            (*eap).line2 = current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T;
        }
        6 => {
            (*eap).line2 = 1 as c_int as linenr_T;
        }
        2 => {
            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as c_int {
                (*eap).line2 = 0 as c_int as linenr_T;
                (*eap).line1 = (*eap).line2;
            } else {
                (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
            }
        }
        7 => {
            (*eap).line2 = qf_get_valid_size(eap) as linenr_T;
            if (*eap).line2 == 0 as linenr_T {
                (*eap).line2 = 1 as c_int as linenr_T;
            }
        }
        11 | 9 | 8 => {
            iemsg(gettext(
                b"INTERNAL: Cannot use EX_DFLALL with ADDR_NONE, ADDR_UNSIGNED or ADDR_QUICKFIX\0"
                    .as_ptr() as *const c_char,
            ));
        }
        _ => {}
    };
}

pub(crate) unsafe extern "C" fn find_excmd_after_range(mut eap: *mut exarg_T) -> *mut c_char {
    let mut cmd: *mut c_char = (*eap).cmd;
    (*eap).cmd = skip_range((*eap).cmd, ::core::ptr::null_mut::<c_int>());
    let mut p: *mut c_char = find_ex_command(eap, ::core::ptr::null_mut::<c_int>());
    (*eap).cmd = cmd;
    return p;
}

pub unsafe extern "C" fn parse_cmd_address(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut silent: bool,
) -> c_int {
    let mut address_count: c_int = 1 as c_int;
    let mut lnum: linenr_T = 0;
    let mut need_check_cursor: bool = false_0 != 0;
    let mut ret: c_int = FAIL;
    '_theend: {
        loop {
            (*eap).line1 = (*eap).line2;
            (*eap).line2 = get_cmd_default_range(eap);
            (*eap).cmd = skipwhite((*eap).cmd);
            let c2rust_fresh29 = address_count;
            address_count = address_count + 1;
            lnum = get_address(
                eap,
                &raw mut (*eap).cmd,
                (*eap).addr_type,
                (*eap).skip != 0,
                silent,
                ((*eap).addr_count == 0 as c_int) as c_int,
                c2rust_fresh29,
                errormsg,
            );
            if (*eap).cmd.is_null() {
                break '_theend;
            }
            if lnum == MAXLNUM as c_int as linenr_T {
                if *(*eap).cmd as c_int == '%' as c_int {
                    (*eap).cmd = (*eap).cmd.offset(1);
                    match (*eap).addr_type as c_uint {
                        0 | 10 => {
                            (*eap).line1 = 1 as c_int as linenr_T;
                            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
                        }
                        3 => {
                            let mut buf: *mut buf_T = firstbuf.get();
                            while !(*buf).b_next.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                                buf = (*buf).b_next;
                            }
                            (*eap).line1 = (*buf).handle as linenr_T;
                            buf = lastbuf.get();
                            while !(*buf).b_prev.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                                buf = (*buf).b_prev;
                            }
                            (*eap).line2 = (*buf).handle as linenr_T;
                        }
                        4 => {
                            (*eap).line1 = (*firstbuf.get()).handle as linenr_T;
                            (*eap).line2 = (*lastbuf.get()).handle as linenr_T;
                        }
                        1 | 5 => {
                            if ((*eap).cmdidx as c_int) < 0 as c_int {
                                (*eap).line1 = 1 as c_int as linenr_T;
                                (*eap).line2 = (if (*eap).addr_type as c_uint
                                    == ADDR_WINDOWS as c_int as c_uint
                                {
                                    current_win_nr(::core::ptr::null::<win_T>())
                                } else {
                                    current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                                }) as linenr_T;
                            } else {
                                *errormsg = gettext(&raw const e_invrange as *const c_char);
                                break '_theend;
                            }
                        }
                        6 | 9 | 8 => {
                            *errormsg = gettext(&raw const e_invrange as *const c_char);
                            break '_theend;
                        }
                        2 => {
                            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as c_int {
                                (*eap).line2 = 0 as c_int as linenr_T;
                                (*eap).line1 = (*eap).line2;
                            } else {
                                (*eap).line1 = 1 as c_int as linenr_T;
                                (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
                            }
                        }
                        7 => {
                            (*eap).line1 = 1 as c_int as linenr_T;
                            (*eap).line2 = qf_get_valid_size(eap) as linenr_T;
                            if (*eap).line2 == 0 as linenr_T {
                                (*eap).line2 = 1 as c_int as linenr_T;
                            }
                        }
                        11 | _ => {}
                    }
                    (*eap).addr_count += 1;
                } else if *(*eap).cmd as c_int == '*' as c_int {
                    if (*eap).addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                        *errormsg = gettext(&raw const e_invrange as *const c_char);
                        break '_theend;
                    } else {
                        (*eap).cmd = (*eap).cmd.offset(1);
                        if (*eap).skip == 0 {
                            let mut fm: *mut fmark_T = mark_get_visual(curbuf.get(), '<' as c_int);
                            if !mark_check(fm, errormsg) {
                                break '_theend;
                            }
                            '_c2rust_label: {
                                if !fm.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm != NULL\0".as_ptr() as *const c_char,
                                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                                        3027 as c_uint,
                                        b"int parse_cmd_address(exarg_T *, const char **, _Bool)\0"
                                            .as_ptr()
                                            as *const c_char,
                                    );
                                }
                            };
                            (*eap).line1 = (*fm).mark.lnum;
                            fm = mark_get_visual(curbuf.get(), '>' as c_int);
                            if !mark_check(fm, errormsg) {
                                break '_theend;
                            }
                            '_c2rust_label_0: {
                                if !fm.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm != NULL\0".as_ptr() as *const c_char,
                                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                                        3033 as c_uint,
                                        b"int parse_cmd_address(exarg_T *, const char **, _Bool)\0"
                                            .as_ptr()
                                            as *const c_char,
                                    );
                                }
                            };
                            (*eap).line2 = (*fm).mark.lnum;
                            (*eap).addr_count += 1;
                        }
                    }
                }
            } else {
                (*eap).line2 = lnum;
            }
            (*eap).addr_count += 1;
            if *(*eap).cmd as c_int == ';' as c_int {
                if (*eap).skip == 0 {
                    (*curwin.get()).w_cursor.lnum = (*eap).line2;
                    if (*eap).line2 > 0 as linenr_T {
                        check_cursor(curwin.get());
                    } else {
                        check_cursor_col(curwin.get());
                    }
                    need_check_cursor = true_0 != 0;
                }
            } else if *(*eap).cmd as c_int != ',' as c_int {
                break;
            }
            (*eap).cmd = (*eap).cmd.offset(1);
        }
        if (*eap).addr_count == 1 as c_int {
            (*eap).line1 = (*eap).line2;
            if lnum == MAXLNUM as c_int as linenr_T {
                (*eap).addr_count = 0 as c_int;
            }
        }
        ret = OK;
    }
    if need_check_cursor {
        check_cursor(curwin.get());
    }
    return ret;
}

pub unsafe extern "C" fn skip_range(mut cmd: *const c_char, mut ctx: *mut c_int) -> *mut c_char {
    while !vim_strchr(
        b" \t0123456789.$%'/?-+,;\\\0".as_ptr() as *const c_char,
        *cmd as uint8_t as c_int,
    )
    .is_null()
    {
        if *cmd as c_int == '\\' as c_int {
            if !(*cmd.offset(1 as c_int as isize) as c_int == '?' as c_int
                || *cmd.offset(1 as c_int as isize) as c_int == '/' as c_int
                || *cmd.offset(1 as c_int as isize) as c_int == '&' as c_int)
            {
                break;
            }
            cmd = cmd.offset(1);
        } else if *cmd as c_int == '\'' as c_int {
            cmd = cmd.offset(1);
            if *cmd as c_int == NUL && !ctx.is_null() {
                *ctx = EXPAND_NOTHING as c_int;
            }
        } else if *cmd as c_int == '/' as c_int || *cmd as c_int == '?' as c_int {
            let c2rust_fresh27 = cmd;
            cmd = cmd.offset(1);
            let mut delim: c_uint = *c2rust_fresh27 as c_uint;
            while *cmd as c_int != NUL && *cmd as c_int != delim as c_char as c_int {
                let c2rust_fresh28 = cmd;
                cmd = cmd.offset(1);
                if *c2rust_fresh28 as c_int == '\\' as c_int && *cmd as c_int != NUL {
                    cmd = cmd.offset(1);
                }
            }
            if *cmd as c_int == NUL && !ctx.is_null() {
                *ctx = EXPAND_NOTHING as c_int;
            }
        }
        if *cmd as c_int != NUL {
            cmd = cmd.offset(1);
        }
    }
    cmd = skip_colon_white(cmd, false_0 != 0);
    if *cmd as c_int == '*' as c_int {
        cmd = skipwhite(cmd.offset(1 as c_int as isize));
    }
    return cmd as *mut c_char;
}

pub(crate) unsafe extern "C" fn addr_error(mut addr_type: cmd_addr_T) -> *const c_char {
    if addr_type as c_uint == ADDR_NONE as c_int as c_uint {
        return gettext(&raw const e_norange as *const c_char);
    } else {
        return gettext(&raw const e_invrange as *const c_char);
    };
}

pub unsafe extern "C" fn get_address(
    mut eap: *mut exarg_T,
    mut ptr: *mut *mut c_char,
    mut addr_type: cmd_addr_T,
    mut skip: bool,
    mut silent: bool,
    mut to_other_file: c_int,
    mut address_count: c_int,
    mut errormsg: *mut *const c_char,
) -> linenr_T {
    let mut c: c_int = 0;
    let mut i: c_int = 0;
    let mut n: linenr_T = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut cmd: *mut c_char = skipwhite(*ptr);
    let mut lnum: linenr_T = MAXLNUM as c_int as linenr_T;
    '_error: loop {
        match *cmd as c_int {
            46 => {
                cmd = cmd.offset(1);
                match addr_type as c_uint {
                    0 | 10 => {
                        lnum = (*curwin.get()).w_cursor.lnum;
                    }
                    1 => {
                        lnum = current_win_nr(curwin.get()) as linenr_T;
                    }
                    2 => {
                        lnum = ((*curwin.get()).w_arg_idx + 1 as c_int) as linenr_T;
                    }
                    3 | 4 => {
                        lnum = (*curbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(curtab.get()) as linenr_T;
                    }
                    11 | 6 | 9 => {
                        *errormsg = addr_error(addr_type);
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    }
                    8 => {
                        lnum = qf_get_cur_idx(eap) as linenr_T;
                    }
                    7 => {
                        lnum = qf_get_cur_valid_idx(eap) as linenr_T;
                    }
                    _ => {}
                }
            }
            36 => {
                cmd = cmd.offset(1);
                match addr_type as c_uint {
                    0 | 10 => {
                        lnum = (*curbuf.get()).b_ml.ml_line_count;
                    }
                    1 => {
                        lnum = current_win_nr(::core::ptr::null::<win_T>()) as linenr_T;
                    }
                    2 => {
                        lnum = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
                    }
                    3 => {
                        buf = lastbuf.get();
                        while (*buf).b_ml.ml_mfp.is_null() {
                            if (*buf).b_prev.is_null() {
                                break;
                            }
                            buf = (*buf).b_prev;
                        }
                        lnum = (*buf).handle as linenr_T;
                    }
                    4 => {
                        lnum = (*lastbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T;
                    }
                    11 | 6 | 9 => {
                        *errormsg = addr_error(addr_type);
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    }
                    8 => {
                        lnum = qf_get_size(eap) as linenr_T;
                        if lnum == 0 as linenr_T {
                            lnum = 1 as c_int as linenr_T;
                        }
                    }
                    7 => {
                        lnum = qf_get_valid_size(eap) as linenr_T;
                        if lnum == 0 as linenr_T {
                            lnum = 1 as c_int as linenr_T;
                        }
                    }
                    _ => {}
                }
            }
            39 => {
                cmd = cmd.offset(1);
                if *cmd as c_int == NUL {
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else if addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else if skip {
                    cmd = cmd.offset(1);
                } else {
                    let mut flag: MarkGet = (if to_other_file != 0
                        && *cmd.offset(1 as c_int as isize) as c_int == NUL
                    {
                        kMarkAll as c_int
                    } else {
                        kMarkBufLocal as c_int
                    }) as MarkGet;
                    let mut fm: *mut fmark_T = mark_get(
                        curbuf.get(),
                        curwin.get(),
                        ::core::ptr::null_mut::<fmark_T>(),
                        flag,
                        *cmd as c_int,
                    );
                    cmd = cmd.offset(1);
                    if !fm.is_null() && (*fm).fnum != (*curbuf.get()).handle {
                        mark_move_to(fm, 0 as MarkMove);
                        lnum = (*curwin.get()).w_cursor.lnum;
                    } else if !mark_check(fm, errormsg) {
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    } else {
                        '_c2rust_label: {
                            if !fm.is_null() {
                            } else {
                                __assert_fail(
                                    b"fm != NULL\0".as_ptr() as *const c_char,
                                    b"src/nvim/ex_docmd.rs\0"
                                        .as_ptr() as *const c_char,
                                    3618 as c_uint,
                                    b"linenr_T get_address(exarg_T *, char **, cmd_addr_T, _Bool, _Bool, int, int, const char **)\0"
                                        .as_ptr() as *const c_char,
                                );
                            }
                        };
                        lnum = (*fm).mark.lnum;
                    }
                }
            }
            47 | 63 => {
                let c2rust_fresh2 = cmd;
                cmd = cmd.offset(1);
                c = *c2rust_fresh2 as uint8_t as c_int;
                if addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else if skip {
                    cmd = skip_regexp(cmd, c, magic_isset() as c_int);
                    if *cmd as c_int == c {
                        cmd = cmd.offset(1);
                    }
                } else {
                    let mut flags: c_int = 0;
                    pos = (*curwin.get()).w_cursor;
                    if lnum > 0 as linenr_T && lnum != MAXLNUM as c_int as linenr_T {
                        (*curwin.get()).w_cursor.lnum = if lnum > (*curbuf.get()).b_ml.ml_line_count
                        {
                            (*curbuf.get()).b_ml.ml_line_count
                        } else {
                            lnum
                        };
                    }
                    (*curwin.get()).w_cursor.col =
                        (if c == '/' as c_int && (*curwin.get()).w_cursor.lnum > 0 as linenr_T {
                            MAXCOL as c_int
                        } else {
                            0 as c_int
                        }) as colnr_T;
                    searchcmdlen.set(0 as c_int);
                    flags = if silent as c_int != 0 {
                        SEARCH_KEEP as c_int
                    } else {
                        SEARCH_HIS as c_int | SEARCH_MSG as c_int
                    };
                    if do_search(
                        ::core::ptr::null_mut::<oparg_T>(),
                        c,
                        c,
                        cmd,
                        strlen(cmd),
                        1 as c_int,
                        flags,
                        ::core::ptr::null_mut::<searchit_arg_T>(),
                    ) == 0
                    {
                        (*curwin.get()).w_cursor = pos;
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    } else {
                        lnum = (*curwin.get()).w_cursor.lnum;
                        (*curwin.get()).w_cursor = pos;
                        cmd = cmd.offset(searchcmdlen.get() as isize);
                    }
                }
            }
            92 => {
                cmd = cmd.offset(1);
                if addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else {
                    if *cmd as c_int == '&' as c_int {
                        i = RE_SUBST as c_int;
                    } else if *cmd as c_int == '?' as c_int || *cmd as c_int == '/' as c_int {
                        i = RE_SEARCH as c_int;
                    } else {
                        *errormsg = gettext(&raw const e_backslash as *const c_char);
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    }
                    if !skip {
                        pos.lnum = if lnum != MAXLNUM as c_int as linenr_T {
                            lnum
                        } else {
                            (*curwin.get()).w_cursor.lnum
                        };
                        pos.col = (if *cmd as c_int != '?' as c_int {
                            MAXCOL as c_int
                        } else {
                            0 as c_int
                        }) as colnr_T;
                        pos.coladd = 0 as c_int as colnr_T;
                        if searchit(
                            curwin.get(),
                            curbuf.get(),
                            &raw mut pos,
                            ::core::ptr::null_mut::<pos_T>(),
                            (if *cmd as c_int == '?' as c_int {
                                BACKWARD as c_int
                            } else {
                                FORWARD as c_int
                            }) as Direction,
                            b"\0".as_ptr() as *const c_char as *mut c_char,
                            0 as size_t,
                            1 as c_int,
                            SEARCH_MSG as c_int,
                            i,
                            ::core::ptr::null_mut::<searchit_arg_T>(),
                        ) != FAIL
                        {
                            lnum = pos.lnum;
                        } else {
                            cmd = ::core::ptr::null_mut::<c_char>();
                            break;
                        }
                    }
                    cmd = cmd.offset(1);
                }
            }
            _ => {
                if ascii_isdigit(*cmd as c_int) {
                    lnum = getdigits(&raw mut cmd, false_0 != 0, 0 as intmax_t) as linenr_T;
                }
            }
        }
        loop {
            cmd = skipwhite(cmd);
            if *cmd as c_int != '-' as c_int
                && *cmd as c_int != '+' as c_int
                && !ascii_isdigit(*cmd as c_int)
            {
                break;
            }
            if lnum == MAXLNUM as c_int as linenr_T {
                match addr_type as c_uint {
                    0 | 10 => {
                        lnum = (*curwin.get()).w_cursor.lnum;
                    }
                    1 => {
                        lnum = current_win_nr(curwin.get()) as linenr_T;
                    }
                    2 => {
                        lnum = ((*curwin.get()).w_arg_idx + 1 as c_int) as linenr_T;
                    }
                    3 | 4 => {
                        lnum = (*curbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(curtab.get()) as linenr_T;
                    }
                    6 => {
                        lnum = 1 as c_int as linenr_T;
                    }
                    8 => {
                        lnum = qf_get_cur_idx(eap) as linenr_T;
                    }
                    7 => {
                        lnum = qf_get_cur_valid_idx(eap) as linenr_T;
                    }
                    11 | 9 => {
                        lnum = 0 as c_int as linenr_T;
                    }
                    _ => {}
                }
            }
            if ascii_isdigit(*cmd as c_int) {
                i = '+' as c_int;
            } else {
                let c2rust_fresh3 = cmd;
                cmd = cmd.offset(1);
                i = *c2rust_fresh3 as uint8_t as c_int;
            }
            if !ascii_isdigit(*cmd as c_int) {
                n = 1 as c_int as linenr_T;
            } else {
                n = getdigits_int32(&raw mut cmd, false_0 != 0, MAXLNUM as c_int as int32_t)
                    as linenr_T;
                if n == MAXLNUM as c_int as linenr_T {
                    *errormsg = gettext(&raw const e_line_number_out_of_range as *const c_char);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break '_error;
                }
            }
            if addr_type as c_uint == ADDR_TABS_RELATIVE as c_int as c_uint {
                *errormsg = gettext(&raw const e_invrange as *const c_char);
                cmd = ::core::ptr::null_mut::<c_char>();
                break '_error;
            } else if addr_type as c_uint == ADDR_LOADED_BUFFERS as c_int as c_uint
                || addr_type as c_uint == ADDR_BUFFERS as c_int as c_uint
            {
                lnum = compute_buffer_local_count(
                    addr_type,
                    lnum,
                    if i == '-' as c_int {
                        -1 as c_int * n as c_int
                    } else {
                        n as c_int
                    },
                ) as linenr_T;
            } else {
                if addr_type as c_uint == ADDR_LINES as c_int as c_uint
                    && (i == '-' as c_int || i == '+' as c_int)
                    && address_count >= 2 as c_int
                {
                    hasFolding(
                        curwin.get(),
                        lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    );
                }
                if i == '-' as c_int {
                    lnum -= n;
                } else if lnum >= 0 as linenr_T && n >= INT32_MAX as linenr_T - lnum {
                    *errormsg = gettext(&raw const e_line_number_out_of_range as *const c_char);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break '_error;
                } else {
                    lnum += n;
                }
            }
        }
        if !(*cmd as c_int == '/' as c_int || *cmd as c_int == '?' as c_int) {
            break;
        }
    }
    *ptr = cmd;
    return lnum;
}

pub unsafe extern "C" fn invalid_range(mut eap: *mut exarg_T) -> *mut c_char {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*eap).line1 < 0 as linenr_T || (*eap).line2 < 0 as linenr_T || (*eap).line1 > (*eap).line2 {
        return gettext(&raw const e_invrange as *const c_char);
    }
    if (*eap).argt & EX_RANGE as uint32_t != 0 {
        match (*eap).addr_type as c_uint {
            0 => {
                if (*eap).line2
                    > (*curbuf.get()).b_ml.ml_line_count
                        + ((*eap).cmdidx as c_int == CMD_diffget as c_int
                            || (*eap).cmdidx as c_int == CMD_diffput as c_int)
                            as c_int
                {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            2 => {
                if (*eap).line2
                    > (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T
                        + ((*(*curwin.get()).w_alist).al_ga.ga_len == 0) as c_int
                {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            4 => {
                if (*eap).line1 < 1 as linenr_T || (*eap).line2 > get_highest_fnum() as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            3 => {
                buf = firstbuf.get();
                while (*buf).b_ml.ml_mfp.is_null() {
                    if (*buf).b_next.is_null() {
                        return gettext(&raw const e_invrange as *const c_char);
                    }
                    buf = (*buf).b_next;
                }
                if (*eap).line1 < (*buf).handle as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
                buf = lastbuf.get();
                while (*buf).b_ml.ml_mfp.is_null() {
                    if (*buf).b_prev.is_null() {
                        return gettext(&raw const e_invrange as *const c_char);
                    }
                    buf = (*buf).b_prev;
                }
                if (*eap).line2 > (*buf).handle as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            1 => {
                if (*eap).line2 > current_win_nr(::core::ptr::null::<win_T>()) as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            5 => {
                if (*eap).line2 > current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            8 => {
                '_c2rust_label: {
                    if (*eap).line2 >= 0 as linenr_T {
                    } else {
                        __assert_fail(
                            b"eap->line2 >= 0\0".as_ptr() as *const c_char,
                            b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                            3906 as c_uint,
                            b"char *invalid_range(exarg_T *)\0".as_ptr() as *const c_char,
                        );
                    }
                };
                if (*eap).line2 <= 0 as linenr_T {
                    if (*eap).addr_count == 0 as c_int {
                        return gettext(&raw const e_no_errors as *const c_char);
                    }
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            7 => {
                if (*eap).line2 != 1 as linenr_T && (*eap).line2 as size_t > qf_get_valid_size(eap)
                    || (*eap).line2 < 0 as linenr_T
                {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            6 | 10 | 9 | 11 | _ => {}
        }
    }
    return ::core::ptr::null_mut::<c_char>();
}

pub(crate) unsafe extern "C" fn correct_range(mut eap: *mut exarg_T) {
    if (*eap).argt & EX_ZEROR as uint32_t == 0 {
        if (*eap).line1 == 0 as linenr_T {
            (*eap).line1 = 1 as c_int as linenr_T;
        }
        if (*eap).line2 == 0 as linenr_T {
            (*eap).line2 = 1 as c_int as linenr_T;
        }
    }
}
