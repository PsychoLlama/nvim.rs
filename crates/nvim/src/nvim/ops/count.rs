//! `g CTRL-G` and `v_g_CTRL-G` -- counting what is in the buffer.
//!
//! `cursor_pos_info` answers the whole of the `g CTRL-G` message: the
//! cursor's position by column, line, word, character and byte, and -- when
//! a Visual region is active -- the same counts for the selection instead,
//! with the blockwise case counted column by column.  `line_count_info`
//! is the per-line counter it drives, and `get_region_bytecount` is the
//! byte-only version the API and quickfix use.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn line_count_info(
    mut line: *mut ::core::ffi::c_char,
    mut wc: *mut varnumber_T,
    mut cc: *mut varnumber_T,
    mut limit: varnumber_T,
    mut eol_size: ::core::ffi::c_int,
) -> varnumber_T {
    unsafe {
        let mut i: varnumber_T = 0;
        let mut words: varnumber_T = 0 as varnumber_T;
        let mut chars: varnumber_T = 0 as varnumber_T;
        let mut is_word: bool = false_0 != 0;
        i = 0 as varnumber_T;
        while i < limit && *line.offset(i as isize) as ::core::ffi::c_int != NUL {
            if is_word {
                if ascii_isspace(*line.offset(i as isize) as ::core::ffi::c_int) {
                    words += 1;
                    is_word = false_0 != 0;
                }
            } else if !ascii_isspace(*line.offset(i as isize) as ::core::ffi::c_int) {
                is_word = true_0 != 0;
            }
            chars += 1;
            i += utfc_ptr2len(line.offset(i as isize)) as varnumber_T;
        }
        if is_word {
            words += 1;
        }
        *wc += words;
        if i < limit && *line.offset(i as isize) as ::core::ffi::c_int == NUL {
            i += eol_size as varnumber_T;
            chars += eol_size as varnumber_T;
        }
        *cc += chars;
        return i;
    }
}

pub unsafe extern "C" fn cursor_pos_info(mut dict: *mut dict_T) {
    unsafe {
        let mut buf1: [::core::ffi::c_char; 50] = [0; 50];
        let mut buf2: [::core::ffi::c_char; 40] = [0; 40];
        let mut byte_count: varnumber_T = 0 as varnumber_T;
        let mut bom_count: varnumber_T = 0 as varnumber_T;
        let mut byte_count_cursor: varnumber_T = 0 as varnumber_T;
        let mut char_count: varnumber_T = 0 as varnumber_T;
        let mut char_count_cursor: varnumber_T = 0 as varnumber_T;
        let mut word_count: varnumber_T = 0 as varnumber_T;
        let mut word_count_cursor: varnumber_T = 0 as varnumber_T;
        let mut min_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut max_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut oparg: oparg_T = oparg_T {
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
        let l_VIsual_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
        let l_VIsual_mode: ::core::ffi::c_int = VIsual_mode.get();
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            if dict.is_null() {
                msg(
                    gettext(no_lines_msg.ptr() as *mut ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
                return;
            }
        } else {
            let mut eol_size: ::core::ffi::c_int = 0;
            let mut last_check: varnumber_T = 100000 as varnumber_T;
            let mut line_count_selected: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if get_fileformat(curbuf.get()) == EOL_DOS {
                eol_size = 2 as ::core::ffi::c_int;
            } else {
                eol_size = 1 as ::core::ffi::c_int;
            }
            if l_VIsual_active != 0 {
                if lt(VIsual.get(), (*curwin.get()).w_cursor) {
                    min_pos = VIsual.get();
                    max_pos = (*curwin.get()).w_cursor;
                } else {
                    min_pos = (*curwin.get()).w_cursor;
                    max_pos = VIsual.get();
                }
                if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                    && max_pos.col > 0 as ::core::ffi::c_int
                {
                    max_pos.col -= 1;
                }
                if l_VIsual_mode == Ctrl_V {
                    let saved_sbr: *mut ::core::ffi::c_char = p_sbr.get();
                    let saved_w_sbr: *mut ::core::ffi::c_char = (*curwin.get()).w_onebuf_opt.wo_sbr;
                    p_sbr.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                    (*curwin.get()).w_onebuf_opt.wo_sbr =
                        empty_string_option.ptr() as *mut ::core::ffi::c_char;
                    oparg.is_VIsual = true_0 != 0;
                    oparg.motion_type = kMTBlockWise;
                    oparg.op_type = OP_NOP;
                    getvcols(
                        curwin.get(),
                        &raw mut min_pos,
                        &raw mut max_pos,
                        &raw mut oparg.start_vcol,
                        &raw mut oparg.end_vcol,
                    );
                    p_sbr.set(saved_sbr);
                    (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;
                    if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                        oparg.end_vcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                    if oparg.end_vcol < oparg.start_vcol {
                        oparg.end_vcol += oparg.start_vcol;
                        oparg.start_vcol = oparg.end_vcol - oparg.start_vcol;
                        oparg.end_vcol -= oparg.start_vcol;
                    }
                }
                line_count_selected =
                    (max_pos.lnum - min_pos.lnum + 1 as linenr_T) as ::core::ffi::c_int;
            }
            let mut lnum: linenr_T = 1 as linenr_T;
            while lnum <= (*curbuf.get()).b_ml.ml_line_count {
                if byte_count > last_check {
                    os_breakcheck();
                    if got_int.get() {
                        return;
                    }
                    last_check = byte_count + 100000 as varnumber_T;
                }
                if l_VIsual_active != 0 && lnum >= min_pos.lnum && lnum <= max_pos.lnum {
                    let mut s: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    match l_VIsual_mode {
                        Ctrl_V => {
                            virtual_op.set(virtual_active(curwin.get()) as TriState);
                            block_prep(&raw mut oparg, &raw mut bd, lnum, false_0 != 0);
                            virtual_op.set(kNone);
                            s = bd.textstart;
                            len = bd.textlen;
                        }
                        86 => {
                            s = ml_get(lnum);
                            len = MAXCOL as ::core::ffi::c_int;
                        }
                        118 => {
                            let mut start_col: colnr_T = if lnum == min_pos.lnum {
                                min_pos.col
                            } else {
                                0 as colnr_T
                            };
                            let mut end_col: colnr_T = if lnum == max_pos.lnum {
                                max_pos.col - start_col + 1 as colnr_T
                            } else {
                                MAXCOL as ::core::ffi::c_int
                            };
                            s = ml_get(lnum).offset(start_col as isize);
                            len = end_col as ::core::ffi::c_int;
                        }
                        _ => {}
                    }
                    if !s.is_null() {
                        byte_count_cursor += line_count_info(
                            s,
                            &raw mut word_count_cursor,
                            &raw mut char_count_cursor,
                            len as varnumber_T,
                            eol_size,
                        );
                        if lnum == (*curbuf.get()).b_ml.ml_line_count
                            && (*curbuf.get()).b_p_eol == 0
                            && ((*curbuf.get()).b_p_bin != 0 || (*curbuf.get()).b_p_fixeol == 0)
                            && (strlen(s) as ::core::ffi::c_int) < len
                        {
                            byte_count_cursor -= eol_size as varnumber_T;
                        }
                    }
                } else if lnum == (*curwin.get()).w_cursor.lnum {
                    word_count_cursor += word_count;
                    char_count_cursor += char_count;
                    byte_count_cursor = byte_count
                        + line_count_info(
                            ml_get(lnum),
                            &raw mut word_count_cursor,
                            &raw mut char_count_cursor,
                            (*curwin.get()).w_cursor.col as varnumber_T + 1 as varnumber_T,
                            eol_size,
                        );
                }
                byte_count += line_count_info(
                    ml_get(lnum),
                    &raw mut word_count,
                    &raw mut char_count,
                    MAXCOL as ::core::ffi::c_int as varnumber_T,
                    eol_size,
                );
                lnum += 1;
            }
            if (*curbuf.get()).b_p_eol == 0
                && ((*curbuf.get()).b_p_bin != 0 || (*curbuf.get()).b_p_fixeol == 0)
            {
                byte_count -= eol_size as varnumber_T;
            }
            if dict.is_null() {
                if l_VIsual_active != 0 {
                    if l_VIsual_mode == Ctrl_V
                        && (*curwin.get()).w_curswant < MAXCOL as ::core::ffi::c_int
                    {
                        getvcols(
                            curwin.get(),
                            &raw mut min_pos,
                            &raw mut max_pos,
                            &raw mut min_pos.col,
                            &raw mut max_pos.col,
                        );
                        let mut cols: int64_t = 0;
                        let (c2rust_result, c2rust_overflowed) =
                            ((oparg.end_vcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                as i128)
                                .overflowing_sub(oparg.start_vcol as i128);
                        let c2rust_result_narrow = c2rust_result as int64_t;
                        *&raw mut cols = c2rust_result_narrow;
                        if c2rust_overflowed || c2rust_result_narrow as i128 != c2rust_result {
                            logmsg(
                                LOGLVL_ERR,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                                b"cursor_pos_info\0".as_ptr() as *const ::core::ffi::c_char,
                                2966 as ::core::ffi::c_int,
                                true_0 != 0,
                                b"STRICT_SUB overflow\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            abort();
                        }
                        vim_snprintf(
                            &raw mut buf1 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                            gettext(b"%ld Cols; \0".as_ptr() as *const ::core::ffi::c_char),
                            cols,
                        );
                    } else {
                        buf1[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    }
                    if char_count_cursor == byte_count_cursor && char_count == byte_count {
                        vim_snprintf(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            gettext(
                                b"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Bytes\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            &raw mut buf1 as *mut ::core::ffi::c_char,
                            line_count_selected as int64_t,
                            (*curbuf.get()).b_ml.ml_line_count as int64_t,
                            word_count_cursor,
                            word_count,
                            byte_count_cursor,
                            byte_count,
                        );
                    } else {
                        vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(
                            b"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Chars; %ld of %ld Bytes\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        line_count_selected as int64_t,
                        (*curbuf.get()).b_ml.ml_line_count as int64_t,
                        word_count_cursor,
                        word_count,
                        char_count_cursor,
                        char_count,
                        byte_count_cursor,
                        byte_count,
                    );
                    }
                } else {
                    let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                    validate_virtcol(curwin.get());
                    col_print(
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                        (*curwin.get()).w_cursor.col + 1 as ::core::ffi::c_int,
                        (*curwin.get()).w_virtcol + 1 as ::core::ffi::c_int,
                    );
                    col_print(
                        &raw mut buf2 as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                        get_cursor_line_len(),
                        linetabsize_str(p),
                    );
                    if char_count_cursor == byte_count_cursor && char_count == byte_count {
                        vim_snprintf(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            gettext(
                                b"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Byte %ld of %ld\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            &raw mut buf1 as *mut ::core::ffi::c_char,
                            &raw mut buf2 as *mut ::core::ffi::c_char,
                            (*curwin.get()).w_cursor.lnum as int64_t,
                            (*curbuf.get()).b_ml.ml_line_count as int64_t,
                            word_count_cursor,
                            word_count,
                            byte_count_cursor,
                            byte_count,
                        );
                    } else {
                        vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(
                            b"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Char %ld of %ld; Byte %ld of %ld\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        &raw mut buf2 as *mut ::core::ffi::c_char,
                        (*curwin.get()).w_cursor.lnum as int64_t,
                        (*curbuf.get()).b_ml.ml_line_count as int64_t,
                        word_count_cursor,
                        word_count,
                        char_count_cursor,
                        char_count,
                        byte_count_cursor,
                        byte_count,
                    );
                    }
                }
            }
            bom_count = bomb_size() as varnumber_T;
            if dict.is_null() && bom_count > 0 as varnumber_T {
                let len_0: size_t = strlen(IObuff.ptr() as *mut ::core::ffi::c_char);
                vim_snprintf(
                    (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len_0 as isize),
                    (IOSIZE as size_t).wrapping_sub(len_0),
                    gettext(b"(+%ld for BOM)\0".as_ptr() as *const ::core::ffi::c_char),
                    bom_count,
                );
            }
            if dict.is_null() {
                let mut p_0: *mut ::core::ffi::c_char = p_shm.get();
                p_shm.set(b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
                if p_ch.get() < 1 as OptInt {
                    msg_start();
                    msg_scroll.set(true_0);
                }
                msg(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
                p_shm.set(p_0);
            }
        }
        if !dict.is_null() {
            tv_dict_add_nr(
                dict,
                b"words\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                word_count,
            );
            tv_dict_add_nr(
                dict,
                b"chars\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                char_count,
            );
            tv_dict_add_nr(
                dict,
                b"bytes\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                byte_count + bom_count,
            );
            tv_dict_add_nr(
                dict,
                if l_VIsual_active != 0 {
                    b"visual_bytes\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"cursor_bytes\0".as_ptr() as *const ::core::ffi::c_char
                },
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
                byte_count_cursor,
            );
            tv_dict_add_nr(
                dict,
                if l_VIsual_active != 0 {
                    b"visual_chars\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"cursor_chars\0".as_ptr() as *const ::core::ffi::c_char
                },
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
                char_count_cursor,
            );
            tv_dict_add_nr(
                dict,
                if l_VIsual_active != 0 {
                    b"visual_words\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"cursor_words\0".as_ptr() as *const ::core::ffi::c_char
                },
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
                word_count_cursor,
            );
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_region_bytecount(
    mut buf: *mut buf_T,
    mut start_lnum: linenr_T,
    mut end_lnum: linenr_T,
    mut start_col: colnr_T,
    mut end_col: colnr_T,
) -> bcount_t {
    unsafe {
        let mut max_lnum: linenr_T = (*buf).b_ml.ml_line_count;
        if start_lnum > max_lnum {
            return 0 as bcount_t;
        }
        if start_lnum == end_lnum {
            return (end_col - start_col) as bcount_t;
        }
        let mut deleted_bytes: bcount_t = (ml_get_buf_len(buf, start_lnum)
            - start_col as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as bcount_t;
        let mut i: linenr_T = 1 as linenr_T;
        while i <= end_lnum - start_lnum - 1 as linenr_T {
            if start_lnum + i > max_lnum {
                return deleted_bytes;
            }
            deleted_bytes +=
                (ml_get_buf_len(buf, start_lnum + i) + 1 as ::core::ffi::c_int) as bcount_t;
            i += 1;
        }
        if end_lnum > max_lnum {
            return deleted_bytes;
        }
        return deleted_bytes + end_col as bcount_t;
    }
}
