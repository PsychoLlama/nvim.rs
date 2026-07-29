//! Scrolling and the view: the page and half-page commands, and the
//! whole `z` prefix tree, including 'scrollbind'.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_vtopline(mut wp: *mut win_T) -> c_int {
    return plines_m_win_fill(wp, 1 as linenr_T, (*wp).w_topline) - (*wp).w_topfill;
}

pub unsafe extern "C" fn do_check_scrollbind(mut check: bool) {
    static old_curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    static old_vtopline: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
    static old_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
    static old_leftcol: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
    let mut vtopline: c_int = get_vtopline(curwin.get());
    if check as c_int != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        if did_syncbind.get() {
            did_syncbind.set(false_0 != 0);
        } else if curwin.get() == old_curwin.get() {
            if ((*curwin.get()).w_buffer == old_buf.get()
                || (*curwin.get()).w_onebuf_opt.wo_diff != 0)
                && (vtopline as linenr_T != old_vtopline.get()
                    || (*curwin.get()).w_leftcol != old_leftcol.get())
            {
                check_scrollbind(
                    vtopline as linenr_T - old_vtopline.get(),
                    (*curwin.get()).w_leftcol as c_int - old_leftcol.get() as c_int,
                );
            }
        } else if !vim_strchr(p_sbo.get(), 'j' as c_int).is_null() {
            check_scrollbind(
                vtopline as linenr_T - (*curwin.get()).w_scbind_pos as linenr_T,
                0 as c_int,
            );
        }
        (*curwin.get()).w_scbind_pos = vtopline;
    }
    old_curwin.set(curwin.get());
    old_vtopline.set(vtopline as linenr_T);
    old_buf.set((*curwin.get()).w_buffer);
    old_leftcol.set((*curwin.get()).w_leftcol);
}

pub unsafe extern "C" fn check_scrollbind(mut vtopline_diff: linenr_T, mut leftcol_diff: c_int) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut old_VIsual_select: c_int = VIsual_select.get() as c_int;
    let mut old_VIsual_active: c_int = VIsual_active.get() as c_int;
    let mut tgt_leftcol: colnr_T = (*curwin.get()).w_leftcol;
    let mut want_ver: bool = (*old_curwin).w_onebuf_opt.wo_diff != 0
        || !vim_strchr(p_sbo.get(), 'v' as c_int).is_null() && vtopline_diff != 0 as linenr_T;
    let mut want_hor: bool = !vim_strchr(p_sbo.get(), 'h' as c_int).is_null()
        && (leftcol_diff != 0 || vtopline_diff != 0 as linenr_T);
    VIsual_active.set(false);
    VIsual_select.set(VIsual_active.get());
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        curwin.set(wp);
        curbuf.set((*curwin.get()).w_buffer);
        if !(curwin.get() == old_curwin || (*curwin.get()).w_onebuf_opt.wo_scb == 0) {
            if want_ver {
                if (*old_curwin).w_onebuf_opt.wo_diff != 0
                    && (*curwin.get()).w_onebuf_opt.wo_diff != 0
                {
                    diff_set_topline(old_curwin, curwin.get());
                } else {
                    (*curwin.get()).w_scbind_pos += vtopline_diff as c_int;
                    let mut curr_vtopline: c_int = get_vtopline(curwin.get());
                    let mut max_vtopline: c_int = curr_vtopline
                        + (*curwin.get()).w_topfill
                        + plines_m_win_fill(
                            curwin.get(),
                            (*curwin.get()).w_topline + 1 as linenr_T,
                            (*curbuf.get()).b_ml.ml_line_count,
                        );
                    let mut new_vtopline: c_int = if (if ((*curwin.get()).w_scbind_pos as linenr_T)
                        < max_vtopline as linenr_T
                    {
                        (*curwin.get()).w_scbind_pos as linenr_T
                    } else {
                        max_vtopline as linenr_T
                    }) > 1 as linenr_T
                    {
                        if ((*curwin.get()).w_scbind_pos as linenr_T) < max_vtopline as linenr_T {
                            (*curwin.get()).w_scbind_pos
                        } else {
                            max_vtopline
                        }
                    } else {
                        1 as c_int
                    };
                    let mut y: c_int = new_vtopline - curr_vtopline;
                    if y > 0 as c_int {
                        scrollup(curwin.get(), y as linenr_T, false_0 != 0);
                    } else {
                        scrolldown(curwin.get(), -(y as linenr_T), false_0);
                    }
                }
                redraw_later(curwin.get(), UPD_VALID as c_int);
                cursor_correct(curwin.get());
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
            if want_hor {
                set_leftcol(tgt_leftcol);
            }
        }
        wp = (*wp).w_next;
    }
    VIsual_select.set(old_VIsual_select != 0);
    VIsual_active.set(old_VIsual_active != 0);
    curwin.set(old_curwin);
    curbuf.set(old_curbuf);
}

pub(crate) unsafe extern "C" fn nv_page(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        if (*cap).arg == BACKWARD as c_int {
            goto_tabpage(-(*cap).count1);
        } else {
            goto_tabpage((*cap).count0);
        }
    } else {
        pagescroll((*cap).arg as Direction, (*cap).count1, false_0 != 0);
    };
}

pub unsafe extern "C" fn nv_scroll_line(mut cap: *mut cmdarg_T) {
    if !checkclearop((*cap).oap) {
        scroll_redraw((*cap).arg, (*cap).count1 as linenr_T);
    }
}

pub(crate) unsafe extern "C" fn nv_z_get_count(
    mut cap: *mut cmdarg_T,
    mut nchar_arg: *mut c_int,
) -> bool {
    let mut nchar: c_int = *nchar_arg;
    if checkclearop((*cap).oap) {
        return false_0 != 0;
    }
    let mut n: c_int = nchar - '0' as c_int;
    loop {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        nchar = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && nchar >= 0 as c_int
        {
            if nchar < 256 as c_int {
                nchar = (*langmap_mapchar.ptr())[nchar as usize] as c_int;
            } else {
                nchar = langmap_adjust_mb(nchar);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(nchar);
        if nchar == K_DEL || nchar == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)) {
            n /= 10 as c_int;
        } else if ascii_isdigit(nchar) {
            if crate::src::nvim::math::vim_append_digit_int(&mut n, nchar - '0' as c_int) {
                continue;
            }
            clearopbeep((*cap).oap);
            break;
        } else if nchar == CAR {
            win_setheight(n);
            break;
        } else if nchar == 'l' as c_int
            || nchar == 'h' as c_int
            || nchar == K_LEFT
            || nchar == K_RIGHT
        {
            (*cap).count1 = if n != 0 {
                n * (*cap).count1
            } else {
                (*cap).count1
            };
            *nchar_arg = nchar;
            return true_0 != 0;
        } else {
            clearopbeep((*cap).oap);
            break;
        }
    }
    (*(*cap).oap).op_type = OP_NOP as c_int;
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn nv_zg_zw(mut cap: *mut cmdarg_T, mut nchar: c_int) -> c_int {
    let mut undo: bool = false_0 != 0;
    if nchar == 'u' as c_int {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        nchar = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && nchar >= 0 as c_int
        {
            if nchar < 256 as c_int {
                nchar = (*langmap_mapchar.ptr())[nchar as usize] as c_int;
            } else {
                nchar = langmap_adjust_mb(nchar);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(nchar);
        if vim_strchr(b"gGwW\0".as_ptr() as *const c_char, nchar).is_null() {
            clearopbeep((*cap).oap);
            return OK;
        }
        undo = true_0 != 0;
    }
    if checkclearop((*cap).oap) {
        return OK;
    }
    let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut len: size_t = 0;
    if VIsual_active.get() as c_int != 0 && !get_visual_text(cap, &raw mut ptr, &raw mut len) {
        return FAIL;
    }
    if ptr.is_null() {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*emsg_off.ptr()) += 1;
        len = spell_move_to(
            curwin.get(),
            FORWARD as c_int,
            SMT_ALL,
            true_0 != 0,
            ::core::ptr::null_mut::<hlf_T>(),
        );
        (*emsg_off.ptr()) -= 1;
        if len != 0 as size_t && (*curwin.get()).w_cursor.col <= pos.col {
            ptr = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
        }
        (*curwin.get()).w_cursor = pos;
    }
    if ptr.is_null() && {
        len = find_ident_under_cursor(
            &raw mut ptr,
            FIND_IDENT as c_int,
            ::core::ptr::null_mut::<c_int>(),
        );
        len == 0 as size_t
    } {
        return FAIL;
    }
    '_c2rust_label: {
        if len <= 2147483647 as c_int as size_t {
        } else {
            __assert_fail(
                b"len <= INT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                2754 as c_uint,
                b"int nv_zg_zw(cmdarg_T *, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    spell_add_word(
        ptr,
        len as c_int,
        (if nchar == 'w' as c_int || nchar == 'W' as c_int {
            SPELL_ADD_BAD as c_int
        } else {
            SPELL_ADD_GOOD as c_int
        }) as SpellAddType,
        if nchar == 'G' as c_int || nchar == 'W' as c_int {
            0 as c_int
        } else {
            (*cap).count1
        },
        undo,
    );
    return OK;
}

pub(crate) unsafe extern "C" fn nv_zet(mut cap: *mut cmdarg_T) {
    let mut col: colnr_T = 0;
    let mut nchar: c_int = (*cap).nchar;
    let mut old_fdl: c_int = (*curwin.get()).w_onebuf_opt.wo_fdl as c_int;
    let mut old_fen: c_int = (*curwin.get()).w_onebuf_opt.wo_fen;
    let mut siso: int64_t = get_sidescrolloff_value(curwin.get());
    if ascii_isdigit(nchar) as c_int != 0 && !nv_z_get_count(cap, &raw mut nchar) {
        return;
    }
    if (*cap).nchar != 'f' as c_int
        && (*cap).nchar != 'F' as c_int
        && !(VIsual_active.get() as c_int != 0
            && !vim_strchr(b"dcCoO\0".as_ptr() as *const c_char, (*cap).nchar).is_null())
        && (*cap).nchar != 'j' as c_int
        && (*cap).nchar != 'k' as c_int
        && checkclearop((*cap).oap) as c_int != 0
    {
        return;
    }
    if !vim_strchr(b"+\r\nt.z^-b\0".as_ptr() as *const c_char, nchar).is_null()
        && (*cap).count0 != 0
        && (*cap).count0 as linenr_T != (*curwin.get()).w_cursor.lnum
    {
        setpcmark();
        if (*cap).count0 as linenr_T > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        } else {
            (*curwin.get()).w_cursor.lnum = (*cap).count0 as linenr_T;
        }
        check_cursor_col(curwin.get());
    }
    's_906: {
        'c_53178: {
            'c_53195: {
                'c_55145: {
                    'c_55198: {
                        'c_53192: {
                            'c_55413: {
                                match nchar {
                                    43 => {
                                        if (*cap).count0 == 0 as c_int {
                                            validate_botline_win(curwin.get());
                                            (*curwin.get()).w_cursor.lnum = if (*curwin.get())
                                                .w_botline
                                                < (*curbuf.get()).b_ml.ml_line_count
                                            {
                                                (*curwin.get()).w_botline
                                            } else {
                                                (*curbuf.get()).b_ml.ml_line_count
                                            };
                                        }
                                        break 'c_55413;
                                    }
                                    NL | CAR | K_KENTER => {
                                        break 'c_55413;
                                    }
                                    116 => {
                                        break 'c_53178;
                                    }
                                    46 => {
                                        beginline(BL_WHITE as c_int | BL_FIX as c_int);
                                    }
                                    122 => {}
                                    94 => {
                                        if (*cap).count0 != 0 as c_int {
                                            scroll_cursor_bot(
                                                curwin.get(),
                                                0 as c_int,
                                                true_0 != 0,
                                            );
                                            (*curwin.get()).w_cursor.lnum =
                                                (*curwin.get()).w_topline;
                                        } else if (*curwin.get()).w_topline == 1 as linenr_T {
                                            (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
                                        } else {
                                            (*curwin.get()).w_cursor.lnum =
                                                (*curwin.get()).w_topline - 1 as linenr_T;
                                        }
                                        break 'c_53192;
                                    }
                                    45 => {
                                        break 'c_53192;
                                    }
                                    98 => {
                                        break 'c_53195;
                                    }
                                    72 => {
                                        (*cap).count1 *= (*curwin.get()).w_view_width / 2 as c_int;
                                        break 'c_55198;
                                    }
                                    104 | K_LEFT => {
                                        break 'c_55198;
                                    }
                                    76 => {
                                        (*cap).count1 *= (*curwin.get()).w_view_width / 2 as c_int;
                                        break 'c_55145;
                                    }
                                    108 | K_RIGHT => {
                                        break 'c_55145;
                                    }
                                    115 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                                            if hasFolding(
                                                curwin.get(),
                                                (*curwin.get()).w_cursor.lnum,
                                                ::core::ptr::null_mut::<linenr_T>(),
                                                ::core::ptr::null_mut::<linenr_T>(),
                                            ) {
                                                col = 0 as c_int as colnr_T;
                                            } else {
                                                getvcol(
                                                    curwin.get(),
                                                    &raw mut (*curwin.get()).w_cursor,
                                                    &raw mut col,
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                );
                                            }
                                            if col as int64_t > siso {
                                                col -= siso as c_int;
                                            } else {
                                                col = 0 as c_int as colnr_T;
                                            }
                                            if (*curwin.get()).w_leftcol != col {
                                                (*curwin.get()).w_leftcol = col;
                                                redraw_later(curwin.get(), UPD_NOT_VALID as c_int);
                                            }
                                        }
                                        break 's_906;
                                    }
                                    101 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                                            if hasFolding(
                                                curwin.get(),
                                                (*curwin.get()).w_cursor.lnum,
                                                ::core::ptr::null_mut::<linenr_T>(),
                                                ::core::ptr::null_mut::<linenr_T>(),
                                            ) {
                                                col = 0 as c_int as colnr_T;
                                            } else {
                                                getvcol(
                                                    curwin.get(),
                                                    &raw mut (*curwin.get()).w_cursor,
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    &raw mut col,
                                                );
                                            }
                                            let mut n: c_int = (*curwin.get()).w_view_width
                                                - win_col_off(curwin.get());
                                            if col as int64_t + siso < n as int64_t {
                                                col = 0 as c_int as colnr_T;
                                            } else if (siso - n as int64_t)
                                                < (INT_MAX - col) as int64_t
                                            {
                                                col = (col as int64_t + siso - n as int64_t
                                                    + 1 as int64_t)
                                                    as c_int
                                                    as colnr_T;
                                            } else {
                                                col = INT_MAX as colnr_T;
                                            }
                                            if (*curwin.get()).w_leftcol != col {
                                                (*curwin.get()).w_leftcol = col;
                                                redraw_later(curwin.get(), UPD_NOT_VALID as c_int);
                                            }
                                        }
                                        break 's_906;
                                    }
                                    80 | 112 => {
                                        nv_put(cap);
                                        break 's_906;
                                    }
                                    121 => {
                                        nv_operator(cap);
                                        break 's_906;
                                    }
                                    70 | 102 => {
                                        if foldManualAllowed(true_0 != 0) != 0 {
                                            (*cap).nchar = 'f' as c_int;
                                            nv_operator(cap);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                            if nchar == 'F' as c_int
                                                && (*(*cap).oap).op_type == OP_FOLD as c_int
                                            {
                                                nv_operator(cap);
                                                finish_op.set(true_0 != 0);
                                            }
                                        } else {
                                            clearopbeep((*cap).oap);
                                        }
                                        break 's_906;
                                    }
                                    100 | 68 => {
                                        if foldManualAllowed(false_0 != 0) != 0 {
                                            if VIsual_active.get() {
                                                nv_operator(cap);
                                            } else {
                                                deleteFold(
                                                    curwin.get(),
                                                    (*curwin.get()).w_cursor.lnum,
                                                    (*curwin.get()).w_cursor.lnum,
                                                    (nchar == 'D' as c_int) as c_int,
                                                    false_0 != 0,
                                                );
                                            }
                                        }
                                        break 's_906;
                                    }
                                    69 => {
                                        if foldmethodIsManual(curwin.get()) {
                                            clearFolding(curwin.get());
                                            changed_window_setting(curwin.get());
                                        } else if foldmethodIsMarker(curwin.get()) {
                                            deleteFold(
                                                curwin.get(),
                                                1 as linenr_T,
                                                (*curbuf.get()).b_ml.ml_line_count,
                                                true_0,
                                                false_0 != 0,
                                            );
                                        } else {
                                            emsg(
                                                gettext(
                                                    b"E352: Cannot erase folds with current 'foldmethod'\0"
                                                        .as_ptr() as *const c_char,
                                                ),
                                            );
                                        }
                                        break 's_906;
                                    }
                                    110 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
                                        break 's_906;
                                    }
                                    78 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    105 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen =
                                            ((*curwin.get()).w_onebuf_opt.wo_fen == 0) as c_int;
                                        break 's_906;
                                    }
                                    97 => {
                                        if hasFolding(
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            ::core::ptr::null_mut::<linenr_T>(),
                                        ) {
                                            openFold((*curwin.get()).w_cursor, (*cap).count1);
                                        } else {
                                            closeFold((*curwin.get()).w_cursor, (*cap).count1);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        }
                                        break 's_906;
                                    }
                                    65 => {
                                        if hasFolding(
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            ::core::ptr::null_mut::<linenr_T>(),
                                        ) {
                                            openFoldRecurse((*curwin.get()).w_cursor);
                                        } else {
                                            closeFoldRecurse((*curwin.get()).w_cursor);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        }
                                        break 's_906;
                                    }
                                    111 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            openFold((*curwin.get()).w_cursor, (*cap).count1);
                                        }
                                        break 's_906;
                                    }
                                    79 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            openFoldRecurse((*curwin.get()).w_cursor);
                                        }
                                        break 's_906;
                                    }
                                    99 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            closeFold((*curwin.get()).w_cursor, (*cap).count1);
                                        }
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    67 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            closeFoldRecurse((*curwin.get()).w_cursor);
                                        }
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    118 => {
                                        foldOpenCursor();
                                        break 's_906;
                                    }
                                    120 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        (*curwin.get()).w_foldinvalid = true_0 != 0;
                                        newFoldLevel();
                                        foldOpenCursor();
                                        break 's_906;
                                    }
                                    88 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        (*curwin.get()).w_foldinvalid = true_0 != 0;
                                        old_fdl = -1 as c_int;
                                        break 's_906;
                                    }
                                    109 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_fdl > 0 as OptInt {
                                            (*curwin.get()).w_onebuf_opt.wo_fdl -=
                                                (*cap).count1 as OptInt;
                                            (*curwin.get()).w_onebuf_opt.wo_fdl = if (*curwin.get())
                                                .w_onebuf_opt
                                                .wo_fdl
                                                > 0 as OptInt
                                            {
                                                (*curwin.get()).w_onebuf_opt.wo_fdl
                                            } else {
                                                0 as OptInt
                                            };
                                        }
                                        old_fdl = -1 as c_int;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    77 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl = 0 as OptInt;
                                        old_fdl = -1 as c_int;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    114 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl +=
                                            (*cap).count1 as OptInt;
                                        let mut d: c_int = getDeepestNesting(curwin.get());
                                        (*curwin.get()).w_onebuf_opt.wo_fdl =
                                            if (*curwin.get()).w_onebuf_opt.wo_fdl < d as OptInt {
                                                (*curwin.get()).w_onebuf_opt.wo_fdl
                                            } else {
                                                d as OptInt
                                            };
                                        break 's_906;
                                    }
                                    82 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl =
                                            getDeepestNesting(curwin.get()) as OptInt;
                                        old_fdl = -1 as c_int;
                                        break 's_906;
                                    }
                                    106 | 107 => {
                                        if foldMoveTo(
                                            true_0 != 0,
                                            if nchar == 'j' as c_int {
                                                FORWARD as c_int
                                            } else {
                                                BACKWARD as c_int
                                            },
                                            (*cap).count1,
                                        ) == false_0
                                        {
                                            clearopbeep((*cap).oap);
                                        }
                                        break 's_906;
                                    }
                                    117 | 103 | 119 | 71 | 87 => {
                                        if nv_zg_zw(cap, nchar) == FAIL {
                                            return;
                                        }
                                        break 's_906;
                                    }
                                    61 => {
                                        if !checkclearop((*cap).oap) {
                                            spell_suggest((*cap).count0);
                                        }
                                        break 's_906;
                                    }
                                    _ => {
                                        clearopbeep((*cap).oap);
                                        break 's_906;
                                    }
                                }
                                scroll_cursor_halfway(curwin.get(), true_0 != 0, false_0 != 0);
                                redraw_later(curwin.get(), UPD_VALID as c_int);
                                set_fraction(curwin.get());
                                break 's_906;
                            }
                            beginline(BL_WHITE as c_int | BL_FIX as c_int);
                            break 'c_53178;
                        }
                        beginline(BL_WHITE as c_int | BL_FIX as c_int);
                        break 'c_53195;
                    }
                    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                        set_leftcol(if (*cap).count1 > (*curwin.get()).w_leftcol {
                            0 as colnr_T
                        } else {
                            (*curwin.get()).w_leftcol - (*cap).count1
                        });
                    }
                    break 's_906;
                }
                if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                    set_leftcol((*curwin.get()).w_leftcol + (*cap).count1);
                }
                break 's_906;
            }
            scroll_cursor_bot(curwin.get(), 0 as c_int, true_0 != 0);
            redraw_later(curwin.get(), UPD_VALID as c_int);
            set_fraction(curwin.get());
            break 's_906;
        }
        scroll_cursor_top(curwin.get(), 0 as c_int, true_0);
        redraw_later(curwin.get(), UPD_VALID as c_int);
        set_fraction(curwin.get());
    }
    if old_fen != (*curwin.get()).w_onebuf_opt.wo_fen {
        if foldmethodIsDiff(curwin.get()) as c_int != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0
        {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if wp != curwin.get()
                    && foldmethodIsDiff(wp) as c_int != 0
                    && (*wp).w_onebuf_opt.wo_scb != 0
                {
                    (*wp).w_onebuf_opt.wo_fen = (*curwin.get()).w_onebuf_opt.wo_fen;
                    changed_window_setting(wp);
                }
                wp = (*wp).w_next;
            }
        }
        changed_window_setting(curwin.get());
    }
    if old_fdl as OptInt != (*curwin.get()).w_onebuf_opt.wo_fdl {
        newFoldLevel();
    }
}

pub(crate) unsafe extern "C" fn nv_Zet(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    match (*cap).nchar {
        90 => {
            do_cmdline_cmd(b"x\0".as_ptr() as *const c_char);
        }
        81 => {
            do_cmdline_cmd(b"q!\0".as_ptr() as *const c_char);
        }
        82 => {
            if (*cap).count0 >= 1 as c_int {
                do_cmdline_cmd(b"restart +qall!\0".as_ptr() as *const c_char);
            } else {
                do_cmdline_cmd(b"restart\0".as_ptr() as *const c_char);
            }
        }
        _ => {
            clearopbeep((*cap).oap);
        }
    };
}

pub(crate) unsafe extern "C" fn nv_halfpage(mut cap: *mut cmdarg_T) {
    if !checkclearop((*cap).oap) {
        pagescroll(
            (if (*cap).cmdchar == Ctrl_D {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            }) as Direction,
            (*cap).count0,
            true_0 != 0,
        );
    }
}
