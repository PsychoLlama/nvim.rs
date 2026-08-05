//! Starting a completion: where it begins and what it is completing.
//!
//! [`ins_complete`] is the entry point every completion key reaches.
//! [`ins_compl_start`] decides `compl_col`, `compl_length` and the pattern by
//! asking the per-mode `get_*_compl_info` function, then hands over to
//! [`super::getexp::ins_compl_get_exp`].

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_normal_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if compl_cont_status.get() & CONT_SOL != 0
            || ctrl_x_mode_path_defines() as ::core::ffi::c_int != 0
        {
            if !compl_status_adding() {
                loop {
                    startcol -= 1;
                    if !(startcol >= 0 as ::core::ffi::c_int
                        && vim_isIDc(
                            *line.offset(startcol as isize) as uint8_t as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0)
                    {
                        break;
                    }
                }
                startcol += 1;
                (*compl_col.ptr()) += startcol;
                compl_length.set(curs_col as ::core::ffi::c_int - startcol);
            }
            if p_ic.get() != 0 {
                compl_pattern.set(cstr_as_string(str_foldcase(
                    line.offset(compl_col.get() as isize),
                    compl_length.get(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                )));
            } else {
                compl_pattern.set(cbuf_to_string(
                    line.offset(compl_col.get() as isize),
                    compl_length.get() as size_t,
                ));
            }
        } else if compl_status_adding() {
            let mut prefix: *mut ::core::ffi::c_char =
                b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            let mut prefixlen: size_t =
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t);
            if !vim_iswordp(line.offset(compl_col.get() as isize))
                || compl_col.get() > 0 as ::core::ffi::c_int
                    && vim_iswordp(mb_prevptr(line, line.offset(compl_col.get() as isize)))
                        as ::core::ffi::c_int
                        != 0
            {
                prefix = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                prefixlen = 0 as size_t;
            }
            let mut n: size_t = (quote_meta(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                line.offset(compl_col.get() as isize),
                compl_length.get(),
            ) as size_t)
                .wrapping_add(prefixlen);
            (*compl_pattern.ptr()).data = xmalloc(n) as *mut ::core::ffi::c_char;
            strcpy((*compl_pattern.ptr()).data, prefix);
            quote_meta(
                (*compl_pattern.ptr()).data.offset(prefixlen as isize),
                line.offset(compl_col.get() as isize),
                compl_length.get(),
            );
            (*compl_pattern.ptr()).size = n.wrapping_sub(1 as size_t);
        } else {
            startcol -= 1;
            if startcol < 0 as ::core::ffi::c_int
                || !vim_iswordp(mb_prevptr(
                    line,
                    line.offset(startcol as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                ))
            {
                compl_pattern.set(cbuf_to_string(
                    b"\\<\\k\\k\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                ));
                (*compl_col.ptr()) += curs_col;
                compl_length.set(0 as ::core::ffi::c_int);
                compl_from_nonkeyword.set(true_0 != 0);
            } else {
                startcol -= utf_head_off(line, line.offset(startcol as isize));
                let mut base_class: ::core::ffi::c_int =
                    mb_get_class(line.offset(startcol as isize));
                loop {
                    startcol -= 1;
                    if startcol < 0 as ::core::ffi::c_int {
                        break;
                    }
                    let mut head_off: ::core::ffi::c_int =
                        utf_head_off(line, line.offset(startcol as isize));
                    if base_class
                        != mb_get_class(line.offset(startcol as isize).offset(-(head_off as isize)))
                    {
                        break;
                    }
                    startcol -= head_off;
                }
                startcol += 1;
                (*compl_col.ptr()) += startcol;
                compl_length.set(curs_col - startcol);
                if compl_length.get() == 1 as ::core::ffi::c_int {
                    (*compl_pattern.ptr()).data = xmalloc(7 as size_t) as *mut ::core::ffi::c_char;
                    strcpy(
                        (*compl_pattern.ptr()).data,
                        b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    quote_meta(
                        (*compl_pattern.ptr())
                            .data
                            .offset(2 as ::core::ffi::c_int as isize),
                        line.offset(compl_col.get() as isize),
                        1 as ::core::ffi::c_int,
                    );
                    strcat(
                        (*compl_pattern.ptr()).data,
                        b"\\k\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    (*compl_pattern.ptr()).size = strlen((*compl_pattern.ptr()).data);
                } else {
                    let mut n_0: size_t = quote_meta(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        line.offset(compl_col.get() as isize),
                        compl_length.get(),
                    )
                    .wrapping_add(2 as ::core::ffi::c_uint)
                        as size_t;
                    (*compl_pattern.ptr()).data = xmalloc(n_0) as *mut ::core::ffi::c_char;
                    strcpy(
                        (*compl_pattern.ptr()).data,
                        b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    quote_meta(
                        (*compl_pattern.ptr())
                            .data
                            .offset(2 as ::core::ffi::c_int as isize),
                        line.offset(compl_col.get() as isize),
                        compl_length.get(),
                    );
                    (*compl_pattern.ptr()).size = n_0.wrapping_sub(1 as size_t);
                }
            }
        }
        if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && compl_cont_status.get() & CONT_LOCAL == 0
        {
            setup_cpt_sources();
            prepare_cpt_compl_funcs();
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_wholeline_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        compl_col.set(getwhitecols(line) as colnr_T);
        compl_length.set(curs_col - compl_col.get());
        if compl_length.get() < 0 as ::core::ffi::c_int {
            compl_length.set(0 as ::core::ffi::c_int);
        }
        if p_ic.get() != 0 {
            compl_pattern.set(cstr_as_string(str_foldcase(
                line.offset(compl_col.get() as isize),
                compl_length.get(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
            )));
        } else {
            compl_pattern.set(cbuf_to_string(
                line.offset(compl_col.get() as isize),
                compl_length.get() as size_t,
            ));
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_filename_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if startcol > 0 as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_char = line.offset(startcol as isize);
            p = p.offset(
                -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
            while p > line && vim_isfilec(utf_ptr2char(p)) as ::core::ffi::c_int != 0 {
                p = p.offset(
                    -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
            let mut p_is_filec: bool = false_0 != 0;
            p_is_filec = p_is_filec as ::core::ffi::c_int != 0
                || vim_isfilec(utf_ptr2char(p)) as ::core::ffi::c_int != 0;
            if p == line && p_is_filec as ::core::ffi::c_int != 0 {
                startcol = 0 as ::core::ffi::c_int;
            } else {
                startcol = p.offset_from(line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
            }
        }
        (*compl_col.ptr()) += startcol;
        compl_length.set(curs_col - startcol);
        compl_pattern.set(cstr_as_string(addstar(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
            EXPAND_FILES,
        )));
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_cmdline_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        compl_pattern.set(cbuf_to_string(line, curs_col as size_t));
        set_cmd_context(
            compl_xp.ptr(),
            (*compl_pattern.ptr()).data,
            (*compl_pattern.ptr()).size as ::core::ffi::c_int,
            curs_col as ::core::ffi::c_int,
            false,
        );
        if (*compl_xp.ptr()).xp_context == EXPAND_LUA {
            nlua_expand_pat(compl_xp.ptr());
        }
        if (*compl_xp.ptr()).xp_context == EXPAND_UNSUCCESSFUL
            || (*compl_xp.ptr()).xp_context == EXPAND_NOTHING
        {
            compl_col.set(curs_col);
        } else {
            compl_col.set(
                (*compl_xp.ptr())
                    .xp_pattern
                    .offset_from((*compl_pattern.ptr()).data) as ::core::ffi::c_int
                    as colnr_T,
            );
        }
        compl_length.set((curs_col - compl_col.get()) as ::core::ffi::c_int);
        return OK;
    }
}

pub(crate) unsafe extern "C" fn set_compl_globals(
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
    mut is_cpt_compl: bool,
) {
    unsafe {
        if is_cpt_compl {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*cpt_compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            (*cpt_compl_pattern.ptr()).size = 0 as size_t;
            if startcol < compl_col.get() {
                prepend_startcol_text(cpt_compl_pattern.ptr(), compl_orig_text.ptr(), startcol);
                return;
            } else {
                cpt_compl_pattern.set(copy_string(
                    compl_orig_text.get(),
                    ::core::ptr::null_mut::<Arena>(),
                ));
            }
        } else {
            if startcol < 0 as ::core::ffi::c_int || startcol > curs_col {
                startcol = curs_col as ::core::ffi::c_int;
            }
            let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
            let mut len: ::core::ffi::c_int = curs_col as ::core::ffi::c_int - startcol;
            compl_pattern.set(cbuf_to_string(
                line.offset(startcol as isize),
                len as size_t,
            ));
            compl_col.set(startcol as colnr_T);
            compl_length.set(len);
        };
    }
}

pub(crate) unsafe extern "C" fn get_userdefined_compl_info(
    mut curs_col: colnr_T,
    mut cb: *mut Callback,
    mut startcol: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let save_State: ::core::ffi::c_int = State.get();
        let is_cpt_function: bool = !cb.is_null();
        if !is_cpt_function {
            let mut funcname: *mut ::core::ffi::c_char = get_complete_funcname(ctrl_x_mode.get());
            if *funcname as ::core::ffi::c_int == NUL {
                semsg(
                    gettext(&raw const e_notset as *const ::core::ffi::c_char),
                    if ctrl_x_mode_function() as ::core::ffi::c_int != 0 {
                        b"completefunc\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"omnifunc\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                return FAIL;
            }
            cb = get_insert_callback(ctrl_x_mode.get());
        }
        let mut args: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        args[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
        args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        args[2 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        args[0 as ::core::ffi::c_int as usize].vval.v_number = 1 as varnumber_T;
        args[1 as ::core::ffi::c_int as usize].vval.v_string =
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*textlock.ptr()) += 1;
        let mut col: colnr_T =
            callback_call_retnr(cb, 2 as ::core::ffi::c_int, &raw mut args as *mut typval_T)
                as colnr_T;
        (*textlock.ptr()) -= 1;
        State.set(save_State);
        (*curwin.get()).w_cursor = pos;
        check_cursor(curwin.get());
        validate_cursor(curwin.get());
        if !equalpos((*curwin.get()).w_cursor, pos) {
            emsg(gettext(
                (e_compldel.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if !startcol.is_null() {
            *startcol = col as ::core::ffi::c_int;
        }
        if col == -2 as ::core::ffi::c_int || aborting() as ::core::ffi::c_int != 0 {
            return FAIL;
        }
        if col == -3 as ::core::ffi::c_int {
            if is_cpt_function {
                return FAIL;
            }
            ctrl_x_mode.set(CTRL_X_NORMAL);
            edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            if !shortmess(SHM_COMPLETIONMENU) {
                msg_clr_cmdline();
            }
            return FAIL;
        }
        compl_opt_refresh_always.set(false_0 != 0);
        if !is_cpt_function {
            set_compl_globals(col as ::core::ffi::c_int, curs_col, false_0 != 0);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_spell_compl_info(
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if spell_bad_len.get() > 0 as size_t {
            '_c2rust_label: {
                if spell_bad_len.get() <= 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                        b"spell_bad_len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        5875 as ::core::ffi::c_uint,
                        b"int get_spell_compl_info(int, colnr_T)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            compl_col.set(
                (curs_col as ::core::ffi::c_int - spell_bad_len.get() as ::core::ffi::c_int)
                    as colnr_T,
            );
        } else {
            compl_col.set(spell_word_start(startcol) as colnr_T);
        }
        if compl_col.get() >= startcol {
            compl_length.set(0 as ::core::ffi::c_int);
            compl_col.set(curs_col);
        } else {
            spell_expand_check_cap(compl_col.get());
            compl_length.set((curs_col - compl_col.get()) as ::core::ffi::c_int);
        }
        let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
        compl_pattern.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
        return OK;
    }
}

pub(crate) unsafe extern "C" fn compl_get_info(
    mut line: *mut ::core::ffi::c_char,
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
    mut line_invalid: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            || ctrl_x_mode_register() as ::core::ffi::c_int != 0
            || ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0
                && !thesaurus_func_complete(ctrl_x_mode.get())
        {
            if get_normal_compl_info(line, startcol, curs_col) != OK {
                return FAIL;
            }
            *line_invalid = true_0 != 0;
        } else if ctrl_x_mode_line_or_eval() {
            return get_wholeline_compl_info(line, curs_col);
        } else if ctrl_x_mode_files() {
            return get_filename_compl_info(line, startcol, curs_col);
        } else if ctrl_x_mode.get() == CTRL_X_CMDLINE {
            return get_cmdline_compl_info(line, curs_col);
        } else if ctrl_x_mode_function() as ::core::ffi::c_int != 0
            || ctrl_x_mode_omni() as ::core::ffi::c_int != 0
            || thesaurus_func_complete(ctrl_x_mode.get()) as ::core::ffi::c_int != 0
        {
            if get_userdefined_compl_info(
                curs_col,
                ::core::ptr::null_mut::<Callback>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != OK
            {
                return FAIL;
            }
            *line_invalid = true_0 != 0;
        } else if ctrl_x_mode_spell() {
            if get_spell_compl_info(startcol, curs_col) == FAIL {
                return FAIL;
            }
            *line_invalid = true_0 != 0;
        } else {
            internal_error(b"ins_complete()\0".as_ptr() as *const ::core::ffi::c_char);
            return FAIL;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_continue_search(mut line: *mut ::core::ffi::c_char) {
    unsafe {
        (*compl_cont_status.ptr()) &= !CONT_INTRPT;
        if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            || ctrl_x_mode_path_patterns() as ::core::ffi::c_int != 0
            || ctrl_x_mode_path_defines() as ::core::ffi::c_int != 0
        {
            if (*compl_startpos.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
                compl_col.set(getwhitecols(line) as colnr_T);
                (*compl_startpos.ptr()).col = compl_col.get();
                (*compl_startpos.ptr()).lnum = (*curwin.get()).w_cursor.lnum;
                (*compl_cont_status.ptr()) &= !CONT_SOL;
            } else {
                if compl_cont_status.get() & CONT_S_IPOS != 0 {
                    (*compl_cont_status.ptr()) |= CONT_SOL;
                    (*compl_startpos.ptr()).col = skipwhite(
                        line.offset(compl_length.get() as isize)
                            .offset((*compl_startpos.ptr()).col as isize),
                    )
                    .offset_from(line) as colnr_T;
                }
                compl_col.set((*compl_startpos.ptr()).col);
            }
            compl_length.set((*curwin.get()).w_cursor.col as ::core::ffi::c_int - compl_col.get());
            if compl_length.get() > IOSIZE - MIN_SPACE {
                (*compl_cont_status.ptr()) &= !CONT_SOL;
                compl_length.set(IOSIZE - MIN_SPACE);
                compl_col.set(
                    ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - compl_length.get())
                        as colnr_T,
                );
            }
            (*compl_cont_status.ptr()) |= CONT_ADDING | CONT_N_ADDS;
            if compl_length.get() < 1 as ::core::ffi::c_int {
                (*compl_cont_status.ptr()) &= CONT_LOCAL;
            }
        } else if ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0
            || ctrl_x_mode_register() as ::core::ffi::c_int != 0
        {
            compl_cont_status.set(CONT_ADDING | CONT_N_ADDS);
        } else {
            compl_cont_status.set(0 as ::core::ffi::c_int);
        };
    }
}

pub(crate) unsafe extern "C" fn ins_compl_start() -> ::core::ffi::c_int {
    unsafe {
        let save_did_ai: bool = did_ai.get();
        did_ai.set(false_0 != 0);
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        if stop_arrow() == FAIL {
            did_ai.set(save_did_ai);
            return FAIL;
        }
        let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
        let mut curs_col: colnr_T = (*curwin.get()).w_cursor.col;
        compl_pending.set(0 as ::core::ffi::c_int);
        compl_lnum.set((*curwin.get()).w_cursor.lnum);
        if compl_cont_status.get() & CONT_INTRPT == CONT_INTRPT
            && compl_cont_mode.get() == ctrl_x_mode.get()
        {
            ins_compl_continue_search(line);
        } else {
            (*compl_cont_status.ptr()) &= CONT_LOCAL;
        }
        let mut startcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !compl_status_adding() {
            compl_cont_mode.set(ctrl_x_mode.get());
            if ctrl_x_mode_not_default() {
                compl_cont_status.set(0 as ::core::ffi::c_int);
            }
            (*compl_cont_status.ptr()) |= CONT_N_ADDS;
            compl_startpos.set((*curwin.get()).w_cursor);
            startcol = curs_col;
            compl_col.set(0 as ::core::ffi::c_int as colnr_T);
        }
        let mut line_invalid: bool = false_0 != 0;
        if compl_get_info(line, startcol, curs_col, &raw mut line_invalid) == FAIL {
            if ctrl_x_mode_function() as ::core::ffi::c_int != 0
                || ctrl_x_mode_omni() as ::core::ffi::c_int != 0
                || thesaurus_func_complete(ctrl_x_mode.get()) as ::core::ffi::c_int != 0
            {
                did_ai.set(save_did_ai);
            }
            return FAIL;
        }
        if line_invalid {
            line = ml_get((*curwin.get()).w_cursor.lnum);
        }
        if compl_status_adding() {
            if !shortmess(SHM_COMPLETIONMENU) {
                edit_submode_pre.set(gettext(b" Adding\0".as_ptr() as *const ::core::ffi::c_char));
            }
            if ctrl_x_mode_line_or_eval() {
                let mut old: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
                (*curbuf.get()).b_p_com =
                    b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                (*compl_startpos.ptr()).lnum = (*curwin.get()).w_cursor.lnum;
                (*compl_startpos.ptr()).col = compl_col.get();
                ins_eol('\r' as ::core::ffi::c_int);
                (*curbuf.get()).b_p_com = old;
                compl_length.set(0 as ::core::ffi::c_int);
                compl_col.set((*curwin.get()).w_cursor.col);
                compl_lnum.set((*curwin.get()).w_cursor.lnum);
            }
        } else {
            edit_submode_pre.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            (*compl_startpos.ptr()).col = compl_col.get();
        }
        if !shortmess(SHM_COMPLETIONMENU) && !compl_autocomplete.get() {
            if compl_cont_status.get() & CONT_LOCAL != 0 {
                edit_submode.set(gettext((*ctrl_x_msgs.ptr())[CTRL_X_LOCAL_MSG as usize]));
            } else {
                edit_submode.set(gettext(
                    (*ctrl_x_msgs.ptr())
                        [(ctrl_x_mode.get() & !(0x100 as ::core::ffi::c_int)) as usize],
                ));
            }
        }
        ins_compl_fixRedoBufForLeader(::core::ptr::null_mut::<::core::ffi::c_char>());
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_orig_text.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*compl_orig_text.ptr()).size = 0 as size_t;
        xfree((*compl_orig_extmarks.ptr()).items as *mut ::core::ffi::c_void);
        (*compl_orig_extmarks.ptr()).capacity = 0 as size_t;
        (*compl_orig_extmarks.ptr()).size = (*compl_orig_extmarks.ptr()).capacity;
        (*compl_orig_extmarks.ptr()).items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
        compl_orig_text.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
        save_orig_extmarks();
        let mut flags: ::core::ffi::c_int = CP_ORIGINAL_TEXT;
        if p_ic.get() != 0 {
            flags |= CP_ICASE;
        }
        if ins_compl_add(
            (*compl_orig_text.ptr()).data,
            (*compl_orig_text.ptr()).size as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<typval_T>(),
            kDirectionNotSet,
            flags,
            false_0 != 0,
            ::core::ptr::null::<::core::ffi::c_int>(),
            FUZZY_SCORE_NONE,
        ) != OK
        {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
            (*compl_pattern.ptr()).size = 0 as size_t;
            let mut ptr__1: *mut *mut ::core::ffi::c_void =
                &raw mut (*compl_orig_text.ptr()).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__1);
            *ptr__1 = NULL;
            let _ = *ptr__1;
            (*compl_orig_text.ptr()).size = 0 as size_t;
            xfree((*compl_orig_extmarks.ptr()).items as *mut ::core::ffi::c_void);
            (*compl_orig_extmarks.ptr()).capacity = 0 as size_t;
            (*compl_orig_extmarks.ptr()).size = (*compl_orig_extmarks.ptr()).capacity;
            (*compl_orig_extmarks.ptr()).items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
            did_ai.set(save_did_ai);
            return FAIL;
        }
        if !shortmess(SHM_COMPLETIONMENU) && !compl_autocomplete.get() {
            edit_submode_extra.set(gettext(
                b"-- Searching...\0".as_ptr() as *const ::core::ffi::c_char
            ));
            edit_submode_highl.set(HLF_COUNT);
            showmode();
            edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            ui_flush();
        }
        did_ai.set(save_did_ai);
        return OK;
    }
}

pub unsafe extern "C" fn ins_complete(
    mut c: ::core::ffi::c_int,
    mut enable_pum: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let disable_ac_delay: bool = compl_started.get() as ::core::ffi::c_int != 0
            && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && (c == Ctrl_N
                || c == Ctrl_P
                || c == Ctrl_R
                || ins_compl_pum_key(c) as ::core::ffi::c_int != 0);
        compl_direction.set(ins_compl_key2dir(c) as Direction);
        let mut insert_match: ::core::ffi::c_int = ins_compl_use_match(c) as ::core::ffi::c_int;
        if !compl_started.get() {
            if ins_compl_start() == FAIL {
                return FAIL;
            }
        } else if insert_match != 0 && stop_arrow() == FAIL {
            return FAIL;
        }
        let mut compl_start_tv: uint64_t = 0 as uint64_t;
        if compl_autocomplete.get() as ::core::ffi::c_int != 0
            && p_acl.get() > 0 as OptInt
            && !disable_ac_delay
        {
            compl_start_tv = os_hrtime();
        }
        compl_curr_win.set(curwin.get());
        compl_curr_buf.set((*curwin.get()).w_buffer);
        compl_shown_match.set(compl_curr_match.get());
        compl_shows_dir.set(compl_direction.get());
        compl_num_bests.set(0 as ::core::ffi::c_int);
        let mut save_w_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
        let mut save_w_leftcol: ::core::ffi::c_int =
            (*curwin.get()).w_leftcol as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int =
            ins_compl_next(true_0 != 0, ins_compl_key2count(c), insert_match != 0);
        if compl_autocomplete.get() {
            compl_time_slice_expired.set(false_0 != 0);
        }
        if n > 1 as ::core::ffi::c_int {
            compl_matches.set(n);
        }
        compl_curr_match.set(compl_shown_match.get());
        compl_direction.set(compl_shows_dir.get());
        if got_int.get() as ::core::ffi::c_int != 0 && global_busy.get() == 0 {
            vgetc();
            got_int.set(false_0 != 0);
        }
        let mut no_matches_found: bool = is_first_match((*compl_first_match.get()).cp_next);
        if no_matches_found {
            if compl_length.get() > 1 as ::core::ffi::c_int
                || compl_status_adding() as ::core::ffi::c_int != 0
                || ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
                    && !ctrl_x_mode_path_patterns()
                    && !ctrl_x_mode_path_defines()
            {
                (*compl_cont_status.ptr()) &= !CONT_N_ADDS;
            }
        }
        if (*compl_curr_match.get()).cp_flags & CP_CONT_S_IPOS != 0 {
            (*compl_cont_status.ptr()) |= CONT_S_IPOS;
        } else {
            (*compl_cont_status.ptr()) &= !CONT_S_IPOS;
        }
        if !shortmess(SHM_COMPLETIONMENU) && !compl_autocomplete.get() {
            ins_compl_show_statusmsg();
        }
        if compl_autocomplete.get() as ::core::ffi::c_int != 0
            && p_acl.get() > 0 as OptInt
            && !disable_ac_delay
            && !no_matches_found
            && os_hrtime()
                .wrapping_sub(compl_start_tv)
                .wrapping_div(1000000 as uint64_t)
                < p_acl.get() as uint64_t
        {
            setcursor();
            ui_flush();
            loop {
                if char_avail() {
                    if ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
                        && ins_compl_win_active(curwin.get()) as ::core::ffi::c_int != 0
                    {
                        ins_compl_delete(false_0 != 0);
                        compl_ins_end_col.set(compl_col.get());
                    }
                    ins_compl_restart();
                    compl_interrupted.set(true_0 != 0);
                    break;
                } else {
                    os_delay(2 as uint64_t, true_0 != 0);
                    if os_hrtime()
                        .wrapping_sub(compl_start_tv)
                        .wrapping_div(1000000 as uint64_t)
                        >= p_acl.get() as uint64_t
                    {
                        break;
                    }
                }
            }
        }
        if enable_pum as ::core::ffi::c_int != 0 && !compl_interrupted.get() {
            show_pum(save_w_wrow, save_w_leftcol);
        }
        compl_was_interrupted.set(compl_interrupted.get());
        compl_interrupted.set(false_0 != 0);
        return OK;
    }
}

pub(crate) unsafe extern "C" fn spell_back_to_badword() {
    unsafe {
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        spell_bad_len.set(spell_move_to(
            curwin.get(),
            BACKWARD as ::core::ffi::c_int,
            SMT_ALL,
            true_0 != 0,
            ::core::ptr::null_mut::<hlf_T>(),
        ));
        if (*curwin.get()).w_cursor.col != tpos.col {
            start_arrow(&raw mut tpos);
        }
    }
}
