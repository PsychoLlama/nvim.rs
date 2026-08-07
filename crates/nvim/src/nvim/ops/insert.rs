//! `I` and `A` in blockwise Visual mode, and `c` everywhere.
//!
//! `op_insert` is the blockwise pair: it enters Insert mode once, at the
//! block's top line, and afterwards replays what was typed into every other
//! line through `block_insert`.  That "afterwards" is why the function is
//! long -- it has to reconstruct what the user typed by diffing the line
//! against a saved copy, and cope with the insert having changed the line's
//! length, moved the cursor, or been abandoned.  `op_change` is `c`: delete
//! the region, then start an insert, with the same blockwise replay on the
//! way out.  `adjust_cursor_eol` is the shared tail that puts the cursor
//! back on a legal column.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn op_insert(mut oap: *mut oparg_T, mut count1: ::core::ffi::c_int) {
    unsafe {
        let mut pre_textlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ind_pre_col: colnr_T = 0 as colnr_T;
        let mut ind_pre_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
        bd.is_MAX =
            ((*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int) as ::core::ffi::c_int;
        (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
        redraw_curbuf_later(UPD_INVERTED);
        update_screen();
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int {
                let mut old_ve_flags: ::core::ffi::c_uint =
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags;
                if u_save_cursor() == FAIL {
                    return;
                }
                (*curwin.get()).w_onebuf_opt.wo_ve_flags =
                    kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint;
                coladvance_force(if (*oap).op_type == OP_APPEND {
                    (*oap).end_vcol + 1 as colnr_T
                } else {
                    getviscol()
                });
                if (*oap).op_type == OP_APPEND {
                    (*curwin.get()).w_cursor.col -= 1;
                }
                (*curwin.get()).w_onebuf_opt.wo_ve_flags = old_ve_flags;
            }
            block_prep(oap, &raw mut bd, (*oap).start.lnum, true_0 != 0);
            ind_pre_col = getwhitecols_curline() as colnr_T;
            ind_pre_vcol = get_indent();
            pre_textlen = (ml_get_len((*oap).start.lnum) - bd.textcol) as ::core::ffi::c_int;
            if (*oap).op_type == OP_APPEND {
                pre_textlen -= bd.textlen;
            }
        }
        if (*oap).op_type == OP_APPEND {
            if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.coladd == 0 as ::core::ffi::c_int
            {
                (*curwin.get()).w_set_curswant = true_0;
                while *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL
                    && (*curwin.get()).w_cursor.col < bd.textcol as ::core::ffi::c_int + bd.textlen
                {
                    (*curwin.get()).w_cursor.col += 1;
                }
                if bd.is_short != 0 && bd.is_MAX == 0 {
                    if u_save_cursor() == FAIL {
                        return;
                    }
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < bd.endspaces {
                        ins_char(' ' as ::core::ffi::c_int);
                        i += 1;
                    }
                    bd.textlen += bd.endspaces;
                }
            } else {
                (*curwin.get()).w_cursor = (*oap).end;
                check_cursor_col(curwin.get());
                if !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL)
                    && (*oap).start_vcol != (*oap).end_vcol
                {
                    inc_cursor();
                }
            }
        }
        let mut t1: pos_T = (*oap).start;
        let start_insert: pos_T = (*curwin.get()).w_cursor;
        edit(NUL, false_0 != 0, count1);
        if t1.lnum == (*curbuf.get()).b_op_start_orig.lnum
            && lt((*curbuf.get()).b_op_start_orig, t1) as ::core::ffi::c_int != 0
        {
            (*oap).start = (*curbuf.get()).b_op_start_orig;
        }
        if (*curwin.get()).w_cursor.lnum != (*oap).start.lnum
            || got_int.get() as ::core::ffi::c_int != 0
        {
            return;
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            let mut ind_post_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut bd2: block_def = block_def {
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
            let mut did_indent: bool = false_0 != 0;
            let mut ind_post_col: colnr_T = getwhitecols_curline() as colnr_T;
            if (*curbuf.get()).b_op_start.col > ind_pre_col && ind_post_col > ind_pre_col {
                bd.textcol += ind_post_col - ind_pre_col;
                ind_post_vcol = get_indent();
                bd.start_vcol += ind_post_vcol - ind_pre_vcol;
                did_indent = true_0 != 0;
            }
            if (*oap).start.lnum == (*curbuf.get()).b_op_start_orig.lnum
                && bd.is_MAX == 0
                && !did_indent
            {
                let t: ::core::ffi::c_int = getviscol2(
                    (*curbuf.get()).b_op_start_orig.col,
                    (*curbuf.get()).b_op_start_orig.coladd,
                );
                if (*oap).op_type == OP_INSERT
                    && (*oap).start.col + (*oap).start.coladd
                        != (*curbuf.get()).b_op_start_orig.col
                            + (*curbuf.get()).b_op_start_orig.coladd
                {
                    (*oap).start.col = (*curbuf.get()).b_op_start_orig.col;
                    pre_textlen -= (t as colnr_T - (*oap).start_vcol) as ::core::ffi::c_int;
                    (*oap).start_vcol = t as colnr_T;
                } else if (*oap).op_type == OP_APPEND
                    && (*oap).start.col + (*oap).start.coladd
                        >= (*curbuf.get()).b_op_start_orig.col
                            + (*curbuf.get()).b_op_start_orig.coladd
                {
                    (*oap).start.col = (*curbuf.get()).b_op_start_orig.col;
                    pre_textlen += bd.textlen;
                    pre_textlen -= (t as colnr_T - (*oap).start_vcol) as ::core::ffi::c_int;
                    (*oap).start_vcol = t as colnr_T;
                    (*oap).op_type = OP_INSERT;
                }
            }
            if did_indent as ::core::ffi::c_int != 0
                && bd.textcol - ind_post_col > 0 as ::core::ffi::c_int
            {
                (*oap).start.col += ind_post_col - ind_pre_col;
                (*oap).start_vcol += ind_post_vcol - ind_pre_vcol;
                (*oap).end.col += ind_post_col - ind_pre_col;
                (*oap).end_vcol += ind_post_vcol - ind_pre_vcol;
            }
            block_prep(oap, &raw mut bd2, (*oap).start.lnum, true_0 != 0);
            if did_indent as ::core::ffi::c_int != 0
                && bd.textcol - ind_post_col > 0 as ::core::ffi::c_int
            {
                (*oap).start.col -= ind_post_col - ind_pre_col;
                (*oap).start_vcol -= ind_post_vcol - ind_pre_vcol;
                (*oap).end.col -= ind_post_col - ind_pre_col;
                (*oap).end_vcol -= ind_post_vcol - ind_pre_vcol;
            }
            if bd.is_MAX == 0 || bd2.textlen < bd.textlen {
                if (*oap).op_type == OP_APPEND {
                    pre_textlen += bd2.textlen - bd.textlen;
                    if bd2.endspaces != 0 {
                        bd2.textlen -= 1;
                    }
                }
                bd.textcol = bd2.textcol;
                bd.textlen = bd2.textlen;
            }
            let mut firstline: *mut ::core::ffi::c_char = ml_get((*oap).start.lnum);
            let mut len: colnr_T = ml_get_len((*oap).start.lnum);
            let mut add: colnr_T = bd.textcol;
            let mut offset: colnr_T = 0 as colnr_T;
            if (*oap).op_type == OP_APPEND {
                add += bd.textlen;
                if bd.is_MAX != 0
                    && start_insert.lnum == (*Insstart.ptr()).lnum
                    && start_insert.col > (*Insstart.ptr()).col
                {
                    offset = start_insert.col - (*Insstart.ptr()).col;
                    add -= offset;
                    if (*oap).end_vcol > offset {
                        (*oap).end_vcol -= offset as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    } else {
                        return;
                    }
                }
            }
            add = if add < len { add } else { len };
            firstline = firstline.offset(add as isize);
            len -= add;
            let mut ins_len: ::core::ffi::c_int =
                len as ::core::ffi::c_int - pre_textlen - offset as ::core::ffi::c_int;
            if pre_textlen >= 0 as ::core::ffi::c_int && ins_len > 0 as ::core::ffi::c_int {
                let mut ins_text: *mut ::core::ffi::c_char =
                    xmemdupz(firstline as *const ::core::ffi::c_void, ins_len as size_t)
                        as *mut ::core::ffi::c_char;
                if u_save((*oap).start.lnum, (*oap).end.lnum + 1 as linenr_T) == OK {
                    block_insert(
                        oap,
                        ins_text,
                        ins_len as size_t,
                        (*oap).op_type == OP_INSERT,
                        &raw mut bd,
                    );
                }
                (*curwin.get()).w_cursor.col = (*oap).start.col;
                check_cursor(curwin.get());
                xfree(ins_text as *mut ::core::ffi::c_void);
            }
        }
    }
}

pub unsafe extern "C" fn op_change(mut oap: *mut oparg_T) -> ::core::ffi::c_int {
    unsafe {
        let mut pre_textlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut pre_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut firstline: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
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
        let mut l: colnr_T = (*oap).start.col;
        if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            l = 0 as ::core::ffi::c_int as colnr_T;
            can_si.set(may_do_si());
        }
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            if u_save_cursor() == FAIL {
                return false_0;
            }
        } else if op_delete(oap) == FAIL {
            return false_0;
        }
        if l > (*curwin.get()).w_cursor.col
            && !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL)
            && virtual_op.get() as u64 == 0
        {
            inc_cursor();
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            if virtual_op.get() as ::core::ffi::c_int != 0
                && ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                    || gchar_cursor() == NUL)
            {
                coladvance_force(getviscol());
            }
            firstline = ml_get((*oap).start.lnum);
            pre_textlen = ml_get_len((*oap).start.lnum) as ::core::ffi::c_int;
            pre_indent = getwhitecols(firstline) as ::core::ffi::c_int;
            bd.textcol = (*curwin.get()).w_cursor.col;
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            fix_indent();
        }
        let save_finish_op: bool = finish_op.get();
        finish_op.set(false_0 != 0);
        let mut retval: ::core::ffi::c_int =
            edit(NUL, false_0 != 0, 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
        finish_op.set(save_finish_op);
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
            && (*oap).start.lnum != (*oap).end.lnum
            && !got_int.get()
        {
            firstline = ml_get((*oap).start.lnum);
            if bd.textcol > pre_indent {
                let mut new_indent: ::core::ffi::c_int =
                    getwhitecols(firstline) as ::core::ffi::c_int;
                pre_textlen += new_indent - pre_indent;
                bd.textcol += new_indent - pre_indent;
            }
            let mut ins_len: ::core::ffi::c_int = ml_get_len((*oap).start.lnum) - pre_textlen;
            if ins_len > 0 as ::core::ffi::c_int {
                let mut ins_text: *mut ::core::ffi::c_char =
                    xmalloc((ins_len as size_t).wrapping_add(1 as size_t))
                        as *mut ::core::ffi::c_char;
                xmemcpyz(
                    ins_text as *mut ::core::ffi::c_void,
                    firstline.offset(bd.textcol as isize) as *const ::core::ffi::c_void,
                    ins_len as size_t,
                );
                let mut linenr: linenr_T = (*oap).start.lnum + 1 as linenr_T;
                while linenr <= (*oap).end.lnum {
                    block_prep(oap, &raw mut bd, linenr, true_0 != 0);
                    if bd.is_short == 0 || virtual_op.get() as ::core::ffi::c_int != 0 {
                        let mut vpos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        if bd.is_short != 0 {
                            vpos.lnum = linenr;
                            getvpos(curwin.get(), &raw mut vpos, (*oap).start_vcol);
                        } else {
                            vpos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        let mut oldp: *mut ::core::ffi::c_char = ml_get(linenr);
                        let mut newp: *mut ::core::ffi::c_char = xmalloc(
                            (ml_get_len(linenr) as size_t)
                                .wrapping_add(vpos.coladd as size_t)
                                .wrapping_add(ins_len as size_t)
                                .wrapping_add(1 as size_t),
                        )
                            as *mut ::core::ffi::c_char;
                        memmove(
                            newp as *mut ::core::ffi::c_void,
                            oldp as *const ::core::ffi::c_void,
                            bd.textcol as size_t,
                        );
                        let mut newlen: ::core::ffi::c_int = bd.textcol as ::core::ffi::c_int;
                        memset(
                            newp.offset(newlen as isize) as *mut ::core::ffi::c_void,
                            ' ' as ::core::ffi::c_int,
                            vpos.coladd as size_t,
                        );
                        newlen += vpos.coladd as ::core::ffi::c_int;
                        memmove(
                            newp.offset(newlen as isize) as *mut ::core::ffi::c_void,
                            ins_text as *const ::core::ffi::c_void,
                            ins_len as size_t,
                        );
                        newlen += ins_len;
                        strcpy(
                            newp.offset(newlen as isize),
                            oldp.offset(bd.textcol as isize),
                        );
                        ml_replace(linenr, newp, false_0 != 0);
                        extmark_splice_cols(
                            curbuf.get(),
                            linenr as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            bd.textcol,
                            0 as colnr_T,
                            vpos.coladd + ins_len as colnr_T,
                            kExtmarkUndo,
                        );
                    }
                    linenr += 1;
                }
                check_cursor(curwin.get());
                changed_lines(
                    curbuf.get(),
                    (*oap).start.lnum + 1 as linenr_T,
                    0 as colnr_T,
                    (*oap).end.lnum + 1 as linenr_T,
                    0 as linenr_T,
                    true_0 != 0,
                );
                xfree(ins_text as *mut ::core::ffi::c_void);
            }
        }
        auto_format(false_0 != 0, true_0 != 0);
        return retval;
    }
}

pub unsafe extern "C" fn adjust_cursor_eol() {
    unsafe {
        let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(curwin.get());
        let adj_cursor: bool = (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            && gchar_cursor() == NUL
            && cur_ve_flags & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            && cur_ve_flags & kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            && !(restart_edit.get() != 0 || State.get() & MODE_INSERT != 0);
        if !adj_cursor {
            return;
        }
        dec_cursor();
        if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint {
            let mut scol: colnr_T = 0;
            let mut ecol: colnr_T = 0;
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                &raw mut scol,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut ecol,
            );
            (*curwin.get()).w_cursor.coladd = (ecol as ::core::ffi::c_int
                - scol as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as colnr_T;
        }
    }
}
