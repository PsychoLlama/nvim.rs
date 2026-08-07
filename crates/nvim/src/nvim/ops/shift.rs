//! `<` and `>` -- moving an indent left or right.
//!
//! Three shapes of the same operation: `shift_line` re-indents one line by a
//! multiple of 'shiftwidth' ('vartabstop' makes that a position-dependent
//! question, which is what the `get_vts*` helpers answer), `op_shift` runs it
//! over a linewise region, and `shift_block` does the blockwise case, which is
//! not a re-indent at all -- it inserts or removes white space at the block's
//! left edge, in the middle of the line, and has to rebuild any tab it
//! splits.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn op_shift(
    mut oap: *mut oparg_T,
    mut curs_top: bool,
    mut amount: ::core::ffi::c_int,
) {
    unsafe {
        let mut block_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if u_save(
            (*oap).start.lnum - 1 as linenr_T,
            (*oap).end.lnum + 1 as linenr_T,
        ) == FAIL
        {
            return;
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            block_col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int =
            (*oap).line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut first_char: ::core::ffi::c_int =
                *get_cursor_line_ptr() as uint8_t as ::core::ffi::c_int;
            if first_char == NUL {
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            } else if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
            {
                shift_block(oap, amount);
            } else if first_char != '#' as ::core::ffi::c_int || !preprocs_left() {
                shift_line(
                    (*oap).op_type == OP_LSHIFT,
                    p_sr.get() != 0,
                    amount,
                    false_0,
                );
            }
            (*curwin.get()).w_cursor.lnum += 1;
            i -= 1;
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
            (*curwin.get()).w_cursor.col = block_col as colnr_T;
        } else if curs_top {
            (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        } else {
            (*curwin.get()).w_cursor.lnum -= 1;
        }
        foldOpenCursor();
        if (*oap).line_count as OptInt > p_report.get() {
            let mut op: *mut ::core::ffi::c_char = (if (*oap).op_type == OP_RSHIFT {
                b">\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"<\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            let mut msg_line_single: *mut ::core::ffi::c_char = ngettext(
                b"%ld line %sed %d time\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld line %sed %d times\0".as_ptr() as *const ::core::ffi::c_char,
                amount as ::core::ffi::c_ulong,
            );
            let mut msg_line_plural: *mut ::core::ffi::c_char = ngettext(
                b"%ld lines %sed %d time\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld lines %sed %d times\0".as_ptr() as *const ::core::ffi::c_char,
                amount as ::core::ffi::c_ulong,
            );
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                ngettext(
                    msg_line_single,
                    msg_line_plural,
                    (*oap).line_count as ::core::ffi::c_ulong,
                ),
                (*oap).line_count as int64_t,
                op,
                amount,
            );
            msg_keep(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
            (*curbuf.get()).b_op_end.col = ml_get_len((*oap).end.lnum);
            if (*curbuf.get()).b_op_end.col > 0 as ::core::ffi::c_int {
                (*curbuf.get()).b_op_end.col -= 1;
            }
        }
        changed_lines(
            curbuf.get(),
            (*oap).start.lnum,
            0 as colnr_T,
            (*oap).end.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
    }
}

unsafe extern "C" fn get_vts(
    mut vts_array: *const ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ts: ::core::ffi::c_int = 0;
        if index < 1 as ::core::ffi::c_int {
            ts = 0 as ::core::ffi::c_int;
        } else if index <= *vts_array.offset(0 as ::core::ffi::c_int as isize) {
            ts = *vts_array.offset(index as isize);
        } else {
            ts = *vts_array.offset(*vts_array.offset(0 as ::core::ffi::c_int as isize) as isize);
        }
        return ts;
    }
}

unsafe extern "C" fn get_vts_sum(
    mut vts_array: *const ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut sum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0;
        i = 1 as ::core::ffi::c_int;
        while i <= index && i <= *vts_array.offset(0 as ::core::ffi::c_int as isize) {
            sum += *vts_array.offset(i as isize);
            i += 1;
        }
        if i <= index {
            sum += *vts_array.offset(*vts_array.offset(0 as ::core::ffi::c_int as isize) as isize)
                * (index - *vts_array.offset(0 as ::core::ffi::c_int as isize));
        }
        return sum;
    }
}

unsafe extern "C" fn get_new_sw_indent(
    mut left: bool,
    mut round: bool,
    mut amount: int64_t,
    mut sw_val: int64_t,
) -> int64_t {
    unsafe {
        let mut count: int64_t = get_indent() as int64_t;
        if round {
            let mut i: int64_t = crate::src::nvim::math::trim_to_int(count / sw_val) as int64_t;
            let mut j: int64_t = crate::src::nvim::math::trim_to_int(count % sw_val) as int64_t;
            if j != 0 && left as ::core::ffi::c_int != 0 {
                amount -= 1;
            }
            if left {
                i = if i - amount > 0 as int64_t {
                    i - amount
                } else {
                    0 as int64_t
                };
            } else {
                i += amount;
            }
            count = i * sw_val;
        } else if left {
            count = if count - sw_val * amount > 0 as int64_t {
                count - sw_val * amount
            } else {
                0 as int64_t
            };
        } else {
            count += sw_val * amount;
        }
        return count;
    }
}

unsafe extern "C" fn get_new_vts_indent(
    mut left: bool,
    mut round: bool,
    mut amount: ::core::ffi::c_int,
    mut vts_array: *mut ::core::ffi::c_int,
) -> int64_t {
    unsafe {
        let mut indent: int64_t = get_indent() as int64_t;
        let mut vtsi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut vts_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ts: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while vts_indent as int64_t <= indent {
            vtsi += 1;
            ts = get_vts(vts_array, vtsi);
            vts_indent += ts;
        }
        vts_indent -= ts;
        vtsi -= 1;
        let mut offset: int64_t = indent - vts_indent as int64_t;
        if round {
            if left {
                if offset == 0 as int64_t {
                    indent = get_vts_sum(vts_array, vtsi - amount) as int64_t;
                } else {
                    indent = get_vts_sum(vts_array, vtsi - (amount - 1 as ::core::ffi::c_int))
                        as int64_t;
                }
            } else {
                indent = get_vts_sum(vts_array, vtsi + amount) as int64_t;
            }
        } else if left {
            if amount > vtsi {
                indent = 0 as int64_t;
            } else {
                indent = get_vts_sum(vts_array, vtsi - amount) as int64_t + offset;
            }
        } else {
            indent = get_vts_sum(vts_array, vtsi + amount) as int64_t + offset;
        }
        return indent;
    }
}

pub unsafe extern "C" fn shift_line(
    mut left: bool,
    mut round: bool,
    mut amount: ::core::ffi::c_int,
    mut call_changed_bytes: ::core::ffi::c_int,
) {
    unsafe {
        let mut count: int64_t = 0;
        let mut sw_val: int64_t = (*curbuf.get()).b_p_sw as int64_t;
        let mut ts_val: int64_t = (*curbuf.get()).b_p_ts as int64_t;
        let mut vts_array: *mut ::core::ffi::c_int =
            (*curbuf.get()).b_p_vts_array as *mut ::core::ffi::c_int;
        if sw_val != 0 as int64_t {
            count = get_new_sw_indent(left, round, amount as int64_t, sw_val);
        } else if vts_array.is_null()
            || *vts_array.offset(0 as ::core::ffi::c_int as isize) == 0 as ::core::ffi::c_int
        {
            count = get_new_sw_indent(left, round, amount as int64_t, ts_val);
        } else {
            count = get_new_vts_indent(left, round, amount, vts_array);
        }
        if State.get() & VREPLACE_FLAG != 0 {
            change_indent(
                INDENT_SET as ::core::ffi::c_int,
                crate::src::nvim::math::trim_to_int(count),
                false_0,
                call_changed_bytes != 0,
            );
        } else {
            set_indent(
                crate::src::nvim::math::trim_to_int(count),
                if call_changed_bytes != 0 {
                    SIN_CHANGED as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        };
    }
}

unsafe extern "C" fn shift_block(mut oap: *mut oparg_T, mut amount: ::core::ffi::c_int) {
    unsafe {
        let left: bool = (*oap).op_type == OP_LSHIFT;
        let oldstate: ::core::ffi::c_int = State.get();
        let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let oldcol: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        let sw_val: ::core::ffi::c_int = get_sw_value_indent(curbuf.get(), left);
        let ts_val: ::core::ffi::c_int = (*curbuf.get()).b_p_ts as ::core::ffi::c_int;
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
        let mut incr: ::core::ffi::c_int = 0;
        let old_p_ri: ::core::ffi::c_int = p_ri.get();
        p_ri.set(0 as ::core::ffi::c_int);
        State.set(MODE_INSERT);
        block_prep(oap, &raw mut bd, (*curwin.get()).w_cursor.lnum, true_0 != 0);
        if bd.is_short != 0 {
            return;
        }
        let mut total: ::core::ffi::c_int = (amount as ::core::ffi::c_uint)
            .wrapping_mul(sw_val as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
        if total / sw_val != amount {
            return;
        }
        let oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let old_line_len: ::core::ffi::c_int = get_cursor_line_len();
        let mut startcol: ::core::ffi::c_int = 0;
        let mut oldlen: ::core::ffi::c_int = 0;
        let mut newlen: ::core::ffi::c_int = 0;
        if !left {
            total += bd.pre_whitesp;
            let mut ws_vcol: colnr_T = bd.start_vcol - bd.pre_whitesp as colnr_T;
            let mut old_textstart: *mut ::core::ffi::c_char = bd.textstart;
            if bd.startspaces != 0 {
                if utfc_ptr2len(bd.textstart) == 1 as ::core::ffi::c_int {
                    bd.textstart = bd.textstart.offset(1);
                } else {
                    ws_vcol = 0 as ::core::ffi::c_int as colnr_T;
                    bd.startspaces = 0 as ::core::ffi::c_int;
                }
            }
            let mut csarg: CharsizeArg = CharsizeArg::default();
            let mut cstype: CharsizeKind = init_charsize_arg(
                &mut csarg,
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                bd.textstart,
            );
            let mut ci: StrCharInfo = utf_ptr2StrCharInfo(bd.textstart);
            let mut vcol: ::core::ffi::c_int = bd.start_vcol as ::core::ffi::c_int;
            while ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
                incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
                ci = utfc_next(ci);
                total += incr;
                vcol += incr;
            }
            bd.textstart = ci.ptr;
            bd.start_vcol = vcol as colnr_T;
            let mut tabs: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if (*curbuf.get()).b_p_et == 0 {
                tabstop_fromto(
                    ws_vcol,
                    ws_vcol + total as colnr_T,
                    ts_val,
                    (*curbuf.get()).b_p_vts_array,
                    &raw mut tabs,
                    &raw mut spaces,
                );
            } else {
                spaces = total;
            }
            let col_pre: ::core::ffi::c_int = bd.pre_whitesp_c
                - (bd.startspaces != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            bd.textcol -= col_pre;
            let new_line_len: ::core::ffi::c_int = bd.textcol as ::core::ffi::c_int
                + tabs
                + spaces
                + (old_line_len - bd.textstart.offset_from(oldp) as ::core::ffi::c_int);
            newp = xmalloc((new_line_len as size_t).wrapping_add(1 as size_t))
                as *mut ::core::ffi::c_char;
            memmove(
                newp as *mut ::core::ffi::c_void,
                oldp as *const ::core::ffi::c_void,
                bd.textcol as size_t,
            );
            startcol = bd.textcol as ::core::ffi::c_int;
            oldlen = bd.textstart.offset_from(old_textstart) as ::core::ffi::c_int + col_pre;
            newlen = tabs + spaces;
            memset(
                newp.offset(bd.textcol as isize) as *mut ::core::ffi::c_void,
                TAB,
                tabs as size_t,
            );
            memset(
                newp.offset(bd.textcol as isize).offset(tabs as isize) as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                spaces as size_t,
            );
            strcpy(
                newp.offset(bd.textcol as isize)
                    .offset(tabs as isize)
                    .offset(spaces as isize),
                bd.textstart,
            );
            '_c2rust_label: {
                if newlen - oldlen == new_line_len - old_line_len {
                } else {
                    __assert_fail(
                        b"newlen - oldlen == new_line_len - old_line_len\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        497 as ::core::ffi::c_uint,
                        b"void shift_block(oparg_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        } else {
            let mut verbatim_copy_end: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut verbatim_copy_width: colnr_T = 0;
            let mut non_white: *mut ::core::ffi::c_char = bd.textstart;
            if bd.startspaces != 0 {
                non_white = non_white.offset(utfc_ptr2len(non_white) as isize);
            }
            let mut non_white_col: colnr_T = bd.start_vcol;
            let mut csarg_0: CharsizeArg = CharsizeArg::default();
            let mut cstype_0: CharsizeKind = init_charsize_arg(
                &mut csarg_0,
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                bd.textstart,
            );
            while ascii_iswhite(*non_white as ::core::ffi::c_int) {
                incr = win_charsize(
                    cstype_0,
                    non_white_col as ::core::ffi::c_int,
                    non_white,
                    *non_white as uint8_t as int32_t,
                    &mut csarg_0,
                )
                .width;
                non_white_col += incr;
                non_white = non_white.offset(1);
            }
            let block_space_width: colnr_T = non_white_col - (*oap).start_vcol;
            let shift_amount: colnr_T = if block_space_width < total {
                block_space_width
            } else {
                total as colnr_T
            };
            let destination_col: colnr_T = non_white_col - shift_amount;
            verbatim_copy_end = bd.textstart;
            verbatim_copy_width = bd.start_vcol;
            if bd.startspaces != 0 {
                verbatim_copy_width -= bd.start_char_vcols;
            }
            cstype_0 = init_charsize_arg(&mut csarg_0, curwin.get(), 0 as linenr_T, bd.textstart);
            let mut ci_0: StrCharInfo = utf_ptr2StrCharInfo(verbatim_copy_end);
            while verbatim_copy_width < destination_col {
                incr = win_charsize(
                    cstype_0,
                    verbatim_copy_width as ::core::ffi::c_int,
                    ci_0.ptr,
                    ci_0.chr.value,
                    &mut csarg_0,
                )
                .width;
                if verbatim_copy_width as ::core::ffi::c_int + incr > destination_col {
                    break;
                }
                verbatim_copy_width += incr;
                ci_0 = utfc_next(ci_0);
            }
            verbatim_copy_end = ci_0.ptr;
            '_c2rust_label_0: {
                if destination_col - verbatim_copy_width >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"destination_col - verbatim_copy_width >= 0\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        561 as ::core::ffi::c_uint,
                        b"void shift_block(oparg_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let fill: ::core::ffi::c_int =
                destination_col as ::core::ffi::c_int - verbatim_copy_width as ::core::ffi::c_int;
            '_c2rust_label_1: {
                if verbatim_copy_end.offset_from(oldp) >= 0 as isize {
                } else {
                    __assert_fail(
                        b"verbatim_copy_end - oldp >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        565 as ::core::ffi::c_uint,
                        b"void shift_block(oparg_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let fixedlen: ::core::ffi::c_int =
                verbatim_copy_end.offset_from(oldp) as ::core::ffi::c_int;
            let new_line_len_0: ::core::ffi::c_int = fixedlen
                + fill
                + (old_line_len - non_white.offset_from(oldp) as ::core::ffi::c_int);
            newp = xmalloc((new_line_len_0 as size_t).wrapping_add(1 as size_t))
                as *mut ::core::ffi::c_char;
            startcol = fixedlen;
            oldlen = bd.textcol as ::core::ffi::c_int
                + non_white.offset_from(bd.textstart) as ::core::ffi::c_int
                - fixedlen;
            newlen = fill;
            memmove(
                newp as *mut ::core::ffi::c_void,
                oldp as *const ::core::ffi::c_void,
                fixedlen as size_t,
            );
            memset(
                newp.offset(fixedlen as isize) as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                fill as size_t,
            );
            strcpy(
                newp.offset(fixedlen as isize).offset(fill as isize),
                non_white,
            );
            '_c2rust_label_2: {
                if newlen - oldlen == new_line_len_0 - old_line_len {
                } else {
                    __assert_fail(
                        b"newlen - oldlen == new_line_len - old_line_len\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        582 as ::core::ffi::c_uint,
                        b"void shift_block(oparg_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        ml_replace((*curwin.get()).w_cursor.lnum, newp, false_0 != 0);
        changed_bytes((*curwin.get()).w_cursor.lnum, bd.textcol);
        extmark_splice_cols(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            startcol as colnr_T,
            oldlen as colnr_T,
            newlen as colnr_T,
            kExtmarkUndo,
        );
        State.set(oldstate);
        (*curwin.get()).w_cursor.col = oldcol as colnr_T;
        p_ri.set(old_p_ri);
    }
}
