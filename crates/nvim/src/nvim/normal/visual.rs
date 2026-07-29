//! Visual mode: entering and leaving it, and the area two corners
//! describe.
//!
//! The off-by-one rules live here -- 'selection' decides whether the end is
//! included, and `unadjust_for_sel` is what puts an exclusive selection back
//! before an operator sees it.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn end_visual_mode() {
    VIsual_select_exclu_adj.set(false_0 != 0);
    VIsual_active.set(false_0 != 0);
    setmouse();
    mouse_dragging.set(0 as c_int);
    (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
    (*curbuf.get()).b_visual.vi_start = VIsual.get();
    (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
    (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
    (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    }
    may_clear_cmdline();
    adjust_cursor_eol();
    may_trigger_modechanged();
}

pub unsafe extern "C" fn reset_VIsual_and_resel() {
    if VIsual_active.get() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    VIsual_reselect.set(false_0);
}

pub unsafe extern "C" fn reset_VIsual() {
    if VIsual_active.get() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as c_int);
        VIsual_reselect.set(false_0);
    }
}

pub unsafe extern "C" fn restore_visual_mode() {
    if VIsual_mode_orig.get() != NUL {
        (*curbuf.get()).b_visual.vi_mode = VIsual_mode_orig.get();
        VIsual_mode_orig.set(NUL);
    }
}

pub unsafe extern "C" fn get_visual_text(
    mut cap: *mut cmdarg_T,
    mut pp: *mut *mut c_char,
    mut lenp: *mut size_t,
) -> bool {
    if VIsual_mode.get() != 'V' as c_int {
        unadjust_for_sel();
    }
    if (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
        if !cap.is_null() {
            clearopbeep((*cap).oap);
        }
        return false_0 != 0;
    }
    if VIsual_mode.get() == 'V' as c_int {
        *pp = get_cursor_line_ptr();
        *lenp = get_cursor_line_len() as size_t;
    } else {
        if lt((*curwin.get()).w_cursor, VIsual.get()) {
            *pp = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
            *lenp = ((*VIsual.ptr()).col as size_t)
                .wrapping_sub((*curwin.get()).w_cursor.col as size_t)
                .wrapping_add(1 as size_t);
        } else {
            *pp = ml_get_pos(VIsual.ptr());
            *lenp = ((*curwin.get()).w_cursor.col as size_t)
                .wrapping_sub((*VIsual.ptr()).col as size_t)
                .wrapping_add(1 as size_t);
        }
        if **pp as c_int == NUL {
            *lenp = 0 as size_t;
        }
        if *lenp > 0 as size_t {
            *lenp = (*lenp).wrapping_add(
                (utfc_ptr2len((*pp).offset((*lenp).wrapping_sub(1 as size_t) as isize))
                    - 1 as c_int) as size_t,
            );
        }
    }
    reset_VIsual_and_resel();
    return true_0 != 0;
}

pub(crate) unsafe extern "C" fn v_swap_corners(mut cmdchar: c_int) {
    let mut left: colnr_T = 0;
    let mut right: colnr_T = 0;
    if cmdchar == 'O' as c_int && VIsual_mode.get() == Ctrl_V {
        let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
        getvcols(
            curwin.get(),
            &raw mut old_cursor,
            VIsual.ptr(),
            &raw mut left,
            &raw mut right,
        );
        (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
        coladvance(curwin.get(), left);
        VIsual.set((*curwin.get()).w_cursor);
        (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
        (*curwin.get()).w_curswant = right;
        if old_cursor.lnum >= (*VIsual.ptr()).lnum && *p_sel.get() as c_int == 'e' as c_int {
            (*curwin.get()).w_curswant += 1;
        }
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
        if (*curwin.get()).w_cursor.col == old_cursor.col
            && (!virtual_active(curwin.get())
                || (*curwin.get()).w_cursor.coladd == old_cursor.coladd)
        {
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            if old_cursor.lnum <= (*VIsual.ptr()).lnum && *p_sel.get() as c_int == 'e' as c_int {
                right += 1;
            }
            coladvance(curwin.get(), right);
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
            coladvance(curwin.get(), left);
            (*curwin.get()).w_curswant = left;
        }
    } else {
        let mut old_cursor_0: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = VIsual.get();
        VIsual.set(old_cursor_0);
        (*curwin.get()).w_set_curswant = true_0;
    };
}

pub(crate) unsafe extern "C" fn v_visop(mut cap: *mut cmdarg_T) {
    static trans: GlobalCell<[c_char; 17]> = GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 17], [c_char; 17]>(*b"YyDdCcxdXdAAIIrr\0")
    });
    if *(*__ctype_b_loc()).offset((*cap).cmdchar as isize) as c_int
        & _ISupper as c_int as c_ushort as c_int
        != 0
    {
        if VIsual_mode.get() != Ctrl_V {
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as c_int);
        } else if (*cap).cmdchar == 'C' as c_int || (*cap).cmdchar == 'D' as c_int {
            (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
        }
    }
    (*cap).cmdchar = *vim_strchr(trans.ptr() as *mut c_char, (*cap).cmdchar)
        .offset(1 as c_int as isize) as uint8_t as c_int;
    nv_operator(cap);
}

pub(crate) unsafe extern "C" fn nv_visual(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == Ctrl_Q {
        (*cap).cmdchar = Ctrl_V;
    }
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        (*(*cap).oap).motion_force = (*cap).cmdchar;
        motion_force.set((*(*cap).oap).motion_force);
        finish_op.set(false_0 != 0);
        return;
    }
    VIsual_select.set((*cap).arg != 0);
    if VIsual_active.get() {
        if VIsual_mode.get() == (*cap).cmdchar {
            end_visual_mode();
        } else {
            VIsual_mode.set((*cap).cmdchar);
            showmode();
            may_trigger_modechanged();
        }
        redraw_curbuf_later(UPD_INVERTED as c_int);
    } else if (*cap).count0 > 0 as c_int && resel_VIsual_mode.get() != NUL {
        VIsual.set((*curwin.get()).w_cursor);
        VIsual_active.set(true_0 != 0);
        VIsual_reselect.set(true_0);
        if (*cap).arg == 0 {
            may_start_select('c' as c_int);
        }
        setmouse();
        if p_smd.get() != 0 && msg_silent.get() == 0 as c_int {
            redraw_cmdline.set(true_0 != 0);
        }
        if resel_VIsual_mode.get() != 'v' as c_int || resel_VIsual_line_count.get() > 1 as linenr_T
        {
            (*curwin.get()).w_cursor.lnum = ((*curwin.get()).w_cursor.lnum as c_int
                + (resel_VIsual_line_count.get() * (*cap).count0 as linenr_T - 1 as linenr_T)
                    as c_int) as linenr_T;
            check_cursor(curwin.get());
        }
        VIsual_mode.set(resel_VIsual_mode.get());
        if VIsual_mode.get() == 'v' as c_int {
            if resel_VIsual_line_count.get() <= 1 as linenr_T {
                update_curswant_force();
                '_c2rust_label: {
                    if (*cap).count0 >= -2147483647 as c_int - 1 as c_int
                        && (*cap).count0 <= 2147483647 as c_int
                    {
                    } else {
                        __assert_fail(
                            b"cap->count0 >= INT_MIN && cap->count0 <= INT_MAX\0".as_ptr()
                                as *const c_char,
                            b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                            5057 as c_uint,
                            b"void nv_visual(cmdarg_T *)\0".as_ptr() as *const c_char,
                        );
                    }
                };
                (*curwin.get()).w_curswant += resel_VIsual_vcol.get() as c_int * (*cap).count0;
                if *p_sel.get() as c_int != 'e' as c_int {
                    (*curwin.get()).w_curswant -= 1;
                }
            } else {
                (*curwin.get()).w_curswant = resel_VIsual_vcol.get();
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }
        if resel_VIsual_vcol.get() == MAXCOL as c_int {
            (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
            coladvance(curwin.get(), MAXCOL as c_int);
        } else if VIsual_mode.get() == Ctrl_V {
            let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            update_curswant_force();
            '_c2rust_label_0: {
                if (*cap).count0 >= -2147483647 as c_int - 1 as c_int
                    && (*cap).count0 <= 2147483647 as c_int
                {
                } else {
                    __assert_fail(
                        b"cap->count0 >= INT_MIN && cap->count0 <= INT_MAX\0".as_ptr()
                            as *const c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                        5075 as c_uint,
                        b"void nv_visual(cmdarg_T *)\0".as_ptr() as *const c_char,
                    );
                }
            };
            (*curwin.get()).w_curswant +=
                resel_VIsual_vcol.get() as c_int * (*cap).count0 - 1 as c_int;
            (*curwin.get()).w_cursor.lnum = lnum;
            if *p_sel.get() as c_int == 'e' as c_int {
                (*curwin.get()).w_curswant += 1;
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        } else {
            (*curwin.get()).w_set_curswant = true_0;
        }
        redraw_curbuf_later(UPD_INVERTED as c_int);
    } else {
        if (*cap).arg == 0 {
            may_start_select('c' as c_int);
        }
        n_start_visual_mode((*cap).cmdchar);
        if VIsual_mode.get() != 'V' as c_int && *p_sel.get() as c_int == 'e' as c_int {
            (*cap).count1 += 1;
        } else {
            VIsual_select_exclu_adj.set(false_0 != 0);
        }
        if (*cap).count0 > 0 as c_int && {
            (*cap).count1 -= 1;
            (*cap).count1 > 0 as c_int
        } {
            if VIsual_mode.get() == 'v' as c_int || VIsual_mode.get() == Ctrl_V {
                nv_right(cap);
            } else if VIsual_mode.get() == 'V' as c_int {
                nv_down(cap);
            }
        }
    };
}

pub unsafe extern "C" fn start_selection() {
    may_start_select('k' as c_int);
    n_start_visual_mode('v' as c_int);
}

pub unsafe extern "C" fn may_start_select(mut c: c_int) {
    VIsual_select.set(
        (c == 'o' as c_int || stuff_empty() as c_int != 0 && typebuf_typed() != 0)
            && !vim_strchr(p_slm.get(), c).is_null(),
    );
}

pub(crate) unsafe extern "C" fn n_start_visual_mode(mut c: c_int) {
    VIsual_mode.set(c);
    VIsual_active.set(true_0 != 0);
    VIsual_reselect.set(true_0);
    if c == Ctrl_V
        && get_ve_flags(curwin.get()) & kOptVeFlagBlock as c_int as c_uint != 0
        && gchar_cursor() == TAB
    {
        validate_virtcol(curwin.get());
        coladvance(curwin.get(), (*curwin.get()).w_virtcol);
    }
    VIsual.set((*curwin.get()).w_cursor);
    foldAdjustVisual();
    may_trigger_modechanged();
    setmouse();
    conceal_check_cursor_line();
    if p_smd.get() != 0 && msg_silent.get() == 0 as c_int {
        redraw_cmdline.set(true_0 != 0);
    }
    if (*curwin.get()).w_redr_type < UPD_INVERTED as c_int {
        (*curwin.get()).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
        (*curwin.get()).w_old_visual_lnum = (*curwin.get()).w_cursor.lnum;
    }
    redraw_curbuf_later(UPD_VALID as c_int);
}

pub(crate) unsafe extern "C" fn nv_gv_cmd(mut cap: *mut cmdarg_T) {
    if (*curbuf.get()).b_visual.vi_start.lnum == 0 as linenr_T
        || (*curbuf.get()).b_visual.vi_start.lnum > (*curbuf.get()).b_ml.ml_line_count
        || (*curbuf.get()).b_visual.vi_end.lnum == 0 as linenr_T
    {
        beep_flush();
        return;
    }
    let mut tpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if VIsual_active.get() {
        let mut i: c_int = VIsual_mode.get();
        VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
        (*curbuf.get()).b_visual.vi_mode = i;
        (*curbuf.get()).b_visual_mode_eval = i;
        i = (*curwin.get()).w_curswant as c_int;
        (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
        (*curbuf.get()).b_visual.vi_curswant = i as colnr_T;
        tpos = (*curbuf.get()).b_visual.vi_end;
        (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
        (*curbuf.get()).b_visual.vi_start = VIsual.get();
    } else {
        VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
        (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
        tpos = (*curbuf.get()).b_visual.vi_end;
        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
    }
    VIsual_active.set(true_0 != 0);
    VIsual_reselect.set(true_0);
    check_cursor(curwin.get());
    VIsual.set((*curwin.get()).w_cursor);
    (*curwin.get()).w_cursor = tpos;
    check_cursor(curwin.get());
    update_topline(curwin.get());
    if (*cap).arg != 0 {
        VIsual_select.set(true_0 != 0);
        VIsual_select_reg.set(0 as c_int);
    } else {
        may_start_select('c' as c_int);
    }
    setmouse();
    redraw_curbuf_later(UPD_INVERTED as c_int);
    showmode();
}

pub(crate) unsafe extern "C" fn adjust_for_sel(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as c_int != 0
        && (*(*cap).oap).inclusive as c_int != 0
        && *p_sel.get() as c_int == 'e' as c_int
        && gchar_cursor() != NUL
        && lt(VIsual.get(), (*curwin.get()).w_cursor) as c_int != 0
    {
        inc_cursor();
        (*(*cap).oap).inclusive = false_0 != 0;
        VIsual_select_exclu_adj.set(true_0 != 0);
    }
}

pub unsafe extern "C" fn unadjust_for_sel() -> bool {
    if *p_sel.get() as c_int == 'e' as c_int && !equalpos(VIsual.get(), (*curwin.get()).w_cursor) {
        return unadjust_for_sel_inner(
            if lt(VIsual.get(), (*curwin.get()).w_cursor) as c_int != 0 {
                &raw mut (*curwin.get()).w_cursor
            } else {
                VIsual.ptr()
            },
        );
    }
    return false_0 != 0;
}

pub unsafe extern "C" fn unadjust_for_sel_inner(mut pp: *mut pos_T) -> bool {
    VIsual_select_exclu_adj.set(false_0 != 0);
    if (*pp).coladd > 0 as c_int {
        (*pp).coladd -= 1;
    } else if (*pp).col > 0 as c_int {
        (*pp).col -= 1;
        mark_mb_adjustpos(curbuf.get(), pp);
        if virtual_active(curwin.get()) {
            let mut cs: colnr_T = 0;
            let mut ce: colnr_T = 0;
            getvcol(
                curwin.get(),
                pp,
                &raw mut cs,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut ce,
            );
            (*pp).coladd = ce - cs;
        }
    } else if (*pp).lnum > 1 as linenr_T {
        (*pp).lnum -= 1;
        (*pp).col = ml_get_len((*pp).lnum);
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn nv_select(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(true_0 != 0);
        VIsual_select_reg.set(0 as c_int);
    } else if VIsual_reselect.get() != 0 {
        (*cap).nchar = 'v' as c_int;
        (*cap).arg = true_0;
        nv_g_cmd(cap);
    }
}

pub(crate) unsafe extern "C" fn nv_object(mut cap: *mut cmdarg_T) {
    let mut flag: bool = false;
    let mut include: bool = false;
    if (*cap).cmdchar == 'i' as c_int {
        include = false_0 != 0;
    } else {
        include = true_0 != 0;
    }
    let mut mps_save: *mut c_char = (*curbuf.get()).b_p_mps;
    (*curbuf.get()).b_p_mps = b"(:),{:},[:],<:>\0".as_ptr() as *const c_char as *mut c_char;
    match (*cap).nchar {
        119 => {
            flag = current_word((*cap).oap, (*cap).count1, include, false_0 != 0) != 0;
        }
        87 => {
            flag = current_word((*cap).oap, (*cap).count1, include, true_0 != 0) != 0;
        }
        98 | 40 | 41 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '(' as c_int,
                ')' as c_int,
            ) != 0;
        }
        66 | 123 | 125 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '{' as c_int,
                '}' as c_int,
            ) != 0;
        }
        91 | 93 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '[' as c_int,
                ']' as c_int,
            ) != 0;
        }
        60 | 62 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '<' as c_int,
                '>' as c_int,
            ) != 0;
        }
        116 => {
            (*cap).retval |= CA_NO_ADJ_OP_END as c_int;
            flag = current_tagblock((*cap).oap, (*cap).count1, include) != 0;
        }
        112 => {
            flag = current_par((*cap).oap, (*cap).count1, include, 'p' as c_int) != 0;
        }
        115 => {
            flag = current_sent((*cap).oap, (*cap).count1, include) != 0;
        }
        34 | 39 | 96 => {
            flag = current_quote((*cap).oap, (*cap).count1, include, (*cap).nchar);
        }
        _ => {
            flag = false_0 != 0;
        }
    }
    (*curbuf.get()).b_p_mps = mps_save;
    if !flag {
        clearopbeep((*cap).oap);
    }
    adjust_cursor_col();
    (*curwin.get()).w_set_curswant = true_0;
}
