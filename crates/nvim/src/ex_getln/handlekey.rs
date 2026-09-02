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
use crate::keycodes::ModMask;
use crate::keycodes::{
    Ctrl__, Ctrl_A, Ctrl_B, Ctrl_C, Ctrl_D, Ctrl_E, Ctrl_G, Ctrl_H, Ctrl_HAT, Ctrl_K, Ctrl_L,
    Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_T, Ctrl_U, Ctrl_V, Ctrl_W, Key, NotAKey,
    is_special,
};
use crate::types::{ExpandContext, FAIL, MB_MAXCHAR, NUL};

/// Handle the erase keys: backspace, delete and CTRL-W.
///
/// Answers [`KeyOutcome::GotoNormalMode`] when erasing emptied a bare `:`
/// line, which leaves the command line altogether.
pub(crate) unsafe fn command_line_erase_chars(mut s: Cls) -> KeyOutcome {
    let mut cc = Cc::current();
    if s.c == Key::Kdel.code() {
        s.c = Key::Del.code();
    }

    // Deleting the current character is the same as a backspace on the
    // next character, except at the end of the line.
    if s.c == Key::Del.code() && cc.cmdpos != cc.len() {
        cc.cmdpos += 1;
    }
    if s.c == Key::Del.code() {
        cc.cmdpos += unsafe { mb_off_next(cc.text(), cc.text().offset(cc.cmdpos as isize)) };
    }

    if cc.cmdpos > 0 {
        let mut j = cc.cmdpos;
        let mut p = unsafe { mb_prevptr(cc.text(), cc.at(j)) };

        if s.c == Ctrl_W {
            while p > cc.text() && ascii_isspace(unsafe { *p } as ::core::ffi::c_int) {
                p = unsafe { mb_prevptr(cc.text(), p) };
            }
            let class = unsafe { mb_get_class(p) };
            while p > cc.text() && unsafe { mb_get_class(p) } == class {
                p = unsafe { mb_prevptr(cc.text(), p) };
            }
            if unsafe { mb_get_class(p) } != class {
                p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
            }
        }

        cc.cmdpos = unsafe { p.offset_from(cc.text()) } as ::core::ffi::c_int;
        // Shift the tail down over the deleted bytes; `set_len` writes the
        // terminator, which is why it comes after the move and not before.
        let tail = (cc.len() - j).max(0) as usize;
        unsafe { ::core::ptr::copy(cc.at(j), cc.at(cc.cmdpos), tail) };
        cc.set_len(cc.len() - (j - cc.cmdpos));
        if cc.is_empty() {
            s.is_state.search_start = s.is_state.save_cursor;
            // Save the view settings, so that the screen won't be
            // restored at the wrong position.
            s.is_state.old_viewstate = s.is_state.init_viewstate;
        }
        unsafe { redrawcmd() };
    } else if cc.is_empty() && s.c != Ctrl_W && cc.cmdprompt.is_null() && s.indent == 0 {
        // In ex and debug mode it doesn't make sense to return.
        if exmode_active.get() || cc.cmdfirstc == '>' as ::core::ffi::c_int {
            return KeyOutcome::NotChanged;
        }

        dealloc_cmdbuff(); // no command line to return

        if !cmd_silent.get() && !ui_has(kUICmdline) {
            msg_col.set(0);
            unsafe { msg_putchar(' ' as ::core::ffi::c_int) }; // delete ':'
        }
        s.is_state.search_start = s.is_state.save_cursor;
        redraw_cmdline.set(true);
        return KeyOutcome::GotoNormalMode;
    }
    KeyOutcome::Changed
}

/// Handle CTRL-^: toggle the use of the language `:lmap` mappings and/or the
/// Input Method.
pub(crate) unsafe fn command_line_toggle_langmap(mut s: Cls) {
    let b_im_ptr = if unsafe { buf_valid(s.b_im_ptr_buf) } {
        s.b_im_ptr
    } else {
        ::core::ptr::null_mut::<OptInt>()
    };
    if unsafe { map_to_exists_mode(c"".as_ptr(), MODE_LANGMAP, false) } {
        // ":lmap" mappings exist; toggle the use of mappings.
        State.set(State.get() ^ MODE_LANGMAP);
        if !b_im_ptr.is_null() {
            unsafe {
                *b_im_ptr = if State.get() & MODE_LANGMAP != 0 {
                    B_IMODE_LMAP as OptInt
                } else {
                    B_IMODE_NONE as OptInt
                }
            };
        }
    }

    if !b_im_ptr.is_null() {
        if b_im_ptr == cur_buf_iminsert() {
            unsafe { set_iminsert_global(curbuf.get()) };
        } else {
            unsafe { set_imsearch_global(curbuf.get()) };
        }
    }
    unsafe { ui_cursor_shape() }; // may show a different cursor shape
    // Show/unshow the value of 'keymap' in status lines later.
    unsafe { status_redraw_curbuf() };
}

/// Handle CTRL-R: insert the contents of a numbered or named register.
pub(crate) unsafe fn command_line_insert_reg(mut s: Cls) -> KeyOutcome {
    let mut cc = Cc::current();
    let save_new_cmdpos = new_cmdpos.get();

    unsafe { putcmdline('"' as ::core::ffi::c_char, true) };
    let raw_key = Keys::unmapped_with_codes();
    s.c = plain_vgetc(); // CTRL-R <char>
    let mut i = s.c;
    if i == Ctrl_O {
        i = Ctrl_R; // CTRL-R CTRL-O == CTRL-R CTRL-R
    }
    if i == Ctrl_R {
        s.c = plain_vgetc(); // CTRL-R CTRL-R <char>
    }
    drop(raw_key);

    // Insert the result of an expression.
    new_cmdpos.set(-1);
    if s.c == '=' as ::core::ffi::c_int {
        if cc.cmdfirstc == '=' as ::core::ffi::c_int  // can't do this recursively
            || cmdline_star.get() > 0
        // or when typing a password
        {
            beep_flush();
            s.c = ESC;
        } else {
            s.c = unsafe { get_expr_register() };
        }
    }

    let mut literally = false;
    if s.c != ESC {
        // ESC cancels inserting the register.
        literally = i == Ctrl_R || is_literal_register(s.c);
        cmdline_paste(s.c, literally, false);

        // When there was a serious error, abort getting the command line.
        if aborting() {
            // Will drop the command line after putting it in the history.
            s.gotesc = true;
            return KeyOutcome::GotoNormalMode;
        }
        KeyTyped.set(false); // don't do 'wildchar' completion
        if new_cmdpos.get() >= 0 {
            // set_cmdline_pos() was used.
            cc.cmdpos = cc.len().min(new_cmdpos.get());
        }
    }
    new_cmdpos.set(save_new_cmdpos);

    cc.special_char = NUL as ::core::ffi::c_char; // remove the double quote
    unsafe { redrawcmd() };

    // With "literally" the command line has already changed; otherwise the
    // text has been stuffed but the command line has not changed yet.
    if literally {
        KeyOutcome::Changed
    } else {
        KeyOutcome::NotChanged
    }
}

/// Handle a left or right mouse click: put the cursor where it landed.
pub(crate) unsafe fn command_line_left_right_mouse(mut s: Cls) {
    let mut cc = Cc::current();
    s.ignore_drag_release = s.c == Key::Leftrelease.code() || s.c == Key::Rightrelease.code();

    cc.cmdspos = cmd_startcol();
    cc.cmdpos = 0;
    while cc.cmdpos < cc.len() {
        let cells = unsafe { cmdline_charsize(cc.cmdpos) };
        if mouse_row.get() <= cmdline_row.get() + cc.cmdspos / Columns.get()
            && mouse_col.get() < cc.cmdspos % Columns.get() + cells
        {
            break;
        }

        // Count ">" for a double-wide character that doesn't fit.
        unsafe { correct_screencol(cc.cmdpos, cells, &raw mut cc.cmdspos) };
        cc.cmdpos += unsafe { utfc_ptr2len(cc.text().offset(cc.cmdpos as isize)) } - 1;
        cc.cmdspos += cells;
        cc.cmdpos += 1;
    }
}

/// The big switch over a typed command-line character.
///
/// `Some(rc)` is [`command_line_handle_key`]'s answer.  `None` stands for the
/// C's `break` out of that switch: the key was not handled specially — or its
/// handler asked for it to be treated as ordinary text — and falls through to
/// the abbreviation check and then to inserting it into the line.
unsafe fn command_line_dispatch_key(mut s: Cls) -> Option<::core::ffi::c_int> {
    let mut cc = Cc::current();
    match Key::try_from(s.c) {
        Ok(Key::Bs | Key::Del | Key::Kdel) | Err(NotAKey(Ctrl_H | Ctrl_W)) => {
            Some(match unsafe { command_line_erase_chars(s) } {
                KeyOutcome::NotChanged => unsafe { command_line_not_changed(s) },
                KeyOutcome::GotoNormalMode => 0, // back to cmd mode
                KeyOutcome::Changed => unsafe { command_line_changed(s) },
            })
        }

        Ok(Key::Ins | Key::Kins) => {
            cc.overstrike = (cc.overstrike == 0) as ::core::ffi::c_int;
            unsafe { ui_cursor_shape() }; // may show a different cursor shape
            unsafe { may_trigger_modechanged() };
            unsafe { status_redraw_curbuf() };
            unsafe { redraw_statuslines() };
            Some(unsafe { command_line_not_changed(s) })
        }

        Err(NotAKey(Ctrl_HAT)) => {
            unsafe { command_line_toggle_langmap(s) };
            Some(unsafe { command_line_not_changed(s) })
        }

        Err(NotAKey(Ctrl_U)) => {
            // Delete all characters left of the cursor.
            // Shift the tail down over everything left of the cursor;
            // `set_len` writes the terminator, so it comes after the move.
            let j = cc.cmdpos;
            let tail = (cc.len() - j).max(0) as usize;
            unsafe { ::core::ptr::copy(cc.at(j), cc.at(0), tail) };
            cc.cmdpos = 0;
            cc.set_len(cc.len() - j);
            if cc.is_empty() {
                s.is_state.search_start = s.is_state.save_cursor;
            }
            unsafe { redrawcmd() };
            Some(unsafe { command_line_changed(s) })
        }

        // Reached if 'wildchar' is not ESC, or when ESC was typed twice.
        Err(NotAKey(ESC | Ctrl_C)) => {
            // In exmode it doesn't make sense to return, except when
            // ":normal" runs out of characters. Also, when the highlight
            // callback is active <C-c> should interrupt only that.
            if (exmode_active.get() && (ex_normal_busy.get() == 0 || !typeahead().is_empty()))
                || (getln_interrupted_highlight.get() && s.c == Ctrl_C)
            {
                getln_interrupted_highlight.set(false);
                return Some(unsafe { command_line_not_changed(s) });
            }

            // Will drop the command line after putting it in the history.
            s.gotesc = true;
            Some(0) // back to cmd mode
        }

        Err(NotAKey(Ctrl_R)) => Some(match unsafe { command_line_insert_reg(s) } {
            KeyOutcome::GotoNormalMode => 0, // back to cmd mode
            KeyOutcome::Changed => unsafe { command_line_changed(s) },
            KeyOutcome::NotChanged => unsafe { command_line_not_changed(s) },
        }),

        Err(NotAKey(Ctrl_D)) => {
            if s.show_matches(false, true, wim_has(0, kOptWimFlagNoselect)) == Expanded::Nothing {
                return None; // use ^D as a normal character instead
            }
            unsafe { redrawcmd() };
            Some(1) // don't do incremental search now
        }

        Ok(Key::Right | Key::SRight | Key::CRight) => {
            while cc.cmdpos < cc.len() {
                let cells = unsafe { cmdline_charsize(cc.cmdpos) };
                if KeyTyped.get() && cc.cmdspos + cells >= Columns.get() * Rows.get() {
                    break;
                }
                cc.cmdspos += cells;
                cc.cmdpos += unsafe { utfc_ptr2len(cc.text().offset(cc.cmdpos as isize)) };
                if !((s.c == Key::SRight.code()
                    || s.c == Key::CRight.code()
                    || mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL))
                    && unsafe { *cc.text().offset(cc.cmdpos as isize) } as ::core::ffi::c_int
                        != ' ' as ::core::ffi::c_int)
                {
                    break;
                }
            }
            cc.cmdspos = unsafe { cmd_screencol(cc.cmdpos) };
            Some(unsafe { command_line_not_changed(s) })
        }

        Ok(Key::Left | Key::SLeft | Key::CLeft) => {
            if cc.cmdpos == 0 {
                return Some(unsafe { command_line_not_changed(s) });
            }
            loop {
                cc.cmdpos -= 1;
                // Move to the first byte of a possibly multibyte char.
                cc.cmdpos -=
                    unsafe { utf_head_off(cc.text(), cc.text().offset(cc.cmdpos as isize)) };
                cc.cmdspos -= unsafe { cmdline_charsize(cc.cmdpos) };
                if !(cc.cmdpos > 0
                    && (s.c == Key::SLeft.code()
                        || s.c == Key::CLeft.code()
                        || mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL))
                    && unsafe { *cc.at(cc.cmdpos - 1) } as ::core::ffi::c_int
                        != ' ' as ::core::ffi::c_int)
                {
                    break;
                }
            }

            cc.cmdspos = unsafe { cmd_screencol(cc.cmdpos) };
            if cc.special_char as ::core::ffi::c_int != NUL {
                unsafe { putcmdline(cc.special_char, cc.special_shift) };
            }
            Some(unsafe { command_line_not_changed(s) })
        }

        // Ignore a mouse event or an open_cmdwin() result.
        Ok(Key::Ignore) => Some(unsafe { command_line_not_changed(s) }),

        // Ignore the mouse.
        Ok(Key::Middledrag | Key::Middlerelease) => Some(unsafe { command_line_not_changed(s) }),

        Ok(Key::Middlemouse) => {
            cmdline_paste(
                if unsafe { eval_has_provider(c"clipboard".as_ptr(), false) } {
                    '*' as ::core::ffi::c_int
                } else {
                    0
                },
                true,
                true,
            );
            unsafe { redrawcmd() };
            Some(unsafe { command_line_changed(s) })
        }

        // Three C arms with a FALLTHROUGH between each pair: the
        // drag/release group falls into K_LEFTMOUSE, which falls into
        // K_RIGHTMOUSE.
        Ok(
            Key::Leftdrag
            | Key::Leftrelease
            | Key::Rightdrag
            | Key::Rightrelease
            | Key::Leftmouse
            | Key::Rightmouse,
        ) => {
            // Ignore drag and release events when the button-down wasn't
            // seen before.
            if s.c != Key::Leftmouse.code()
                && s.c != Key::Rightmouse.code()
                && s.ignore_drag_release
            {
                return Some(unsafe { command_line_not_changed(s) });
            }
            // Return on a left click above a number prompt.
            if s.c != Key::Rightmouse.code()
                && !cc.mouse_used.is_null()
                && mouse_row.get() < cmdline_row.get()
            {
                unsafe { *cc.mouse_used = true };
                return Some(0);
            }
            unsafe { command_line_left_right_mouse(s) };
            Some(unsafe { command_line_not_changed(s) })
        }

        // The mouse scroll wheel and the alternate buttons are ignored
        // here, as is the end of a Select-mode mapping.
        Ok(
            Key::Mousedown
            | Key::Mouseup
            | Key::Mouseleft
            | Key::Mouseright
            | Key::X1mouse
            | Key::X1drag
            | Key::X1release
            | Key::X2mouse
            | Key::X2drag
            | Key::X2release
            | Key::Mousemove
            | Key::Select,
        ) => Some(unsafe { command_line_not_changed(s) }),

        // Beginning of the command line.
        Ok(Key::Home | Key::Khome | Key::SHome | Key::CHome) | Err(NotAKey(Ctrl_B)) => {
            cc.cmdpos = 0;
            cc.cmdspos = cmd_startcol();
            Some(unsafe { command_line_not_changed(s) })
        }

        // End of the command line.
        Ok(Key::End | Key::Kend | Key::SEnd | Key::CEnd) | Err(NotAKey(Ctrl_E)) => {
            cc.cmdpos = cc.len();
            cc.cmdspos = unsafe { cmd_screencol(cc.cmdpos) };
            Some(unsafe { command_line_not_changed(s) })
        }

        Err(NotAKey(Ctrl_A)) => {
            // all matches
            if cmdline_pum_active() {
                // As Ctrl-A completes all the matches, close the popup
                // menu if there is one.
                unsafe { cmdline_pum_cleanup(cc) };
            }
            if s.next_wild(WildMode::All, WildOpts::NONE) == FAIL {
                return None;
            }
            s.xpc.xp_context = ExpandContext::Nothing;
            s.did_wild_list = false;
            Some(unsafe { command_line_changed(s) })
        }

        Err(NotAKey(Ctrl_L)) => {
            let (firstc, is_state) = (s.firstc, s.is_state());
            if unsafe { may_add_char_to_search(firstc, &mut s.c, is_state) }.is_ok() {
                return Some(unsafe { command_line_not_changed(s) });
            }
            // Completion: the longest common part.
            if s.next_wild(WildMode::Longest, WildOpts::NONE) == FAIL {
                return None;
            }
            Some(unsafe { command_line_changed(s) })
        }

        // Ctrl_N/Ctrl_P are the next/previous match while completing, and
        // FALL THROUGH into the history keys otherwise.
        Ok(
            Key::Up
            | Key::Down
            | Key::SUp
            | Key::SDown
            | Key::Pageup
            | Key::Kpageup
            | Key::Pagedown
            | Key::Kpagedown,
        )
        | Err(NotAKey(Ctrl_N | Ctrl_P)) => {
            if (s.c == Ctrl_N || s.c == Ctrl_P) && s.xpc.xp_numfiles > 0 {
                let wild_type = if s.c == Ctrl_P {
                    WildMode::Prev
                } else {
                    WildMode::Next
                };
                if s.next_wild(wild_type, WildOpts::NONE) == FAIL {
                    return None;
                }
                return Some(unsafe { command_line_changed(s) });
            }

            if cmdline_pum_active()
                && (s.c == Key::Pageup.code()
                    || s.c == Key::Pagedown.code()
                    || s.c == Key::Kpageup.code()
                    || s.c == Key::Kpagedown.code())
            {
                // If the popup menu is displayed, PageUp and PageDown
                // scroll the menu.
                let wild_type = if s.c == Key::Pagedown.code() || s.c == Key::Kpagedown.code() {
                    WildMode::PageDown
                } else {
                    WildMode::PageUp
                };
                if s.next_wild(wild_type, WildOpts::NONE) == FAIL {
                    return None;
                }
                Some(unsafe { command_line_changed(s) })
            } else {
                Some(match unsafe { command_line_browse_history(s.raw()) } {
                    KeyOutcome::Changed => {
                        s.did_hist_navigate = true;
                        unsafe { command_line_changed(s) }
                    }
                    KeyOutcome::GotoNormalMode => 0,
                    KeyOutcome::NotChanged => unsafe { command_line_not_changed(s) },
                })
            }
        }

        // Next (CTRL-G) or previous (CTRL-T) 'incsearch' match.
        Err(NotAKey(Ctrl_G | Ctrl_T)) => {
            if unsafe {
                may_do_command_line_next_incsearch(s.firstc, s.count, s.is_state(), s.c == Ctrl_G)
            }
            .is_err()
            {
                return Some(unsafe { command_line_not_changed(s) });
            }
            None
        }

        Err(NotAKey(Ctrl_V | Ctrl_Q)) => {
            s.ignore_drag_release = true;
            unsafe { putcmdline('^' as ::core::ffi::c_char, true) };

            // Get the next (two) characters. Do not include the modifiers
            // in the key, for CTRL-SHIFT-V.
            s.c = unsafe { get_literal(mod_mask.get().has(ModMask::SHIFT)) };

            s.do_abbr = false; // don't do abbreviation now
            cc.special_char = NUL as ::core::ffi::c_char;
            // May need to remove the ^ when a composing char was typed.
            if utf_iscomposing_first(s.c) && !cmd_silent.get() {
                if ui_has(kUICmdline) {
                    // TODO(bfredl): why not make unputcmdline also work
                    // with true?
                    unsafe { unputcmdline() };
                } else {
                    unsafe { draw_cmdline(cc.cmdpos, cc.len() - cc.cmdpos) };
                    unsafe { msg_putchar(' ' as ::core::ffi::c_int) };
                    unsafe { cursorcmd() };
                }
            }
            None
        }

        Err(NotAKey(Ctrl_K)) => {
            s.ignore_drag_release = true;
            unsafe { putcmdline('?' as ::core::ffi::c_char, true) };
            s.c = get_digraph(true);
            cc.special_char = NUL as ::core::ffi::c_char;

            if s.c != NUL {
                return None;
            }
            unsafe { redrawcmd() };
            Some(unsafe { command_line_not_changed(s) })
        }

        // CTRL-_: switch language mode.
        Err(NotAKey(Ctrl__)) => {
            if p_ari.get() == 0 {
                return None;
            }
            Some(unsafe { command_line_not_changed(s) })
        }

        _ => {
            let c = s.c;
            // Number prompts use the mouse and return on a 'q' press;
            // otherwise 'q' FALLS THROUGH to the default arm.
            if c == 'q' as ::core::ffi::c_int && !cc.mouse_used.is_null() {
                unsafe { *cc.at(0) = NUL as ::core::ffi::c_char };
                return Some(0);
            }
            // A normal character with no special meaning. Just set
            // mod_mask to 0x0, so that typing Shift-Space in the GUI
            // doesn't enter the string <S-Space>. This should only happen
            // after ^V.
            if !is_special(c) {
                mod_mask.set(ModMask::NONE);
            }
            None
        }
    }
}

pub(crate) unsafe fn command_line_handle_key(mut s: Cls) -> ::core::ffi::c_int {
    // One character, its own buffer: `put_on_cmdline` reaches the message
    // machinery, which writes upstream's shared `IObuff`.
    let mut ch = [0 as ::core::ffi::c_char; MB_MAXCHAR + 1];
    let mut cc = Cc::current();
    // For a one-key prompt, avoid putting ESC and Ctrl-C onto the cmdline.
    // For all other keys, just put it onto the cmdline and exit — which is
    // the C's `goto end`.
    if !(cc.one_key && s.c != ESC && s.c != Ctrl_C) {
        if let Some(rc) = unsafe { command_line_dispatch_key(s) } {
            return rc;
        }

        // We come here if we have a normal character.
        if s.do_abbr
            && (is_special(s.c) || !unsafe { vim_iswordc(s.c) })
            // Add ABBR_OFF for characters above 0x100; this is what
            // check_abbr() expects.
            && (ccheck_abbr(if s.c >= 0x100 {
                s.c + ABBR_OFF
            } else {
                s.c
            }) || s.c == Ctrl_RSB)
        {
            return unsafe { command_line_changed(s) };
        }
    }

    // C's `end:` — put the character in the command line.
    if is_special(s.c) || !mod_mask.get().is_empty() {
        let name = get_special_key_name(s.c, mod_mask.get());
        unsafe { put_on_cmdline(name.as_ptr().cast_mut(), -1, true) };
    } else {
        let j = unsafe { utf_char2bytes(s.c, ch.as_mut_ptr()) };
        ch[j as usize] = NUL as ::core::ffi::c_char; // exclude composing chars
        unsafe { put_on_cmdline(ch.as_mut_ptr(), j, true) };
    }
    if cc.one_key {
        0
    } else {
        unsafe { command_line_changed(s) }
    }
}
