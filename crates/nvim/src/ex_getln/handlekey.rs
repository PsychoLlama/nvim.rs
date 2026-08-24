//! The editing keys.
//!
//! [`command_line_handle_key`] is the big switch over every key that is not
//! handled earlier: the cursor motions, the erase keys, the register and
//! digraph insertions, history, and the keys that end the line.  The arms
//! long enough to need it have a helper of their own next to it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{Expanded, WildMode, WildOpts};
use crate::getchar::typeahead;
use crate::guard::Keys;
use crate::keycodes::{
    Ctrl__, Ctrl_A, Ctrl_B, Ctrl_C, Ctrl_D, Ctrl_E, Ctrl_G, Ctrl_H, Ctrl_HAT, Ctrl_K, Ctrl_L,
    Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_T, Ctrl_U, Ctrl_V, Ctrl_W, is_special,
};
use crate::types::{ExpandContext, FAIL, NUL, OK};

/// Handle the erase keys: backspace, delete and CTRL-W.
///
/// Answers [`KeyOutcome::GotoNormalMode`] when erasing emptied a bare `:`
/// line, which leaves the command line altogether.
pub(crate) unsafe fn command_line_erase_chars(s: *mut CommandLineState) -> KeyOutcome {
    unsafe {
        let cc = ccline.ptr();
        if (*s).c == K_KDEL {
            (*s).c = K_DEL;
        }

        // Deleting the current character is the same as a backspace on the
        // next character, except at the end of the line.
        if (*s).c == K_DEL && (*cc).cmdpos != (*cc).cmdlen {
            (*cc).cmdpos += 1;
        }
        if (*s).c == K_DEL {
            (*cc).cmdpos += mb_off_next((*cc).cmdbuff, (*cc).cmdbuff.offset((*cc).cmdpos as isize));
        }

        if (*cc).cmdpos > 0 {
            let mut j = (*cc).cmdpos;
            let mut p = mb_prevptr((*cc).cmdbuff, (*cc).cmdbuff.offset(j as isize));

            if (*s).c == Ctrl_W {
                while p > (*cc).cmdbuff && ascii_isspace(*p as ::core::ffi::c_int) {
                    p = mb_prevptr((*cc).cmdbuff, p);
                }
                let class = mb_get_class(p);
                while p > (*cc).cmdbuff && mb_get_class(p) == class {
                    p = mb_prevptr((*cc).cmdbuff, p);
                }
                if mb_get_class(p) != class {
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            }

            (*cc).cmdpos = p.offset_from((*cc).cmdbuff) as ::core::ffi::c_int;
            (*cc).cmdlen -= j - (*cc).cmdpos;
            let mut i = (*cc).cmdpos;
            while i < (*cc).cmdlen {
                *(*cc).cmdbuff.offset(i as isize) = *(*cc).cmdbuff.offset(j as isize);
                i += 1;
                j += 1;
            }

            // Truncate at the end, required for multi-byte characters.
            *(*cc).cmdbuff.offset((*cc).cmdlen as isize) = NUL as ::core::ffi::c_char;
            if (*cc).cmdlen == 0 {
                (*s).is_state.search_start = (*s).is_state.save_cursor;
                // Save the view settings, so that the screen won't be
                // restored at the wrong position.
                (*s).is_state.old_viewstate = (*s).is_state.init_viewstate;
            }
            redrawcmd();
        } else if (*cc).cmdlen == 0
            && (*s).c != Ctrl_W
            && (*cc).cmdprompt.is_null()
            && (*s).indent == 0
        {
            // In ex and debug mode it doesn't make sense to return.
            if exmode_active.get() || (*cc).cmdfirstc == '>' as ::core::ffi::c_int {
                return KeyOutcome::NotChanged;
            }

            dealloc_cmdbuff(); // no command line to return

            if !cmd_silent.get() && !ui_has(kUICmdline) {
                msg_col.set(0);
                msg_putchar(' ' as ::core::ffi::c_int); // delete ':'
            }
            (*s).is_state.search_start = (*s).is_state.save_cursor;
            redraw_cmdline.set(true);
            return KeyOutcome::GotoNormalMode;
        }
        KeyOutcome::Changed
    }
}

/// Handle CTRL-^: toggle the use of the language `:lmap` mappings and/or the
/// Input Method.
pub(crate) unsafe fn command_line_toggle_langmap(s: *mut CommandLineState) {
    unsafe {
        let b_im_ptr = if buf_valid((*s).b_im_ptr_buf) {
            (*s).b_im_ptr
        } else {
            ::core::ptr::null_mut::<OptInt>()
        };
        if map_to_exists_mode(c"".as_ptr(), MODE_LANGMAP, false) {
            // ":lmap" mappings exist; toggle the use of mappings.
            (*State.ptr()) ^= MODE_LANGMAP;
            if !b_im_ptr.is_null() {
                *b_im_ptr = if State.get() & MODE_LANGMAP != 0 {
                    B_IMODE_LMAP as OptInt
                } else {
                    B_IMODE_NONE as OptInt
                };
            }
        }

        if !b_im_ptr.is_null() {
            if b_im_ptr == &raw mut (*curbuf.get()).b_p_iminsert {
                set_iminsert_global(curbuf.get());
            } else {
                set_imsearch_global(curbuf.get());
            }
        }
        ui_cursor_shape(); // may show a different cursor shape
        // Show/unshow the value of 'keymap' in status lines later.
        status_redraw_curbuf();
    }
}

/// Handle CTRL-R: insert the contents of a numbered or named register.
pub(crate) unsafe fn command_line_insert_reg(s: *mut CommandLineState) -> KeyOutcome {
    unsafe {
        let cc = ccline.ptr();
        let save_new_cmdpos = new_cmdpos.get();

        putcmdline('"' as ::core::ffi::c_char, true);
        let raw_key = Keys::unmapped_with_codes();
        (*s).c = plain_vgetc(); // CTRL-R <char>
        let mut i = (*s).c;
        if i == Ctrl_O {
            i = Ctrl_R; // CTRL-R CTRL-O == CTRL-R CTRL-R
        }
        if i == Ctrl_R {
            (*s).c = plain_vgetc(); // CTRL-R CTRL-R <char>
        }
        drop(raw_key);

        // Insert the result of an expression.
        new_cmdpos.set(-1);
        if (*s).c == '=' as ::core::ffi::c_int {
            if (*cc).cmdfirstc == '=' as ::core::ffi::c_int  // can't do this recursively
                || cmdline_star.get() > 0
            // or when typing a password
            {
                beep_flush();
                (*s).c = ESC;
            } else {
                (*s).c = get_expr_register();
            }
        }

        let mut literally = false;
        if (*s).c != ESC {
            // ESC cancels inserting the register.
            literally = i == Ctrl_R || is_literal_register((*s).c);
            cmdline_paste((*s).c, literally, false);

            // When there was a serious error, abort getting the command line.
            if aborting() {
                // Will free ccline.cmdbuff after putting it in the history.
                (*s).gotesc = true;
                return KeyOutcome::GotoNormalMode;
            }
            KeyTyped.set(false); // don't do 'wildchar' completion
            if new_cmdpos.get() >= 0 {
                // set_cmdline_pos() was used.
                (*cc).cmdpos = (*cc).cmdlen.min(new_cmdpos.get());
            }
        }
        new_cmdpos.set(save_new_cmdpos);

        (*cc).special_char = NUL as ::core::ffi::c_char; // remove the double quote
        redrawcmd();

        // With "literally" the command line has already changed; otherwise the
        // text has been stuffed but the command line has not changed yet.
        if literally {
            KeyOutcome::Changed
        } else {
            KeyOutcome::NotChanged
        }
    }
}

/// Handle a left or right mouse click: put the cursor where it landed.
pub(crate) unsafe fn command_line_left_right_mouse(s: *mut CommandLineState) {
    unsafe {
        let cc = ccline.ptr();
        (*s).ignore_drag_release = (*s).c == K_LEFTRELEASE || (*s).c == K_RIGHTRELEASE;

        (*cc).cmdspos = cmd_startcol();
        (*cc).cmdpos = 0;
        while (*cc).cmdpos < (*cc).cmdlen {
            let cells = cmdline_charsize((*cc).cmdpos);
            if mouse_row.get() <= cmdline_row.get() + (*cc).cmdspos / Columns.get()
                && mouse_col.get() < (*cc).cmdspos % Columns.get() + cells
            {
                break;
            }

            // Count ">" for a double-wide character that doesn't fit.
            correct_screencol((*cc).cmdpos, cells, &raw mut (*cc).cmdspos);
            (*cc).cmdpos += utfc_ptr2len((*cc).cmdbuff.offset((*cc).cmdpos as isize)) - 1;
            (*cc).cmdspos += cells;
            (*cc).cmdpos += 1;
        }
    }
}

/// The big switch over a typed command-line character.
///
/// `Some(rc)` is [`command_line_handle_key`]'s answer.  `None` stands for the
/// C's `break` out of that switch: the key was not handled specially — or its
/// handler asked for it to be treated as ordinary text — and falls through to
/// the abbreviation check and then to inserting it into the line.
unsafe fn command_line_dispatch_key(s: *mut CommandLineState) -> Option<::core::ffi::c_int> {
    unsafe {
        let cc = ccline.ptr();
        match (*s).c {
            K_BS | Ctrl_H | K_DEL | K_KDEL | Ctrl_W => Some(match command_line_erase_chars(s) {
                KeyOutcome::NotChanged => command_line_not_changed(s),
                KeyOutcome::GotoNormalMode => 0, // back to cmd mode
                KeyOutcome::Changed => command_line_changed(s),
            }),

            K_INS | K_KINS => {
                (*cc).overstrike = ((*cc).overstrike == 0) as ::core::ffi::c_int;
                ui_cursor_shape(); // may show a different cursor shape
                may_trigger_modechanged();
                status_redraw_curbuf();
                redraw_statuslines();
                Some(command_line_not_changed(s))
            }

            Ctrl_HAT => {
                command_line_toggle_langmap(s);
                Some(command_line_not_changed(s))
            }

            Ctrl_U => {
                // Delete all characters left of the cursor.
                let mut j = (*cc).cmdpos;
                (*cc).cmdlen -= j;
                (*cc).cmdpos = 0;
                let mut i = 0;
                while i < (*cc).cmdlen {
                    *(*cc).cmdbuff.offset(i as isize) = *(*cc).cmdbuff.offset(j as isize);
                    i += 1;
                    j += 1;
                }

                // Truncate at the end, required for multi-byte characters.
                *(*cc).cmdbuff.offset((*cc).cmdlen as isize) = NUL as ::core::ffi::c_char;
                if (*cc).cmdlen == 0 {
                    (*s).is_state.search_start = (*s).is_state.save_cursor;
                }
                redrawcmd();
                Some(command_line_changed(s))
            }

            // Reached if 'wildchar' is not ESC, or when ESC was typed twice.
            ESC | Ctrl_C => {
                // In exmode it doesn't make sense to return, except when
                // ":normal" runs out of characters. Also, when the highlight
                // callback is active <C-c> should interrupt only that.
                if (exmode_active.get() && (ex_normal_busy.get() == 0 || !typeahead().is_empty()))
                    || (getln_interrupted_highlight.get() && (*s).c == Ctrl_C)
                {
                    getln_interrupted_highlight.set(false);
                    return Some(command_line_not_changed(s));
                }

                // Will free ccline.cmdbuff after putting it in the history.
                (*s).gotesc = true;
                Some(0) // back to cmd mode
            }

            Ctrl_R => Some(match command_line_insert_reg(s) {
                KeyOutcome::GotoNormalMode => 0, // back to cmd mode
                KeyOutcome::Changed => command_line_changed(s),
                KeyOutcome::NotChanged => command_line_not_changed(s),
            }),

            Ctrl_D => {
                if showmatches(
                    &raw mut (*s).xpc,
                    false,
                    true,
                    wim_has(0, kOptWimFlagNoselect),
                ) == Expanded::Nothing
                {
                    return None; // use ^D as a normal character instead
                }
                redrawcmd();
                Some(1) // don't do incremental search now
            }

            K_RIGHT | K_S_RIGHT | K_C_RIGHT => {
                while (*cc).cmdpos < (*cc).cmdlen {
                    let cells = cmdline_charsize((*cc).cmdpos);
                    if KeyTyped.get() && (*cc).cmdspos + cells >= Columns.get() * Rows.get() {
                        break;
                    }
                    (*cc).cmdspos += cells;
                    (*cc).cmdpos += utfc_ptr2len((*cc).cmdbuff.offset((*cc).cmdpos as isize));
                    if !(((*s).c == K_S_RIGHT
                        || (*s).c == K_C_RIGHT
                        || mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0)
                        && *(*cc).cmdbuff.offset((*cc).cmdpos as isize) as ::core::ffi::c_int
                            != ' ' as ::core::ffi::c_int)
                    {
                        break;
                    }
                }
                (*cc).cmdspos = cmd_screencol((*cc).cmdpos);
                Some(command_line_not_changed(s))
            }

            K_LEFT | K_S_LEFT | K_C_LEFT => {
                if (*cc).cmdpos == 0 {
                    return Some(command_line_not_changed(s));
                }
                loop {
                    (*cc).cmdpos -= 1;
                    // Move to the first byte of a possibly multibyte char.
                    (*cc).cmdpos -=
                        utf_head_off((*cc).cmdbuff, (*cc).cmdbuff.offset((*cc).cmdpos as isize));
                    (*cc).cmdspos -= cmdline_charsize((*cc).cmdpos);
                    if !((*cc).cmdpos > 0
                        && ((*s).c == K_S_LEFT
                            || (*s).c == K_C_LEFT
                            || mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0)
                        && *(*cc).cmdbuff.offset(((*cc).cmdpos - 1) as isize) as ::core::ffi::c_int
                            != ' ' as ::core::ffi::c_int)
                    {
                        break;
                    }
                }

                (*cc).cmdspos = cmd_screencol((*cc).cmdpos);
                if (*cc).special_char as ::core::ffi::c_int != NUL {
                    putcmdline((*cc).special_char, (*cc).special_shift);
                }
                Some(command_line_not_changed(s))
            }

            // Ignore a mouse event or an open_cmdwin() result.
            K_IGNORE => Some(command_line_not_changed(s)),

            // Ignore the mouse.
            K_MIDDLEDRAG | K_MIDDLERELEASE => Some(command_line_not_changed(s)),

            K_MIDDLEMOUSE => {
                cmdline_paste(
                    if eval_has_provider(c"clipboard".as_ptr(), false) {
                        '*' as ::core::ffi::c_int
                    } else {
                        0
                    },
                    true,
                    true,
                );
                redrawcmd();
                Some(command_line_changed(s))
            }

            // Three C arms with a FALLTHROUGH between each pair: the
            // drag/release group falls into K_LEFTMOUSE, which falls into
            // K_RIGHTMOUSE.
            K_LEFTDRAG | K_LEFTRELEASE | K_RIGHTDRAG | K_RIGHTRELEASE | K_LEFTMOUSE
            | K_RIGHTMOUSE => {
                // Ignore drag and release events when the button-down wasn't
                // seen before.
                if (*s).c != K_LEFTMOUSE && (*s).c != K_RIGHTMOUSE && (*s).ignore_drag_release {
                    return Some(command_line_not_changed(s));
                }
                // Return on a left click above a number prompt.
                if (*s).c != K_RIGHTMOUSE
                    && !(*cc).mouse_used.is_null()
                    && mouse_row.get() < cmdline_row.get()
                {
                    *(*cc).mouse_used = true;
                    return Some(0);
                }
                command_line_left_right_mouse(s);
                Some(command_line_not_changed(s))
            }

            // The mouse scroll wheel and the alternate buttons are ignored
            // here, as is the end of a Select-mode mapping.
            K_MOUSEDOWN | K_MOUSEUP | K_MOUSELEFT | K_MOUSERIGHT | K_X1MOUSE | K_X1DRAG
            | K_X1RELEASE | K_X2MOUSE | K_X2DRAG | K_X2RELEASE | K_MOUSEMOVE | K_SELECT => {
                Some(command_line_not_changed(s))
            }

            // Beginning of the command line.
            Ctrl_B | K_HOME | K_KHOME | K_S_HOME | K_C_HOME => {
                (*cc).cmdpos = 0;
                (*cc).cmdspos = cmd_startcol();
                Some(command_line_not_changed(s))
            }

            // End of the command line.
            Ctrl_E | K_END | K_KEND | K_S_END | K_C_END => {
                (*cc).cmdpos = (*cc).cmdlen;
                (*cc).cmdspos = cmd_screencol((*cc).cmdpos);
                Some(command_line_not_changed(s))
            }

            Ctrl_A => {
                // all matches
                if cmdline_pum_active() {
                    // As Ctrl-A completes all the matches, close the popup
                    // menu if there is one.
                    cmdline_pum_cleanup(cc);
                }
                if nextwild(
                    &raw mut (*s).xpc,
                    WildMode::All,
                    WildOpts::NONE,
                    (*s).firstc != '@' as ::core::ffi::c_int,
                ) == FAIL
                {
                    return None;
                }
                (*s).xpc.xp_context = ExpandContext::Nothing;
                (*s).did_wild_list = false;
                Some(command_line_changed(s))
            }

            Ctrl_L => {
                if may_add_char_to_search((*s).firstc, &raw mut (*s).c, &raw mut (*s).is_state)
                    == OK
                {
                    return Some(command_line_not_changed(s));
                }
                // Completion: the longest common part.
                if nextwild(
                    &raw mut (*s).xpc,
                    WildMode::Longest,
                    WildOpts::NONE,
                    (*s).firstc != '@' as ::core::ffi::c_int,
                ) == FAIL
                {
                    return None;
                }
                Some(command_line_changed(s))
            }

            // Ctrl_N/Ctrl_P are the next/previous match while completing, and
            // FALL THROUGH into the history keys otherwise.
            Ctrl_N | Ctrl_P | K_UP | K_DOWN | K_S_UP | K_S_DOWN | K_PAGEUP | K_KPAGEUP
            | K_PAGEDOWN | K_KPAGEDOWN => {
                if ((*s).c == Ctrl_N || (*s).c == Ctrl_P) && (*s).xpc.xp_numfiles > 0 {
                    let wild_type = if (*s).c == Ctrl_P {
                        WildMode::Prev
                    } else {
                        WildMode::Next
                    };
                    if nextwild(
                        &raw mut (*s).xpc,
                        wild_type,
                        WildOpts::NONE,
                        (*s).firstc != '@' as ::core::ffi::c_int,
                    ) == FAIL
                    {
                        return None;
                    }
                    return Some(command_line_changed(s));
                }

                if cmdline_pum_active()
                    && ((*s).c == K_PAGEUP
                        || (*s).c == K_PAGEDOWN
                        || (*s).c == K_KPAGEUP
                        || (*s).c == K_KPAGEDOWN)
                {
                    // If the popup menu is displayed, PageUp and PageDown
                    // scroll the menu.
                    let wild_type = if (*s).c == K_PAGEDOWN || (*s).c == K_KPAGEDOWN {
                        WildMode::PageDown
                    } else {
                        WildMode::PageUp
                    };
                    if nextwild(
                        &raw mut (*s).xpc,
                        wild_type,
                        WildOpts::NONE,
                        (*s).firstc != '@' as ::core::ffi::c_int,
                    ) == FAIL
                    {
                        return None;
                    }
                    Some(command_line_changed(s))
                } else {
                    Some(match command_line_browse_history(s) {
                        KeyOutcome::Changed => {
                            (*s).did_hist_navigate = true;
                            command_line_changed(s)
                        }
                        KeyOutcome::GotoNormalMode => 0,
                        KeyOutcome::NotChanged => command_line_not_changed(s),
                    })
                }
            }

            // Next (CTRL-G) or previous (CTRL-T) 'incsearch' match.
            Ctrl_G | Ctrl_T => {
                if may_do_command_line_next_incsearch(
                    (*s).firstc,
                    (*s).count,
                    &raw mut (*s).is_state,
                    (*s).c == Ctrl_G,
                ) == FAIL
                {
                    return Some(command_line_not_changed(s));
                }
                None
            }

            Ctrl_V | Ctrl_Q => {
                (*s).ignore_drag_release = true;
                putcmdline('^' as ::core::ffi::c_char, true);

                // Get the next (two) characters. Do not include the modifiers
                // in the key, for CTRL-SHIFT-V.
                (*s).c = get_literal(mod_mask.get() & MOD_MASK_SHIFT != 0);

                (*s).do_abbr = false; // don't do abbreviation now
                (*cc).special_char = NUL as ::core::ffi::c_char;
                // May need to remove the ^ when a composing char was typed.
                if utf_iscomposing_first((*s).c) && !cmd_silent.get() {
                    if ui_has(kUICmdline) {
                        // TODO(bfredl): why not make unputcmdline also work
                        // with true?
                        unputcmdline();
                    } else {
                        draw_cmdline((*cc).cmdpos, (*cc).cmdlen - (*cc).cmdpos);
                        msg_putchar(' ' as ::core::ffi::c_int);
                        cursorcmd();
                    }
                }
                None
            }

            Ctrl_K => {
                (*s).ignore_drag_release = true;
                putcmdline('?' as ::core::ffi::c_char, true);
                (*s).c = get_digraph(true);
                (*cc).special_char = NUL as ::core::ffi::c_char;

                if (*s).c != NUL {
                    return None;
                }
                redrawcmd();
                Some(command_line_not_changed(s))
            }

            // CTRL-_: switch language mode.
            Ctrl__ => {
                if p_ari.get() == 0 {
                    return None;
                }
                Some(command_line_not_changed(s))
            }

            c => {
                // Number prompts use the mouse and return on a 'q' press;
                // otherwise 'q' FALLS THROUGH to the default arm.
                if c == 'q' as ::core::ffi::c_int && !(*cc).mouse_used.is_null() {
                    *(*cc).cmdbuff = NUL as ::core::ffi::c_char;
                    return Some(0);
                }
                // A normal character with no special meaning. Just set
                // mod_mask to 0x0, so that typing Shift-Space in the GUI
                // doesn't enter the string <S-Space>. This should only happen
                // after ^V.
                if !is_special(c) {
                    mod_mask.set(0);
                }
                None
            }
        }
    }
}

pub(crate) unsafe fn command_line_handle_key(s: *mut CommandLineState) -> ::core::ffi::c_int {
    unsafe {
        let cc = ccline.ptr();

        // For a one-key prompt, avoid putting ESC and Ctrl-C onto the cmdline.
        // For all other keys, just put it onto the cmdline and exit — which is
        // the C's `goto end`.
        if !((*cc).one_key && (*s).c != ESC && (*s).c != Ctrl_C) {
            if let Some(rc) = command_line_dispatch_key(s) {
                return rc;
            }

            // We come here if we have a normal character.
            if (*s).do_abbr
                && (is_special((*s).c) || !vim_iswordc((*s).c))
                // Add ABBR_OFF for characters above 0x100; this is what
                // check_abbr() expects.
                && (ccheck_abbr(if (*s).c >= 0x100 {
                    (*s).c + ABBR_OFF
                } else {
                    (*s).c
                }) || (*s).c == Ctrl_RSB)
            {
                return command_line_changed(s);
            }
        }

        // C's `end:` — put the character in the command line.
        if is_special((*s).c) || mod_mask.get() != 0 {
            put_on_cmdline(get_special_key_name((*s).c, mod_mask.get()), -1, true);
        } else {
            let j = utf_char2bytes((*s).c, IObuff.ptr() as *mut ::core::ffi::c_char);
            (*IObuff.ptr())[j as usize] = NUL as ::core::ffi::c_char; // exclude composing chars
            put_on_cmdline(IObuff.ptr() as *mut ::core::ffi::c_char, j, true);
        }
        if (*cc).one_key {
            0
        } else {
            command_line_changed(s)
        }
    }
}
