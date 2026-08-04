//! The editing keys.
//!
//! [`command_line_handle_key`] is the big switch over every key that is not
//! handled earlier: the cursor motions, the erase keys, the register and
//! digraph insertions, history, and the keys that end the line.  The arms
//! long enough to need it have a helper of their own next to it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn command_line_erase_chars(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*s).c = K_DEL;
        }
        if (*s).c == K_DEL && (*ccline.ptr()).cmdpos != (*ccline.ptr()).cmdlen {
            (*ccline.ptr()).cmdpos += 1;
        }
        if (*s).c == K_DEL {
            (*ccline.ptr()).cmdpos += mb_off_next(
                (*ccline.ptr()).cmdbuff,
                (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize),
            );
        }
        if (*ccline.ptr()).cmdpos > 0 as ::core::ffi::c_int {
            let mut j: ::core::ffi::c_int = (*ccline.ptr()).cmdpos;
            let mut p: *mut ::core::ffi::c_char = mb_prevptr(
                (*ccline.ptr()).cmdbuff,
                (*ccline.ptr()).cmdbuff.offset(j as isize),
            );
            if (*s).c == Ctrl_W {
                while p > (*ccline.ptr()).cmdbuff
                    && ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    p = mb_prevptr((*ccline.ptr()).cmdbuff, p);
                }
                let mut i: ::core::ffi::c_int = mb_get_class(p);
                while p > (*ccline.ptr()).cmdbuff && mb_get_class(p) == i {
                    p = mb_prevptr((*ccline.ptr()).cmdbuff, p);
                }
                if mb_get_class(p) != i {
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            }
            (*ccline.ptr()).cmdpos = p.offset_from((*ccline.ptr()).cmdbuff) as ::core::ffi::c_int;
            (*ccline.ptr()).cmdlen -= j - (*ccline.ptr()).cmdpos;
            let mut i_0: ::core::ffi::c_int = (*ccline.ptr()).cmdpos;
            while i_0 < (*ccline.ptr()).cmdlen {
                let c2rust_fresh29 = j;
                j = j + 1;
                let c2rust_fresh30 = i_0;
                i_0 = i_0 + 1;
                *(*ccline.ptr()).cmdbuff.offset(c2rust_fresh30 as isize) =
                    *(*ccline.ptr()).cmdbuff.offset(c2rust_fresh29 as isize);
            }
            *(*ccline.ptr())
                .cmdbuff
                .offset((*ccline.ptr()).cmdlen as isize) = NUL as ::core::ffi::c_char;
            if (*ccline.ptr()).cmdlen == 0 as ::core::ffi::c_int {
                (*s).is_state.search_start = (*s).is_state.save_cursor;
                (*s).is_state.old_viewstate = (*s).is_state.init_viewstate;
            }
            redrawcmd();
        } else if (*ccline.ptr()).cmdlen == 0 as ::core::ffi::c_int
            && (*s).c != Ctrl_W
            && (*ccline.ptr()).cmdprompt.is_null()
            && (*s).indent == 0 as ::core::ffi::c_int
        {
            if exmode_active.get() as ::core::ffi::c_int != 0
                || (*ccline.ptr()).cmdfirstc == '>' as ::core::ffi::c_int
            {
                return CMDLINE_NOT_CHANGED;
            }
            dealloc_cmdbuff();
            if !cmd_silent.get() && !ui_has(kUICmdline) {
                msg_col.set(0 as ::core::ffi::c_int);
                msg_putchar(' ' as ::core::ffi::c_int);
            }
            (*s).is_state.search_start = (*s).is_state.save_cursor;
            redraw_cmdline.set(true_0 != 0);
            return GOTO_NORMAL_MODE;
        }
        return CMDLINE_CHANGED;
    }
}

pub(crate) unsafe extern "C" fn command_line_toggle_langmap(mut s: *mut CommandLineState) {
    unsafe {
        let mut b_im_ptr: *mut OptInt = if buf_valid((*s).b_im_ptr_buf) as ::core::ffi::c_int != 0 {
            (*s).b_im_ptr
        } else {
            ::core::ptr::null_mut::<OptInt>()
        };
        if map_to_exists_mode(
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            MODE_LANGMAP,
            false_0 != 0,
        ) {
            (*State.ptr()) ^= MODE_LANGMAP;
            if !b_im_ptr.is_null() {
                if State.get() & MODE_LANGMAP != 0 {
                    *b_im_ptr = B_IMODE_LMAP as OptInt;
                } else {
                    *b_im_ptr = B_IMODE_NONE as OptInt;
                }
            }
        }
        if !b_im_ptr.is_null() {
            if b_im_ptr == &raw mut (*curbuf.get()).b_p_iminsert {
                set_iminsert_global(curbuf.get());
            } else {
                set_imsearch_global(curbuf.get());
            }
        }
        ui_cursor_shape();
        status_redraw_curbuf();
    }
}

pub(crate) unsafe extern "C" fn command_line_insert_reg(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        let save_new_cmdpos: ::core::ffi::c_int = new_cmdpos.get();
        putcmdline('"' as ::core::ffi::c_char, true_0 != 0);
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        let mut i: ::core::ffi::c_int = (*s).c;
        if i == Ctrl_O {
            i = Ctrl_R;
        }
        if i == Ctrl_R {
            (*s).c = plain_vgetc();
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        new_cmdpos.set(-1 as ::core::ffi::c_int);
        if (*s).c == '=' as ::core::ffi::c_int {
            if (*ccline.ptr()).cmdfirstc == '=' as ::core::ffi::c_int
                || cmdline_star.get() > 0 as ::core::ffi::c_int
            {
                beep_flush();
                (*s).c = ESC;
            } else {
                (*s).c = get_expr_register();
            }
        }
        let mut literally: bool = false_0 != 0;
        if (*s).c != ESC {
            literally = i == Ctrl_R || is_literal_register((*s).c) as ::core::ffi::c_int != 0;
            cmdline_paste((*s).c, literally, false_0 != 0);
            if aborting() {
                (*s).gotesc = true_0 != 0;
                return GOTO_NORMAL_MODE;
            }
            KeyTyped.set(false_0 != 0);
            if new_cmdpos.get() >= 0 as ::core::ffi::c_int {
                (*ccline.ptr()).cmdpos = if (*ccline.ptr()).cmdlen < new_cmdpos.get() {
                    (*ccline.ptr()).cmdlen
                } else {
                    new_cmdpos.get()
                };
            }
        }
        new_cmdpos.set(save_new_cmdpos);
        (*ccline.ptr()).special_char = NUL as ::core::ffi::c_char;
        redrawcmd();
        return if literally as ::core::ffi::c_int != 0 {
            CMDLINE_CHANGED
        } else {
            CMDLINE_NOT_CHANGED
        };
    }
}

pub(crate) unsafe extern "C" fn command_line_left_right_mouse(mut s: *mut CommandLineState) {
    unsafe {
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || (*s).c
                == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*s).ignore_drag_release = true_0 != 0;
        } else {
            (*s).ignore_drag_release = false_0 != 0;
        }
        (*ccline.ptr()).cmdspos = cmd_startcol();
        (*ccline.ptr()).cmdpos = 0 as ::core::ffi::c_int;
        while (*ccline.ptr()).cmdpos < (*ccline.ptr()).cmdlen {
            let mut cells: ::core::ffi::c_int = cmdline_charsize((*ccline.ptr()).cmdpos);
            if mouse_row.get() <= cmdline_row.get() + (*ccline.ptr()).cmdspos / Columns.get()
                && mouse_col.get() < (*ccline.ptr()).cmdspos % Columns.get() + cells
            {
                break;
            }
            correct_screencol(
                (*ccline.ptr()).cmdpos,
                cells,
                &raw mut (*ccline.ptr()).cmdspos,
            );
            (*ccline.ptr()).cmdpos += utfc_ptr2len(
                (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize),
            ) - 1 as ::core::ffi::c_int;
            (*ccline.ptr()).cmdspos += cells;
            (*ccline.ptr()).cmdpos += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn command_line_handle_key(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        if !((*ccline.ptr()).one_key as ::core::ffi::c_int != 0
            && (*s).c != ESC
            && (*s).c != Ctrl_C)
        {
            's_680: {
                'c_46093: {
                    'c_46136: {
                        'c_46090: {
                            match (*s).c {
                                K_BS | Ctrl_H | K_DEL | -20733 | Ctrl_W => {
                                    match command_line_erase_chars(s) {
                                        1 => return command_line_not_changed(s),
                                        3 => return 0 as ::core::ffi::c_int,
                                        _ => return command_line_changed(s),
                                    }
                                }
                                K_INS | K_KINS => {
                                    (*ccline.ptr()).overstrike =
                                        ((*ccline.ptr()).overstrike == 0) as ::core::ffi::c_int;
                                    ui_cursor_shape();
                                    may_trigger_modechanged();
                                    status_redraw_curbuf();
                                    redraw_statuslines();
                                    return command_line_not_changed(s);
                                }
                                Ctrl_HAT => {
                                    command_line_toggle_langmap(s);
                                    return command_line_not_changed(s);
                                }
                                Ctrl_U => {
                                    let mut j: ::core::ffi::c_int = (*ccline.ptr()).cmdpos;
                                    (*ccline.ptr()).cmdlen -= j;
                                    (*ccline.ptr()).cmdpos = 0 as ::core::ffi::c_int;
                                    let mut i: ::core::ffi::c_int = (*ccline.ptr()).cmdpos;
                                    while i < (*ccline.ptr()).cmdlen {
                                        let c2rust_fresh7 = j;
                                        j = j + 1;
                                        let c2rust_fresh8 = i;
                                        i = i + 1;
                                        *(*ccline.ptr()).cmdbuff.offset(c2rust_fresh8 as isize) =
                                            *(*ccline.ptr()).cmdbuff.offset(c2rust_fresh7 as isize);
                                    }
                                    *(*ccline.ptr())
                                        .cmdbuff
                                        .offset((*ccline.ptr()).cmdlen as isize) =
                                        NUL as ::core::ffi::c_char;
                                    if (*ccline.ptr()).cmdlen == 0 as ::core::ffi::c_int {
                                        (*s).is_state.search_start = (*s).is_state.save_cursor;
                                    }
                                    redrawcmd();
                                    return command_line_changed(s);
                                }
                                ESC | Ctrl_C => {
                                    if exmode_active.get() as ::core::ffi::c_int != 0
                                        && (ex_normal_busy.get() == 0 as ::core::ffi::c_int
                                            || (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int)
                                        || getln_interrupted_highlight.get() as ::core::ffi::c_int
                                            != 0
                                            && (*s).c == Ctrl_C
                                    {
                                        getln_interrupted_highlight.set(false_0 != 0);
                                        return command_line_not_changed(s);
                                    }
                                    (*s).gotesc = true_0 != 0;
                                    return 0 as ::core::ffi::c_int;
                                }
                                Ctrl_R => match command_line_insert_reg(s) {
                                    3 => return 0 as ::core::ffi::c_int,
                                    2 => return command_line_changed(s),
                                    _ => return command_line_not_changed(s),
                                },
                                Ctrl_D => {
                                    if showmatches(
                                        &raw mut (*s).xpc,
                                        false_0 != 0,
                                        true_0 != 0,
                                        (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize]
                                            as ::core::ffi::c_int
                                            & kOptWimFlagNoselect as ::core::ffi::c_int
                                            != 0,
                                    ) == EXPAND_NOTHING as ::core::ffi::c_int
                                    {
                                        break 's_680;
                                    } else {
                                        redrawcmd();
                                        return 1 as ::core::ffi::c_int;
                                    }
                                }
                                K_RIGHT | K_S_RIGHT | -22269 => {
                                    while (*ccline.ptr()).cmdpos < (*ccline.ptr()).cmdlen {
                                        let mut cells: ::core::ffi::c_int =
                                            cmdline_charsize((*ccline.ptr()).cmdpos);
                                        if KeyTyped.get() as ::core::ffi::c_int != 0
                                            && (*ccline.ptr()).cmdspos + cells
                                                >= Columns.get() * Rows.get()
                                        {
                                            break;
                                        }
                                        (*ccline.ptr()).cmdspos += cells;
                                        (*ccline.ptr()).cmdpos += utfc_ptr2len(
                                            (*ccline.ptr())
                                                .cmdbuff
                                                .offset((*ccline.ptr()).cmdpos as isize),
                                        );
                                        if !(((*s).c == K_S_RIGHT
                                            || (*s).c
                                                == -(253 as ::core::ffi::c_int
                                                    + ((KE_C_RIGHT as ::core::ffi::c_int)
                                                        << 8 as ::core::ffi::c_int))
                                            || mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL)
                                                != 0)
                                            && *(*ccline.ptr())
                                                .cmdbuff
                                                .offset((*ccline.ptr()).cmdpos as isize)
                                                as ::core::ffi::c_int
                                                != ' ' as ::core::ffi::c_int)
                                        {
                                            break;
                                        }
                                    }
                                    (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
                                    return command_line_not_changed(s);
                                }
                                K_LEFT | K_S_LEFT | -22013 => {
                                    if (*ccline.ptr()).cmdpos == 0 as ::core::ffi::c_int {
                                        return command_line_not_changed(s);
                                    }
                                    loop {
                                        (*ccline.ptr()).cmdpos -= 1;
                                        (*ccline.ptr()).cmdpos -= utf_head_off(
                                            (*ccline.ptr()).cmdbuff,
                                            (*ccline.ptr())
                                                .cmdbuff
                                                .offset((*ccline.ptr()).cmdpos as isize),
                                        );
                                        (*ccline.ptr()).cmdspos -=
                                            cmdline_charsize((*ccline.ptr()).cmdpos);
                                        if !((*ccline.ptr()).cmdpos > 0 as ::core::ffi::c_int
                                            && ((*s).c == K_S_LEFT
                                                || (*s).c
                                                    == -(253 as ::core::ffi::c_int
                                                        + ((KE_C_LEFT as ::core::ffi::c_int)
                                                            << 8 as ::core::ffi::c_int))
                                                || mod_mask.get()
                                                    & (MOD_MASK_SHIFT | MOD_MASK_CTRL)
                                                    != 0)
                                            && *(*ccline.ptr()).cmdbuff.offset(
                                                ((*ccline.ptr()).cmdpos - 1 as ::core::ffi::c_int)
                                                    as isize,
                                            )
                                                as ::core::ffi::c_int
                                                != ' ' as ::core::ffi::c_int)
                                        {
                                            break;
                                        }
                                    }
                                    (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
                                    if (*ccline.ptr()).special_char as ::core::ffi::c_int != NUL {
                                        putcmdline(
                                            (*ccline.ptr()).special_char,
                                            (*ccline.ptr()).special_shift,
                                        );
                                    }
                                    return command_line_not_changed(s);
                                }
                                -13821 => return command_line_not_changed(s),
                                K_MIDDLEDRAG | K_MIDDLERELEASE => {
                                    return command_line_not_changed(s);
                                }
                                K_MIDDLEMOUSE => {
                                    cmdline_paste(
                                        if eval_has_provider(
                                            b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                                            false_0 != 0,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                        {
                                            '*' as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        },
                                        true_0 != 0,
                                        true_0 != 0,
                                    );
                                    redrawcmd();
                                    return command_line_changed(s);
                                }
                                K_LEFTDRAG | -12029 | K_RIGHTDRAG | -13565 => {
                                    if (*s).ignore_drag_release {
                                        return command_line_not_changed(s);
                                    }
                                    break 'c_46090;
                                }
                                K_LEFTMOUSE => {
                                    break 'c_46090;
                                }
                                K_RIGHTMOUSE => {
                                    break 'c_46093;
                                }
                                K_MOUSEDOWN | K_MOUSEUP | K_MOUSELEFT | K_MOUSERIGHT
                                | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE | K_X2DRAG
                                | K_X2RELEASE | K_MOUSEMOVE => return command_line_not_changed(s),
                                K_SELECT => return command_line_not_changed(s),
                                Ctrl_B | K_HOME | K_KHOME | K_S_HOME | K_C_HOME => {
                                    (*ccline.ptr()).cmdpos = 0 as ::core::ffi::c_int;
                                    (*ccline.ptr()).cmdspos = cmd_startcol();
                                    return command_line_not_changed(s);
                                }
                                Ctrl_E | K_END | K_KEND | K_S_END | K_C_END => {
                                    (*ccline.ptr()).cmdpos = (*ccline.ptr()).cmdlen;
                                    (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
                                    return command_line_not_changed(s);
                                }
                                Ctrl_A => {
                                    if cmdline_pum_active() {
                                        cmdline_pum_cleanup(ccline.ptr());
                                    }
                                    if nextwild(
                                        &raw mut (*s).xpc,
                                        WILD_ALL,
                                        0 as ::core::ffi::c_int,
                                        (*s).firstc != '@' as ::core::ffi::c_int,
                                    ) == FAIL
                                    {
                                        break 's_680;
                                    } else {
                                        (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                                        (*s).did_wild_list = false_0 != 0;
                                        return command_line_changed(s);
                                    }
                                }
                                Ctrl_L => {
                                    if may_add_char_to_search(
                                        (*s).firstc,
                                        &raw mut (*s).c,
                                        &raw mut (*s).is_state,
                                    ) == OK
                                    {
                                        return command_line_not_changed(s);
                                    }
                                    if nextwild(
                                        &raw mut (*s).xpc,
                                        WILD_LONGEST,
                                        0 as ::core::ffi::c_int,
                                        (*s).firstc != '@' as ::core::ffi::c_int,
                                    ) == FAIL
                                    {
                                        break 's_680;
                                    } else {
                                        return command_line_changed(s);
                                    }
                                }
                                Ctrl_N | Ctrl_P => {
                                    if (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int {
                                        let wild_type: ::core::ffi::c_int = if (*s).c == Ctrl_P {
                                            WILD_PREV
                                        } else {
                                            WILD_NEXT
                                        };
                                        if nextwild(
                                            &raw mut (*s).xpc,
                                            wild_type,
                                            0 as ::core::ffi::c_int,
                                            (*s).firstc != '@' as ::core::ffi::c_int,
                                        ) == FAIL
                                        {
                                            break 's_680;
                                        } else {
                                            return command_line_changed(s);
                                        }
                                    }
                                }
                                K_UP | K_DOWN | -1277 | -1533 | K_PAGEUP | K_KPAGEUP
                                | K_PAGEDOWN | K_KPAGEDOWN => {}
                                Ctrl_G | Ctrl_T => {
                                    if may_do_command_line_next_incsearch(
                                        (*s).firstc,
                                        (*s).count,
                                        &raw mut (*s).is_state,
                                        (*s).c == Ctrl_G,
                                    ) == FAIL
                                    {
                                        return command_line_not_changed(s);
                                    }
                                    break 's_680;
                                }
                                Ctrl_V | Ctrl_Q => {
                                    (*s).ignore_drag_release = true_0 != 0;
                                    putcmdline('^' as ::core::ffi::c_char, true_0 != 0);
                                    (*s).c = get_literal(mod_mask.get() & MOD_MASK_SHIFT != 0);
                                    (*s).do_abbr = false_0 != 0;
                                    (*ccline.ptr()).special_char = NUL as ::core::ffi::c_char;
                                    if utf_iscomposing_first((*s).c) as ::core::ffi::c_int != 0
                                        && !cmd_silent.get()
                                    {
                                        if ui_has(kUICmdline) {
                                            unputcmdline();
                                        } else {
                                            draw_cmdline(
                                                (*ccline.ptr()).cmdpos,
                                                (*ccline.ptr()).cmdlen - (*ccline.ptr()).cmdpos,
                                            );
                                            msg_putchar(' ' as ::core::ffi::c_int);
                                            cursorcmd();
                                        }
                                    }
                                    break 's_680;
                                }
                                Ctrl_K => {
                                    (*s).ignore_drag_release = true_0 != 0;
                                    putcmdline('?' as ::core::ffi::c_char, true_0 != 0);
                                    (*s).c = get_digraph(true_0 != 0);
                                    (*ccline.ptr()).special_char = NUL as ::core::ffi::c_char;
                                    if (*s).c != NUL {
                                        break 's_680;
                                    } else {
                                        redrawcmd();
                                        return command_line_not_changed(s);
                                    }
                                }
                                Ctrl__ => {
                                    if p_ari.get() == 0 {
                                        break 's_680;
                                    } else {
                                        return command_line_not_changed(s);
                                    }
                                }
                                113 => {
                                    if !(*ccline.ptr()).mouse_used.is_null() {
                                        *(*ccline.ptr()).cmdbuff = NUL as ::core::ffi::c_char;
                                        return 0 as ::core::ffi::c_int;
                                    }
                                    break 'c_46136;
                                }
                                _ => {
                                    break 'c_46136;
                                }
                            }
                            if cmdline_pum_active() as ::core::ffi::c_int != 0
                                && ((*s).c == K_PAGEUP
                                    || (*s).c == K_PAGEDOWN
                                    || (*s).c == K_KPAGEUP
                                    || (*s).c == K_KPAGEDOWN)
                            {
                                let wild_type_0: ::core::ffi::c_int =
                                    if (*s).c == K_PAGEDOWN || (*s).c == K_KPAGEDOWN {
                                        WILD_PAGEDOWN
                                    } else {
                                        WILD_PAGEUP
                                    };
                                if nextwild(
                                    &raw mut (*s).xpc,
                                    wild_type_0,
                                    0 as ::core::ffi::c_int,
                                    (*s).firstc != '@' as ::core::ffi::c_int,
                                ) == FAIL
                                {
                                    break 's_680;
                                } else {
                                    return command_line_changed(s);
                                }
                            } else {
                                match command_line_browse_history(s) {
                                    2 => {
                                        (*s).did_hist_navigate = true_0 != 0;
                                        return command_line_changed(s);
                                    }
                                    3 => return 0 as ::core::ffi::c_int,
                                    _ => return command_line_not_changed(s),
                                }
                            }
                        }
                        if !(*ccline.ptr()).mouse_used.is_null()
                            && mouse_row.get() < cmdline_row.get()
                        {
                            *(*ccline.ptr()).mouse_used = true_0 != 0;
                            return 0 as ::core::ffi::c_int;
                        }
                        break 'c_46093;
                    }
                    if !((*s).c < 0 as ::core::ffi::c_int) {
                        mod_mask.set(0 as ::core::ffi::c_int);
                    }
                    break 's_680;
                }
                command_line_left_right_mouse(s);
                return command_line_not_changed(s);
            }
            if (*s).do_abbr as ::core::ffi::c_int != 0
                && ((*s).c < 0 as ::core::ffi::c_int || !vim_iswordc((*s).c))
                && (ccheck_abbr(if (*s).c >= 0x100 as ::core::ffi::c_int {
                    (*s).c + ABBR_OFF
                } else {
                    (*s).c
                }) != 0
                    || (*s).c == Ctrl_RSB)
            {
                return command_line_changed(s);
            }
        }
        if (*s).c < 0 as ::core::ffi::c_int || mod_mask.get() != 0 as ::core::ffi::c_int {
            put_on_cmdline(
                get_special_key_name((*s).c, mod_mask.get()),
                -1 as ::core::ffi::c_int,
                true_0 != 0,
            );
        } else {
            let mut j_0: ::core::ffi::c_int =
                utf_char2bytes((*s).c, IObuff.ptr() as *mut ::core::ffi::c_char);
            (*IObuff.ptr())[j_0 as usize] = NUL as ::core::ffi::c_char;
            put_on_cmdline(IObuff.ptr() as *mut ::core::ffi::c_char, j_0, true_0 != 0);
        }
        return if (*ccline.ptr()).one_key as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            command_line_changed(s)
        };
    }
}
