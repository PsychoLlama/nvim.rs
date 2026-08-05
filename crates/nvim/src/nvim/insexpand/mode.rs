//! Which completion is running: the CTRL-X modes and the state queries.
//!
//! `ctrl_x_mode` is the mode the user selected with a CTRL-X chord, and
//! [`vim_is_ctrl_x_key`] decides whether the key just typed still belongs to
//! it.  [`set_ctrl_x_mode`] is the chord dispatch itself.  The rest are the
//! one-line queries the editor asks about a completion in progress.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ins_ctrl_x() {
    unsafe {
        if !ctrl_x_mode_cmdline() {
            if compl_cont_status.get() & CONT_N_ADDS != 0 {
                (*compl_cont_status.ptr()) |= CONT_INTRPT;
            } else {
                compl_cont_status.set(0 as ::core::ffi::c_int);
            }
            ctrl_x_mode.set(CTRL_X_NOT_DEFINED_YET);
            edit_submode.set(gettext(
                (*ctrl_x_msgs.ptr())[(ctrl_x_mode.get() & !(0x100 as ::core::ffi::c_int)) as usize],
            ));
            edit_submode_pre.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            redraw_mode.set(true_0 != 0);
        } else {
            ctrl_x_mode.set(CTRL_X_CMDLINE_CTRL_X);
        }
        may_trigger_modechanged();
    }
}

pub unsafe extern "C" fn ctrl_x_mode_none() -> bool {
    return ctrl_x_mode.get() == 0 as ::core::ffi::c_int;
}

pub unsafe extern "C" fn ctrl_x_mode_normal() -> bool {
    return ctrl_x_mode.get() == CTRL_X_NORMAL;
}

pub unsafe extern "C" fn ctrl_x_mode_scroll() -> bool {
    return ctrl_x_mode.get() == CTRL_X_SCROLL;
}

pub unsafe extern "C" fn ctrl_x_mode_whole_line() -> bool {
    return ctrl_x_mode.get() == CTRL_X_WHOLE_LINE;
}

pub unsafe extern "C" fn ctrl_x_mode_files() -> bool {
    return ctrl_x_mode.get() == CTRL_X_FILES;
}

pub unsafe extern "C" fn ctrl_x_mode_tags() -> bool {
    return ctrl_x_mode.get() == CTRL_X_TAGS;
}

pub unsafe extern "C" fn ctrl_x_mode_path_patterns() -> bool {
    return ctrl_x_mode.get() == CTRL_X_PATH_PATTERNS;
}

pub unsafe extern "C" fn ctrl_x_mode_path_defines() -> bool {
    return ctrl_x_mode.get() == CTRL_X_PATH_DEFINES;
}

pub unsafe extern "C" fn ctrl_x_mode_dictionary() -> bool {
    return ctrl_x_mode.get() == CTRL_X_DICTIONARY;
}

pub unsafe extern "C" fn ctrl_x_mode_thesaurus() -> bool {
    return ctrl_x_mode.get() == CTRL_X_THESAURUS;
}

pub unsafe extern "C" fn ctrl_x_mode_cmdline() -> bool {
    return ctrl_x_mode.get() == CTRL_X_CMDLINE || ctrl_x_mode.get() == CTRL_X_CMDLINE_CTRL_X;
}

pub unsafe extern "C" fn ctrl_x_mode_function() -> bool {
    return ctrl_x_mode.get() == CTRL_X_FUNCTION;
}

pub unsafe extern "C" fn ctrl_x_mode_omni() -> bool {
    return ctrl_x_mode.get() == CTRL_X_OMNI;
}

pub unsafe extern "C" fn ctrl_x_mode_spell() -> bool {
    return ctrl_x_mode.get() == CTRL_X_SPELL;
}

pub(crate) unsafe extern "C" fn ctrl_x_mode_eval() -> bool {
    return ctrl_x_mode.get() == CTRL_X_EVAL;
}

pub unsafe extern "C" fn ctrl_x_mode_line_or_eval() -> bool {
    return ctrl_x_mode.get() == CTRL_X_WHOLE_LINE || ctrl_x_mode.get() == CTRL_X_EVAL;
}

pub unsafe extern "C" fn ctrl_x_mode_register() -> bool {
    return ctrl_x_mode.get() == CTRL_X_REGISTER;
}

pub unsafe extern "C" fn ctrl_x_mode_not_default() -> bool {
    return ctrl_x_mode.get() != CTRL_X_NORMAL;
}

pub unsafe extern "C" fn ctrl_x_mode_not_defined_yet() -> bool {
    return ctrl_x_mode.get() == CTRL_X_NOT_DEFINED_YET;
}

pub unsafe extern "C" fn compl_status_adding() -> bool {
    return compl_cont_status.get() & CONT_ADDING != 0;
}

pub unsafe extern "C" fn compl_status_sol() -> bool {
    return compl_cont_status.get() & CONT_SOL != 0;
}

pub unsafe extern "C" fn compl_status_local() -> bool {
    return compl_cont_status.get() & CONT_LOCAL != 0;
}

pub unsafe extern "C" fn compl_status_clear() {
    compl_cont_status.set(0 as ::core::ffi::c_int);
}

pub(crate) unsafe extern "C" fn compl_dir_forward() -> bool {
    return compl_direction.get() as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int;
}

pub(crate) unsafe extern "C" fn compl_shows_dir_forward() -> bool {
    return compl_shows_dir.get() as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int;
}

pub(crate) unsafe extern "C" fn compl_shows_dir_backward() -> bool {
    return compl_shows_dir.get() as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int;
}

pub unsafe extern "C" fn check_compl_option(mut dict_opt: bool) -> bool {
    unsafe {
        if if dict_opt as ::core::ffi::c_int != 0 {
            (*(*curbuf.get()).b_p_dict as ::core::ffi::c_int == NUL
                && *p_dict.get() as ::core::ffi::c_int == NUL
                && (*curwin.get()).w_onebuf_opt.wo_spell == 0) as ::core::ffi::c_int
        } else {
            (*(*curbuf.get()).b_p_tsr as ::core::ffi::c_int == NUL
                && *p_tsr.get() as ::core::ffi::c_int == NUL
                && *(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int == NUL
                && *p_tsrfu.get() as ::core::ffi::c_int == NUL) as ::core::ffi::c_int
        } != 0
        {
            ctrl_x_mode.set(CTRL_X_NORMAL);
            edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            emsg(if dict_opt as ::core::ffi::c_int != 0 {
                gettext(b"'dictionary' option is empty\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                gettext(b"'thesaurus' option is empty\0".as_ptr() as *const ::core::ffi::c_char)
            });
            if emsg_silent.get() == 0 as ::core::ffi::c_int && !in_assert_fails.get() {
                vim_beep(kOptBoFlagComplete as ::core::ffi::c_int as ::core::ffi::c_uint);
                setcursor();
                msg_delay(2004 as uint64_t, false_0 != 0);
            }
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn vim_is_ctrl_x_key(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        if c == Ctrl_R && ctrl_x_mode.get() != CTRL_X_REGISTER {
            return true_0 != 0;
        }
        if ins_compl_pum_key(c) {
            return true_0 != 0;
        }
        match ctrl_x_mode.get() {
            0 => return c == Ctrl_N || c == Ctrl_P || c == Ctrl_X,
            1 | 17 => {
                return c == Ctrl_X
                    || c == Ctrl_Y
                    || c == Ctrl_E
                    || c == Ctrl_L
                    || c == Ctrl_F
                    || c == Ctrl_RSB
                    || c == Ctrl_I
                    || c == Ctrl_D
                    || c == Ctrl_P
                    || c == Ctrl_N
                    || c == Ctrl_T
                    || c == Ctrl_V
                    || c == Ctrl_Q
                    || c == Ctrl_U
                    || c == Ctrl_O
                    || c == Ctrl_S
                    || c == Ctrl_K
                    || c == 's' as ::core::ffi::c_int
                    || c == Ctrl_Z
                    || c == Ctrl_R;
            }
            2 => return c == Ctrl_Y || c == Ctrl_E,
            3 => return c == Ctrl_L || c == Ctrl_P || c == Ctrl_N,
            4 => return c == Ctrl_F || c == Ctrl_P || c == Ctrl_N,
            265 => return c == Ctrl_K || c == Ctrl_P || c == Ctrl_N,
            266 => return c == Ctrl_T || c == Ctrl_P || c == Ctrl_N,
            261 => return c == Ctrl_RSB || c == Ctrl_P || c == Ctrl_N,
            262 => return c == Ctrl_P || c == Ctrl_N,
            263 => return c == Ctrl_D || c == Ctrl_P || c == Ctrl_N,
            11 => {
                return c == Ctrl_V || c == Ctrl_Q || c == Ctrl_P || c == Ctrl_N || c == Ctrl_X;
            }
            12 => return c == Ctrl_U || c == Ctrl_P || c == Ctrl_N,
            13 => return c == Ctrl_O || c == Ctrl_P || c == Ctrl_N,
            14 => return c == Ctrl_S || c == Ctrl_P || c == Ctrl_N,
            16 => return c == Ctrl_P || c == Ctrl_N,
            18 => return c == Ctrl_P || c == Ctrl_N,
            19 => return c == Ctrl_R || c == Ctrl_P || c == Ctrl_N,
            _ => {}
        }
        internal_error(b"vim_is_ctrl_x_key()\0".as_ptr() as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn match_at_original_text(match_0: *const compl_T) -> bool {
    unsafe {
        return (*match_0).cp_flags & CP_ORIGINAL_TEXT != 0;
    }
}

pub(crate) unsafe extern "C" fn is_first_match(match_0: *const compl_T) -> bool {
    return match_0 == compl_first_match.get() as *const compl_T;
}

pub unsafe extern "C" fn ins_compl_accept_char(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        if compl_autocomplete.get() as ::core::ffi::c_int != 0
            && compl_from_nonkeyword.get() as ::core::ffi::c_int != 0
        {
            return false_0 != 0;
        }
        if ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0 {
            return vim_isIDc(c);
        }
        match ctrl_x_mode.get() {
            4 => return vim_isfilec(c) as ::core::ffi::c_int != 0 && !vim_ispathsep(c),
            11 | 17 | 13 => {
                return vim_isprintc(c) as ::core::ffi::c_int != 0 && !ascii_iswhite(c);
            }
            3 => return vim_isprintc(c),
            _ => {}
        }
        return vim_iswordc(c);
    }
}

pub(crate) unsafe extern "C" fn cot_fuzzy() -> bool {
    unsafe {
        return get_cot_flags() & kOptCotFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint
            && !ctrl_x_mode_thesaurus();
    }
}

pub(crate) unsafe extern "C" fn is_nearest_active() -> bool {
    unsafe {
        return (compl_autocomplete.get() as ::core::ffi::c_int != 0
            || get_cot_flags() & kOptCotFlagNearest as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0)
            && !cot_fuzzy();
    }
}

pub unsafe extern "C" fn ins_compl_is_match_selected() -> bool {
    unsafe {
        return !(*compl_shown_match.ptr()).is_null() && !is_first_match(compl_shown_match.get());
    }
}

pub unsafe extern "C" fn ins_compl_preinsert_longest() -> bool {
    unsafe {
        return compl_autocomplete.get() as ::core::ffi::c_int != 0
            && get_cot_flags()
                & (kOptCotFlagLongest as ::core::ffi::c_int
                    | kOptCotFlagPreinsert as ::core::ffi::c_int
                    | kOptCotFlagFuzzy as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                == kOptCotFlagLongest as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
}

pub unsafe extern "C" fn ins_compl_leader() -> *mut ::core::ffi::c_char {
    unsafe {
        return if !(*compl_leader.ptr()).data.is_null() {
            (*compl_leader.ptr()).data
        } else {
            (*compl_orig_text.ptr()).data
        };
    }
}

pub(crate) unsafe extern "C" fn ins_compl_leader_len() -> size_t {
    unsafe {
        return if !(*compl_leader.ptr()).data.is_null() {
            (*compl_leader.ptr()).size
        } else {
            (*compl_orig_text.ptr()).size
        };
    }
}

pub(crate) unsafe extern "C" fn ins_compl_has_multiple() -> bool {
    unsafe {
        return !vim_strchr(
            (*compl_shown_match.get()).cp_str.data,
            '\n' as ::core::ffi::c_int,
        )
        .is_null();
    }
}

pub unsafe extern "C" fn ins_compl_lnum_in_range(mut lnum: linenr_T) -> bool {
    unsafe {
        if !ins_compl_has_multiple() {
            return false_0 != 0;
        }
        return lnum >= compl_lnum.get() && lnum <= (*curwin.get()).w_cursor.lnum;
    }
}

pub unsafe extern "C" fn ins_compl_has_shown_match() -> bool {
    unsafe {
        return (*compl_shown_match.ptr()).is_null()
            || compl_shown_match.get() != (*compl_shown_match.get()).cp_next;
    }
}

pub unsafe extern "C" fn ins_compl_long_shown_match() -> bool {
    unsafe {
        return !(*compl_shown_match.ptr()).is_null()
            && !(*compl_shown_match.get()).cp_str.data.is_null()
            && (*compl_shown_match.get()).cp_str.size as colnr_T
                > (*curwin.get()).w_cursor.col - compl_col.get();
    }
}

pub unsafe extern "C" fn get_cot_flags() -> ::core::ffi::c_uint {
    unsafe {
        return if (*curbuf.get()).b_cot_flags != 0 as ::core::ffi::c_uint {
            (*curbuf.get()).b_cot_flags
        } else {
            cot_flags.get()
        };
    }
}

pub unsafe extern "C" fn ins_compl_active() -> bool {
    return compl_started.get();
}

pub unsafe extern "C" fn ins_compl_win_active(mut wp: *mut win_T) -> bool {
    unsafe {
        return ins_compl_active() as ::core::ffi::c_int != 0
            && wp == compl_curr_win.get()
            && (*wp).w_buffer == compl_curr_buf.get();
    }
}

pub unsafe extern "C" fn ins_compl_used_match() -> bool {
    return compl_used_match.get();
}

pub unsafe extern "C" fn ins_compl_init_get_longest() {
    compl_get_longest.set(false_0 != 0);
}

pub unsafe extern "C" fn ins_compl_interrupted() -> bool {
    return compl_interrupted.get() as ::core::ffi::c_int != 0
        || compl_time_slice_expired.get() as ::core::ffi::c_int != 0;
}

pub unsafe extern "C" fn ins_compl_enter_selects() -> bool {
    return compl_enter_selects.get();
}

pub unsafe extern "C" fn ins_compl_col() -> colnr_T {
    return compl_col.get();
}

pub unsafe extern "C" fn ins_compl_len() -> ::core::ffi::c_int {
    return compl_length.get();
}

pub unsafe extern "C" fn ins_compl_has_preinsert() -> bool {
    unsafe {
        let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
        if compl_autocomplete.get() as ::core::ffi::c_int != 0
            && p_ic.get() != 0
            && p_inf.get() == 0
        {
            return false_0 != 0;
        }
        return if !compl_autocomplete.get() {
            (cur_cot_flags
                & (kOptCotFlagPreinsert as ::core::ffi::c_int
                    | kOptCotFlagFuzzy as ::core::ffi::c_int
                    | kOptCotFlagMenuone as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                == (kOptCotFlagPreinsert as ::core::ffi::c_int
                    | kOptCotFlagMenuone as ::core::ffi::c_int)
                    as ::core::ffi::c_uint) as ::core::ffi::c_int
        } else {
            (cur_cot_flags
                & (kOptCotFlagPreinsert as ::core::ffi::c_int
                    | kOptCotFlagFuzzy as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                == kOptCotFlagPreinsert as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int
        } != 0;
    }
}

pub unsafe extern "C" fn ins_compl_preinsert_effect() -> bool {
    unsafe {
        if !ins_compl_has_preinsert() && !ins_compl_preinsert_longest() {
            return false_0 != 0;
        }
        return (*curwin.get()).w_cursor.col < compl_ins_end_col.get();
    }
}

pub(crate) unsafe extern "C" fn ins_compl_refresh_always() -> bool {
    unsafe {
        return (ctrl_x_mode_function() as ::core::ffi::c_int != 0
            || ctrl_x_mode_omni() as ::core::ffi::c_int != 0)
            && compl_opt_refresh_always.get() as ::core::ffi::c_int != 0;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_need_restart() -> bool {
    unsafe {
        return compl_was_interrupted.get() as ::core::ffi::c_int != 0
            || ins_compl_refresh_always() as ::core::ffi::c_int != 0;
    }
}

pub unsafe extern "C" fn ins_compl_has_autocomplete() -> bool {
    unsafe {
        return if (*curbuf.get()).b_p_ac >= 0 as ::core::ffi::c_int {
            (*curbuf.get()).b_p_ac
        } else {
            p_ac.get()
        } != 0;
    }
}

pub(crate) unsafe extern "C" fn get_compl_len() -> ::core::ffi::c_int {
    unsafe {
        let mut off: ::core::ffi::c_int = (*curwin.get()).w_cursor.col - compl_col.get();
        return if 0 as ::core::ffi::c_int > off {
            0 as ::core::ffi::c_int
        } else {
            off
        };
    }
}

pub(crate) unsafe extern "C" fn set_ctrl_x_mode(c: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut retval: bool = false_0 != 0;
        's_241: {
            match c {
                Ctrl_E | Ctrl_Y => {
                    ctrl_x_mode.set(CTRL_X_SCROLL);
                    if State.get() & REPLACE_FLAG == 0 {
                        edit_submode
                            .set(gettext(b" (insert) Scroll (^E/^Y)\0".as_ptr()
                                as *const ::core::ffi::c_char));
                    } else {
                        edit_submode
                            .set(gettext(b" (replace) Scroll (^E/^Y)\0".as_ptr()
                                as *const ::core::ffi::c_char));
                    }
                    edit_submode_pre.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                    redraw_mode.set(true_0 != 0);
                    break 's_241;
                }
                Ctrl_L => {
                    ctrl_x_mode.set(CTRL_X_WHOLE_LINE);
                    break 's_241;
                }
                Ctrl_F => {
                    ctrl_x_mode.set(CTRL_X_FILES);
                    break 's_241;
                }
                Ctrl_K => {
                    ctrl_x_mode.set(CTRL_X_DICTIONARY);
                    break 's_241;
                }
                Ctrl_R => {
                    if vpeekc() == '=' as ::core::ffi::c_int {
                        break 's_241;
                    } else {
                        ctrl_x_mode.set(CTRL_X_REGISTER);
                        break 's_241;
                    }
                }
                Ctrl_T => {
                    ctrl_x_mode.set(CTRL_X_THESAURUS);
                    break 's_241;
                }
                Ctrl_U => {
                    ctrl_x_mode.set(CTRL_X_FUNCTION);
                    break 's_241;
                }
                Ctrl_O => {
                    ctrl_x_mode.set(CTRL_X_OMNI);
                    break 's_241;
                }
                115 | Ctrl_S => {
                    ctrl_x_mode.set(CTRL_X_SPELL);
                    (*emsg_off.ptr()) += 1;
                    spell_back_to_badword();
                    (*emsg_off.ptr()) -= 1;
                    break 's_241;
                }
                Ctrl_RSB => {
                    ctrl_x_mode.set(CTRL_X_TAGS);
                    break 's_241;
                }
                Ctrl_I | K_S_TAB => {
                    ctrl_x_mode.set(CTRL_X_PATH_PATTERNS);
                    break 's_241;
                }
                Ctrl_D => {
                    ctrl_x_mode.set(CTRL_X_PATH_DEFINES);
                    break 's_241;
                }
                Ctrl_V | Ctrl_Q => {
                    ctrl_x_mode.set(CTRL_X_CMDLINE);
                    break 's_241;
                }
                Ctrl_Z => {
                    ctrl_x_mode.set(CTRL_X_NORMAL);
                    edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                    redraw_mode.set(true_0 != 0);
                    retval = true_0 != 0;
                    break 's_241;
                }
                Ctrl_P | Ctrl_N => {
                    if compl_cont_status.get() & CONT_INTRPT == 0 {
                        (*compl_cont_status.ptr()) |= CONT_LOCAL;
                    } else if compl_cont_mode.get() != 0 as ::core::ffi::c_int {
                        (*compl_cont_status.ptr()) &= !CONT_LOCAL;
                    }
                }
                _ => {}
            }
            if c == Ctrl_X {
                if compl_cont_mode.get() != 0 as ::core::ffi::c_int {
                    compl_cont_status.set(0 as ::core::ffi::c_int);
                } else {
                    compl_cont_mode.set(CTRL_X_NOT_DEFINED_YET);
                }
            }
            ctrl_x_mode.set(CTRL_X_NORMAL);
            edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            redraw_mode.set(true_0 != 0);
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_mode() -> *mut ::core::ffi::c_char {
    unsafe {
        if ctrl_x_mode_not_defined_yet() as ::core::ffi::c_int != 0
            || ctrl_x_mode_scroll() as ::core::ffi::c_int != 0
            || compl_started.get() as ::core::ffi::c_int != 0
        {
            return (*ctrl_x_mode_names.ptr())[(ctrl_x_mode.get() & !CTRL_X_WANT_IDENT) as usize];
        }
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_key2dir(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if c == -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            return if (*pum_want.ptr()).item < compl_selected_item.get() {
                BACKWARD as ::core::ffi::c_int
            } else {
                FORWARD as ::core::ffi::c_int
            };
        }
        if c == Ctrl_P
            || c == Ctrl_L
            || c == K_PAGEUP
            || c == K_KPAGEUP
            || c == -(253 as ::core::ffi::c_int
                + ((KE_S_UP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_UP
        {
            return BACKWARD as ::core::ffi::c_int;
        }
        return FORWARD as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_pum_key(mut c: ::core::ffi::c_int) -> bool {
    return pum_visible() as ::core::ffi::c_int != 0
        && (c == K_PAGEUP
            || c == K_KPAGEUP
            || c == -(253 as ::core::ffi::c_int
                + ((KE_S_UP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_PAGEDOWN
            || c == K_KPAGEDOWN
            || c == -(253 as ::core::ffi::c_int
                + ((KE_S_DOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_UP
            || c == K_DOWN);
}

pub(crate) unsafe extern "C" fn ins_compl_key2count(
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if c == -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            let mut offset: ::core::ffi::c_int = (*pum_want.ptr()).item - compl_selected_item.get();
            return abs(offset);
        }
        if ins_compl_pum_key(c) as ::core::ffi::c_int != 0 && c != K_UP && c != K_DOWN {
            let mut h: ::core::ffi::c_int = pum_get_height();
            if h > 3 as ::core::ffi::c_int {
                h -= 2 as ::core::ffi::c_int;
            }
            return h;
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_use_match(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        match c {
            K_UP | K_DOWN | K_PAGEDOWN | K_KPAGEDOWN | -1533 | K_PAGEUP | K_KPAGEUP | -1277 => {
                return false_0 != 0;
            }
            -26365 | -26877 | -26621 => {
                return (*pum_want.ptr()).active as ::core::ffi::c_int != 0
                    && (*pum_want.ptr()).insert as ::core::ffi::c_int != 0;
            }
            _ => {}
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn ins_compl_enable_autocomplete() {
    compl_autocomplete.set(true_0 != 0);
    compl_get_longest.set(false_0 != 0);
}

pub unsafe extern "C" fn f_preinserted(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if ins_compl_preinsert_effect() {
            (*rettv).vval.v_number = 1 as varnumber_T;
        }
    }
}
