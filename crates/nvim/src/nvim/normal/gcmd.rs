//! The `g` prefix tree.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nv_g_home_m_cmd(mut cap: *mut cmdarg_T) {
    let mut i: c_int = 0;
    let flag: bool = (*cap).nchar == '^' as c_int;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && (*curwin.get()).w_view_width != 0 as c_int {
        let mut width1: c_int = (*curwin.get()).w_view_width - win_col_off(curwin.get());
        let mut width2: c_int = width1 + win_col_off2(curwin.get());
        validate_virtcol(curwin.get());
        i = 0 as c_int;
        if (*curwin.get()).w_virtcol >= width1 && width2 > 0 as c_int {
            i = ((*curwin.get()).w_virtcol as c_int - width1) / width2 * width2 + width1;
        }
        if (*curwin.get()).w_skipcol > 0 as c_int
            && (*curwin.get()).w_cursor.lnum == (*curwin.get()).w_topline
        {
            let mut overlap: c_int =
                sms_marker_overlap(curwin.get(), (*curwin.get()).w_view_width - width2);
            if overlap > 0 as c_int && i == (*curwin.get()).w_skipcol {
                i += overlap;
            }
        }
    } else {
        i = (*curwin.get()).w_leftcol as c_int;
    }
    if (*cap).nchar == 'm' as c_int {
        i += ((*curwin.get()).w_view_width - win_col_off(curwin.get())
            + (if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && i > 0 as c_int {
                win_col_off2(curwin.get())
            } else {
                0 as c_int
            }))
            / 2 as c_int;
    }
    coladvance(curwin.get(), i);
    if flag {
        loop {
            i = gchar_cursor();
            if !(ascii_iswhite(i) as c_int != 0 && oneright() == OK) {
                break;
            }
        }
        (*curwin.get()).w_valid &= !VALID_WCOL;
    }
    (*curwin.get()).w_set_curswant = true_0;
    if hasAnyFolding(curwin.get()) != 0 {
        validate_cheight(curwin.get());
        if (*curwin.get()).w_cline_folded {
            update_curswant_force();
        }
    }
    adjust_skipcol();
}

pub(crate) unsafe extern "C" fn nv_g_underscore_cmd(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = true_0 != 0;
    (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
    if cursor_down(
        (*cap).count1 - 1 as c_int,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
        return;
    }
    let mut ptr: *mut c_char = get_cursor_line_ptr();
    if (*curwin.get()).w_cursor.col > 0 as c_int
        && *ptr.offset((*curwin.get()).w_cursor.col as isize) as c_int == NUL
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
    while (*curwin.get()).w_cursor.col > 0 as c_int
        && ascii_iswhite(*ptr.offset((*curwin.get()).w_cursor.col as isize) as c_int) as c_int != 0
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
    (*curwin.get()).w_set_curswant = true_0;
    adjust_for_sel(cap);
}

pub(crate) unsafe extern "C" fn nv_g_dollar_cmd(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut i: c_int = 0;
    let mut col_off: c_int = win_col_off(curwin.get());
    let flag: bool = (*cap).nchar == K_END || (*cap).nchar == K_KEND;
    (*oap).motion_type = kMTCharWise;
    (*oap).inclusive = true_0 != 0;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && (*curwin.get()).w_view_width != 0 as c_int {
        (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
        if (*cap).count1 == 1 as c_int {
            let mut width1: c_int = (*curwin.get()).w_view_width - col_off;
            let mut width2: c_int = width1 + win_col_off2(curwin.get());
            validate_virtcol(curwin.get());
            i = width1 - 1 as c_int;
            if (*curwin.get()).w_virtcol >= width1 {
                i += (((*curwin.get()).w_virtcol as c_int - width1) / width2 + 1 as c_int) * width2;
            }
            coladvance(curwin.get(), i);
            update_curswant_force();
            if (*curwin.get()).w_cursor.col > 0 as c_int
                && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            {
                if (*curwin.get()).w_virtcol > i {
                    (*curwin.get()).w_cursor.col -= 1;
                }
            }
        } else if nv_screengo(
            oap,
            FORWARD as c_int,
            (*cap).count1 - 1 as c_int,
            false_0 != 0,
        ) as c_int
            == false_0
        {
            clearopbeep(oap);
        }
    } else {
        if (*cap).count1 > 1 as c_int {
            cursor_down((*cap).count1 - 1 as c_int, false_0 != 0);
        }
        i = (*curwin.get()).w_leftcol as c_int + (*curwin.get()).w_view_width
            - col_off
            - 1 as c_int;
        coladvance(curwin.get(), i);
        if (*curwin.get()).w_cursor.col > 0 as c_int
            && utf_ptr2cells(get_cursor_pos_ptr()) > 1 as c_int
        {
            let mut vcol: colnr_T = 0;
            getvvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol,
            );
            if vcol >= (*curwin.get()).w_leftcol as c_int + (*curwin.get()).w_view_width - col_off {
                (*curwin.get()).w_cursor.col -= 1;
            }
        }
        update_curswant_force();
    }
    if flag {
        loop {
            i = gchar_cursor();
            if !(ascii_iswhite_or_nul(i) as c_int != 0 && oneleft() == OK) {
                break;
            }
        }
        (*curwin.get()).w_valid &= !VALID_WCOL;
    }
}

pub(crate) unsafe extern "C" fn nv_gi_cmd(mut cap: *mut cmdarg_T) {
    if (*curbuf.get()).b_last_insert.mark.lnum != 0 as linenr_T {
        (*curwin.get()).w_cursor = (*curbuf.get()).b_last_insert.mark;
        check_cursor_lnum(curwin.get());
        let mut i: c_int = get_cursor_line_len();
        if (*curwin.get()).w_cursor.col > i {
            if virtual_active(curwin.get()) {
                (*curwin.get()).w_cursor.coladd += (*curwin.get()).w_cursor.col as c_int - i;
            }
            (*curwin.get()).w_cursor.col = i as colnr_T;
        }
    }
    (*cap).cmdchar = 'i' as c_int;
    nv_edit(cap);
}

pub(crate) unsafe extern "C" fn nv_g_cmd(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut i: c_int = 0;
    's_650: {
        'c_40473: {
            'c_36907: {
                match (*cap).nchar {
                    Ctrl_A | Ctrl_X => {
                        if VIsual_active.get() {
                            (*cap).arg = true_0;
                            (*cap).cmdchar = (*cap).nchar;
                            (*cap).nchar = NUL;
                            nv_addsub(cap);
                        } else {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    82 => {
                        (*cap).arg = true_0;
                        nv_Replace(cap);
                        break 's_650;
                    }
                    114 => {
                        nv_vreplace(cap);
                        break 's_650;
                    }
                    38 => {
                        do_cmdline_cmd(b"%s//~/&\0".as_ptr() as *const c_char);
                        break 's_650;
                    }
                    118 => {
                        nv_gv_cmd(cap);
                        break 's_650;
                    }
                    86 => {
                        VIsual_reselect.set(false_0);
                        break 's_650;
                    }
                    K_BS => {
                        (*cap).nchar = Ctrl_H;
                    }
                    104 | 72 | Ctrl_H => {}
                    78 | 110 => {
                        if current_search((*cap).count1, (*cap).nchar == 'n' as c_int) == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    106 | K_DOWN => {
                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                            (*oap).motion_type = kMTLineWise;
                            i = cursor_down((*cap).count1, (*oap).op_type == OP_NOP as c_int);
                        } else {
                            i = nv_screengo(oap, FORWARD as c_int, (*cap).count1, false_0 != 0)
                                as c_int;
                        }
                        if i == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    107 | K_UP => {
                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                            (*oap).motion_type = kMTLineWise;
                            i = cursor_up(
                                (*cap).count1 as linenr_T,
                                (*oap).op_type == OP_NOP as c_int,
                            );
                        } else {
                            i = nv_screengo(oap, BACKWARD as c_int, (*cap).count1, false_0 != 0)
                                as c_int;
                        }
                        if i == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    74 => {
                        nv_join(cap);
                        break 's_650;
                    }
                    94 | 48 | 109 | K_HOME | K_KHOME => {
                        nv_g_home_m_cmd(cap);
                        break 's_650;
                    }
                    77 => {
                        (*oap).motion_type = kMTCharWise;
                        (*oap).inclusive = false_0 != 0;
                        i = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                        if (*cap).count0 > 0 as c_int && (*cap).count0 <= 100 as c_int {
                            coladvance(curwin.get(), i * (*cap).count0 / 100 as c_int);
                        } else {
                            coladvance(curwin.get(), i / 2 as c_int);
                        }
                        (*curwin.get()).w_set_curswant = true_0;
                        break 's_650;
                    }
                    95 => {
                        nv_g_underscore_cmd(cap);
                        break 's_650;
                    }
                    36 | K_END | K_KEND => {
                        nv_g_dollar_cmd(cap);
                        break 's_650;
                    }
                    42 | 35 | POUND | Ctrl_RSB | 93 => {
                        nv_ident(cap);
                        break 's_650;
                    }
                    101 | 69 => {
                        (*oap).motion_type = kMTCharWise;
                        (*curwin.get()).w_set_curswant = true_0;
                        (*oap).inclusive = true_0 != 0;
                        if bckend_word((*cap).count1, (*cap).nchar == 'E' as c_int, false_0 != 0)
                            == false_0
                        {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    Ctrl_G => {
                        cursor_pos_info(::core::ptr::null_mut::<dict_T>());
                        break 's_650;
                    }
                    105 => {
                        nv_gi_cmd(cap);
                        break 's_650;
                    }
                    73 => {
                        beginline(0 as c_int);
                        if !checkclearopq(oap) {
                            invoke_edit(cap, false_0, 'g' as c_int, false_0);
                        }
                        break 's_650;
                    }
                    102 | 70 => {
                        nv_gotofile(cap);
                        break 's_650;
                    }
                    39 => {
                        (*cap).arg = true_0;
                        break 'c_36907;
                    }
                    96 => {
                        break 'c_36907;
                    }
                    115 => {
                        do_sleep(((*cap).count1 * 1000 as c_int) as int64_t, false_0 != 0);
                        break 's_650;
                    }
                    97 => {
                        do_ascii(::core::ptr::null_mut::<exarg_T>());
                        break 's_650;
                    }
                    56 => {
                        if (*cap).count0 == 8 as c_int {
                            utf_find_illegal();
                        } else {
                            show_utf8();
                        }
                        break 's_650;
                    }
                    60 => {
                        show_sb_text();
                        break 's_650;
                    }
                    103 => {
                        (*cap).arg = false_0;
                        nv_goto(cap);
                        break 's_650;
                    }
                    113 | 119 => {
                        (*oap).cursor_start = (*curwin.get()).w_cursor;
                        break 'c_40473;
                    }
                    126 | 117 | 85 | 63 | 64 => {
                        break 'c_40473;
                    }
                    100 | 68 => {
                        nv_gd(oap, (*cap).nchar, (*cap).count0);
                        break 's_650;
                    }
                    -12285 | -12541 | -12797 | -11517 | -11773 | -12029 | -25853 | -13053
                    | -13309 | -13565 | -23037 | -23293 | -23549 | -23805 | -24061 | -24317 => {
                        mod_mask.set(MOD_MASK_CTRL);
                        do_mouse(oap, (*cap).nchar, BACKWARD as c_int, (*cap).count1, false);
                        break 's_650;
                    }
                    -13821 => {
                        break 's_650;
                    }
                    112 | 80 => {
                        nv_put(cap);
                        break 's_650;
                    }
                    111 => {
                        (*oap).inclusive = false_0 != 0;
                        goto_byte((*cap).count0);
                        break 's_650;
                    }
                    81 => {
                        if !check_text_locked((*cap).oap) && !checkclearopq(oap) {
                            do_exmode();
                        }
                        break 's_650;
                    }
                    44 => {
                        nv_pcmark(cap);
                        break 's_650;
                    }
                    59 => {
                        (*cap).count1 = -(*cap).count1;
                        nv_pcmark(cap);
                        break 's_650;
                    }
                    116 => {
                        if !checkclearop(oap) {
                            goto_tabpage((*cap).count0);
                        }
                        break 's_650;
                    }
                    84 => {
                        if !checkclearop(oap) {
                            goto_tabpage(-(*cap).count1);
                        }
                        break 's_650;
                    }
                    TAB => {
                        if !checkclearop(oap) && !goto_tabpage_lastused() {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    43 | 45 => {
                        if !checkclearopq(oap) {
                            undo_time(
                                if (*cap).nchar == '-' as c_int {
                                    -(*cap).count1
                                } else {
                                    (*cap).count1
                                },
                                false_0 != 0,
                                false_0 != 0,
                                false_0 != 0,
                            );
                        }
                        break 's_650;
                    }
                    _ => {
                        clearopbeep(oap);
                        break 's_650;
                    }
                }
                (*cap).cmdchar = (*cap).nchar + ('v' as c_int - 'h' as c_int);
                (*cap).arg = true_0;
                nv_visual(cap);
                break 's_650;
            }
            nv_gomark(cap);
            break 's_650;
        }
        nv_operator(cap);
    };
}
