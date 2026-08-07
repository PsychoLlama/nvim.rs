//! `do_pending_operator` -- the dispatcher.
//!
//! Normal mode reads an operator and then a motion; this is what runs when
//! both have arrived.  Roughly half of it is deciding what the region
//! actually *is*: swapping the ends if the motion went backwards, applying
//! `v`/`V`/CTRL-V forcing, expanding a linewise region over closed folds,
//! handling the Visual-mode case (where the region is the selection and the
//! operator was typed after it), recording the whole thing for `.`, and
//! choosing the register.  The other half is a switch on `oap->op_type`
//! that calls one of this module's operators.  `clear_oparg` resets the
//! struct between commands.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn clear_oparg(mut oap: *mut oparg_T) {
    unsafe {
        memset(
            oap as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<oparg_T>(),
        );
    }
}

unsafe extern "C" fn is_ex_cmdchar(mut cap: *mut cmdarg_T) -> bool {
    unsafe {
        return (*cap).cmdchar == ':' as ::core::ffi::c_int
            || (*cap).cmdchar
                == -(253 as ::core::ffi::c_int
                    + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    }
}

pub unsafe extern "C" fn do_pending_operator(
    mut cap: *mut cmdarg_T,
    mut old_col: ::core::ffi::c_int,
    mut gui_yank: bool,
) {
    unsafe {
        let mut oap: *mut oparg_T = (*cap).oap;
        let mut lbr_saved: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_lbr;
        static redo_VIsual: GlobalCell<redo_VIsual_T> = GlobalCell::new(redo_VIsual_T {
            rv_mode: NUL,
            rv_line_count: 0 as linenr_T,
            rv_vcol: 0 as colnr_T,
            rv_count: 0 as ::core::ffi::c_int,
            rv_arg: 0 as ::core::ffi::c_int,
        });
        let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
        if (finish_op.get() as ::core::ffi::c_int != 0
            || VIsual_active.get() as ::core::ffi::c_int != 0)
            && (*oap).op_type != OP_NOP as ::core::ffi::c_int
        {
            let mut empty_region_error: bool = false;
            let mut restart_edit_save: ::core::ffi::c_int = 0;
            let mut include_line_break: bool = false_0 != 0;
            let redo_yank: bool = !vim_strchr(p_cpo.get(), CPO_YANK).is_null() && !gui_yank;
            reset_lbr();
            (*oap).is_VIsual = VIsual_active.get();
            if (*oap).motion_force == 'V' as ::core::ffi::c_int {
                (*oap).motion_type = kMTLineWise;
            } else if (*oap).motion_force == 'v' as ::core::ffi::c_int {
                if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                    (*oap).inclusive = false_0 != 0;
                } else if (*oap).motion_type as ::core::ffi::c_int
                    == kMTCharWise as ::core::ffi::c_int
                {
                    (*oap).inclusive = !(*oap).inclusive;
                }
                (*oap).motion_type = kMTCharWise;
            } else if (*oap).motion_force == Ctrl_V {
                if !VIsual_active.get() {
                    VIsual_active.set(true_0 != 0);
                    VIsual.set((*oap).start);
                }
                VIsual_mode.set(Ctrl_V);
                VIsual_select.set(false_0 != 0);
                VIsual_reselect.set(false_0);
            }
            if (redo_yank as ::core::ffi::c_int != 0
                || (*oap).op_type != OP_YANK as ::core::ffi::c_int)
                && (!VIsual_active.get()
                    || (*oap).motion_force != 0
                    || (is_ex_cmdchar(cap) as ::core::ffi::c_int != 0
                        || (*cap).cmdchar
                            == -(253 as ::core::ffi::c_int
                                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
                        && (*oap).op_type != OP_COLON as ::core::ffi::c_int)
                && (*cap).cmdchar != 'D' as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLD as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDOPEN as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDOPENREC as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDCLOSE as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDCLOSEREC as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDDEL as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDDELREC as ::core::ffi::c_int
            {
                prep_redo(
                    (*oap).regname,
                    (*cap).count0,
                    get_op_char((*oap).op_type),
                    get_extra_op_char((*oap).op_type),
                    (*oap).motion_force,
                    (*cap).cmdchar,
                    (*cap).nchar,
                );
                if (*cap).cmdchar == '/' as ::core::ffi::c_int
                    || (*cap).cmdchar == '?' as ::core::ffi::c_int
                {
                    if vim_strchr(p_cpo.get(), CPO_REDO).is_null() {
                        AppendToRedobuffLit((*cap).searchbuf, -1 as ::core::ffi::c_int);
                    }
                    AppendToRedobuff(NL_STR.as_ptr());
                } else if is_ex_cmdchar(cap) {
                    if (*repeat_cmdline.ptr()).is_null() {
                        ResetRedobuff();
                    } else {
                        if (*cap).cmdchar == ':' as ::core::ffi::c_int {
                            AppendToRedobuffLit(repeat_cmdline.get(), -1 as ::core::ffi::c_int);
                        } else {
                            AppendToRedobuffSpec(repeat_cmdline.get());
                        }
                        AppendToRedobuff(NL_STR.as_ptr());
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            repeat_cmdline.ptr() as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        let _ = *ptr_;
                    }
                } else if (*cap).cmdchar
                    == -(253 as ::core::ffi::c_int
                        + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    AppendNumberToRedobuff(repeat_luaref.get() as ::core::ffi::c_int);
                    AppendToRedobuff(NL_STR.as_ptr());
                }
            }
            if redo_VIsual_busy.get() {
                (*oap).start = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor.lnum = ((*curwin.get()).w_cursor.lnum
                    as ::core::ffi::c_int
                    + ((*redo_VIsual.ptr()).rv_line_count - 1 as linenr_T) as ::core::ffi::c_int)
                    as linenr_T;
                (*curwin.get()).w_cursor.lnum =
                    if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                        (*curwin.get()).w_cursor.lnum
                    } else {
                        (*curbuf.get()).b_ml.ml_line_count
                    };
                VIsual_mode.set((*redo_VIsual.ptr()).rv_mode);
                if (*redo_VIsual.ptr()).rv_vcol == MAXCOL as ::core::ffi::c_int
                    || VIsual_mode.get() == 'v' as ::core::ffi::c_int
                {
                    if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
                        if (*redo_VIsual.ptr()).rv_line_count <= 1 as linenr_T {
                            validate_virtcol(curwin.get());
                            (*curwin.get()).w_curswant = ((*curwin.get()).w_virtcol
                                as ::core::ffi::c_int
                                + (*redo_VIsual.ptr()).rv_vcol as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int)
                                as colnr_T;
                        } else {
                            (*curwin.get()).w_curswant = (*redo_VIsual.ptr()).rv_vcol;
                        }
                    } else {
                        (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                    coladvance(curwin.get(), (*curwin.get()).w_curswant);
                }
                (*cap).count0 = (*redo_VIsual.ptr()).rv_count;
                (*cap).count1 = if (*cap).count0 == 0 as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    (*cap).count0
                };
            } else if VIsual_active.get() {
                if !gui_yank {
                    (*curbuf.get()).b_visual.vi_start = VIsual.get();
                    (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
                    (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
                    restore_visual_mode();
                    (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
                    (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
                }
                if VIsual_select.get() as ::core::ffi::c_int != 0
                    && VIsual_mode.get() == 'V' as ::core::ffi::c_int
                    && (*(*cap).oap).op_type != OP_DELETE as ::core::ffi::c_int
                {
                    if lt(VIsual.get(), (*curwin.get()).w_cursor) {
                        (*VIsual.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
                        (*curwin.get()).w_cursor.col = ml_get_len((*curwin.get()).w_cursor.lnum);
                    } else {
                        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                        (*VIsual.ptr()).col = ml_get_len((*VIsual.ptr()).lnum);
                    }
                    VIsual_mode.set('v' as ::core::ffi::c_int);
                } else if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
                    include_line_break = unadjust_for_sel();
                }
                (*oap).start = VIsual.get();
                if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                    (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*oap).start.coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
            if lt((*oap).start, (*curwin.get()).w_cursor) {
                if !VIsual_active.get() {
                    if hasFolding(
                        curwin.get(),
                        (*oap).start.lnum,
                        &raw mut (*oap).start.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                    ) {
                        (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    if ((*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                        || (*oap).inclusive as ::core::ffi::c_int != 0
                        || (*oap).motion_type as ::core::ffi::c_int
                            == kMTLineWise as ::core::ffi::c_int)
                        && hasFolding(
                            curwin.get(),
                            (*curwin.get()).w_cursor.lnum,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut (*curwin.get()).w_cursor.lnum,
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        (*curwin.get()).w_cursor.col = get_cursor_line_len();
                    }
                }
                (*oap).end = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor = (*oap).start;
                (*curwin.get()).w_valid &= !VALID_VIRTCOL;
            } else {
                if !VIsual_active.get()
                    && (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                {
                    if hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        &raw mut (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                    ) {
                        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    if hasFolding(
                        curwin.get(),
                        (*oap).start.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut (*oap).start.lnum,
                    ) {
                        (*oap).start.col = ml_get_len((*oap).start.lnum);
                    }
                }
                (*oap).end = (*oap).start;
                (*oap).start = (*curwin.get()).w_cursor;
            }
            check_pos((*curwin.get()).w_buffer, &raw mut (*oap).end);
            (*oap).line_count = (*oap).end.lnum - (*oap).start.lnum + 1 as linenr_T;
            virtual_op.set(virtual_active(curwin.get()) as TriState);
            if VIsual_active.get() as ::core::ffi::c_int != 0
                || redo_VIsual_busy.get() as ::core::ffi::c_int != 0
            {
                get_op_vcol(oap, (*redo_VIsual.ptr()).rv_vcol, true_0 != 0);
                if !redo_VIsual_busy.get() && !gui_yank {
                    resel_VIsual_mode.set(VIsual_mode.get());
                    if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                        resel_VIsual_vcol.set(MAXCOL as ::core::ffi::c_int as colnr_T);
                    } else {
                        if VIsual_mode.get() != Ctrl_V {
                            getvvcol(
                                curwin.get(),
                                &raw mut (*oap).end,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut (*oap).end_vcol,
                            );
                        }
                        if VIsual_mode.get() == Ctrl_V || (*oap).line_count <= 1 as linenr_T {
                            if VIsual_mode.get() != Ctrl_V {
                                getvvcol(
                                    curwin.get(),
                                    &raw mut (*oap).start,
                                    &raw mut (*oap).start_vcol,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                            }
                            resel_VIsual_vcol.set(
                                ((*oap).end_vcol as ::core::ffi::c_int
                                    - (*oap).start_vcol as ::core::ffi::c_int
                                    + 1 as ::core::ffi::c_int)
                                    as colnr_T,
                            );
                        } else {
                            resel_VIsual_vcol.set((*oap).end_vcol);
                        }
                    }
                    resel_VIsual_line_count.set((*oap).line_count);
                }
                if (redo_yank as ::core::ffi::c_int != 0
                    || (*oap).op_type != OP_YANK as ::core::ffi::c_int)
                    && (*oap).op_type != OP_COLON as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLD as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLDOPEN as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLDOPENREC as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLDCLOSE as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLDCLOSEREC as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLDDEL as ::core::ffi::c_int
                    && (*oap).op_type != OP_FOLDDELREC as ::core::ffi::c_int
                    && (*oap).motion_force == NUL
                {
                    if (*cap).cmdchar == 'g' as ::core::ffi::c_int
                        && ((*cap).nchar == 'n' as ::core::ffi::c_int
                            || (*cap).nchar == 'N' as ::core::ffi::c_int)
                    {
                        prep_redo(
                            (*oap).regname,
                            (*cap).count0,
                            get_op_char((*oap).op_type),
                            get_extra_op_char((*oap).op_type),
                            (*oap).motion_force,
                            (*cap).cmdchar,
                            (*cap).nchar,
                        );
                    } else if !is_ex_cmdchar(cap)
                        && (*cap).cmdchar
                            != -(253 as ::core::ffi::c_int
                                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    {
                        let mut opchar: ::core::ffi::c_int = get_op_char((*oap).op_type);
                        let mut extra_opchar: ::core::ffi::c_int =
                            get_extra_op_char((*oap).op_type);
                        let mut nchar: ::core::ffi::c_int =
                            if (*oap).op_type == OP_REPLACE as ::core::ffi::c_int {
                                (*cap).nchar
                            } else {
                                NUL
                            };
                        if nchar == REPLACE_CR_NCHAR as ::core::ffi::c_int {
                            nchar = CAR;
                        } else if nchar == REPLACE_NL_NCHAR as ::core::ffi::c_int {
                            nchar = NL;
                        }
                        if opchar == 'g' as ::core::ffi::c_int
                            && extra_opchar == '@' as ::core::ffi::c_int
                        {
                            prep_redo_num2(
                                (*oap).regname,
                                0 as ::core::ffi::c_int,
                                NUL,
                                'v' as ::core::ffi::c_int,
                                (*cap).count0,
                                opchar,
                                extra_opchar,
                                nchar,
                            );
                        } else {
                            prep_redo(
                                (*oap).regname,
                                0 as ::core::ffi::c_int,
                                NUL,
                                'v' as ::core::ffi::c_int,
                                opchar,
                                extra_opchar,
                                nchar,
                            );
                        }
                    }
                    if !redo_VIsual_busy.get() {
                        (*redo_VIsual.ptr()).rv_mode = resel_VIsual_mode.get();
                        (*redo_VIsual.ptr()).rv_vcol = resel_VIsual_vcol.get();
                        (*redo_VIsual.ptr()).rv_line_count = resel_VIsual_line_count.get();
                        (*redo_VIsual.ptr()).rv_count = (*cap).count0;
                        (*redo_VIsual.ptr()).rv_arg = (*cap).arg;
                    }
                }
                if (*oap).motion_force == NUL
                    || (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                {
                    (*oap).inclusive = true_0 != 0;
                }
                if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                    (*oap).motion_type = kMTLineWise;
                } else if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
                    (*oap).motion_type = kMTCharWise;
                    if *ml_get_pos(&raw mut (*oap).end) as ::core::ffi::c_int == NUL
                        && (include_line_break as ::core::ffi::c_int != 0
                            || virtual_op.get() as u64 == 0)
                    {
                        (*oap).inclusive = false_0 != 0;
                        if *p_sel.get() as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
                            && op_on_lines((*oap).op_type) == 0
                            && (*oap).end.lnum < (*curbuf.get()).b_ml.ml_line_count
                        {
                            (*oap).end.lnum += 1;
                            (*oap).end.col = 0 as ::core::ffi::c_int as colnr_T;
                            (*oap).end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                            (*oap).line_count += 1;
                        }
                    }
                }
                redo_VIsual_busy.set(false_0 != 0);
                if !gui_yank {
                    VIsual_active.set(false_0 != 0);
                    setmouse();
                    mouse_dragging.set(0 as ::core::ffi::c_int);
                    may_clear_cmdline();
                    if ((*oap).op_type == OP_YANK as ::core::ffi::c_int
                        || (*oap).op_type == OP_COLON as ::core::ffi::c_int
                        || (*oap).op_type == OP_FUNCTION as ::core::ffi::c_int
                        || (*oap).op_type == OP_FILTER as ::core::ffi::c_int)
                        && (*oap).motion_force == NUL
                    {
                        restore_lbr(lbr_saved != 0);
                        redraw_curbuf_later(UPD_INVERTED);
                    }
                }
            }
            if (*oap).inclusive {
                let l: ::core::ffi::c_int = utfc_ptr2len(ml_get_pos(&raw mut (*oap).end));
                if l > 1 as ::core::ffi::c_int {
                    (*oap).end.col += l - 1 as ::core::ffi::c_int;
                }
            }
            (*curwin.get()).w_set_curswant = true_0;
            (*oap).empty = (*oap).motion_type as ::core::ffi::c_int
                != kMTLineWise as ::core::ffi::c_int
                && (!(*oap).inclusive
                    || (*oap).op_type == OP_YANK as ::core::ffi::c_int
                        && gchar_pos(&raw mut (*oap).end) == NUL)
                && equalpos((*oap).start, (*oap).end) as ::core::ffi::c_int != 0
                && !(virtual_op.get() as ::core::ffi::c_int != 0
                    && (*oap).start.coladd != (*oap).end.coladd);
            empty_region_error = (*oap).empty as ::core::ffi::c_int != 0
                && !vim_strchr(p_cpo.get(), CPO_EMPTYREGION).is_null();
            if (*oap).is_VIsual as ::core::ffi::c_int != 0
                && ((*oap).empty as ::core::ffi::c_int != 0
                    || (*curbuf.get()).b_p_ma == 0
                    || (*oap).op_type == OP_FOLD as ::core::ffi::c_int)
            {
                restore_lbr(lbr_saved != 0);
                redraw_curbuf_later(UPD_INVERTED);
            }
            if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                && (*oap).inclusive as ::core::ffi::c_int == false_0
                && (*cap).retval & CA_NO_ADJ_OP_END as ::core::ffi::c_int == 0
                && (*oap).end.col == 0 as ::core::ffi::c_int
                && (!(*oap).is_VIsual
                    || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
                && (*oap).line_count > 1 as linenr_T
            {
                (*oap).end_adjusted = true_0 != 0;
                (*oap).line_count -= 1;
                (*oap).end.lnum -= 1;
                if inindent(0 as ::core::ffi::c_int) {
                    (*oap).motion_type = kMTLineWise;
                } else {
                    (*oap).end.col = ml_get_len((*oap).end.lnum);
                    if (*oap).end.col != 0 {
                        (*oap).end.col -= 1;
                        (*oap).inclusive = true_0 != 0;
                    }
                }
            } else {
                (*oap).end_adjusted = false_0 != 0;
            }
            's_1511: {
                match (*oap).op_type {
                    4 | 5 => {
                        op_shift(
                            oap,
                            true_0 != 0,
                            if (*oap).is_VIsual as ::core::ffi::c_int != 0 {
                                (*cap).count1
                            } else {
                                1 as ::core::ffi::c_int
                            },
                        );
                        auto_format(false_0 != 0, true_0 != 0);
                        break 's_1511;
                    }
                    14 | 13 => {
                        (*oap).line_count = if (*oap).line_count > 2 as linenr_T {
                            (*oap).line_count
                        } else {
                            2 as linenr_T
                        };
                        if (*curwin.get()).w_cursor.lnum + (*oap).line_count - 1 as linenr_T
                            > (*curbuf.get()).b_ml.ml_line_count
                        {
                            beep_flush();
                        } else {
                            do_join(
                                (*oap).line_count as size_t,
                                (*oap).op_type == OP_JOIN as ::core::ffi::c_int,
                                true_0 != 0,
                                true_0 != 0,
                                true_0 != 0,
                            );
                            auto_format(false_0 != 0, true_0 != 0);
                        }
                        break 's_1511;
                    }
                    1 => {
                        VIsual_reselect.set(false_0);
                        if empty_region_error {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        } else {
                            op_delete(oap);
                            if (*oap).motion_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                                && has_format_option(FO_AUTO) as ::core::ffi::c_int != 0
                                && u_save_cursor() == OK
                            {
                                auto_format(false_0 != 0, true_0 != 0);
                            }
                        }
                        break 's_1511;
                    }
                    2 => {
                        if empty_region_error {
                            if !gui_yank {
                                vim_beep(
                                    kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                                );
                                CancelRedo();
                            }
                        } else {
                            restore_lbr(lbr_saved != 0);
                            (*oap).excl_tr_ws = (*cap).cmdchar == 'z' as ::core::ffi::c_int;
                            op_yank(oap, !gui_yank);
                        }
                        check_cursor_col(curwin.get());
                        break 's_1511;
                    }
                    3 => {
                        VIsual_reselect.set(false_0);
                        if empty_region_error {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        } else {
                            if !KeyTyped.get() {
                                restart_edit_save = restart_edit.get();
                            } else {
                                restart_edit_save = 0 as ::core::ffi::c_int;
                            }
                            restart_edit.set(0 as ::core::ffi::c_int);
                            restore_lbr(lbr_saved != 0);
                            (*curbuf.get()).b_last_changedtick_i =
                                buf_get_changedtick(curbuf.get());
                            if op_change(oap) != 0 {
                                (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
                            }
                            if restart_edit.get() == 0 as ::core::ffi::c_int {
                                restart_edit.set(restart_edit_save);
                            }
                        }
                        break 's_1511;
                    }
                    6 => {
                        if !vim_strchr(p_cpo.get(), CPO_FILTER).is_null() {
                            AppendToRedobuff(b"!\r\0".as_ptr() as *const ::core::ffi::c_char);
                        } else {
                            bangredo.set(true_0 != 0);
                        }
                    }
                    8 | 10 => {}
                    7 | 11 | 12 | 15 => {
                        if empty_region_error {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        } else {
                            op_tilde(oap);
                        }
                        check_cursor_col(curwin.get());
                        break 's_1511;
                    }
                    9 => {
                        if *(*curbuf.get()).b_p_fex as ::core::ffi::c_int != NUL {
                            op_formatexpr(oap);
                        } else if *p_fp.get() as ::core::ffi::c_int != NUL
                            || *(*curbuf.get()).b_p_fp as ::core::ffi::c_int != NUL
                        {
                            op_colon(oap);
                        } else {
                            op_format(oap, false_0 != 0);
                        }
                        break 's_1511;
                    }
                    26 => {
                        op_format(oap, true_0 != 0);
                        break 's_1511;
                    }
                    27 => {
                        let mut save_redo_VIsual: redo_VIsual_T = redo_VIsual.get();
                        restore_lbr(lbr_saved != 0);
                        op_function(oap);
                        redo_VIsual.set(save_redo_VIsual);
                        break 's_1511;
                    }
                    17 | 18 => {
                        VIsual_reselect.set(false_0);
                        if empty_region_error {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        } else {
                            restart_edit_save = restart_edit.get();
                            restart_edit.set(0 as ::core::ffi::c_int);
                            restore_lbr(lbr_saved != 0);
                            (*curbuf.get()).b_last_changedtick_i =
                                buf_get_changedtick(curbuf.get());
                            op_insert(oap, (*cap).count1);
                            reset_lbr();
                            auto_format(false_0 != 0, true_0 != 0);
                            if restart_edit.get() == 0 as ::core::ffi::c_int {
                                restart_edit.set(restart_edit_save);
                            } else {
                                (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
                            }
                        }
                        break 's_1511;
                    }
                    16 => {
                        VIsual_reselect.set(false_0);
                        if empty_region_error {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        } else {
                            restore_lbr(lbr_saved != 0);
                            op_replace(oap, (*cap).nchar);
                        }
                        break 's_1511;
                    }
                    19 => {
                        VIsual_reselect.set(false_0);
                        foldCreate(curwin.get(), (*oap).start, (*oap).end);
                        break 's_1511;
                    }
                    20 | 21 | 22 | 23 => {
                        VIsual_reselect.set(false_0);
                        opFoldRange(
                            (*oap).start,
                            (*oap).end,
                            ((*oap).op_type == OP_FOLDOPEN as ::core::ffi::c_int
                                || (*oap).op_type == OP_FOLDOPENREC as ::core::ffi::c_int)
                                as ::core::ffi::c_int,
                            ((*oap).op_type == OP_FOLDOPENREC as ::core::ffi::c_int
                                || (*oap).op_type == OP_FOLDCLOSEREC as ::core::ffi::c_int)
                                as ::core::ffi::c_int,
                            (*oap).is_VIsual,
                        );
                        break 's_1511;
                    }
                    24 | 25 => {
                        VIsual_reselect.set(false_0);
                        deleteFold(
                            curwin.get(),
                            (*oap).start.lnum,
                            (*oap).end.lnum,
                            ((*oap).op_type == OP_FOLDDELREC as ::core::ffi::c_int)
                                as ::core::ffi::c_int,
                            (*oap).is_VIsual,
                        );
                        break 's_1511;
                    }
                    28 | 29 => {
                        if empty_region_error {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        } else {
                            VIsual_active.set(true_0 != 0);
                            restore_lbr(lbr_saved != 0);
                            op_addsub(
                                oap,
                                (*cap).count1 as linenr_T,
                                (*redo_VIsual.ptr()).rv_arg != 0,
                            );
                            VIsual_active.set(false_0 != 0);
                        }
                        check_cursor_col(curwin.get());
                        break 's_1511;
                    }
                    _ => {
                        clearopbeep(oap);
                        break 's_1511;
                    }
                }
                if (*oap).op_type == OP_INDENT as ::core::ffi::c_int
                    && *get_equalprg() as ::core::ffi::c_int == NUL
                {
                    if (*curbuf.get()).b_p_lisp != 0 {
                        if use_indentexpr_for_lisp() {
                            op_reindent(
                                oap,
                                Some(
                                    get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
                                ),
                            );
                        } else {
                            op_reindent(
                                oap,
                                Some(
                                    get_lisp_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
                                ),
                            );
                        }
                    } else {
                        op_reindent(
                            oap,
                            if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
                                Some(
                                    get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
                                )
                            } else {
                                Some(get_c_indent as unsafe extern "C" fn() -> ::core::ffi::c_int)
                            },
                        );
                    }
                } else {
                    op_colon(oap);
                }
            }
            virtual_op.set(kNone);
            if !gui_yank {
                if p_sol.get() == 0
                    && (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                    && !(*oap).end_adjusted
                    && ((*oap).op_type == OP_LSHIFT as ::core::ffi::c_int
                        || (*oap).op_type == OP_RSHIFT as ::core::ffi::c_int
                        || (*oap).op_type == OP_DELETE as ::core::ffi::c_int)
                {
                    reset_lbr();
                    (*curwin.get()).w_curswant = old_col as colnr_T;
                    coladvance(curwin.get(), (*curwin.get()).w_curswant);
                }
            } else {
                (*curwin.get()).w_cursor = old_cursor;
            }
            clearop(oap);
            motion_force.set(NUL);
        }
        restore_lbr(lbr_saved != 0);
    }
}
