//! The arrow keys, and the two that start a selection.
//!
//! Every one of these is a Normal-mode motion plus the Insert-mode
//! bookkeeping around it: `start_arrow` to close the undo block, an
//! `undisplay_dollar`, a check against 'whichwrap' for whether the motion
//! may leave the line, and -- for the shifted forms -- `ins_start_select`,
//! which turns the key into a Select-mode selection when 'keymodel'
//! contains "startsel".  `ins_up`/`ins_down` also have to preserve the
//! column the user wants, which is `Insstart.col` for CTRL-Home and the
//! current virtual column otherwise.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_start_select(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        if !km_startsel.get() {
            return false_0 != 0;
        }
        's_78: {
            match c {
                K_KHOME | K_KEND | K_PAGEUP | K_KPAGEUP | K_PAGEDOWN | K_KPAGEDOWN => {
                    if mod_mask.get() & MOD_MASK_SHIFT == 0 {
                        break 's_78;
                    }
                }
                K_S_LEFT | K_S_RIGHT | K_S_UP | K_S_DOWN | K_S_END | K_S_HOME => {}
                _ => {
                    break 's_78;
                }
            }
            start_selection();
            stuffcharReadbuff(Ctrl_O);
            if mod_mask.get() != 0 {
                let buf: [::core::ffi::c_char; 4] = [
                    K_SPECIAL as ::core::ffi::c_char,
                    KS_MODIFIER as ::core::ffi::c_char,
                    mod_mask.get() as uint8_t as ::core::ffi::c_char,
                    NUL as ::core::ffi::c_char,
                ];
                stuffReadbuffLen(&raw const buf as *const ::core::ffi::c_char, 3 as ptrdiff_t);
            }
            stuffcharReadbuff(c);
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn ins_left() {
    unsafe {
        let end_change: bool =
            dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        undisplay_dollar();
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if oneleft() == OK {
            start_arrow_with_change(&raw mut tpos, end_change);
            if !end_change {
                AppendCharToRedobuff(K_LEFT);
            }
            if revins_scol.get() != -1 as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.col >= revins_scol.get()
            {
                (*revins_legal.ptr()) += 1;
            }
            (*revins_chars.ptr()) += 1;
        } else if !vim_strchr(p_ww.get(), '[' as ::core::ffi::c_int).is_null()
            && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
        {
            start_arrow(&raw mut tpos);
            (*curwin.get()).w_cursor.lnum -= 1;
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
            (*curwin.get()).w_set_curswant = true_0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

pub(crate) unsafe extern "C" fn ins_home(mut c: ::core::ffi::c_int) {
    unsafe {
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        undisplay_dollar();
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if c == -(253 as ::core::ffi::c_int
            + ((KE_C_HOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        }
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
        start_arrow(&raw mut tpos);
    }
}

pub(crate) unsafe extern "C" fn ins_end(mut c: ::core::ffi::c_int) {
    unsafe {
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        undisplay_dollar();
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if c == -(253 as ::core::ffi::c_int
            + ((KE_C_END as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        }
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
        start_arrow(&raw mut tpos);
    }
}

pub(crate) unsafe extern "C" fn ins_s_left() {
    unsafe {
        let end_change: bool =
            dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        undisplay_dollar();
        if (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            || (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        {
            start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
            if !end_change {
                AppendCharToRedobuff(K_S_LEFT);
            }
            bck_word(1 as ::core::ffi::c_int, false_0 != 0, false_0 != 0);
            (*curwin.get()).w_set_curswant = true_0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

pub(crate) unsafe extern "C" fn ins_right() {
    unsafe {
        let end_change: bool =
            dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        undisplay_dollar();
        if gchar_cursor() != NUL || virtual_active(curwin.get()) as ::core::ffi::c_int != 0 {
            start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
            if !end_change {
                AppendCharToRedobuff(K_RIGHT);
            }
            (*curwin.get()).w_set_curswant = true_0;
            if virtual_active(curwin.get()) {
                oneright();
            } else {
                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
            }
            (*revins_legal.ptr()) += 1;
            if revins_chars.get() != 0 {
                (*revins_chars.ptr()) -= 1;
            }
        } else if !vim_strchr(p_ww.get(), ']' as ::core::ffi::c_int).is_null()
            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
        {
            start_arrow(&raw mut (*curwin.get()).w_cursor);
            (*curwin.get()).w_set_curswant = true_0;
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

pub(crate) unsafe extern "C" fn ins_s_right() {
    unsafe {
        let end_change: bool =
            dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        undisplay_dollar();
        if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            || gchar_cursor() != NUL
        {
            start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
            if !end_change {
                AppendCharToRedobuff(K_S_RIGHT);
            }
            fwd_word(1 as ::core::ffi::c_int, false_0 != 0, false);
            (*curwin.get()).w_set_curswant = true_0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

pub(crate) unsafe extern "C" fn ins_up(mut startcol: bool) {
    unsafe {
        let mut old_topline: linenr_T = (*curwin.get()).w_topline;
        let mut old_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
        undisplay_dollar();
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if cursor_up(1 as linenr_T, true_0 != 0) == OK {
            if startcol {
                coladvance(curwin.get(), getvcol_nolist(Insstart.ptr()));
            }
            if old_topline != (*curwin.get()).w_topline || old_topfill != (*curwin.get()).w_topfill
            {
                redraw_later(curwin.get(), UPD_VALID);
            }
            start_arrow(&raw mut tpos);
            can_cindent.set(true_0 != 0);
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        };
    }
}

pub(crate) unsafe extern "C" fn ins_pageup() {
    unsafe {
        undisplay_dollar();
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            if !(*first_tabpage.get()).tp_next.is_null() {
                start_arrow(&raw mut (*curwin.get()).w_cursor);
                goto_tabpage(-1 as ::core::ffi::c_int);
            }
            return;
        }
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if pagescroll(BACKWARD, 1 as ::core::ffi::c_int, false_0 != 0) == OK {
            start_arrow(&raw mut tpos);
            can_cindent.set(true_0 != 0);
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        };
    }
}

pub(crate) unsafe extern "C" fn ins_down(mut startcol: bool) {
    unsafe {
        let mut old_topline: linenr_T = (*curwin.get()).w_topline;
        let mut old_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
        undisplay_dollar();
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if cursor_down(1 as ::core::ffi::c_int, true_0 != 0) == OK {
            if startcol {
                coladvance(curwin.get(), getvcol_nolist(Insstart.ptr()));
            }
            if old_topline != (*curwin.get()).w_topline || old_topfill != (*curwin.get()).w_topfill
            {
                redraw_later(curwin.get(), UPD_VALID);
            }
            start_arrow(&raw mut tpos);
            can_cindent.set(true_0 != 0);
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        };
    }
}

pub(crate) unsafe extern "C" fn ins_pagedown() {
    unsafe {
        undisplay_dollar();
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            if !(*first_tabpage.get()).tp_next.is_null() {
                start_arrow(&raw mut (*curwin.get()).w_cursor);
                goto_tabpage(0 as ::core::ffi::c_int);
            }
            return;
        }
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if pagescroll(FORWARD, 1 as ::core::ffi::c_int, false_0 != 0) == OK {
            start_arrow(&raw mut tpos);
            can_cindent.set(true_0 != 0);
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
        };
    }
}
