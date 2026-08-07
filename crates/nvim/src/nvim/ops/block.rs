//! Blockwise geometry: turning a CTRL-V region into per-line byte ranges.
//!
//! A blockwise operator sees a rectangle of *screen columns*, and the buffer
//! holds bytes; `block_prep` is the translation, filling a `block_def` for one
//! line with where the block starts and ends in that line, how much white
//! space has to be padded on either side because a tab or a wide character
//! straddles an edge, and whether the line is too short to reach the block at
//! all.  `charwise_block_prep` answers the same question for a charwise region
//! being treated as a block (the `"=` and API paths), `get_op_vcol` decides the
//! column pair an operator's region spans, and `block_insert` is the write
//! half -- `I` and `A` inserting the same text into every line at that
//! column.
//!
//! `reset_lbr`/`restore_lbr` bracket all of it: 'linebreak' changes what
//! `getvcol` answers, and every column here has to be measured with it off.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn block_insert(
    mut oap: *mut oparg_T,
    mut s: *const ::core::ffi::c_char,
    mut slen: size_t,
    mut b_insert: bool,
    mut bdp: *mut block_def,
) {
    unsafe {
        let mut ts_val: ::core::ffi::c_int = 0;
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut offset: colnr_T = 0;
        let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut oldp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut oldstate: ::core::ffi::c_int = State.get();
        State.set(MODE_INSERT);
        let mut lnum: linenr_T = (*oap).start.lnum + 1 as linenr_T;
        while lnum <= (*oap).end.lnum {
            block_prep(oap, bdp, lnum, true_0 != 0);
            if !((*bdp).is_short != 0 && b_insert as ::core::ffi::c_int != 0) {
                oldp = ml_get(lnum);
                if b_insert {
                    ts_val = (*bdp).start_char_vcols as ::core::ffi::c_int;
                    spaces = (*bdp).startspaces;
                    if spaces != 0 as ::core::ffi::c_int {
                        count = ts_val - 1 as ::core::ffi::c_int;
                    }
                    offset = (*bdp).textcol;
                } else {
                    ts_val = (*bdp).end_char_vcols as ::core::ffi::c_int;
                    if (*bdp).is_short == 0 {
                        spaces = if (*bdp).endspaces != 0 {
                            ts_val - (*bdp).endspaces
                        } else {
                            0 as ::core::ffi::c_int
                        };
                        if spaces != 0 as ::core::ffi::c_int {
                            count = ts_val - 1 as ::core::ffi::c_int;
                        }
                        offset = ((*bdp).textcol as ::core::ffi::c_int + (*bdp).textlen
                            - (spaces != 0 as ::core::ffi::c_int) as ::core::ffi::c_int)
                            as colnr_T;
                    } else {
                        if (*bdp).is_MAX == 0 {
                            spaces = (*oap).end_vcol as ::core::ffi::c_int
                                - (*bdp).end_vcol as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int;
                        }
                        count = spaces;
                        offset = ((*bdp).textcol as ::core::ffi::c_int + (*bdp).textlen) as colnr_T;
                    }
                }
                if spaces > 0 as ::core::ffi::c_int {
                    offset -= utf_head_off(oldp, oldp.offset(offset as isize));
                }
                spaces = if spaces > 0 as ::core::ffi::c_int {
                    spaces
                } else {
                    0 as ::core::ffi::c_int
                };
                '_c2rust_label: {
                    if count >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ops.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        647 as ::core::ffi::c_uint,
                        b"void block_insert(oparg_T *, const char *, size_t, _Bool, struct block_def *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                newp = xmalloc(
                    (ml_get_len(lnum) as size_t)
                        .wrapping_add(spaces as size_t)
                        .wrapping_add(slen)
                        .wrapping_add(
                            if spaces > 0 as ::core::ffi::c_int && (*bdp).is_short == 0 {
                                (ts_val - spaces) as size_t
                            } else {
                                0 as size_t
                            },
                        )
                        .wrapping_add(count as size_t)
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                memmove(
                    newp as *mut ::core::ffi::c_void,
                    oldp as *const ::core::ffi::c_void,
                    offset as size_t,
                );
                oldp = oldp.offset(offset as isize);
                let mut startcol: ::core::ffi::c_int = offset as ::core::ffi::c_int;
                memset(
                    newp.offset(offset as isize) as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    spaces as size_t,
                );
                memmove(
                    newp.offset(offset as isize).offset(spaces as isize)
                        as *mut ::core::ffi::c_void,
                    s as *const ::core::ffi::c_void,
                    slen,
                );
                offset += slen as ::core::ffi::c_int;
                let mut skipped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if spaces > 0 as ::core::ffi::c_int && (*bdp).is_short == 0 {
                    if *oldp as ::core::ffi::c_int == TAB {
                        memset(
                            newp.offset(offset as isize).offset(spaces as isize)
                                as *mut ::core::ffi::c_void,
                            ' ' as ::core::ffi::c_int,
                            (ts_val - spaces) as size_t,
                        );
                        oldp = oldp.offset(1);
                        count += 1;
                        skipped = 1 as ::core::ffi::c_int;
                    } else {
                        count = spaces;
                    }
                }
                if spaces > 0 as ::core::ffi::c_int {
                    offset += count;
                }
                strcpy(newp.offset(offset as isize), oldp);
                ml_replace(lnum, newp, false_0 != 0);
                extmark_splice_cols(
                    curbuf.get(),
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    startcol as colnr_T,
                    skipped as colnr_T,
                    offset - startcol as colnr_T,
                    kExtmarkUndo,
                );
                if lnum == (*oap).end.lnum {
                    (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
                    (*curbuf.get()).b_op_end.col = offset;
                    if (*curbuf.get()).b_visual.vi_end.coladd != 0 {
                        (*curbuf.get()).b_visual.vi_end.col +=
                            (*curbuf.get()).b_visual.vi_end.coladd;
                        (*curbuf.get()).b_visual.vi_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    }
                }
            }
            lnum += 1;
        }
        State.set(oldstate);
        if (*oap).start.lnum < (*oap).end.lnum {
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum + 1 as linenr_T,
                0 as colnr_T,
                (*oap).end.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
        }
    }
}

pub unsafe extern "C" fn reset_lbr() -> bool {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_lbr == 0 {
            return false_0 != 0;
        }
        (*curwin.get()).w_onebuf_opt.wo_lbr = false_0;
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn restore_lbr(mut lbr_saved: bool) {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 || !lbr_saved {
            return;
        }
        (*curwin.get()).w_onebuf_opt.wo_lbr = true_0;
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    }
}

pub unsafe extern "C" fn block_prep(
    mut oap: *mut oparg_T,
    mut bdp: *mut block_def,
    mut lnum: linenr_T,
    mut is_del: bool,
) {
    unsafe {
        let mut incr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let lbr_saved: bool = reset_lbr();
        (*bdp).startspaces = 0 as ::core::ffi::c_int;
        (*bdp).endspaces = 0 as ::core::ffi::c_int;
        (*bdp).textlen = 0 as ::core::ffi::c_int;
        (*bdp).start_vcol = 0 as ::core::ffi::c_int as colnr_T;
        (*bdp).end_vcol = 0 as ::core::ffi::c_int as colnr_T;
        (*bdp).is_short = false_0;
        (*bdp).is_oneChar = false_0;
        (*bdp).pre_whitesp = 0 as ::core::ffi::c_int;
        (*bdp).pre_whitesp_c = 0 as ::core::ffi::c_int;
        (*bdp).end_char_vcols = 0 as ::core::ffi::c_int as colnr_T;
        (*bdp).start_char_vcols = 0 as ::core::ffi::c_int as colnr_T;
        let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut prev_pstart: *mut ::core::ffi::c_char = line;
        let mut csarg: CharsizeArg = CharsizeArg::default();
        let mut cstype: CharsizeKind = init_charsize_arg(&mut csarg, curwin.get(), lnum, line);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        let mut vcol: ::core::ffi::c_int = (*bdp).start_vcol as ::core::ffi::c_int;
        while vcol < (*oap).start_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
            incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            vcol += incr;
            if ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
                (*bdp).pre_whitesp += incr;
                (*bdp).pre_whitesp_c += 1;
            } else {
                (*bdp).pre_whitesp = 0 as ::core::ffi::c_int;
                (*bdp).pre_whitesp_c = 0 as ::core::ffi::c_int;
            }
            prev_pstart = ci.ptr;
            ci = utfc_next(ci);
        }
        (*bdp).start_vcol = vcol as colnr_T;
        let mut pstart: *mut ::core::ffi::c_char = ci.ptr;
        (*bdp).start_char_vcols = incr as colnr_T;
        if (*bdp).start_vcol < (*oap).start_vcol {
            (*bdp).end_vcol = (*bdp).start_vcol;
            (*bdp).is_short = true_0;
            if !is_del || (*oap).op_type == OP_APPEND {
                (*bdp).endspaces = (*oap).end_vcol as ::core::ffi::c_int
                    - (*oap).start_vcol as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
            }
        } else {
            (*bdp).startspaces = ((*bdp).start_vcol - (*oap).start_vcol) as ::core::ffi::c_int;
            if is_del as ::core::ffi::c_int != 0 && (*bdp).startspaces != 0 {
                (*bdp).startspaces =
                    (*bdp).start_char_vcols as ::core::ffi::c_int - (*bdp).startspaces;
            }
            let mut pend: *mut ::core::ffi::c_char = pstart;
            (*bdp).end_vcol = (*bdp).start_vcol;
            if (*bdp).end_vcol > (*oap).end_vcol {
                (*bdp).is_oneChar = true_0;
                if (*oap).op_type == OP_INSERT {
                    (*bdp).endspaces =
                        (*bdp).start_char_vcols as ::core::ffi::c_int - (*bdp).startspaces;
                } else if (*oap).op_type == OP_APPEND {
                    (*bdp).startspaces += (*oap).end_vcol as ::core::ffi::c_int
                        - (*oap).start_vcol as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    (*bdp).endspaces =
                        (*bdp).start_char_vcols as ::core::ffi::c_int - (*bdp).startspaces;
                } else {
                    (*bdp).startspaces = (*oap).end_vcol as ::core::ffi::c_int
                        - (*oap).start_vcol as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    if is_del as ::core::ffi::c_int != 0 && (*oap).op_type != OP_LSHIFT {
                        (*bdp).startspaces = ((*bdp).start_char_vcols
                            - ((*bdp).start_vcol - (*oap).start_vcol))
                            as ::core::ffi::c_int;
                        (*bdp).endspaces = (*bdp).end_vcol as ::core::ffi::c_int
                            - (*oap).end_vcol as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int;
                    }
                }
            } else {
                cstype = init_charsize_arg(&mut csarg, curwin.get(), lnum, line);
                ci = utf_ptr2StrCharInfo(pend);
                vcol = (*bdp).end_vcol as ::core::ffi::c_int;
                let mut prev_pend: *mut ::core::ffi::c_char = pend;
                while vcol <= (*oap).end_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
                    prev_pend = ci.ptr;
                    incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
                    vcol += incr;
                    ci = utfc_next(ci);
                }
                (*bdp).end_vcol = vcol as colnr_T;
                pend = ci.ptr;
                if (*bdp).end_vcol <= (*oap).end_vcol
                    && (!is_del || (*oap).op_type == OP_APPEND || (*oap).op_type == OP_REPLACE)
                {
                    (*bdp).is_short = true_0;
                    if (*oap).op_type == OP_APPEND || virtual_op.get() as ::core::ffi::c_int != 0 {
                        (*bdp).endspaces = (*oap).end_vcol as ::core::ffi::c_int
                            - (*bdp).end_vcol as ::core::ffi::c_int
                            + (*oap).inclusive as ::core::ffi::c_int;
                    }
                } else if (*bdp).end_vcol > (*oap).end_vcol {
                    (*bdp).endspaces = (*bdp).end_vcol as ::core::ffi::c_int
                        - (*oap).end_vcol as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int;
                    if !is_del && (*bdp).endspaces != 0 {
                        (*bdp).endspaces = incr - (*bdp).endspaces;
                        if pend != pstart {
                            pend = prev_pend;
                        }
                    }
                }
            }
            (*bdp).end_char_vcols = incr as colnr_T;
            if is_del as ::core::ffi::c_int != 0 && (*bdp).startspaces != 0 {
                pstart = prev_pstart;
            }
            (*bdp).textlen = pend.offset_from(pstart) as ::core::ffi::c_int;
        }
        (*bdp).textcol = pstart.offset_from(line) as colnr_T;
        (*bdp).textstart = pstart;
        restore_lbr(lbr_saved);
    }
}

pub unsafe extern "C" fn charwise_block_prep(
    mut start: pos_T,
    mut end: pos_T,
    mut bdp: *mut block_def,
    mut lnum: linenr_T,
    mut inclusive: bool,
) {
    unsafe {
        let mut startcol: colnr_T = 0 as colnr_T;
        let mut endcol: colnr_T = MAXCOL as ::core::ffi::c_int;
        let mut cs: colnr_T = 0;
        let mut ce: colnr_T = 0;
        let mut p: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut plen: ::core::ffi::c_int = ml_get_len(lnum);
        (*bdp).startspaces = 0 as ::core::ffi::c_int;
        (*bdp).endspaces = 0 as ::core::ffi::c_int;
        (*bdp).is_oneChar = false_0;
        (*bdp).start_char_vcols = 0 as ::core::ffi::c_int as colnr_T;
        if lnum == start.lnum {
            startcol = start.col;
            if virtual_op.get() as u64 != 0 {
                getvcol(
                    curwin.get(),
                    &raw mut start,
                    &raw mut cs,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut ce,
                );
                if ce != cs && start.coladd > 0 as ::core::ffi::c_int {
                    (*bdp).start_char_vcols = (ce as ::core::ffi::c_int - cs as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int)
                        as colnr_T;
                    (*bdp).startspaces =
                        if (*bdp).start_char_vcols - start.coladd > 0 as ::core::ffi::c_int {
                            (*bdp).start_char_vcols as ::core::ffi::c_int
                                - start.coladd as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        };
                    startcol += 1;
                }
            }
        }
        if lnum == end.lnum {
            endcol = end.col;
            if virtual_op.get() as u64 != 0 {
                getvcol(
                    curwin.get(),
                    &raw mut end,
                    &raw mut cs,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut ce,
                );
                if *p.offset(endcol as isize) as ::core::ffi::c_int == NUL
                    || cs + end.coladd < ce
                        && utf_head_off(p, p.offset(endcol as isize)) == 0 as ::core::ffi::c_int
                {
                    if start.lnum == end.lnum && start.col == end.col {
                        (*bdp).is_oneChar = true_0;
                        (*bdp).startspaces = end.coladd as ::core::ffi::c_int
                            - start.coladd as ::core::ffi::c_int
                            + inclusive as ::core::ffi::c_int;
                        endcol = startcol;
                    } else {
                        (*bdp).endspaces =
                            end.coladd as ::core::ffi::c_int + inclusive as ::core::ffi::c_int;
                        endcol -= inclusive as ::core::ffi::c_int;
                    }
                }
            }
        }
        if endcol == MAXCOL as ::core::ffi::c_int {
            endcol = ml_get_len(lnum);
        }
        if startcol > endcol || (*bdp).is_oneChar != 0 {
            (*bdp).textlen = 0 as ::core::ffi::c_int;
        } else {
            (*bdp).textlen = endcol as ::core::ffi::c_int - startcol as ::core::ffi::c_int
                + inclusive as ::core::ffi::c_int;
        }
        (*bdp).textcol = startcol;
        (*bdp).textstart = if startcol <= plen {
            p.offset(startcol as isize)
        } else {
            p
        };
    }
}

pub(crate) unsafe extern "C" fn get_op_vcol(
    mut oap: *mut oparg_T,
    mut redo_VIsual_vcol: colnr_T,
    mut initial: bool,
) {
    unsafe {
        let mut start: colnr_T = 0;
        let mut end: colnr_T = 0;
        if VIsual_mode.get() != Ctrl_V || !initial && (*oap).end.col < (*curwin.get()).w_view_width
        {
            return;
        }
        (*oap).motion_type = kMTBlockWise;
        mark_mb_adjustpos((*curwin.get()).w_buffer, &raw mut (*oap).end);
        getvvcol(
            curwin.get(),
            &raw mut (*oap).start,
            &raw mut (*oap).start_vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut (*oap).end_vcol,
        );
        if !redo_VIsual_busy.get() {
            getvvcol(
                curwin.get(),
                &raw mut (*oap).end,
                &raw mut start,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut end,
            );
            (*oap).start_vcol = if (*oap).start_vcol < start {
                (*oap).start_vcol
            } else {
                start
            };
            if end > (*oap).end_vcol {
                if initial as ::core::ffi::c_int != 0
                    && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                    && start >= 1 as ::core::ffi::c_int
                    && start as ::core::ffi::c_int - 1 as ::core::ffi::c_int >= (*oap).end_vcol
                {
                    (*oap).end_vcol =
                        (start as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
                } else {
                    (*oap).end_vcol = end;
                }
            }
        }
        if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col = MAXCOL as ::core::ffi::c_int as colnr_T;
            (*oap).end_vcol = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
            while (*curwin.get()).w_cursor.lnum <= (*oap).end.lnum {
                getvvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    ::core::ptr::null_mut::<colnr_T>(),
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut end,
                );
                (*oap).end_vcol = if (*oap).end_vcol > end {
                    (*oap).end_vcol
                } else {
                    end
                };
                (*curwin.get()).w_cursor.lnum += 1;
            }
        } else if redo_VIsual_busy.get() {
            (*oap).end_vcol = ((*oap).start_vcol as ::core::ffi::c_int
                + redo_VIsual_vcol as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as colnr_T;
        }
        (*curwin.get()).w_cursor.lnum = (*oap).end.lnum;
        coladvance(curwin.get(), (*oap).end_vcol);
        (*oap).end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = (*oap).start;
        coladvance(curwin.get(), (*oap).start_vcol);
        (*oap).start = (*curwin.get()).w_cursor;
    }
}
