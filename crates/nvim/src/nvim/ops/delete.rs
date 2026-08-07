//! `d` -- deleting the operator's region.
//!
//! One function with three arms (linewise, blockwise, charwise) and a long
//! prologue they share: yank into the register first unless 'cpoptions'
//! says otherwise, decide whether a linewise delete should really become a
//! charwise one ('cpoptions' `E`, the empty-region rule), and honour
//! `oap->excl_tr_ws`.  The charwise arm is the delicate one, because a
//! region may span lines and the join at the seam has to preserve the
//! cursor's virtual column.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn op_delete(mut oap: *mut oparg_T) -> ::core::ffi::c_int {
    unsafe {
        let mut lnum: linenr_T = 0;
        let mut bd: block_def = block_def {
            startspaces: 0 as ::core::ffi::c_int,
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
        let mut old_lcount: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            return OK;
        }
        if (*oap).empty {
            return u_save_cursor();
        }
        if (*curbuf.get()).b_p_ma == 0 {
            emsg(gettext(
                &raw const e_modifiable as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if VIsual_select.get() as ::core::ffi::c_int != 0
            && (*oap).is_VIsual as ::core::ffi::c_int != 0
        {
            (*oap).regname = VIsual_select_reg.get();
        }
        mb_adjust_opend(oap);
        if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && !(*oap).is_VIsual
            && (*oap).line_count > 1 as linenr_T
            && (*oap).motion_force == NUL
            && (*oap).op_type == OP_DELETE
        {
            let mut ptr: *mut ::core::ffi::c_char =
                ml_get((*oap).end.lnum).offset((*oap).end.col as isize);
            if *ptr as ::core::ffi::c_int != NUL {
                ptr = ptr.offset((*oap).inclusive as ::core::ffi::c_int as isize);
            }
            ptr = skipwhite(ptr);
            if *ptr as ::core::ffi::c_int == NUL
                && inindent(0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            {
                (*oap).motion_type = kMTLineWise;
            }
        }
        if (*oap).motion_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
            && (*oap).line_count == 1 as linenr_T
            && (*oap).op_type == OP_DELETE
            && *ml_get((*oap).start.lnum) as ::core::ffi::c_int == NUL
        {
            if virtual_op.get() as u64 == 0 {
                if !vim_strchr(p_cpo.get(), CPO_EMPTYREGION).is_null() {
                    beep_flush();
                }
                return OK;
            }
        } else {
            if (*oap).regname != '_' as ::core::ffi::c_int {
                let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
                let mut did_yank: bool = false_0 != 0;
                if (*oap).regname != 0 as ::core::ffi::c_int {
                    if !valid_yank_reg((*oap).regname, true_0 != 0) {
                        beep_flush();
                        return OK;
                    }
                    reg = get_yank_register((*oap).regname, YREG_YANK as ::core::ffi::c_int);
                    op_yank_reg(oap, false_0 != 0, reg, is_append_register((*oap).regname));
                    did_yank = true_0 != 0;
                }
                if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                    || (*oap).line_count > 1 as linenr_T
                    || (*oap).use_reg_one as ::core::ffi::c_int != 0
                {
                    shift_delete_registers(is_append_register((*oap).regname));
                    reg = get_y_register(1 as ::core::ffi::c_int);
                    op_yank_reg(oap, false_0 != 0, reg, false_0 != 0);
                    did_yank = true_0 != 0;
                }
                if (*oap).regname == 0 as ::core::ffi::c_int
                    && (*oap).motion_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
                    && (*oap).line_count == 1 as linenr_T
                {
                    reg = get_yank_register(
                        '-' as ::core::ffi::c_int,
                        YREG_YANK as ::core::ffi::c_int,
                    );
                    op_yank_reg(oap, false_0 != 0, reg, false_0 != 0);
                    did_yank = true_0 != 0;
                }
                if did_yank as ::core::ffi::c_int != 0 || (*oap).regname == 0 as ::core::ffi::c_int
                {
                    if reg.is_null() {
                        abort();
                    }
                    crate::src::nvim::clipboard::set_clipboard((*oap).regname, reg as *mut _);
                    do_autocmd_textyankpost(oap, reg);
                }
            }
            if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                if u_save(
                    (*oap).start.lnum - 1 as linenr_T,
                    (*oap).end.lnum + 1 as linenr_T,
                ) == FAIL
                {
                    return FAIL;
                }
                lnum = (*curwin.get()).w_cursor.lnum;
                while lnum <= (*oap).end.lnum {
                    block_prep(oap, &raw mut bd, lnum, true_0 != 0);
                    if bd.textlen != 0 as ::core::ffi::c_int {
                        if lnum == (*curwin.get()).w_cursor.lnum {
                            (*curwin.get()).w_cursor.col =
                                (bd.textcol as ::core::ffi::c_int + bd.startspaces) as colnr_T;
                            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        let mut n: ::core::ffi::c_int = bd.textlen - bd.startspaces - bd.endspaces;
                        let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
                        let mut newp: *mut ::core::ffi::c_char = xmalloc(
                            (ml_get_len(lnum) as size_t)
                                .wrapping_sub(n as size_t)
                                .wrapping_add(1 as size_t),
                        )
                            as *mut ::core::ffi::c_char;
                        memmove(
                            newp as *mut ::core::ffi::c_void,
                            oldp as *const ::core::ffi::c_void,
                            bd.textcol as size_t,
                        );
                        memset(
                            newp.offset(bd.textcol as isize) as *mut ::core::ffi::c_void,
                            ' ' as ::core::ffi::c_int,
                            (bd.startspaces as size_t).wrapping_add(bd.endspaces as size_t),
                        );
                        strcpy(
                            newp.offset(bd.textcol as isize)
                                .offset(bd.startspaces as isize)
                                .offset(bd.endspaces as isize),
                            oldp.offset(bd.textcol as isize).offset(bd.textlen as isize),
                        );
                        ml_replace(lnum, newp, false_0 != 0);
                        extmark_splice_cols(
                            curbuf.get(),
                            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            bd.textcol,
                            bd.textlen as colnr_T,
                            bd.startspaces as colnr_T + bd.endspaces as colnr_T,
                            kExtmarkUndo,
                        );
                    }
                    lnum += 1;
                }
                check_cursor_col(curwin.get());
                changed_lines(
                    curbuf.get(),
                    (*curwin.get()).w_cursor.lnum,
                    (*curwin.get()).w_cursor.col,
                    (*oap).end.lnum + 1 as linenr_T,
                    0 as linenr_T,
                    true_0 != 0,
                );
                (*oap).line_count = 0 as ::core::ffi::c_int as linenr_T;
            } else if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            {
                if (*oap).op_type == OP_CHANGE {
                    if (*oap).line_count > 1 as linenr_T {
                        lnum = (*curwin.get()).w_cursor.lnum;
                        (*curwin.get()).w_cursor.lnum += 1;
                        del_lines((*oap).line_count - 1 as linenr_T, true_0 != 0);
                        (*curwin.get()).w_cursor.lnum = lnum;
                    }
                    if u_save_cursor() == FAIL {
                        return FAIL;
                    }
                    if (*curbuf.get()).b_p_ai != 0 {
                        beginline(BL_WHITE as ::core::ffi::c_int);
                        did_ai.set(true_0 != 0);
                        ai_col.set((*curwin.get()).w_cursor.col);
                    } else {
                        beginline(0 as ::core::ffi::c_int);
                    }
                    truncate_line(false_0);
                    if (*oap).line_count > 1 as linenr_T {
                        u_clearline(curbuf.get());
                    }
                } else {
                    del_lines((*oap).line_count, true_0 != 0);
                    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                    u_clearline(curbuf.get());
                }
            } else {
                if virtual_op.get() as u64 != 0 {
                    if gchar_pos(&raw mut (*oap).start) == '\t' as ::core::ffi::c_int {
                        let mut endcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if u_save_cursor() == FAIL {
                            return FAIL;
                        }
                        if (*oap).line_count == 1 as linenr_T {
                            endcol = getviscol2((*oap).end.col, (*oap).end.coladd);
                        }
                        coladvance_force(getviscol2((*oap).start.col, (*oap).start.coladd));
                        (*oap).start = (*curwin.get()).w_cursor;
                        if (*oap).line_count == 1 as linenr_T {
                            coladvance(curwin.get(), endcol as colnr_T);
                            (*oap).end.col = (*curwin.get()).w_cursor.col;
                            (*oap).end.coladd = (*curwin.get()).w_cursor.coladd;
                            (*curwin.get()).w_cursor = (*oap).start;
                        }
                    }
                    if gchar_pos(&raw mut (*oap).end) == '\t' as ::core::ffi::c_int
                        && (*oap).end.coladd == 0 as ::core::ffi::c_int
                        && (*oap).inclusive as ::core::ffi::c_int != 0
                    {
                        if u_save(
                            (*oap).end.lnum - 1 as linenr_T,
                            (*oap).end.lnum + 1 as linenr_T,
                        ) == FAIL
                        {
                            return FAIL;
                        }
                        (*curwin.get()).w_cursor = (*oap).end;
                        coladvance_force(getviscol2((*oap).end.col, (*oap).end.coladd));
                        (*oap).end = (*curwin.get()).w_cursor;
                        (*curwin.get()).w_cursor = (*oap).start;
                    }
                    mb_adjust_opend(oap);
                }
                if (*oap).line_count == 1 as linenr_T {
                    if u_save_cursor() == FAIL {
                        return FAIL;
                    }
                    if !vim_strchr(p_cpo.get(), CPO_DOLLAR).is_null()
                        && (*oap).op_type == OP_CHANGE
                        && (*oap).end.lnum == (*curwin.get()).w_cursor.lnum
                        && !(*oap).is_VIsual
                    {
                        display_dollar((*oap).end.col - !(*oap).inclusive as ::core::ffi::c_int);
                    }
                    let mut n_0: ::core::ffi::c_int = (*oap).end.col as ::core::ffi::c_int
                        - (*oap).start.col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        - !(*oap).inclusive as ::core::ffi::c_int;
                    if virtual_op.get() as u64 != 0 {
                        let mut len: ::core::ffi::c_int = get_cursor_line_len();
                        if (*oap).end.coladd != 0 as ::core::ffi::c_int
                            && (*oap).end.col >= len - 1 as ::core::ffi::c_int
                            && !((*oap).start.coladd != 0
                                && (*oap).end.col >= len - 1 as ::core::ffi::c_int)
                        {
                            n_0 += 1;
                        }
                        if n_0 == 0 as ::core::ffi::c_int
                            && (*oap).start.coladd != (*oap).end.coladd
                        {
                            n_0 = 1 as ::core::ffi::c_int;
                        }
                        if gchar_cursor() != NUL {
                            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        }
                    }
                    del_bytes(
                        n_0,
                        virtual_op.get() as u64 == 0,
                        (*oap).op_type == OP_DELETE && !(*oap).is_VIsual,
                    );
                } else {
                    let mut curpos: pos_T = pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    };
                    if u_save(
                        (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                        (*curwin.get()).w_cursor.lnum + (*oap).line_count,
                    ) == FAIL
                    {
                        return FAIL;
                    }
                    (*curbuf_splice_pending.ptr()) += 1;
                    let mut startpos: pos_T = (*curwin.get()).w_cursor;
                    let mut deleted_bytes: bcount_t = get_region_bytecount(
                        curbuf.get(),
                        startpos.lnum,
                        (*oap).end.lnum,
                        startpos.col,
                        (*oap).end.col,
                    ) + (*oap).inclusive as bcount_t;
                    truncate_line(true_0);
                    curpos = (*curwin.get()).w_cursor;
                    (*curwin.get()).w_cursor.lnum += 1;
                    del_lines((*oap).line_count - 2 as linenr_T, false_0 != 0);
                    let mut n_1: ::core::ffi::c_int = (*oap).end.col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        - !(*oap).inclusive as ::core::ffi::c_int;
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    del_bytes(
                        n_1,
                        virtual_op.get() as u64 == 0,
                        (*oap).op_type == OP_DELETE && !(*oap).is_VIsual,
                    );
                    (*curwin.get()).w_cursor = curpos;
                    do_join(
                        2 as size_t,
                        false_0 != 0,
                        false_0 != 0,
                        false_0 != 0,
                        false_0 != 0,
                    );
                    (*curbuf_splice_pending.ptr()) -= 1;
                    extmark_splice(
                        curbuf.get(),
                        startpos.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        startpos.col,
                        (*oap).line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        n_1 as colnr_T,
                        deleted_bytes,
                        0 as ::core::ffi::c_int,
                        0 as colnr_T,
                        0 as bcount_t,
                        kExtmarkUndo,
                    );
                }
                if (*oap).op_type == OP_DELETE {
                    auto_format(false_0 != 0, true_0 != 0);
                }
            }
            msgmore(
                (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int
                    - old_lcount as ::core::ffi::c_int,
            );
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
                (*curbuf.get()).b_op_end.col = (*oap).start.col;
            } else {
                (*curbuf.get()).b_op_end = (*oap).start;
            }
            (*curbuf.get()).b_op_start = (*oap).start;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn mb_adjust_opend(mut oap: *mut oparg_T) {
    unsafe {
        if !(*oap).inclusive {
            return;
        }
        let mut line: *const ::core::ffi::c_char = ml_get((*oap).end.lnum);
        let mut ptr: *const ::core::ffi::c_char = line.offset((*oap).end.col as isize);
        if *ptr as ::core::ffi::c_int != NUL {
            ptr = ptr.offset(-(utf_head_off(line, ptr) as isize));
            ptr = ptr.offset((utfc_ptr2len(ptr) - 1 as ::core::ffi::c_int) as isize);
            (*oap).end.col = ptr.offset_from(line) as colnr_T;
        }
    }
}

#[inline]
unsafe extern "C" fn is_append_register(mut regname: ::core::ffi::c_int) -> bool {
    return regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint;
}
