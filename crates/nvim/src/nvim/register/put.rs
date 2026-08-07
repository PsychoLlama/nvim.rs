//! `do_put` -- `p`, `P`, `gp`, `gP`, `]p`, `[p` and `zp`.
//!
//! Still one 1,094-line function here, and still over the file cap: a carve
//! cannot split a single over-cap item, so this file is a holding pen until
//! the rewrite decomposes it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_put(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        let mut split_pos: colnr_T = 0;
        let mut col: colnr_T = 0;
        let mut len_0: ::core::ffi::c_int = 0;
        let mut totlen: size_t = 0 as size_t;
        let mut lnum: linenr_T = 0 as linenr_T;
        let mut y_type: MotionType = kMTCharWise;
        let mut y_size: size_t = 0;
        let mut y_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut vcol: colnr_T = 0 as colnr_T;
        let mut y_array: *mut String_0 = ::core::ptr::null_mut::<String_0>();
        let mut nr_lines: linenr_T = 0 as linenr_T;
        let mut allocated: bool = false;
        let orig_start: pos_T = (*curbuf.get()).b_op_start;
        let orig_end: pos_T = (*curbuf.get()).b_op_end;
        let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(curwin.get());
        if ins_compl_preinsert_effect() {
            ins_compl_delete(false);
        }
        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        if regname == '.' as ::core::ffi::c_int && reg.is_null() {
            let mut non_linewise_vis: bool = VIsual_active.get() as ::core::ffi::c_int != 0
                && VIsual_mode.get() != 'V' as ::core::ffi::c_int;
            let mut command_start_char: ::core::ffi::c_char =
                (if non_linewise_vis as ::core::ffi::c_int != 0 {
                    'c' as ::core::ffi::c_int
                } else if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                    'i' as ::core::ffi::c_int
                } else if dir == FORWARD as ::core::ffi::c_int {
                    'a' as ::core::ffi::c_int
                } else {
                    'i' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
            if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                do_put(
                    '_' as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<yankreg_T>(),
                    dir,
                    1 as ::core::ffi::c_int,
                    PUT_LINE as ::core::ffi::c_int,
                );
            }
            if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                stuffcharReadbuff(command_start_char as ::core::ffi::c_int);
                while count > 0 as ::core::ffi::c_int {
                    stuff_inserted(
                        NUL,
                        1 as ::core::ffi::c_int,
                        (count != 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                    );
                    if count != 1 as ::core::ffi::c_int {
                        stuffReadbuff(c"\n ".as_ptr());
                        stuffcharReadbuff(Ctrl_U);
                    }
                    count -= 1;
                }
            } else {
                stuff_inserted(command_start_char as ::core::ffi::c_int, count, false_0);
            }
            if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                    stuffReadbuff(c"j0".as_ptr());
                } else {
                    let mut cursor_pos: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                    let mut one_past_line: bool = *cursor_pos as ::core::ffi::c_int == NUL;
                    let mut eol: bool = false;
                    if !one_past_line {
                        eol = *cursor_pos.offset(utfc_ptr2len(cursor_pos) as isize)
                            as ::core::ffi::c_int
                            == NUL;
                    }
                    let mut ve_allows: bool = cur_ve_flags
                        == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        || cur_ve_flags
                            == kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint;
                    let mut eof: bool = (*curbuf.get()).b_ml.ml_line_count
                        == (*curwin.get()).w_cursor.lnum
                        && one_past_line as ::core::ffi::c_int != 0;
                    if ve_allows as ::core::ffi::c_int != 0
                        || !(eol as ::core::ffi::c_int != 0 || eof as ::core::ffi::c_int != 0)
                    {
                        stuffcharReadbuff('l' as ::core::ffi::c_int);
                    }
                }
            } else if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                stuffReadbuff(c"g'[".as_ptr());
            }
            if command_start_char as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                if u_save(
                    (*curwin.get()).w_cursor.lnum,
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                ) == FAIL
                {
                    return;
                }
            }
            return;
        }
        let mut insert_string: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
        if reg.is_null()
            && get_spec_reg(
                regname,
                &raw mut insert_string.data,
                &raw mut allocated,
                true,
            ) as ::core::ffi::c_int
                != 0
        {
            if insert_string.data.is_null() {
                return;
            }
        }
        if (*curbuf.get()).terminal.is_null() {
            if u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            ) == FAIL
            {
                return;
            }
        }
        if !insert_string.data.is_null() {
            insert_string.size = strlen(insert_string.data);
            y_type = kMTCharWise;
            if regname == '=' as ::core::ffi::c_int {
                loop {
                    y_size = 0 as size_t;
                    let mut ptr: *mut ::core::ffi::c_char = insert_string.data;
                    let mut ptrlen: size_t = insert_string.size;
                    while !ptr.is_null() {
                        if !y_array.is_null() {
                            (*y_array.add(y_size)).data = ptr;
                        }
                        y_size = y_size.wrapping_add(1);
                        let mut tmp: *mut ::core::ffi::c_char =
                            vim_strchr(ptr, '\n' as ::core::ffi::c_int);
                        if tmp.is_null() {
                            if !y_array.is_null() {
                                (*y_array.add(y_size.wrapping_sub(1 as size_t))).size = ptrlen;
                            }
                        } else {
                            if !y_array.is_null() {
                                *tmp = NUL as ::core::ffi::c_char;
                                (*y_array.add(y_size.wrapping_sub(1 as size_t))).size =
                                    tmp.offset_from(ptr) as size_t;
                                ptrlen = ptrlen.wrapping_sub(
                                    (*y_array.add(y_size.wrapping_sub(1 as size_t)))
                                        .size
                                        .wrapping_add(1 as size_t),
                                );
                            }
                            tmp = tmp.offset(1);
                            if *tmp as ::core::ffi::c_int == NUL {
                                y_type = kMTLineWise;
                                break;
                            }
                        }
                        ptr = tmp;
                    }
                    if !y_array.is_null() {
                        break;
                    }
                    y_array = xmalloc(y_size.wrapping_mul(::core::mem::size_of::<String_0>()))
                        as *mut String_0;
                }
            } else {
                y_size = 1 as size_t;
                y_array = &raw mut insert_string;
            }
        } else {
            if reg.is_null() {
                reg = get_yank_register(regname, YREG_PASTE);
            }
            y_type = (*reg).y_type;
            y_width = (*reg).y_width as ::core::ffi::c_int;
            y_size = (*reg).y_size;
            y_array = (*reg).y_array;
        }
        '_end: {
            if !(*curbuf.get()).terminal.is_null() {
                terminal_paste(count, y_array, y_size);
            } else {
                split_pos = 0 as colnr_T;
                if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                    if flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0 {
                        if u_save_cursor() == FAIL {
                            break '_end;
                        } else {
                            let mut curline: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                            let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                            let p_orig: *mut ::core::ffi::c_char = p;
                            let plen: size_t = get_cursor_pos_len() as size_t;
                            if dir == FORWARD as ::core::ffi::c_int
                                && *p as ::core::ffi::c_int != NUL
                            {
                                p = p.offset(utfc_ptr2len(p) as isize);
                            }
                            split_pos = p.offset_from(curline) as colnr_T;
                            let mut ptr_0: *mut ::core::ffi::c_char = xmemdupz(
                                p as *const ::core::ffi::c_void,
                                plen.wrapping_sub(p.offset_from(p_orig) as size_t),
                            )
                                as *mut ::core::ffi::c_char;
                            ml_append((*curwin.get()).w_cursor.lnum, ptr_0, 0 as colnr_T, false);
                            xfree(ptr_0 as *mut ::core::ffi::c_void);
                            ptr_0 = xmemdupz(
                                get_cursor_line_ptr() as *const ::core::ffi::c_void,
                                split_pos as size_t,
                            ) as *mut ::core::ffi::c_char;
                            ml_replace((*curwin.get()).w_cursor.lnum, ptr_0, false);
                            nr_lines += 1;
                            dir = FORWARD as ::core::ffi::c_int;
                            buf_updates_send_changes(
                                curbuf.get(),
                                (*curwin.get()).w_cursor.lnum,
                                1 as int64_t,
                                1 as int64_t,
                            );
                        }
                    }
                    if flags & PUT_LINE_FORWARD as ::core::ffi::c_int != 0 {
                        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_end;
                        dir = FORWARD as ::core::ffi::c_int;
                    }
                    (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                }
                if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                    y_type = kMTLineWise;
                }
                if y_size == 0 as size_t || y_array.is_null() {
                    semsg(
                        gettext(c"E353: Nothing in register %s".as_ptr()),
                        if regname == 0 as ::core::ffi::c_int {
                            c"\"".as_ptr()
                        } else {
                            transchar(regname) as *const ::core::ffi::c_char
                        },
                    );
                } else {
                    if y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                        lnum = (*curwin.get()).w_cursor.lnum + y_size as linenr_T + 1 as linenr_T;
                        lnum = if lnum < (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T {
                            lnum
                        } else {
                            (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T
                        };
                        if u_save((*curwin.get()).w_cursor.lnum - 1 as linenr_T, lnum) == FAIL {
                            break '_end;
                        }
                    } else if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                        lnum = (*curwin.get()).w_cursor.lnum;
                        if dir == BACKWARD as ::core::ffi::c_int {
                            hasFolding(
                                curwin.get(),
                                lnum,
                                &raw mut lnum,
                                ::core::ptr::null_mut::<linenr_T>(),
                            );
                        } else {
                            hasFolding(
                                curwin.get(),
                                lnum,
                                ::core::ptr::null_mut::<linenr_T>(),
                                &raw mut lnum,
                            );
                        }
                        if dir == FORWARD as ::core::ffi::c_int {
                            lnum += 1;
                        }
                        if (if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0 {
                            u_save(0 as linenr_T, 2 as linenr_T)
                        } else {
                            u_save(lnum - 1 as linenr_T, lnum)
                        }) == FAIL
                        {
                            break '_end;
                        } else {
                            if dir == FORWARD as ::core::ffi::c_int {
                                (*curwin.get()).w_cursor.lnum = lnum - 1 as linenr_T;
                            } else {
                                (*curwin.get()).w_cursor.lnum = lnum;
                            }
                            (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                        }
                    } else if u_save_cursor() == FAIL {
                        break '_end;
                    }
                    if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        && y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                    {
                        if gchar_cursor() == TAB {
                            let mut viscol: ::core::ffi::c_int = getviscol();
                            let mut ts: OptInt = (*curbuf.get()).b_p_ts;
                            if if dir == FORWARD as ::core::ffi::c_int {
                                (tabstop_padding(
                                    viscol as colnr_T,
                                    ts,
                                    (*curbuf.get()).b_p_vts_array,
                                ) != 1 as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            } else {
                                ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            } != 0
                            {
                                coladvance_force(viscol as colnr_T);
                            } else {
                                (*curwin.get()).w_cursor.coladd =
                                    0 as ::core::ffi::c_int as colnr_T;
                            }
                        } else if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                            || gchar_cursor() == NUL
                        {
                            coladvance_force(
                                getviscol()
                                    + (dir == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
                            );
                        }
                    }
                    lnum = (*curwin.get()).w_cursor.lnum;
                    col = (*curwin.get()).w_cursor.col;
                    if y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                        let mut incr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut bd: block_def = block_def {
                            startspaces: 0,
                            endspaces: 0,
                            textlen: 0,
                            textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            textcol: 0,
                            start_vcol: 0,
                            end_vcol: 0,
                            is_short: 0,
                            is_MAX: 0,
                            is_oneChar: 0,
                            pre_whitesp: 0,
                            pre_whitesp_c: 0,
                            end_char_vcols: 0,
                            start_char_vcols: 0,
                        };
                        let mut c: ::core::ffi::c_int = gchar_cursor();
                        let mut endcol2: colnr_T = 0 as colnr_T;
                        if dir == FORWARD as ::core::ffi::c_int && c != NUL {
                            if cur_ve_flags
                                == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                getvcol(
                                    curwin.get(),
                                    &raw mut (*curwin.get()).w_cursor,
                                    &raw mut col,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    &raw mut endcol2,
                                );
                            } else {
                                getvcol(
                                    curwin.get(),
                                    &raw mut (*curwin.get()).w_cursor,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    &raw mut col,
                                );
                            }
                            (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                            col += 1;
                        } else {
                            getvcol(
                                curwin.get(),
                                &raw mut (*curwin.get()).w_cursor,
                                &raw mut col,
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut endcol2,
                            );
                        }
                        col += (*curwin.get()).w_cursor.coladd;
                        if cur_ve_flags
                            == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                            && ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                                || endcol2 == (*curwin.get()).w_cursor.col)
                        {
                            if dir == FORWARD as ::core::ffi::c_int && c == NUL {
                                col += 1;
                            }
                            if dir != FORWARD as ::core::ffi::c_int
                                && c != NUL
                                && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                            {
                                (*curwin.get()).w_cursor.col += 1;
                            }
                            if c == TAB {
                                if dir == BACKWARD as ::core::ffi::c_int
                                    && (*curwin.get()).w_cursor.col != 0
                                {
                                    (*curwin.get()).w_cursor.col -= 1;
                                }
                                if dir == FORWARD as ::core::ffi::c_int
                                    && col as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                        == endcol2
                                {
                                    (*curwin.get()).w_cursor.col += 1;
                                }
                            }
                        }
                        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        bd.textcol = 0 as ::core::ffi::c_int as colnr_T;
                        let mut i: size_t = 0 as size_t;
                        while i < y_size {
                            let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut shortline: ::core::ffi::c_char = 0;
                            let mut lines_appended: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            bd.startspaces = 0 as ::core::ffi::c_int;
                            bd.endspaces = 0 as ::core::ffi::c_int;
                            vcol = 0 as ::core::ffi::c_int as colnr_T;
                            let mut delcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
                                if ml_append(
                                    (*curbuf.get()).b_ml.ml_line_count,
                                    c"".as_ptr() as *mut ::core::ffi::c_char,
                                    1 as colnr_T,
                                    false,
                                ) == FAIL
                                {
                                    break;
                                }
                                nr_lines += 1;
                                lines_appended = 1 as ::core::ffi::c_int;
                            }
                            let mut oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                            let mut oldlen: colnr_T = get_cursor_line_len();
                            let mut csarg: CharsizeArg = CharsizeArg::default();
                            let mut cstype: CharsizeKind = init_charsize_arg(
                                &mut csarg,
                                curwin.get(),
                                (*curwin.get()).w_cursor.lnum,
                                oldp,
                            );
                            let mut ci: StrCharInfo = utf_ptr2StrCharInfo(oldp);
                            vcol = 0 as ::core::ffi::c_int as colnr_T;
                            while vcol < col && *ci.ptr as ::core::ffi::c_int != NUL {
                                incr = win_charsize(
                                    cstype,
                                    vcol as ::core::ffi::c_int,
                                    ci.ptr,
                                    ci.chr.value,
                                    &mut csarg,
                                )
                                .width;
                                vcol += incr;
                                ci = utfc_next(ci);
                            }
                            let mut ptr_1: *mut ::core::ffi::c_char = ci.ptr;
                            bd.textcol = ptr_1.offset_from(oldp) as colnr_T;
                            shortline = (vcol < col || vcol == col && *ptr_1 == 0)
                                as ::core::ffi::c_int
                                as ::core::ffi::c_char;
                            if vcol < col {
                                bd.startspaces = (col - vcol) as ::core::ffi::c_int;
                            } else if vcol > col {
                                bd.endspaces = (vcol - col) as ::core::ffi::c_int;
                                bd.startspaces = incr - bd.endspaces;
                                bd.textcol -= 1;
                                delcount = 1 as ::core::ffi::c_int;
                                bd.textcol -= utf_head_off(oldp, oldp.offset(bd.textcol as isize));
                                if *oldp.offset(bd.textcol as isize) as ::core::ffi::c_int != TAB {
                                    delcount = 0 as ::core::ffi::c_int;
                                    bd.endspaces = 0 as ::core::ffi::c_int;
                                }
                            }
                            let yanklen: ::core::ffi::c_int =
                                (*y_array.add(i)).size as ::core::ffi::c_int;
                            if flags & PUT_BLOCK_INNER as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                spaces = y_width + 1 as ::core::ffi::c_int;
                                cstype = init_charsize_arg(
                                    &mut csarg,
                                    curwin.get(),
                                    0 as linenr_T,
                                    (*y_array.add(i)).data,
                                );
                                ci = utf_ptr2StrCharInfo((*y_array.add(i)).data);
                                while *ci.ptr as ::core::ffi::c_int != NUL {
                                    spaces -= win_charsize(
                                        cstype,
                                        0 as ::core::ffi::c_int,
                                        ci.ptr,
                                        ci.chr.value,
                                        &mut csarg,
                                    )
                                    .width;
                                    ci = utfc_next(ci);
                                }
                                spaces = if spaces > 0 as ::core::ffi::c_int {
                                    spaces
                                } else {
                                    0 as ::core::ffi::c_int
                                };
                            }
                            if yanklen + spaces != 0 as ::core::ffi::c_int
                                && count
                                    > (INT_MAX - (bd.startspaces + bd.endspaces))
                                        / (yanklen + spaces)
                            {
                                emsg(gettext(
                                    &raw const e_resulting_text_too_long
                                        as *const ::core::ffi::c_char,
                                ));
                                break;
                            } else {
                                totlen = (count as size_t)
                                    .wrapping_mul((yanklen + spaces) as size_t)
                                    .wrapping_add(bd.startspaces as size_t)
                                    .wrapping_add(bd.endspaces as size_t);
                                let mut newp: *mut ::core::ffi::c_char = xmalloc(
                                    totlen
                                        .wrapping_add(oldlen as size_t)
                                        .wrapping_add(1 as size_t),
                                )
                                    as *mut ::core::ffi::c_char;
                                ptr_1 = newp;
                                memmove(
                                    ptr_1 as *mut ::core::ffi::c_void,
                                    oldp as *const ::core::ffi::c_void,
                                    bd.textcol as size_t,
                                );
                                ptr_1 = ptr_1.offset(bd.textcol as isize);
                                memset(
                                    ptr_1 as *mut ::core::ffi::c_void,
                                    ' ' as ::core::ffi::c_int,
                                    bd.startspaces as size_t,
                                );
                                ptr_1 = ptr_1.offset(bd.startspaces as isize);
                                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while j < count {
                                    memmove(
                                        ptr_1 as *mut ::core::ffi::c_void,
                                        (*y_array.add(i)).data as *const ::core::ffi::c_void,
                                        yanklen as size_t,
                                    );
                                    ptr_1 = ptr_1.offset(yanklen as isize);
                                    if (j < count - 1 as ::core::ffi::c_int || shortline == 0)
                                        && spaces > 0 as ::core::ffi::c_int
                                    {
                                        memset(
                                            ptr_1 as *mut ::core::ffi::c_void,
                                            ' ' as ::core::ffi::c_int,
                                            spaces as size_t,
                                        );
                                        ptr_1 = ptr_1.offset(spaces as isize);
                                    } else {
                                        totlen = totlen.wrapping_sub(spaces as size_t);
                                    }
                                    j += 1;
                                }
                                memset(
                                    ptr_1 as *mut ::core::ffi::c_void,
                                    ' ' as ::core::ffi::c_int,
                                    bd.endspaces as size_t,
                                );
                                ptr_1 = ptr_1.offset(bd.endspaces as isize);
                                let mut columns: ::core::ffi::c_int = oldlen as ::core::ffi::c_int
                                    - bd.textcol as ::core::ffi::c_int
                                    - delcount
                                    + 1 as ::core::ffi::c_int;
                                '_c2rust_label: {
                                    if columns >= 0 as ::core::ffi::c_int {
                                    } else {
                                        __assert_fail(
                                            c"columns >= 0".as_ptr(),
                                            c"src/nvim/register.rs".as_ptr(),
                                            1731 as ::core::ffi::c_uint,
                                            c"void do_put(int, yankreg_T *, int, int, int)"
                                                .as_ptr(),
                                        );
                                    }
                                };
                                memmove(
                                    ptr_1 as *mut ::core::ffi::c_void,
                                    oldp.offset(bd.textcol as isize).offset(delcount as isize)
                                        as *const ::core::ffi::c_void,
                                    columns as size_t,
                                );
                                ml_replace((*curwin.get()).w_cursor.lnum, newp, false);
                                extmark_splice_cols(
                                    curbuf.get(),
                                    (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int,
                                    bd.textcol,
                                    delcount as colnr_T,
                                    totlen as colnr_T + lines_appended as colnr_T,
                                    kExtmarkUndo,
                                );
                                (*curwin.get()).w_cursor.lnum += 1;
                                if i == 0 as size_t {
                                    (*curwin.get()).w_cursor.col += bd.startspaces;
                                }
                                i = i.wrapping_add(1);
                            }
                        }
                        changed_lines(
                            curbuf.get(),
                            lnum,
                            0 as colnr_T,
                            (*curbuf.get()).b_op_start.lnum + y_size as linenr_T - nr_lines,
                            nr_lines,
                            true,
                        );
                        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                        (*curbuf.get()).b_op_start.lnum = lnum;
                        (*curbuf.get()).b_op_end.lnum =
                            (*curwin.get()).w_cursor.lnum - 1 as linenr_T;
                        (*curbuf.get()).b_op_end.col = (if bd.textcol as ::core::ffi::c_int
                            + totlen as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int
                            > 0 as ::core::ffi::c_int
                        {
                            bd.textcol as ::core::ffi::c_int + totlen as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as colnr_T;
                        (*curbuf.get()).b_op_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                            (*curwin.get()).w_cursor = (*curbuf.get()).b_op_end;
                            (*curwin.get()).w_cursor.col += 1;
                            let mut len: colnr_T = get_cursor_line_len();
                            (*curwin.get()).w_cursor.col = if (*curwin.get()).w_cursor.col < len {
                                (*curwin.get()).w_cursor.col
                            } else {
                                len
                            };
                        } else {
                            (*curwin.get()).w_cursor.lnum = lnum;
                        }
                    } else {
                        let yanklen_0: ::core::ffi::c_int =
                            (*y_array.offset(0 as ::core::ffi::c_int as isize)).size
                                as ::core::ffi::c_int;
                        if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                            if dir == FORWARD as ::core::ffi::c_int && gchar_cursor() != NUL {
                                let mut bytelen: ::core::ffi::c_int =
                                    utfc_ptr2len(get_cursor_pos_ptr());
                                col += bytelen;
                                if yanklen_0 != 0 {
                                    (*curwin.get()).w_cursor.col += bytelen;
                                    (*curbuf.get()).b_op_end.col += bytelen;
                                }
                            }
                            (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                        } else if dir == BACKWARD as ::core::ffi::c_int {
                            lnum -= 1;
                        }
                        let mut new_cursor: pos_T = (*curwin.get()).w_cursor;
                        if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                            && y_size == 1 as size_t
                        {
                            let mut end_lnum: linenr_T = 0 as linenr_T;
                            let mut start_lnum: linenr_T = lnum;
                            let mut first_byte_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if VIsual_active.get() {
                                end_lnum = if (*curbuf.get()).b_visual.vi_end.lnum
                                    > (*curbuf.get()).b_visual.vi_start.lnum
                                {
                                    (*curbuf.get()).b_visual.vi_end.lnum
                                } else {
                                    (*curbuf.get()).b_visual.vi_start.lnum
                                };
                                if end_lnum > start_lnum {
                                    let mut pos: pos_T = pos_T {
                                        lnum: lnum,
                                        col: col,
                                        coladd: 0 as colnr_T,
                                    };
                                    getvcol(
                                        curwin.get(),
                                        &raw mut pos,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        &raw mut vcol,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                    );
                                }
                            }
                            if count == 0 as ::core::ffi::c_int
                                || yanklen_0 == 0 as ::core::ffi::c_int
                            {
                                if VIsual_active.get() {
                                    lnum = end_lnum;
                                }
                            } else if count > INT_MAX / yanklen_0 {
                                emsg(gettext(
                                    &raw const e_resulting_text_too_long
                                        as *const ::core::ffi::c_char,
                                ));
                            } else {
                                totlen = (count as size_t).wrapping_mul(yanklen_0 as size_t);
                                loop {
                                    let mut oldp_0: *mut ::core::ffi::c_char = ml_get(lnum);
                                    let mut oldlen_0: colnr_T = ml_get_len(lnum);
                                    if lnum > start_lnum {
                                        let mut pos_0: pos_T = pos_T {
                                            lnum: lnum,
                                            col: 0,
                                            coladd: 0,
                                        };
                                        if getvpos(curwin.get(), &raw mut pos_0, vcol) {
                                            col = pos_0.col;
                                        } else {
                                            col = MAXCOL as ::core::ffi::c_int as colnr_T;
                                        }
                                    }
                                    if VIsual_active.get() as ::core::ffi::c_int != 0
                                        && col > oldlen_0
                                    {
                                        lnum += 1;
                                    } else {
                                        let mut newp_0: *mut ::core::ffi::c_char = xmalloc(
                                            totlen
                                                .wrapping_add(oldlen_0 as size_t)
                                                .wrapping_add(1 as size_t),
                                        )
                                            as *mut ::core::ffi::c_char;
                                        memmove(
                                            newp_0 as *mut ::core::ffi::c_void,
                                            oldp_0 as *const ::core::ffi::c_void,
                                            col as size_t,
                                        );
                                        let mut ptr_2: *mut ::core::ffi::c_char =
                                            newp_0.offset(col as isize);
                                        let mut i_0: size_t = 0 as size_t;
                                        while i_0 < count as size_t {
                                            memmove(
                                                ptr_2 as *mut ::core::ffi::c_void,
                                                (*y_array.offset(0 as ::core::ffi::c_int as isize))
                                                    .data
                                                    as *const ::core::ffi::c_void,
                                                yanklen_0 as size_t,
                                            );
                                            ptr_2 = ptr_2.offset(yanklen_0 as isize);
                                            i_0 = i_0.wrapping_add(1);
                                        }
                                        memmove(
                                            ptr_2 as *mut ::core::ffi::c_void,
                                            oldp_0.offset(col as isize)
                                                as *const ::core::ffi::c_void,
                                            ((oldlen_0 - col) as size_t).wrapping_add(1 as size_t),
                                        );
                                        ml_replace(lnum, newp_0, false);
                                        first_byte_off = utf_head_off(
                                            newp_0,
                                            ptr_2.offset(-(1 as ::core::ffi::c_int as isize)),
                                        );
                                        if lnum == (*curwin.get()).w_cursor.lnum {
                                            changed_cline_bef_curs(curwin.get());
                                            invalidate_botline_win(curwin.get());
                                            (*curwin.get()).w_cursor.col +=
                                                totlen.wrapping_sub(1 as size_t) as colnr_T;
                                        }
                                        changed_bytes(lnum, col);
                                        extmark_splice_cols(
                                            curbuf.get(),
                                            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                            col,
                                            0 as colnr_T,
                                            totlen as colnr_T,
                                            kExtmarkUndo,
                                        );
                                        if VIsual_active.get() {
                                            lnum += 1;
                                        }
                                    }
                                    if !(VIsual_active.get() as ::core::ffi::c_int != 0
                                        && lnum <= end_lnum)
                                    {
                                        break;
                                    }
                                }
                                if VIsual_active.get() {
                                    lnum -= 1;
                                }
                            }
                            (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                            (*curbuf.get()).b_op_end.col -= first_byte_off;
                            if totlen != 0
                                && (restart_edit.get() != 0 as ::core::ffi::c_int
                                    || flags & PUT_CURSEND as ::core::ffi::c_int != 0)
                            {
                                (*curwin.get()).w_cursor.col += 1;
                            } else {
                                (*curwin.get()).w_cursor.col -= first_byte_off;
                            }
                        } else {
                            let mut new_lnum: linenr_T = new_cursor.lnum;
                            let mut indent: ::core::ffi::c_int = 0;
                            let mut orig_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut indent_diff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut first_indent: bool = true;
                            let mut lendiff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if flags & PUT_FIXINDENT as ::core::ffi::c_int != 0 {
                                orig_indent = get_indent();
                            }
                            let mut cnt: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            '_error: while cnt <= count {
                                let mut i_1: size_t = 0 as size_t;
                                if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                                {
                                    lnum = new_cursor.lnum;
                                    let mut ptr_3: *mut ::core::ffi::c_char =
                                        ml_get(lnum).offset(col as isize);
                                    let mut ptrlen_0: size_t =
                                        (ml_get_len(lnum) as size_t).wrapping_sub(col as size_t);
                                    totlen = (*y_array.add(y_size.wrapping_sub(1 as size_t))).size;
                                    let mut newp_1: *mut ::core::ffi::c_char = xmalloc(
                                        ptrlen_0.wrapping_add(totlen).wrapping_add(1 as size_t),
                                    )
                                        as *mut ::core::ffi::c_char;
                                    strcpy(
                                        newp_1,
                                        (*y_array.add(y_size.wrapping_sub(1 as size_t))).data,
                                    );
                                    strcpy(newp_1.add(totlen), ptr_3);
                                    ml_append(lnum, newp_1, 0 as colnr_T, false);
                                    new_lnum += 1;
                                    xfree(newp_1 as *mut ::core::ffi::c_void);
                                    let mut oldp_1: *mut ::core::ffi::c_char = ml_get(lnum);
                                    newp_1 = xmalloc(
                                        (col as size_t)
                                            .wrapping_add(yanklen_0 as size_t)
                                            .wrapping_add(1 as size_t),
                                    )
                                        as *mut ::core::ffi::c_char;
                                    memmove(
                                        newp_1 as *mut ::core::ffi::c_void,
                                        oldp_1 as *const ::core::ffi::c_void,
                                        col as size_t,
                                    );
                                    memmove(
                                        newp_1.offset(col as isize) as *mut ::core::ffi::c_void,
                                        (*y_array.offset(0 as ::core::ffi::c_int as isize)).data
                                            as *const ::core::ffi::c_void,
                                        (yanklen_0 as size_t).wrapping_add(1 as size_t),
                                    );
                                    ml_replace(lnum, newp_1, false);
                                    (*curwin.get()).w_cursor.lnum = lnum;
                                    i_1 = 1 as size_t;
                                }
                                while i_1 < y_size {
                                    if y_type as ::core::ffi::c_int
                                        != kMTCharWise as ::core::ffi::c_int
                                        || i_1 < y_size.wrapping_sub(1 as size_t)
                                    {
                                        if ml_append(
                                            lnum,
                                            (*y_array.add(i_1)).data,
                                            0 as colnr_T,
                                            false,
                                        ) == FAIL
                                        {
                                            break '_error;
                                        }
                                        new_lnum += 1;
                                    }
                                    lnum += 1;
                                    nr_lines += 1;
                                    if flags & PUT_FIXINDENT as ::core::ffi::c_int != 0 {
                                        let mut old_pos: pos_T = (*curwin.get()).w_cursor;
                                        (*curwin.get()).w_cursor.lnum = lnum;
                                        let mut ptr_4: *mut ::core::ffi::c_char = ml_get(lnum);
                                        if cnt == count && i_1 == y_size.wrapping_sub(1 as size_t) {
                                            lendiff = ml_get_len(lnum) as ::core::ffi::c_int;
                                        }
                                        if *ptr_4 as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                                            && preprocs_left() as ::core::ffi::c_int != 0
                                        {
                                            indent = 0 as ::core::ffi::c_int;
                                        } else if *ptr_4 as ::core::ffi::c_int == NUL {
                                            indent = 0 as ::core::ffi::c_int;
                                        } else if first_indent {
                                            indent_diff = orig_indent - get_indent();
                                            indent = orig_indent;
                                            first_indent = false;
                                        } else {
                                            indent = get_indent() + indent_diff;
                                            if indent < 0 as ::core::ffi::c_int {
                                                indent = 0 as ::core::ffi::c_int;
                                            }
                                        }
                                        set_indent(indent, SIN_NOMARK);
                                        (*curwin.get()).w_cursor = old_pos;
                                        if cnt == count && i_1 == y_size.wrapping_sub(1 as size_t) {
                                            lendiff -= ml_get_len(lnum) as ::core::ffi::c_int;
                                        }
                                    }
                                    i_1 = i_1.wrapping_add(1);
                                }
                                let mut totsize: bcount_t = 0 as bcount_t;
                                let mut lastsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                                    || y_type as ::core::ffi::c_int
                                        == kMTLineWise as ::core::ffi::c_int
                                        && flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0
                                {
                                    i_1 = 0 as size_t;
                                    while i_1 < y_size.wrapping_sub(1 as size_t) {
                                        totsize +=
                                            (*y_array.add(i_1)).size as bcount_t + 1 as bcount_t;
                                        i_1 = i_1.wrapping_add(1);
                                    }
                                    lastsize = (*y_array.add(y_size.wrapping_sub(1 as size_t))).size
                                        as ::core::ffi::c_int;
                                    totsize += lastsize as bcount_t;
                                }
                                if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                                {
                                    extmark_splice(
                                        curbuf.get(),
                                        new_cursor.lnum as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int,
                                        col,
                                        0 as ::core::ffi::c_int,
                                        0 as colnr_T,
                                        0 as bcount_t,
                                        y_size as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                        lastsize as colnr_T,
                                        totsize,
                                        kExtmarkUndo,
                                    );
                                } else if y_type as ::core::ffi::c_int
                                    == kMTLineWise as ::core::ffi::c_int
                                    && flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0
                                {
                                    extmark_splice(
                                        curbuf.get(),
                                        new_cursor.lnum as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int,
                                        split_pos,
                                        0 as ::core::ffi::c_int,
                                        0 as colnr_T,
                                        0 as bcount_t,
                                        y_size as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                                        0 as colnr_T,
                                        totsize + 2 as bcount_t,
                                        kExtmarkUndo,
                                    );
                                }
                                if cnt == 1 as ::core::ffi::c_int {
                                    new_lnum = lnum;
                                }
                                cnt += 1;
                            }
                            if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                                (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                                if dir == FORWARD as ::core::ffi::c_int {
                                    (*curbuf.get()).b_op_start.lnum += 1;
                                }
                            }
                            let mut kind: ExtmarkOp = (if y_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                                && flags & PUT_LINE_SPLIT as ::core::ffi::c_int == 0
                            {
                                kExtmarkUndo as ::core::ffi::c_int
                            } else {
                                kExtmarkNOOP as ::core::ffi::c_int
                            }) as ExtmarkOp;
                            mark_adjust(
                                (*curbuf.get()).b_op_start.lnum
                                    + (y_type as ::core::ffi::c_int
                                        == kMTCharWise as ::core::ffi::c_int)
                                        as ::core::ffi::c_int,
                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                nr_lines,
                                0 as linenr_T,
                                kind,
                            );
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                                changed_lines(
                                    curbuf.get(),
                                    (*curwin.get()).w_cursor.lnum,
                                    col,
                                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                                    nr_lines,
                                    true,
                                );
                            } else {
                                changed_lines(
                                    curbuf.get(),
                                    (*curbuf.get()).b_op_start.lnum,
                                    0 as colnr_T,
                                    (*curbuf.get()).b_op_start.lnum,
                                    nr_lines,
                                    true,
                                );
                            }
                            (*curbuf.get()).b_op_end.lnum = new_lnum;
                            col = (if 0 as ::core::ffi::c_int
                                > (*y_array.add(y_size.wrapping_sub(1 as size_t))).size
                                    as ::core::ffi::c_int
                                    - lendiff
                            {
                                0 as ::core::ffi::c_int
                            } else {
                                (*y_array.add(y_size.wrapping_sub(1 as size_t))).size
                                    as ::core::ffi::c_int
                                    - lendiff
                            }) as colnr_T;
                            if col > 1 as ::core::ffi::c_int {
                                (*curbuf.get()).b_op_end.col = (col as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int)
                                    as colnr_T;
                                if (*y_array.add(y_size.wrapping_sub(1 as size_t))).size
                                    > 0 as size_t
                                {
                                    (*curbuf.get()).b_op_end.col -= utf_head_off(
                                        (*y_array.add(y_size.wrapping_sub(1 as size_t))).data,
                                        (*y_array.add(y_size.wrapping_sub(1 as size_t)))
                                            .data
                                            .add(
                                                (*y_array.add(y_size.wrapping_sub(1 as size_t)))
                                                    .size,
                                            )
                                            .offset(-(1 as ::core::ffi::c_int as isize)),
                                    );
                                }
                            } else {
                                (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
                            }
                            if flags & PUT_CURSLINE as ::core::ffi::c_int != 0 {
                                (*curwin.get()).w_cursor.lnum = lnum;
                                beginline(BL_WHITE | BL_FIX);
                            } else if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                                if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                                {
                                    if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                                        (*curwin.get()).w_cursor.lnum =
                                            (*curbuf.get()).b_ml.ml_line_count;
                                    } else {
                                        (*curwin.get()).w_cursor.lnum = lnum + 1 as linenr_T;
                                    }
                                    (*curwin.get()).w_cursor.col =
                                        0 as ::core::ffi::c_int as colnr_T;
                                } else {
                                    (*curwin.get()).w_cursor.lnum = new_lnum;
                                    (*curwin.get()).w_cursor.col = col;
                                    (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                                    if col > 1 as ::core::ffi::c_int {
                                        (*curbuf.get()).b_op_end.col = (col as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int)
                                            as colnr_T;
                                    }
                                }
                            } else if y_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                            {
                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                if dir == FORWARD as ::core::ffi::c_int {
                                    (*curwin.get()).w_cursor.lnum += 1;
                                }
                                beginline(BL_WHITE | BL_FIX);
                            } else {
                                (*curwin.get()).w_cursor = new_cursor;
                            }
                        }
                    }
                    msgmore(nr_lines as ::core::ffi::c_int);
                    (*curwin.get()).w_set_curswant = true_0;
                    len_0 = get_cursor_line_len();
                    if (*curwin.get()).w_cursor.col > len_0 {
                        if cur_ve_flags
                            == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            (*curwin.get()).w_cursor.coladd =
                                ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - len_0)
                                    as colnr_T;
                        }
                        (*curwin.get()).w_cursor.col = len_0 as colnr_T;
                    }
                }
            }
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
            (*curbuf.get()).b_op_start = orig_start;
            (*curbuf.get()).b_op_end = orig_end;
        }
        if allocated {
            xfree(insert_string.data as *mut ::core::ffi::c_void);
        }
        if regname == '=' as ::core::ffi::c_int {
            xfree(y_array as *mut ::core::ffi::c_void);
        }
        if (*curbuf.get()).terminal.is_null() {
            VIsual_active.set(false);
        }
        adjust_cursor_eol();
    }
}
