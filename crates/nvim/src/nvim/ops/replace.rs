//! `r` -- overwriting every character in the region with one character.
//!
//! `op_replace` walks the region and `pbyte` writes one byte at a time
//! through the undo layer.  Two things make it more than a memset: a
//! multi-byte replacement character is a different width from what it
//! replaces, so the line has to be rebuilt rather than patched, and a
//! blockwise replace has to pad short lines out to the block's right edge
//! first.  `replace_character` is the Insert-mode Replace-mode entry.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn pbyte(mut lp: pos_T, mut c: ::core::ffi::c_int) {
    unsafe {
        '_c2rust_label: {
            if c <= 127 as ::core::ffi::c_int * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"c <= UCHAR_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1054 as ::core::ffi::c_uint,
                    b"void pbyte(pos_T, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut p: *mut ::core::ffi::c_char = ml_get_buf_mut(curbuf.get(), lp.lnum);
        let mut len: colnr_T = (*curbuf.get()).b_ml.ml_line_textlen;
        if lp.col >= len {
            lp.col = (if len > 1 as ::core::ffi::c_int {
                len as ::core::ffi::c_int - 2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
        }
        *p.offset(lp.col as isize) = c as ::core::ffi::c_char;
        if curbuf_splice_pending.get() == 0 {
            extmark_splice_cols(
                curbuf.get(),
                lp.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                lp.col,
                1 as colnr_T,
                1 as colnr_T,
                kExtmarkUndo,
            );
        }
    }
}

unsafe extern "C" fn replace_character(mut c: ::core::ffi::c_int) {
    unsafe {
        let n: ::core::ffi::c_int = State.get();
        State.set(MODE_REPLACE);
        ins_char(c);
        State.set(n);
        dec_cursor();
    }
}

pub(crate) unsafe extern "C" fn op_replace(
    mut oap: *mut oparg_T,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut n: ::core::ffi::c_int = 0;
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
        let mut after_p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut had_ctrl_v_cr: bool = false_0 != 0;
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 || (*oap).empty as ::core::ffi::c_int != 0
        {
            return OK;
        }
        if c == REPLACE_CR_NCHAR as ::core::ffi::c_int {
            had_ctrl_v_cr = true_0 != 0;
            c = CAR;
        } else if c == REPLACE_NL_NCHAR as ::core::ffi::c_int {
            had_ctrl_v_cr = true_0 != 0;
            c = NL;
        }
        mb_adjust_opend(oap);
        if u_save(
            (*oap).start.lnum - 1 as linenr_T,
            (*oap).end.lnum + 1 as linenr_T,
        ) == FAIL
        {
            return FAIL;
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            bd.is_MAX =
                ((*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int) as ::core::ffi::c_int;
            while (*curwin.get()).w_cursor.lnum <= (*oap).end.lnum {
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                block_prep(oap, &raw mut bd, (*curwin.get()).w_cursor.lnum, true_0 != 0);
                if !(bd.textlen == 0 as ::core::ffi::c_int
                    && (virtual_op.get() as u64 == 0 || bd.is_MAX != 0))
                {
                    if virtual_op.get() as ::core::ffi::c_int != 0
                        && bd.is_short != 0
                        && *bd.textstart as ::core::ffi::c_int == NUL
                    {
                        let mut vpos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        vpos.lnum = (*curwin.get()).w_cursor.lnum;
                        getvpos(curwin.get(), &raw mut vpos, (*oap).start_vcol);
                        bd.startspaces += vpos.coladd as ::core::ffi::c_int;
                        n = bd.startspaces;
                    } else {
                        n = if bd.startspaces != 0 {
                            bd.start_char_vcols as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        };
                    }
                    n += if bd.endspaces != 0
                        && bd.is_oneChar == 0
                        && bd.end_char_vcols > 0 as ::core::ffi::c_int
                    {
                        bd.end_char_vcols as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    let mut numc: ::core::ffi::c_int = (*oap).end_vcol as ::core::ffi::c_int
                        - (*oap).start_vcol as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    if bd.is_short != 0 && (virtual_op.get() as u64 == 0 || bd.is_MAX != 0) {
                        numc -= (*oap).end_vcol as ::core::ffi::c_int
                            - bd.end_vcol as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int;
                    }
                    if utf_char2cells(c) > 1 as ::core::ffi::c_int {
                        if numc & 1 as ::core::ffi::c_int != 0 && bd.is_short == 0 {
                            bd.endspaces += 1;
                            n += 1;
                        }
                        numc = numc / 2 as ::core::ffi::c_int;
                    }
                    let mut num_chars: ::core::ffi::c_int = numc;
                    numc *= utf_char2len(c);
                    let mut oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                    let mut oldlen: colnr_T = get_cursor_line_len();
                    let mut newp_size: size_t =
                        (bd.textcol as size_t).wrapping_add(bd.startspaces as size_t);
                    if had_ctrl_v_cr as ::core::ffi::c_int != 0
                        || c != '\r' as ::core::ffi::c_int && c != '\n' as ::core::ffi::c_int
                    {
                        newp_size = newp_size.wrapping_add(numc as size_t);
                        if bd.is_short == 0 {
                            newp_size = newp_size.wrapping_add(
                                (bd.endspaces + oldlen as ::core::ffi::c_int
                                    - bd.textcol as ::core::ffi::c_int
                                    - bd.textlen) as size_t,
                            );
                        }
                    }
                    let mut newp: *mut ::core::ffi::c_char =
                        xmallocz(newp_size) as *mut ::core::ffi::c_char;
                    memmove(
                        newp as *mut ::core::ffi::c_void,
                        oldp as *const ::core::ffi::c_void,
                        bd.textcol as size_t,
                    );
                    oldp = oldp.offset((bd.textcol as ::core::ffi::c_int + bd.textlen) as isize);
                    memset(
                        newp.offset(bd.textcol as isize) as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        bd.startspaces as size_t,
                    );
                    let mut after_p_len: size_t = 0 as size_t;
                    let mut col: ::core::ffi::c_int = oldlen as ::core::ffi::c_int
                        - bd.textcol as ::core::ffi::c_int
                        - bd.textlen
                        + 1 as ::core::ffi::c_int;
                    '_c2rust_label: {
                        if col >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"col >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1179 as ::core::ffi::c_uint,
                                b"int op_replace(oparg_T *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    let mut newrows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut newcols: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if had_ctrl_v_cr as ::core::ffi::c_int != 0
                        || c != '\r' as ::core::ffi::c_int && c != '\n' as ::core::ffi::c_int
                    {
                        let mut newp_len: ::core::ffi::c_int =
                            bd.textcol as ::core::ffi::c_int + bd.startspaces;
                        loop {
                            num_chars -= 1;
                            if num_chars < 0 as ::core::ffi::c_int {
                                break;
                            }
                            newp_len += utf_char2bytes(c, newp.offset(newp_len as isize));
                        }
                        if bd.is_short == 0 {
                            memset(
                                newp.offset(newp_len as isize) as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                bd.endspaces as size_t,
                            );
                            newp_len += bd.endspaces;
                            memmove(
                                newp.offset(newp_len as isize) as *mut ::core::ffi::c_void,
                                oldp as *const ::core::ffi::c_void,
                                col as size_t,
                            );
                        }
                        newcols = (newp_len as colnr_T - bd.textcol) as ::core::ffi::c_int;
                    } else {
                        after_p_len = col as size_t;
                        after_p = xmalloc(after_p_len) as *mut ::core::ffi::c_char;
                        memmove(
                            after_p as *mut ::core::ffi::c_void,
                            oldp as *const ::core::ffi::c_void,
                            after_p_len,
                        );
                        newrows = 1 as ::core::ffi::c_int;
                    }
                    ml_replace((*curwin.get()).w_cursor.lnum, newp, false_0 != 0);
                    (*curbuf_splice_pending.ptr()) += 1;
                    let mut baselnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                    if !after_p.is_null() {
                        let c2rust_fresh7 = (*curwin.get()).w_cursor.lnum;
                        (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_cursor.lnum + 1;
                        ml_append(c2rust_fresh7, after_p, after_p_len as colnr_T, false_0 != 0);
                        appended_lines_mark((*curwin.get()).w_cursor.lnum, 1 as ::core::ffi::c_int);
                        (*oap).end.lnum += 1;
                        xfree(after_p as *mut ::core::ffi::c_void);
                    }
                    (*curbuf_splice_pending.ptr()) -= 1;
                    extmark_splice(
                        curbuf.get(),
                        baselnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        bd.textcol,
                        0 as ::core::ffi::c_int,
                        bd.textlen as colnr_T,
                        bd.textlen as bcount_t,
                        newrows,
                        newcols as colnr_T,
                        (newrows + newcols) as bcount_t,
                        kExtmarkUndo,
                    );
                }
                (*curwin.get()).w_cursor.lnum += 1;
            }
        } else {
            if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                (*oap).end.col = ml_get_len((*oap).end.lnum);
                if (*oap).end.col != 0 {
                    (*oap).end.col -= 1;
                }
            } else if !(*oap).inclusive {
                dec(&raw mut (*oap).end);
            }
            while ltoreq((*curwin.get()).w_cursor, (*oap).end) {
                let mut done: bool = false_0 != 0;
                n = gchar_cursor();
                if n != NUL {
                    let mut new_byte_len: ::core::ffi::c_int = utf_char2len(c);
                    let mut old_byte_len: ::core::ffi::c_int = utfc_ptr2len(get_cursor_pos_ptr());
                    if new_byte_len > 1 as ::core::ffi::c_int
                        || old_byte_len > 1 as ::core::ffi::c_int
                    {
                        if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                            (*oap).end.col += new_byte_len - old_byte_len;
                        }
                        replace_character(c);
                        done = true_0 != 0;
                    } else {
                        if n == TAB {
                            let mut end_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                                end_vcol = getviscol2((*oap).end.col, (*oap).end.coladd);
                            }
                            coladvance_force(getviscol());
                            if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                                getvpos(curwin.get(), &raw mut (*oap).end, end_vcol as colnr_T);
                            }
                        }
                        if gchar_cursor() != NUL {
                            pbyte((*curwin.get()).w_cursor, c);
                            done = true_0 != 0;
                        }
                    }
                }
                if !done
                    && virtual_op.get() as ::core::ffi::c_int != 0
                    && (*curwin.get()).w_cursor.lnum == (*oap).end.lnum
                {
                    let mut virtcols: ::core::ffi::c_int = (*oap).end.coladd as ::core::ffi::c_int;
                    if (*curwin.get()).w_cursor.lnum == (*oap).start.lnum
                        && (*oap).start.col == (*oap).end.col
                        && (*oap).start.coladd != 0
                    {
                        virtcols -= (*oap).start.coladd as ::core::ffi::c_int;
                    }
                    coladvance_force(getviscol2((*oap).end.col, (*oap).end.coladd) + 1 as colnr_T);
                    (*curwin.get()).w_cursor.col -= virtcols + 1 as ::core::ffi::c_int;
                    while virtcols >= 0 as ::core::ffi::c_int {
                        if utf_char2len(c) > 1 as ::core::ffi::c_int {
                            replace_character(c);
                        } else {
                            pbyte((*curwin.get()).w_cursor, c);
                        }
                        if inc(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                            break;
                        }
                        virtcols -= 1;
                    }
                }
                if inc_cursor() == -1 as ::core::ffi::c_int {
                    break;
                }
            }
        }
        (*curwin.get()).w_cursor = (*oap).start;
        check_cursor(curwin.get());
        changed_lines(
            curbuf.get(),
            (*oap).start.lnum,
            (*oap).start.col,
            (*oap).end.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end = (*oap).end;
        }
        return OK;
    }
}
