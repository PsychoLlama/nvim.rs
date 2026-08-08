//! `insert_handle_key` -- what a key means in Insert mode.
//!
//! One switch over every key that is *not* simply inserted: the CTRL-
//! commands, the arrows and their shifted forms, backspace and delete, TAB
//! and CR, the completion keys, and the two dozen that only differ from a
//! plain character under an option ('paste', 'revins', 'digraph',
//! 'keymodel', 'startsel').  Each arm either calls one of this module's
//! `ins_*` functions and asks for the next key, or falls through to the
//! normal-character path at the bottom, which is the only place a byte
//! reaches `insertchar`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn insert_handle_key(mut s: *mut InsertState) -> ::core::ffi::c_int {
    unsafe {
        's_1398: {
            '_normalchar: {
                '_check_pum: {
                    'c_31045: {
                        'c_42507: {
                            'c_31081: {
                                'c_31145: {
                                    'c_35097: {
                                        match (*s).c {
                                            ESC => {
                                                if echeck_abbr(ESC + ABBR_OFF) {
                                                    break 's_1398;
                                                } else {
                                                    break 'c_31045;
                                                }
                                            }
                                            Ctrl_C => {
                                                break 'c_31045;
                                            }
                                            Ctrl_O => {
                                                if ctrl_x_mode_omni() {
                                                    insert_do_complete(s);
                                                    break 's_1398;
                                                } else if echeck_abbr(Ctrl_O + ABBR_OFF) {
                                                    break 's_1398;
                                                } else {
                                                    ins_ctrl_o();
                                                    if get_ve_flags(curwin.get())
                                                        & kOptVeFlagOnemore as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        != 0
                                                    {
                                                        ins_at_eol.set(false_0 != 0);
                                                        (*s).nomove = true_0 != 0;
                                                    }
                                                    (*s).count = 0 as ::core::ffi::c_int;
                                                    return 0 as ::core::ffi::c_int;
                                                }
                                            }
                                            K_INS | K_KINS => {
                                                ins_insert((*s).replaceState);
                                                break 's_1398;
                                            }
                                            K_HELP | K_F1 | K_XF1 => {
                                                stuffcharReadbuff(K_HELP);
                                                return 0 as ::core::ffi::c_int;
                                            }
                                            32 => {
                                                if mod_mask.get() != MOD_MASK_CTRL {
                                                    break '_normalchar;
                                                } else {
                                                    break 'c_42507;
                                                }
                                            }
                                            K_ZERO | NUL | Ctrl_A => {
                                                break 'c_42507;
                                            }
                                            Ctrl_R => {
                                                if ctrl_x_mode_register() as ::core::ffi::c_int != 0
                                                    && !ins_compl_active()
                                                {
                                                    insert_do_complete(s);
                                                    break 's_1398;
                                                } else {
                                                    ins_reg();
                                                    auto_format(false_0 != 0, true_0 != 0);
                                                    (*s).inserted_space = false_0;
                                                    break 's_1398;
                                                }
                                            }
                                            Ctrl_G => {
                                                ins_ctrl_g();
                                                break 's_1398;
                                            }
                                            Ctrl_HAT => {
                                                ins_ctrl_hat();
                                                break 's_1398;
                                            }
                                            Ctrl__ => {
                                                if p_ari.get() == 0 {
                                                    break '_normalchar;
                                                } else {
                                                    ins_ctrl_();
                                                    break 's_1398;
                                                }
                                            }
                                            Ctrl_D => {
                                                if ctrl_x_mode_path_defines() {
                                                    insert_do_complete(s);
                                                    break 's_1398;
                                                } else {
                                                    break 'c_31081;
                                                }
                                            }
                                            Ctrl_T => {
                                                break 'c_31081;
                                            }
                                            K_DEL | K_KDEL => {
                                                ins_del();
                                                auto_format(false_0 != 0, true_0 != 0);
                                                break 's_1398;
                                            }
                                            K_BS | Ctrl_H => {
                                                (*s).did_backspace = ins_bs(
                                                    (*s).c,
                                                    Backspace::Char,
                                                    &raw mut (*s).inserted_space,
                                                );
                                                auto_format(false_0 != 0, true_0 != 0);
                                                if (*s).did_backspace {
                                                    if ins_compl_has_autocomplete()
                                                        as ::core::ffi::c_int
                                                        != 0
                                                        && !char_avail()
                                                        && (*curwin.get()).w_cursor.col
                                                            > 0 as ::core::ffi::c_int
                                                    {
                                                        (*s).c = char_before_cursor();
                                                        if vim_isprintc((*s).c) {
                                                            redraw_later(curwin.get(), UPD_VALID);
                                                            update_screen();
                                                            ui_flush();
                                                            ins_compl_enable_autocomplete();
                                                            insert_do_complete(s);
                                                        }
                                                    }
                                                }
                                                break 's_1398;
                                            }
                                            Ctrl_W => {
                                                if bt_prompt(curbuf.get()) as ::core::ffi::c_int
                                                    != 0
                                                    && mod_mask.get() & MOD_MASK_SHIFT
                                                        == 0 as ::core::ffi::c_int
                                                {
                                                    stuffcharReadbuff(Ctrl_W);
                                                    restart_edit.set('A' as ::core::ffi::c_int);
                                                    (*s).nomove = true_0 != 0;
                                                    (*s).count = 0 as ::core::ffi::c_int;
                                                    return 0 as ::core::ffi::c_int;
                                                }
                                                (*s).did_backspace = ins_bs(
                                                    (*s).c,
                                                    Backspace::Word,
                                                    &raw mut (*s).inserted_space,
                                                );
                                                auto_format(false_0 != 0, true_0 != 0);
                                                if (*s).did_backspace {
                                                    if ins_compl_has_autocomplete()
                                                        as ::core::ffi::c_int
                                                        != 0
                                                        && !char_avail()
                                                        && (*curwin.get()).w_cursor.col
                                                            > 0 as ::core::ffi::c_int
                                                    {
                                                        (*s).c = char_before_cursor();
                                                        if vim_isprintc((*s).c) {
                                                            redraw_later(curwin.get(), UPD_VALID);
                                                            update_screen();
                                                            ui_flush();
                                                            ins_compl_enable_autocomplete();
                                                            insert_do_complete(s);
                                                        }
                                                    }
                                                }
                                                break 's_1398;
                                            }
                                            Ctrl_U => {
                                                if ctrl_x_mode_function() {
                                                    insert_do_complete(s);
                                                } else {
                                                    (*s).did_backspace = ins_bs(
                                                        (*s).c,
                                                        Backspace::Line,
                                                        &raw mut (*s).inserted_space,
                                                    );
                                                    auto_format(false_0 != 0, true_0 != 0);
                                                    (*s).inserted_space = false_0;
                                                    if (*s).did_backspace {
                                                        if ins_compl_has_autocomplete()
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && !char_avail()
                                                            && (*curwin.get()).w_cursor.col
                                                                > 0 as ::core::ffi::c_int
                                                        {
                                                            (*s).c = char_before_cursor();
                                                            if vim_isprintc((*s).c) {
                                                                redraw_later(
                                                                    curwin.get(),
                                                                    UPD_VALID,
                                                                );
                                                                update_screen();
                                                                ui_flush();
                                                                ins_compl_enable_autocomplete();
                                                                insert_do_complete(s);
                                                            }
                                                        }
                                                    }
                                                }
                                                break 's_1398;
                                            }
                                            K_LEFTMOUSE | K_LEFTMOUSE_NM | K_LEFTDRAG
                                            | K_LEFTRELEASE | K_LEFTRELEASE_NM | K_MOUSEMOVE
                                            | K_MIDDLEMOUSE | K_MIDDLEDRAG | K_MIDDLERELEASE
                                            | K_RIGHTMOUSE | K_RIGHTDRAG | K_RIGHTRELEASE
                                            | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE
                                            | K_X2DRAG | K_X2RELEASE => {
                                                ins_mouse((*s).c);
                                                break 's_1398;
                                            }
                                            K_MOUSEDOWN => {
                                                ins_mousescroll(MSCR_DOWN);
                                                break 's_1398;
                                            }
                                            K_MOUSEUP => {
                                                ins_mousescroll(MSCR_UP);
                                                break 's_1398;
                                            }
                                            K_MOUSELEFT => {
                                                ins_mousescroll(MSCR_LEFT);
                                                break 's_1398;
                                            }
                                            K_MOUSERIGHT => {
                                                ins_mousescroll(MSCR_RIGHT);
                                                break 's_1398;
                                            }
                                            K_SELECT | -13821 => {
                                                break 's_1398;
                                            }
                                            K_PASTE_START => {
                                                paste_repeat(1 as ::core::ffi::c_int);
                                                break '_check_pum;
                                            }
                                            -26365 => {
                                                state_handle_k_event();
                                                if dont_sync_undo.get() as ::core::ffi::c_int
                                                    == kTrue as ::core::ffi::c_int
                                                {
                                                    dont_sync_undo.set(kNone);
                                                }
                                                break '_check_pum;
                                            }
                                            K_COMMAND => {
                                                do_cmdline(
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                Some(
                                                    getcmdkeycmd
                                                        as unsafe extern "C" fn(
                                                            ::core::ffi::c_int,
                                                            *mut ::core::ffi::c_void,
                                                            ::core::ffi::c_int,
                                                            bool,
                                                        ) -> *mut ::core::ffi::c_char,
                                                ),
                                                NULL,
                                                0 as ::core::ffi::c_int,
                                            );
                                                break '_check_pum;
                                            }
                                            K_LUA => {
                                                map_execute_lua(false_0 != 0, false_0 != 0);
                                                break '_check_pum;
                                            }
                                            K_HOME | K_KHOME | K_S_HOME | -22525 => {
                                                ins_home((*s).c);
                                                break 's_1398;
                                            }
                                            K_END | K_KEND | K_S_END | -22781 => {
                                                ins_end((*s).c);
                                                break 's_1398;
                                            }
                                            K_LEFT => {
                                                if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL)
                                                    != 0
                                                {
                                                    ins_s_left();
                                                } else {
                                                    ins_left();
                                                }
                                                break 's_1398;
                                            }
                                            K_S_LEFT | -22013 => {
                                                ins_s_left();
                                                break 's_1398;
                                            }
                                            K_RIGHT => {
                                                if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL)
                                                    != 0
                                                {
                                                    ins_s_right();
                                                } else {
                                                    ins_right();
                                                }
                                                break 's_1398;
                                            }
                                            K_S_RIGHT | -22269 => {
                                                ins_s_right();
                                                break 's_1398;
                                            }
                                            K_UP => {
                                                if pum_visible() {
                                                    insert_do_complete(s);
                                                } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                                                    ins_page(true);
                                                } else {
                                                    ins_updown(true, false);
                                                }
                                                break 's_1398;
                                            }
                                            K_S_UP | K_PAGEUP | K_KPAGEUP => {
                                                if pum_visible() {
                                                    insert_do_complete(s);
                                                } else {
                                                    ins_page(true);
                                                }
                                                break 's_1398;
                                            }
                                            K_DOWN => {
                                                if pum_visible() {
                                                    insert_do_complete(s);
                                                } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                                                    ins_page(false);
                                                } else {
                                                    ins_updown(false, false);
                                                }
                                                break 's_1398;
                                            }
                                            K_S_DOWN | K_PAGEDOWN | K_KPAGEDOWN => {
                                                if pum_visible() {
                                                    insert_do_complete(s);
                                                } else {
                                                    ins_page(false);
                                                }
                                                break 's_1398;
                                            }
                                            K_S_TAB => {
                                                (*s).c = TAB;
                                                break 'c_31145;
                                            }
                                            TAB => {
                                                break 'c_31145;
                                            }
                                            K_KENTER => {
                                                (*s).c = CAR;
                                                break 'c_35097;
                                            }
                                            CAR | NL => {
                                                break 'c_35097;
                                            }
                                            Ctrl_K => {
                                                if ctrl_x_mode_dictionary() {
                                                    if check_compl_option(true_0 != 0) {
                                                        insert_do_complete(s);
                                                    }
                                                    break 's_1398;
                                                } else {
                                                    (*s).c = ins_digraph();
                                                    if (*s).c == NUL {
                                                        break 's_1398;
                                                    } else {
                                                        break '_normalchar;
                                                    }
                                                }
                                            }
                                            Ctrl_X => {
                                                ins_ctrl_x();
                                                break 's_1398;
                                            }
                                            Ctrl_RSB => {
                                                if !ctrl_x_mode_tags() {
                                                    break '_normalchar;
                                                } else {
                                                    insert_do_complete(s);
                                                    break 's_1398;
                                                }
                                            }
                                            Ctrl_F => {
                                                if !ctrl_x_mode_files() {
                                                    break '_normalchar;
                                                } else {
                                                    insert_do_complete(s);
                                                    break 's_1398;
                                                }
                                            }
                                            115 | Ctrl_S => {
                                                if !ctrl_x_mode_spell() {
                                                    break '_normalchar;
                                                } else {
                                                    insert_do_complete(s);
                                                    break 's_1398;
                                                }
                                            }
                                            Ctrl_L => {
                                                if !ctrl_x_mode_whole_line() {
                                                    break '_normalchar;
                                                }
                                            }
                                            Ctrl_P | Ctrl_N => {}
                                            Ctrl_Y | Ctrl_E => {
                                                (*s).c = ins_ctrl_ey((*s).c);
                                                break 's_1398;
                                            }
                                            Ctrl_Z | _ => {
                                                break '_normalchar;
                                            }
                                        }
                                        if *(*curbuf.get()).b_p_cpt as ::core::ffi::c_int == NUL
                                            && (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                                                || ctrl_x_mode_whole_line() as ::core::ffi::c_int
                                                    != 0)
                                            && !compl_status_local()
                                        {
                                            break '_normalchar;
                                        } else {
                                            insert_do_complete(s);
                                            break 's_1398;
                                        }
                                    }
                                    if bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
                                        && (*s).c == CAR
                                    {
                                        if (*curwin.get()).w_llist_ref.is_null() {
                                            do_cmdline_cmd(
                                                b".cc\0".as_ptr() as *const ::core::ffi::c_char
                                            );
                                        } else {
                                            do_cmdline_cmd(
                                                b".ll\0".as_ptr() as *const ::core::ffi::c_char
                                            );
                                        }
                                        break 's_1398;
                                    } else {
                                        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                            cmdwin_result.set(CAR);
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        if mod_mask.get() & MOD_MASK_SHIFT
                                            == 0 as ::core::ffi::c_int
                                            && bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
                                        {
                                            prompt_invoke_callback();
                                            if !bt_prompt(curbuf.get()) {
                                                return 0 as ::core::ffi::c_int;
                                            }
                                            break 's_1398;
                                        } else {
                                            if !ins_eol((*s).c) {
                                                return 0 as ::core::ffi::c_int;
                                            }
                                            auto_format(false_0 != 0, false_0 != 0);
                                            (*s).inserted_space = false_0;
                                            break 's_1398;
                                        }
                                    }
                                }
                                if ctrl_x_mode_path_patterns() {
                                    insert_do_complete(s);
                                    break 's_1398;
                                } else {
                                    (*s).inserted_space = false_0;
                                    if ins_tab() {
                                        break '_normalchar;
                                    } else {
                                        auto_format(false_0 != 0, true_0 != 0);
                                        break 's_1398;
                                    }
                                }
                            }
                            if (*s).c == Ctrl_T
                                && ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0
                            {
                                if check_compl_option(false_0 != 0) {
                                    insert_do_complete(s);
                                }
                                break 's_1398;
                            } else {
                                ins_shift((*s).c, (*s).lastc);
                                auto_format(false_0 != 0, true_0 != 0);
                                (*s).inserted_space = false_0;
                                break 's_1398;
                            }
                        }
                        if stuff_inserted(
                            NUL,
                            1 as ::core::ffi::c_int,
                            ((*s).c == Ctrl_A) as ::core::ffi::c_int,
                        ) == FAIL
                            && (*s).c != Ctrl_A
                        {
                            return 0 as ::core::ffi::c_int;
                        }
                        (*s).inserted_space = false_0;
                        break 's_1398;
                    }
                    if (*s).c == Ctrl_C && cmdwin_type.get() != 0 as ::core::ffi::c_int {
                        cmdwin_result.set(
                            -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
                        );
                        got_int.set(false_0 != 0);
                        (*s).nomove = true_0 != 0;
                        return 0 as ::core::ffi::c_int;
                    }
                    if (*s).c == Ctrl_C && bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 {
                        if invoke_prompt_interrupt() {
                            if !bt_prompt(curbuf.get()) {
                                return 0 as ::core::ffi::c_int;
                            }
                            break 's_1398;
                        }
                    }
                    return 0 as ::core::ffi::c_int;
                }
                if (*pum_want.ptr()).active {
                    if pum_visible() {
                        edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                        insert_do_complete(s);
                        if (*pum_want.ptr()).finish {
                            ins_compl_prep(Ctrl_Y);
                        }
                    }
                    (*pum_want.ptr()).active = false_0 != 0;
                }
                if (*curbuf.get()).b_u_synced {
                    ins_need_undo.set(true_0 != 0);
                }
                break 's_1398;
            }
            if p_paste.get() == 0 {
                let mut str: *mut ::core::ffi::c_char = do_insert_char_pre((*s).c);
                if !str.is_null() {
                    if *str as ::core::ffi::c_int != NUL && stop_arrow() != FAIL {
                        let mut p: *mut ::core::ffi::c_char = str;
                        while *p as ::core::ffi::c_int != NUL {
                            (*s).c = utf_ptr2char(p);
                            if (*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL {
                                ins_eol((*s).c);
                            } else {
                                ins_char((*s).c);
                            }
                            p = p.offset(utfc_ptr2len(p) as isize);
                        }
                        AppendToRedobuffLit(str, -1 as ::core::ffi::c_int);
                    }
                    xfree(str as *mut ::core::ffi::c_void);
                    (*s).c = NUL;
                }
                if (*s).c == NUL {
                    break 's_1398;
                }
            }
            ins_try_si((*s).c);
            if (*s).c == ' ' as ::core::ffi::c_int {
                (*s).inserted_space = true_0;
                if inindent(0 as ::core::ffi::c_int) {
                    can_cindent.set(false_0 != 0);
                }
                if Insstart_blank_vcol.get() == MAXCOL as ::core::ffi::c_int
                    && (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
                {
                    Insstart_blank_vcol.set(get_nolist_virtcol());
                }
            }
            if vim_iswordc((*s).c) as ::core::ffi::c_int != 0
                || !echeck_abbr(if (*s).c >= 0x100 as ::core::ffi::c_int {
                    (*s).c + ABBR_OFF
                } else {
                    (*s).c
                }) && (*s).c != Ctrl_RSB
            {
                insert_special((*s).c, false_0, false_0);
                (*revins_legal.ptr()) += 1;
                (*revins_chars.ptr()) += 1;
            }
            auto_format(false_0 != 0, true_0 != 0);
            foldOpenCursor();
            if ins_compl_has_autocomplete() as ::core::ffi::c_int != 0
                && !char_avail()
                && vim_isprintc((*s).c) as ::core::ffi::c_int != 0
            {
                redraw_later(curwin.get(), UPD_VALID);
                update_screen();
                ui_flush();
                ins_compl_enable_autocomplete();
                insert_do_complete(s);
            }
        }
        insert_handle_key_post(s);
        return 1 as ::core::ffi::c_int;
    }
}
