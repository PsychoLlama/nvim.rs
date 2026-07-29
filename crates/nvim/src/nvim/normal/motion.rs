//! Cursor motions that are not searches: by character, word, line,
//! screen line, paragraph and sentence.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nv_screengo(
    mut oap: *mut oparg_T,
    mut dir: c_int,
    mut dist: c_int,
    mut skip_conceal: bool,
) -> bool {
    let mut linelen: c_int = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
    let mut retval: bool = true_0 != 0;
    let mut atend: bool = false_0 != 0;
    let mut col_off1: c_int = 0;
    let mut col_off2: c_int = 0;
    let mut width1: c_int = 0;
    let mut width2: c_int = 0;
    (*oap).motion_type = kMTCharWise;
    (*oap).inclusive = (*curwin.get()).w_curswant == MAXCOL as c_int;
    col_off1 = win_col_off(curwin.get());
    col_off2 = col_off1 - win_col_off2(curwin.get());
    width1 = (*curwin.get()).w_view_width - col_off1;
    width2 = (*curwin.get()).w_view_width - col_off2;
    if width2 == 0 as c_int {
        width2 = 1 as c_int;
    }
    if (*curwin.get()).w_view_width != 0 as c_int {
        let mut n: c_int = 0;
        if (*curwin.get()).w_curswant == MAXCOL as c_int {
            atend = true_0 != 0;
            validate_virtcol(curwin.get());
            if width1 <= 0 as c_int {
                (*curwin.get()).w_curswant = 0 as c_int as colnr_T;
            } else {
                (*curwin.get()).w_curswant = (width1 - 1 as c_int) as colnr_T;
                if (*curwin.get()).w_virtcol > (*curwin.get()).w_curswant {
                    (*curwin.get()).w_curswant += (((*curwin.get()).w_virtcol as c_int
                        - (*curwin.get()).w_curswant as c_int
                        - 1 as c_int)
                        / width2
                        + 1 as c_int)
                        * width2;
                }
            }
        } else {
            if linelen > width1 {
                n = ((linelen - width1 - 1 as c_int) / width2 + 1 as c_int) * width2 + width1;
            } else {
                n = width1;
            }
            (*curwin.get()).w_curswant = (if (*curwin.get()).w_curswant < n - 1 as c_int {
                (*curwin.get()).w_curswant as c_int
            } else {
                n - 1 as c_int
            }) as colnr_T;
        }
        loop {
            let c2rust_fresh10 = dist;
            dist = dist - 1;
            if c2rust_fresh10 == 0 {
                break;
            }
            if dir == BACKWARD as c_int {
                if (*curwin.get()).w_curswant >= width1
                    && !hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    (*curwin.get()).w_curswant -= width2;
                } else if (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
                    retval = false_0 != 0;
                    break;
                } else {
                    cursor_up_inner(curwin.get(), 1 as linenr_T, skip_conceal);
                    linelen = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                    if linelen > width1 {
                        let mut w: c_int =
                            ((linelen - width1 - 1 as c_int) / width2 + 1 as c_int) * width2;
                        '_c2rust_label: {
                            if w <= 0 as c_int
                                || (*curwin.get()).w_curswant <= 2147483647 as c_int - w
                            {
                            } else {
                                __assert_fail(
                                    b"w <= 0 || curwin->w_curswant <= INT_MAX - w\0".as_ptr()
                                        as *const c_char,
                                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                                    2570 as c_uint,
                                    b"_Bool nv_screengo(oparg_T *, int, int, _Bool)\0".as_ptr()
                                        as *const c_char,
                                );
                            }
                        };
                        (*curwin.get()).w_curswant += w;
                    }
                }
            } else {
                if linelen > width1 {
                    n = ((linelen - width1 - 1 as c_int) / width2 + 1 as c_int) * width2 + width1;
                } else {
                    n = width1;
                }
                if (*curwin.get()).w_curswant as c_int + width2 < n
                    && !hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    (*curwin.get()).w_curswant += width2;
                } else if (*curwin.get()).w_cursor.lnum
                    >= (*(*curwin.get()).w_buffer).b_ml.ml_line_count
                {
                    retval = false_0 != 0;
                    break;
                } else {
                    cursor_down_inner(curwin.get(), 1 as c_int, skip_conceal);
                    (*curwin.get()).w_curswant %= width2;
                    if (*curwin.get()).w_curswant >= width1 {
                        (*curwin.get()).w_curswant -= width2;
                    }
                    linelen = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                }
            }
        }
    }
    if virtual_active(curwin.get()) as c_int != 0 && atend as c_int != 0 {
        coladvance(curwin.get(), MAXCOL as c_int);
    } else {
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
    }
    if (*curwin.get()).w_cursor.col > 0 as c_int && (*curwin.get()).w_onebuf_opt.wo_wrap != 0 {
        validate_virtcol(curwin.get());
        let mut virtcol: colnr_T = (*curwin.get()).w_virtcol;
        if virtcol > width1 && *get_showbreak_value(curwin.get()) as c_int != NUL {
            virtcol -= vim_strsize(get_showbreak_value(curwin.get()));
        }
        let mut c: c_int = utf_ptr2char(get_cursor_pos_ptr());
        if dir == FORWARD as c_int
            && virtcol < (*curwin.get()).w_curswant
            && (*curwin.get()).w_curswant <= width1
            && !vim_isprintc(c)
            && c > 255 as c_int
        {
            oneright();
        }
        if virtcol > (*curwin.get()).w_curswant
            && (if (*curwin.get()).w_curswant < width1 {
                ((*curwin.get()).w_curswant > width1 / 2 as c_int) as c_int
            } else {
                (((*curwin.get()).w_curswant as c_int - width1) % width2 > width2 / 2 as c_int)
                    as c_int
            }) != 0
        {
            (*curwin.get()).w_cursor.col -= 1;
        }
    }
    if atend {
        (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
    }
    adjust_skipcol();
    return retval;
}

pub(crate) unsafe extern "C" fn nv_scroll(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    let mut lnum: linenr_T = 0;
    (*(*cap).oap).motion_type = kMTLineWise;
    setpcmark();
    if (*cap).cmdchar == 'L' as c_int {
        validate_botline_win(curwin.get());
        (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_botline - 1 as linenr_T;
        if (*cap).count1 as linenr_T - 1 as linenr_T >= (*curwin.get()).w_cursor.lnum {
            (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        } else if win_lines_concealed(curwin.get()) {
            n = (*cap).count1 - 1 as c_int;
            while n > 0 as c_int && (*curwin.get()).w_cursor.lnum > (*curwin.get()).w_topline {
                hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum,
                    &raw mut (*curwin.get()).w_cursor.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                n += decor_conceal_line(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum as c_int,
                    true_0 != 0,
                ) as c_int;
                if (*curwin.get()).w_cursor.lnum > (*curwin.get()).w_topline {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                n -= 1;
            }
        } else {
            (*curwin.get()).w_cursor.lnum =
                ((*curwin.get()).w_cursor.lnum as c_int - ((*cap).count1 - 1 as c_int)) as linenr_T;
        }
    } else {
        if (*cap).cmdchar == 'M' as c_int {
            let mut used: c_int = 0 as c_int;
            used -=
                win_get_fill(curwin.get(), (*curwin.get()).w_topline) - (*curwin.get()).w_topfill;
            validate_botline_win(curwin.get());
            let mut half: c_int = ((*curwin.get()).w_view_height - (*curwin.get()).w_empty_rows
                + 1 as c_int)
                / 2 as c_int;
            n = 0 as c_int;
            while ((*curwin.get()).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                if n > 0 as c_int
                    && used
                        + win_get_fill(curwin.get(), (*curwin.get()).w_topline + n as linenr_T)
                            / 2 as c_int
                        >= half
                {
                    n -= 1;
                    break;
                } else {
                    used += plines_win(
                        curwin.get(),
                        (*curwin.get()).w_topline + n as linenr_T,
                        true_0 != 0,
                    );
                    if used >= half {
                        break;
                    }
                    if hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_topline + n as linenr_T,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    ) {
                        n = (lnum - (*curwin.get()).w_topline) as c_int;
                    }
                    n += 1;
                }
            }
            if n > 0 as c_int && used > (*curwin.get()).w_view_height {
                n -= 1;
            }
        } else {
            n = (*cap).count1 - 1 as c_int;
            if win_lines_concealed(curwin.get()) {
                lnum = (*curwin.get()).w_topline;
                while (decor_conceal_line(curwin.get(), lnum as c_int - 1 as c_int, true_0 != 0)
                    as c_int
                    != 0
                    || {
                        let c2rust_fresh12 = n;
                        n = n - 1;
                        c2rust_fresh12 > 0 as c_int
                    })
                    && lnum < (*curwin.get()).w_botline - 1 as linenr_T
                {
                    hasFolding(
                        curwin.get(),
                        lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    );
                    lnum += 1;
                }
                n = (lnum - (*curwin.get()).w_topline) as c_int;
            }
        }
        (*curwin.get()).w_cursor.lnum =
            if ((*curwin.get()).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_topline + n as linenr_T
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int {
        cursor_correct(curwin.get());
    }
    beginline(BL_SOL as c_int | BL_FIX as c_int);
}

pub(crate) unsafe extern "C" fn nv_right(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = true_0;
        }
        nv_wordcmd(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    let mut past_line: bool =
        VIsual_active.get() as c_int != 0 && *p_sel.get() as c_int != 'o' as c_int;
    if virtual_active(curwin.get()) {
        past_line = false_0 != 0;
    }
    n = (*cap).count1;
    while n > 0 as c_int {
        if !past_line && oneright() == false_0
            || past_line as c_int != 0 && *get_cursor_pos_ptr() as c_int == NUL
        {
            if ((*cap).cmdchar == ' ' as c_int && !vim_strchr(p_ww.get(), 's' as c_int).is_null()
                || (*cap).cmdchar == 'l' as c_int
                    && !vim_strchr(p_ww.get(), 'l' as c_int).is_null()
                || (*cap).cmdchar == K_RIGHT && !vim_strchr(p_ww.get(), '>' as c_int).is_null())
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                if (*(*cap).oap).op_type != OP_NOP as c_int
                    && !(*(*cap).oap).inclusive
                    && !(*ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL)
                {
                    (*(*cap).oap).inclusive = true_0 != 0;
                } else {
                    (*curwin.get()).w_cursor.lnum += 1;
                    (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
                    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
                    (*curwin.get()).w_set_curswant = true_0;
                    (*(*cap).oap).inclusive = false_0 != 0;
                }
            } else {
                if (*(*cap).oap).op_type == OP_NOP as c_int {
                    if n == (*cap).count1 {
                        beep_flush();
                    }
                } else if !(*ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL) {
                    (*(*cap).oap).inclusive = true_0 != 0;
                }
                break;
            }
        } else if past_line {
            (*curwin.get()).w_set_curswant = true_0;
            if virtual_active(curwin.get()) {
                oneright();
            } else {
                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
            }
        }
        n -= 1;
    }
    if n != (*cap).count1
        && fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_left(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = 1 as c_int;
        }
        nv_bck_word(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    n = (*cap).count1;
    while n > 0 as c_int {
        if oneleft() == false_0 {
            if (((*cap).cmdchar == K_BS || (*cap).cmdchar == Ctrl_H)
                && !vim_strchr(p_ww.get(), 'b' as c_int).is_null()
                || (*cap).cmdchar == 'h' as c_int
                    && !vim_strchr(p_ww.get(), 'h' as c_int).is_null()
                || (*cap).cmdchar == K_LEFT && !vim_strchr(p_ww.get(), '<' as c_int).is_null())
                && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            {
                (*curwin.get()).w_cursor.lnum -= 1;
                coladvance(curwin.get(), MAXCOL as c_int);
                (*curwin.get()).w_set_curswant = true_0;
                if ((*(*cap).oap).op_type == OP_DELETE as c_int
                    || (*(*cap).oap).op_type == OP_CHANGE as c_int)
                    && !(*ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL)
                {
                    let mut cp: *mut c_char = get_cursor_pos_ptr();
                    if *cp as c_int != NUL {
                        (*curwin.get()).w_cursor.col += utfc_ptr2len(cp);
                    }
                    (*cap).retval |= CA_NO_ADJ_OP_END as c_int;
                }
            } else {
                if (*(*cap).oap).op_type == OP_NOP as c_int && n == (*cap).count1 {
                    beep_flush();
                }
                break;
            }
        }
        n -= 1;
    }
    if n != (*cap).count1
        && fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_up(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        (*cap).arg = BACKWARD as c_int;
        nv_page(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTLineWise;
    if cursor_up(
        (*cap).count1 as linenr_T,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if (*cap).arg != 0 {
        beginline(BL_WHITE as c_int | BL_FIX as c_int);
    }
}

pub(crate) unsafe extern "C" fn nv_down(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        (*cap).arg = FORWARD as c_int;
        nv_page(cap);
    } else if bt_quickfix(curbuf.get()) as c_int != 0 && (*cap).cmdchar == CAR {
        qf_view_result(false_0 != 0);
    } else if cmdwin_type.get() != 0 as c_int && (*cap).cmdchar == CAR {
        cmdwin_result.set(CAR);
    } else if bt_prompt(curbuf.get()) as c_int != 0
        && (*cap).cmdchar == CAR
        && (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count
    {
        prompt_invoke_callback();
        if restart_edit.get() == 0 as c_int {
            restart_edit.set('a' as c_int);
        }
    } else {
        (*(*cap).oap).motion_type = kMTLineWise;
        if cursor_down((*cap).count1, (*(*cap).oap).op_type == OP_NOP as c_int) == false_0 {
            clearopbeep((*cap).oap);
        } else if (*cap).arg != 0 {
            beginline(BL_WHITE as c_int | BL_FIX as c_int);
        }
    };
}

pub(crate) unsafe extern "C" fn nv_end(mut cap: *mut cmdarg_T) {
    if (*cap).arg != 0 || mod_mask.get() & MOD_MASK_CTRL != 0 {
        (*cap).arg = true_0;
        nv_goto(cap);
        (*cap).count1 = 1 as c_int;
    }
    nv_dollar(cap);
}

pub(crate) unsafe extern "C" fn nv_dollar(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = true_0 != 0;
    if !virtual_active(curwin.get())
        || gchar_cursor() != NUL
        || (*(*cap).oap).op_type == OP_NOP as c_int
    {
        (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
    }
    if cursor_down(
        (*cap).count1 - 1 as c_int,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_csearch(mut cap: *mut cmdarg_T) {
    let mut cursor_dec: bool = false_0 != 0;
    if *p_sel.get() as c_int == 'e' as c_int
        && VIsual_active.get() as c_int != 0
        && VIsual_mode.get() == 'v' as c_int
        && VIsual_select_exclu_adj.get() as c_int != 0
    {
        unadjust_for_sel();
        cursor_dec = true_0 != 0;
    }
    let mut t_cmd: bool = (*cap).cmdchar == 't' as c_int || (*cap).cmdchar == 'T' as c_int;
    (*(*cap).oap).motion_type = kMTCharWise;
    if (*cap).nchar < 0 as c_int || searchc(cap, t_cmd) == false_0 {
        clearopbeep((*cap).oap);
        if cursor_dec {
            adjust_for_sel(cap);
        }
        return;
    }
    (*curwin.get()).w_set_curswant = true_0;
    if gchar_cursor() == TAB
        && virtual_active(curwin.get()) as c_int != 0
        && (*cap).arg == FORWARD as c_int
        && (t_cmd as c_int != 0 || (*(*cap).oap).op_type != OP_NOP as c_int)
    {
        let mut scol: colnr_T = 0;
        let mut ecol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut scol,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ecol,
        );
        (*curwin.get()).w_cursor.coladd = ecol - scol;
    } else {
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    }
    adjust_for_sel(cap);
    if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_percent(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    (*(*cap).oap).inclusive = true_0 != 0;
    if (*cap).count0 != 0 {
        if (*cap).count0 > 100 as c_int {
            clearopbeep((*cap).oap);
        } else {
            (*(*cap).oap).motion_type = kMTLineWise;
            setpcmark();
            if (*curbuf.get()).b_ml.ml_line_count >= 21474836 as linenr_T {
                (*curwin.get()).w_cursor.lnum =
                    ((*curbuf.get()).b_ml.ml_line_count + 99 as linenr_T) / 100 as linenr_T
                        * (*cap).count0 as linenr_T;
            } else {
                (*curwin.get()).w_cursor.lnum = ((*curbuf.get()).b_ml.ml_line_count
                    * (*cap).count0 as linenr_T
                    + 99 as linenr_T)
                    / 100 as linenr_T;
            }
            (*curwin.get()).w_cursor.lnum = if (if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                (*curwin.get()).w_cursor.lnum
            } else {
                1 as linenr_T
            }) < (*curbuf.get()).b_ml.ml_line_count
            {
                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                    (*curwin.get()).w_cursor.lnum
                } else {
                    1 as linenr_T
                }
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
    } else {
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).use_reg_one = true_0 != 0;
        pos = findmatch((*cap).oap, NUL);
        if pos.is_null() {
            clearopbeep((*cap).oap);
        } else {
            setpcmark();
            (*curwin.get()).w_cursor = *pos;
            (*curwin.get()).w_set_curswant = true_0;
            (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
            adjust_for_sel(cap);
        }
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int
        && lnum != (*curwin.get()).w_cursor.lnum
        && fdo_flags.get() & kOptFdoFlagPercent as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_brace(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if findsent((*cap).arg as Direction, (*cap).count1) == FAIL {
        clearopbeep((*cap).oap);
        return;
    }
    adjust_cursor((*cap).oap);
    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_findpar(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if !findpar(
        &raw mut (*(*cap).oap).inclusive,
        (*cap).arg,
        (*cap).count1,
        NUL,
        false_0 != 0,
    ) {
        clearopbeep((*cap).oap);
        return;
    }
    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_home(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        nv_goto(cap);
    } else {
        (*cap).count0 = 1 as c_int;
        nv_pipe(cap);
    }
    ins_at_eol.set(false_0 != 0);
}

pub(crate) unsafe extern "C" fn nv_pipe(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    beginline(0 as c_int);
    if (*cap).count0 > 0 as c_int {
        coladvance(curwin.get(), (*cap).count0 - 1 as c_int);
        (*curwin.get()).w_curswant = (*cap).count0 - 1 as c_int;
    } else {
        (*curwin.get()).w_curswant = 0 as c_int as colnr_T;
    }
    (*curwin.get()).w_set_curswant = false_0;
}

pub(crate) unsafe extern "C" fn nv_bck_word(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if bck_word((*cap).count1, (*cap).arg != 0, false_0 != 0) == false_0 {
        clearopbeep((*cap).oap);
    } else if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_wordcmd(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    let mut word_end: bool = false;
    let mut flag: bool = false_0 != 0;
    let mut startpos: pos_T = (*curwin.get()).w_cursor;
    if (*cap).cmdchar == 'e' as c_int || (*cap).cmdchar == 'E' as c_int {
        word_end = true_0 != 0;
    } else {
        word_end = false_0 != 0;
    }
    (*(*cap).oap).inclusive = word_end;
    if !word_end && (*(*cap).oap).op_type == OP_CHANGE as c_int {
        n = gchar_cursor();
        if n != NUL && !ascii_iswhite(n) {
            if !vim_strchr(p_cpo.get(), CPO_CHANGEW).is_null() {
                (*(*cap).oap).inclusive = true_0 != 0;
                word_end = true_0 != 0;
            }
            flag = true_0 != 0;
        }
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*curwin.get()).w_set_curswant = true_0;
    if word_end {
        n = end_word((*cap).count1, (*cap).arg != 0, flag, false_0 != 0);
    } else {
        n = fwd_word(
            (*cap).count1,
            (*cap).arg != 0,
            (*(*cap).oap).op_type != OP_NOP as c_int,
        );
    }
    if lt(startpos, (*curwin.get()).w_cursor) {
        adjust_cursor((*cap).oap);
    }
    if n == false_0 && (*(*cap).oap).op_type == OP_NOP as c_int {
        clearopbeep((*cap).oap);
    } else {
        adjust_for_sel(cap);
        if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
            && (*(*cap).oap).op_type == OP_NOP as c_int
        {
            foldOpenCursor();
        }
    };
}

pub(crate) unsafe extern "C" fn adjust_cursor(mut oap: *mut oparg_T) {
    if (*curwin.get()).w_cursor.col > 0 as c_int
        && gchar_cursor() == NUL
        && (!VIsual_active.get() || *p_sel.get() as c_int == 'o' as c_int)
        && !virtual_active(curwin.get())
        && get_ve_flags(curwin.get()) & kOptVeFlagOnemore as c_int as c_uint == 0 as c_uint
    {
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*oap).inclusive = true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn nv_beginline(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    beginline((*cap).arg);
    if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
    ins_at_eol.set(false_0 != 0);
}

pub(crate) unsafe extern "C" fn nv_goto(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = 0;
    if (*cap).arg != 0 {
        lnum = (*curbuf.get()).b_ml.ml_line_count;
    } else {
        lnum = 1 as c_int as linenr_T;
    }
    (*(*cap).oap).motion_type = kMTLineWise;
    setpcmark();
    if (*cap).count0 != 0 as c_int {
        lnum = (*cap).count0 as linenr_T;
    }
    lnum = if (if lnum > 1 as linenr_T {
        lnum
    } else {
        1 as linenr_T
    }) < (*curbuf.get()).b_ml.ml_line_count
    {
        if lnum > 1 as linenr_T {
            lnum
        } else {
            1 as linenr_T
        }
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    (*curwin.get()).w_cursor.lnum = lnum;
    beginline(BL_SOL as c_int | BL_FIX as c_int);
    if fdo_flags.get() & kOptFdoFlagJump as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
