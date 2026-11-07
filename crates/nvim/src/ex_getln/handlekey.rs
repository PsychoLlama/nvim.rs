//! The editing keys.
//!
//! [`command_line_handle_key`] is the big switch over every key that is not
//! handled earlier: the cursor motions, the erase keys, the register and
//! digraph insertions, history, and the keys that end the line.  The arms
//! long enough to need it have a helper of their own next to it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cmdexpand::{Expanded, WildMode, WildOpts};
use crate::cstr;
use crate::getchar::typeahead;
use crate::guard::Keys;
use crate::keycodes::ModMask;
use crate::keycodes::SpecialKey;
use crate::narrow::len_as_int;
use crate::state::Step;
use crate::types::{ExpandContext, MB_MAXCHAR};
use crate::winlayer::Buf;

/// Where the character before byte `at` of `line` starts: C's
/// `mb_prevptr`, which stays put at the start of the line.
fn prev_char(line: &[u8], at: usize) -> usize {
    if at == 0 {
        0
    } else {
        at - 1 - head_off(line, at - 1)
    }
}

/// Handle the erase keys: backspace, delete and CTRL-W.
///
/// Answers [`KeyOutcome::GotoNormalMode`] when erasing emptied a bare `:`
/// line, which leaves the command line altogether.
pub(crate) fn command_line_erase_chars(s: &mut CommandLineState) -> KeyOutcome {
    let mut cc = Cc::current();
    if s.c == SpecialKey::Kdel {
        s.c = Key::from(SpecialKey::Del);
    }

    // Deleting the current character is the same as a backspace on the
    // next character, except at the end of the line.
    if s.c == SpecialKey::Del && cc.cmdpos != cc.len() {
        cc.cmdpos += 1;
    }
    if s.c == SpecialKey::Del {
        // C's `mb_off_next`: forward to the start of the next character
        // when the cursor is inside one.
        let line = cc.bytes();
        let at = usize::try_from(cc.cmdpos).unwrap_or(0).min(line.len());
        let back = head_off(line, at);
        let fwd = if back == 0 {
            0
        } else {
            c_cluster_len(&line[at - back..]) - back
        };
        cc.cmdpos += len_as_int(fwd);
    }

    if cc.cmdpos > 0 {
        let j = cc.cmdpos;
        let line = cc.bytes();
        let mut p = prev_char(line, usize::try_from(j).unwrap_or(0).min(line.len()));

        if s.c == Key::CTRL_W {
            while p > 0 && ascii_isspace(::core::ffi::c_int::from(line[p])) {
                p = prev_char(line, p);
            }
            let class = class_in(&line[p..], Buf::current());
            while p > 0 && class_in(&line[p..], Buf::current()) == class {
                p = prev_char(line, p);
            }
            if class_in(&line[p..], Buf::current()) != class {
                p += c_cluster_len(&line[p..]);
            }
        }

        cc.cmdpos = len_as_int(p);
        // Take the deleted bytes out; the tail and its terminator move down.
        let pos = cc.cmdpos;
        cc.cmdbuff.splice(pos, j, &[]);
        if cc.is_empty() {
            s.is_state.search_start = s.is_state.save_cursor;
            // Save the view settings, so that the screen won't be
            // restored at the wrong position.
            s.is_state.old_viewstate = s.is_state.init_viewstate;
        }
        redrawcmd();
    } else if cc.is_empty() && s.c != Key::CTRL_W && cc.cmdprompt.is_none() && s.indent == 0 {
        // In ex and debug mode it doesn't make sense to return.
        if exmode_active.get() || cc.cmdfirstc == b'>' {
            return KeyOutcome::NotChanged;
        }

        dealloc_cmdbuff(); // no command line to return

        if !cmd_silent.get() && !ui_has(kUICmdline) {
            msg_col.set(0);
            msg_putchar(::core::ffi::c_int::from(b' ')); // delete ':'
        }
        s.is_state.search_start = s.is_state.save_cursor;
        redraw_cmdline.set(true);
        return KeyOutcome::GotoNormalMode;
    }
    KeyOutcome::Changed
}

/// Handle CTRL-^: toggle the use of the language `:lmap` mappings and/or the
/// Input Method.
pub(crate) fn command_line_toggle_langmap(s: &CommandLineState) {
    let b_im = s
        .b_im
        .filter(|&(_, id)| buf_valid(id))
        .and_then(|(which, id)| id.get().map(|buffer| (which, buffer)));
    if map_to_exists_mode(b"", MODE_LANGMAP, false) {
        // ":lmap" mappings exist; toggle the use of mappings.
        State.set(State.get() ^ MODE_LANGMAP);
        if let Some((which, buffer)) = b_im {
            let value = if State.get() & MODE_LANGMAP != 0 {
                OptInt::from(B_IMODE_LMAP)
            } else {
                OptInt::from(B_IMODE_NONE)
            };
            set_im_value(buffer, which, value);
        }
    }

    if let Some((which, buffer)) = b_im {
        // C compared `b_im_ptr` with `&curbuf->b_p_iminsert`: the global
        // follows `'iminsert'` only when the option is the current buffer's.
        if which == ImOption::Insert && buffer == Buf::current() {
            set_iminsert_global(Buf::current());
        } else {
            set_imsearch_global(Buf::current());
        }
    }
    ui_cursor_shape(); // may show a different cursor shape
    // Show/unshow the value of 'keymap' in status lines later.
    status_redraw_curbuf();
}

/// Handle CTRL-R: insert the contents of a numbered or named register.
pub(crate) fn command_line_insert_reg(s: &mut CommandLineState) -> KeyOutcome {
    let mut cc = Cc::current();
    let save_new_cmdpos = new_cmdpos.get();

    putcmdline(b'"'.cast_signed(), true);
    let raw_key = Keys::unmapped_with_codes();
    s.c = plain_vgetc(); // CTRL-R <char>
    let mut i = s.c;
    if i == Key::CTRL_O {
        i = Key::CTRL_R; // CTRL-R CTRL-O == CTRL-R CTRL-R
    }
    if i == Key::CTRL_R {
        s.c = plain_vgetc(); // CTRL-R CTRL-R <char>
    }
    drop(raw_key);

    // Insert the result of an expression.
    new_cmdpos.set(-1);
    if s.c.is('=') {
        if cc.cmdfirstc == b'='  // can't do this recursively
            || cmdline_star.get() > 0
        // or when typing a password
        {
            beep_flush();
            s.c = Key::ESC;
        } else {
            s.c = if get_expr_register() {
                Key::ascii(b'=')
            } else {
                Key::NUL
            };
        }
    }

    let mut literally = false;
    if s.c != Key::ESC {
        // ESC cancels inserting the register.
        // A key that is not a byte names no register, and pastes nothing.
        let regname = register_name(s.c);
        literally = i == Key::CTRL_R || regname.is_some_and(is_literal_register);
        if let Some(regname) = regname {
            cmdline_paste(regname, literally, false);
        }

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

    cc.special_char = 0; // remove the double quote
    redrawcmd();

    // With "literally" the command line has already changed; otherwise the
    // text has been stuffed but the command line has not changed yet.
    if literally {
        KeyOutcome::Changed
    } else {
        KeyOutcome::NotChanged
    }
}

/// Handle a left or right mouse click: put the cursor where it landed.
pub(crate) fn command_line_left_right_mouse(s: &mut CommandLineState) {
    let mut cc = Cc::current();
    s.ignore_drag_release = s.c == SpecialKey::Leftrelease || s.c == SpecialKey::Rightrelease;

    cc.cmdspos = cmd_startcol();
    cc.cmdpos = 0;
    while cc.cmdpos < cc.len() {
        let cells = cmdline_charsize(cc.cmdpos);
        if mouse_row.get() <= cmdline_row.get() + cc.cmdspos / Columns.get()
            && mouse_col.get() < cc.cmdspos % Columns.get() + cells
        {
            break;
        }

        // Count ">" for a double-wide character that doesn't fit.
        cc.cmdspos = correct_screencol(cc.cmdpos, cells, cc.cmdspos);
        cc.cmdpos += cluster_len_at(cc, cc.cmdpos) - 1;
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
fn command_line_dispatch_key(s: &mut CommandLineState) -> Option<Step> {
    let mut cc = Cc::current();
    match s.c.special().ok_or(s.c) {
        Ok(SpecialKey::Bs | SpecialKey::Del | SpecialKey::Kdel)
        | Err(Key::CTRL_H | Key::CTRL_W) => {
            Some(match command_line_erase_chars(s) {
                KeyOutcome::NotChanged => command_line_not_changed(s),
                KeyOutcome::GotoNormalMode => Step::Leave, // back to cmd mode
                KeyOutcome::Changed => command_line_changed(s),
            })
        }

        Ok(SpecialKey::Ins | SpecialKey::Kins) => {
            cc.overstrike = !cc.overstrike;
            ui_cursor_shape(); // may show a different cursor shape
            may_trigger_modechanged();
            status_redraw_curbuf();
            redraw_statuslines();
            Some(command_line_not_changed(s))
        }

        Err(Key::CTRL_HAT) => {
            command_line_toggle_langmap(s);
            Some(command_line_not_changed(s))
        }

        Err(Key::CTRL_U) => {
            // Delete all characters left of the cursor.
            // Shift the tail down over everything left of the cursor;
            // `set_len` writes the terminator, so it comes after the move.
            let j = cc.cmdpos;
            cc.cmdbuff.splice(0, j, &[]);
            cc.cmdpos = 0;
            if cc.is_empty() {
                s.is_state.search_start = s.is_state.save_cursor;
            }
            redrawcmd();
            Some(command_line_changed(s))
        }

        // Reached if 'wildchar' is not ESC, or when ESC was typed twice.
        Err(Key::ESC | Key::CTRL_C) => {
            // In exmode it doesn't make sense to return, except when
            // ":normal" runs out of characters. Also, when the highlight
            // callback is active <C-c> should interrupt only that.
            if (exmode_active.get() && (ex_normal_busy.get() == 0 || !typeahead().is_empty()))
                || (getln_interrupted_highlight.get() && s.c == Key::CTRL_C)
            {
                getln_interrupted_highlight.set(false);
                return Some(command_line_not_changed(s));
            }

            // Will drop the command line after putting it in the history.
            s.gotesc = true;
            Some(Step::Leave) // back to cmd mode
        }

        Err(Key::CTRL_R) => Some(match command_line_insert_reg(s) {
            KeyOutcome::GotoNormalMode => Step::Leave, // back to cmd mode
            KeyOutcome::Changed => command_line_changed(s),
            KeyOutcome::NotChanged => command_line_not_changed(s),
        }),

        Err(Key::CTRL_D) => {
            if show_matches(false, true, wim_has(0, kOptWimFlagNoselect)) == Expanded::Nothing {
                return None; // use ^D as a normal character instead
            }
            redrawcmd();
            Some(Step::Go) // don't do incremental search now
        }

        Ok(SpecialKey::Right | SpecialKey::SRight | SpecialKey::CRight) => {
            while cc.cmdpos < cc.len() {
                let cells = cmdline_charsize(cc.cmdpos);
                if KeyTyped.get() && cc.cmdspos + cells >= Columns.get() * Rows.get() {
                    break;
                }
                cc.cmdspos += cells;
                cc.cmdpos += cluster_len_at(cc, cc.cmdpos);
                if !((s.c == SpecialKey::SRight
                    || s.c == SpecialKey::CRight
                    || mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL))
                    && cc
                        .cmdbuff
                        .raw_byte_at(usize::try_from(cc.cmdpos).unwrap_or(0))
                        != b' ')
                {
                    break;
                }
            }
            cc.cmdspos = cmd_screencol(cc.cmdpos);
            Some(command_line_not_changed(s))
        }

        Ok(SpecialKey::Left | SpecialKey::SLeft | SpecialKey::CLeft) => {
            if cc.cmdpos == 0 {
                return Some(command_line_not_changed(s));
            }
            loop {
                cc.cmdpos -= 1;
                // Move to the first byte of a possibly multibyte char.
                cc.cmdpos -= len_as_int(head_off(
                    cc.bytes(),
                    usize::try_from(cc.cmdpos).unwrap_or(0),
                ));
                cc.cmdspos -= cmdline_charsize(cc.cmdpos);
                if !(cc.cmdpos > 0
                    && (s.c == SpecialKey::SLeft
                        || s.c == SpecialKey::CLeft
                        || mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL))
                    && cc.byte_at(cc.cmdpos - 1) != b' ')
                {
                    break;
                }
            }

            cc.cmdspos = cmd_screencol(cc.cmdpos);
            if cc.special_char != 0 {
                putcmdline(cc.special_char, cc.special_shift);
            }
            Some(command_line_not_changed(s))
        }

        // Ignore a mouse event or an open_cmdwin() result.
        Ok(SpecialKey::Ignore) => Some(command_line_not_changed(s)),

        // Ignore the mouse.
        Ok(SpecialKey::Middledrag | SpecialKey::Middlerelease) => Some(command_line_not_changed(s)),

        Ok(SpecialKey::Middlemouse) => {
            cmdline_paste(
                if eval_has_provider(c"clipboard") {
                    b'*'
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
        Ok(
            SpecialKey::Leftdrag
            | SpecialKey::Leftrelease
            | SpecialKey::Rightdrag
            | SpecialKey::Rightrelease
            | SpecialKey::Leftmouse
            | SpecialKey::Rightmouse,
        ) => {
            // Ignore drag and release events when the button-down wasn't
            // seen before.
            if s.c != SpecialKey::Leftmouse
                && s.c != SpecialKey::Rightmouse
                && s.ignore_drag_release
            {
                return Some(command_line_not_changed(s));
            }
            // Return on a left click above a number prompt.
            if s.c != SpecialKey::Rightmouse
                && cc.mouse_prompt
                && mouse_row.get() < cmdline_row.get()
            {
                cc.mouse_used = true;
                return Some(Step::Leave);
            }
            command_line_left_right_mouse(s);
            Some(command_line_not_changed(s))
        }

        // The mouse scroll wheel and the alternate buttons are ignored
        // here, as is the end of a Select-mode mapping.
        Ok(
            SpecialKey::Mousedown
            | SpecialKey::Mouseup
            | SpecialKey::Mouseleft
            | SpecialKey::Mouseright
            | SpecialKey::X1mouse
            | SpecialKey::X1drag
            | SpecialKey::X1release
            | SpecialKey::X2mouse
            | SpecialKey::X2drag
            | SpecialKey::X2release
            | SpecialKey::Mousemove
            | SpecialKey::Select,
        ) => Some(command_line_not_changed(s)),

        // Beginning of the command line.
        Ok(SpecialKey::Home | SpecialKey::Khome | SpecialKey::SHome | SpecialKey::CHome)
        | Err(Key::CTRL_B) => {
            cc.cmdpos = 0;
            cc.cmdspos = cmd_startcol();
            Some(command_line_not_changed(s))
        }

        // End of the command line.
        Ok(SpecialKey::End | SpecialKey::Kend | SpecialKey::SEnd | SpecialKey::CEnd)
        | Err(Key::CTRL_E) => {
            cc.cmdpos = cc.len();
            cc.cmdspos = cmd_screencol(cc.cmdpos);
            Some(command_line_not_changed(s))
        }

        Err(Key::CTRL_A) => {
            // all matches
            if cmdline_pum_active() {
                // As Ctrl-A completes all the matches, close the popup
                // menu if there is one.
                cmdline_pum_cleanup(cc);
            }
            if !s.next_wild(WildMode::All, WildOpts::NONE) {
                return None;
            }
            completion_mut(|xpc| xpc.context = ExpandContext::Nothing);
            s.did_wild_list = false;
            Some(command_line_changed(s))
        }

        Err(Key::CTRL_L) => {
            if may_add_char_to_search(s.firstc, &mut s.c, &mut s.is_state).is_ok() {
                return Some(command_line_not_changed(s));
            }
            // Completion: the longest common part.
            if !s.next_wild(WildMode::Longest, WildOpts::NONE) {
                return None;
            }
            Some(command_line_changed(s))
        }

        // Ctrl_N/Ctrl_P are the next/previous match while completing, and
        // FALL THROUGH into the history keys otherwise.
        Ok(
            SpecialKey::Up
            | SpecialKey::Down
            | SpecialKey::SUp
            | SpecialKey::SDown
            | SpecialKey::Pageup
            | SpecialKey::Kpageup
            | SpecialKey::Pagedown
            | SpecialKey::Kpagedown,
        )
        | Err(Key::CTRL_N | Key::CTRL_P) => {
            if (s.c == Key::CTRL_N || s.c == Key::CTRL_P) && match_count() > 0 {
                let wild_type = if s.c == Key::CTRL_P {
                    WildMode::Prev
                } else {
                    WildMode::Next
                };
                if !s.next_wild(wild_type, WildOpts::NONE) {
                    return None;
                }
                return Some(command_line_changed(s));
            }

            if cmdline_pum_active()
                && (s.c == SpecialKey::Pageup
                    || s.c == SpecialKey::Pagedown
                    || s.c == SpecialKey::Kpageup
                    || s.c == SpecialKey::Kpagedown)
            {
                // If the popup menu is displayed, PageUp and PageDown
                // scroll the menu.
                let wild_type = if s.c == SpecialKey::Pagedown || s.c == SpecialKey::Kpagedown {
                    WildMode::PageDown
                } else {
                    WildMode::PageUp
                };
                if !s.next_wild(wild_type, WildOpts::NONE) {
                    return None;
                }
                Some(command_line_changed(s))
            } else {
                Some(match command_line_browse_history(s) {
                    KeyOutcome::Changed => {
                        s.did_hist_navigate = true;
                        command_line_changed(s)
                    }
                    KeyOutcome::GotoNormalMode => Step::Leave,
                    KeyOutcome::NotChanged => command_line_not_changed(s),
                })
            }
        }

        // Next (CTRL-G) or previous (CTRL-T) 'incsearch' match.
        Err(Key::CTRL_G | Key::CTRL_T) => {
            let next = s.c == Key::CTRL_G;
            if may_do_command_line_next_incsearch(s.firstc, s.count, &mut s.is_state, next).is_err()
            {
                return Some(command_line_not_changed(s));
            }
            None
        }

        Err(Key::CTRL_V | Key::CTRL_Q) => {
            s.ignore_drag_release = true;
            putcmdline(b'^'.cast_signed(), true);

            // Get the next (two) characters. Do not include the modifiers
            // in the key, for CTRL-SHIFT-V.
            s.c = get_literal(mod_mask.get().has(ModMask::SHIFT));

            s.do_abbr = false; // don't do abbreviation now
            cc.special_char = 0;
            // May need to remove the ^ when a composing char was typed.
            if utf_iscomposing_first(s.c.code()) && !cmd_silent.get() {
                if ui_has(kUICmdline) {
                    // TODO(bfredl): why not make unputcmdline also work
                    // with true?
                    unputcmdline();
                } else {
                    draw_cmdline(cc.cmdpos, cc.len() - cc.cmdpos);
                    msg_putchar(::core::ffi::c_int::from(b' '));
                    cursorcmd();
                }
            }
            None
        }

        Err(Key::CTRL_K) => {
            s.ignore_drag_release = true;
            putcmdline(b'?'.cast_signed(), true);
            s.c = get_digraph(true);
            cc.special_char = 0;

            if !s.c.is_nul() {
                return None;
            }
            redrawcmd();
            Some(command_line_not_changed(s))
        }

        // CTRL-_: switch language mode.
        Err(Key::CTRL_UNDERSCORE) => {
            if !p_ari() {
                return None;
            }
            Some(command_line_not_changed(s))
        }

        _ => {
            let c = s.c;
            // Number prompts use the mouse and return on a 'q' press;
            // otherwise 'q' FALLS THROUGH to the default arm.
            if c.is('q') && cc.mouse_prompt {
                cc.cmdbuff.set_byte(0, 0);
                return Some(Step::Leave);
            }
            // A normal character with no special meaning. Just set
            // mod_mask to 0x0, so that typing Shift-Space in the GUI
            // doesn't enter the string <S-Space>. This should only happen
            // after ^V.
            if !c.is_special() {
                mod_mask.set(ModMask::NONE);
            }
            None
        }
    }
}

pub(crate) fn command_line_handle_key(s: &mut CommandLineState) -> Step {
    let cc = Cc::current();
    // For a one-key prompt, avoid putting ESC and Ctrl-C onto the cmdline.
    // For all other keys, just put it onto the cmdline and exit — which is
    // the C's `goto end`.
    if !(cc.one_key && s.c != Key::ESC && s.c != Key::CTRL_C) {
        if let Some(rc) = command_line_dispatch_key(s) {
            return rc;
        }

        // We come here if we have a normal character.
        if s.do_abbr
            && (s.c.is_special() || !vim_iswordc(s.c.code()))
            // A character above 0x100 needs no CTRL-V in front of it.
            && (ccheck_abbr(s.c, s.c.is_char() && s.c.as_byte().is_none())
                || s.c == Key::CTRL_RSB)
        {
            return command_line_changed(s);
        }
    }

    // C's `end:` — put the character in the command line.
    if s.c.is_special() || !mod_mask.get().is_empty() {
        let name = get_special_key_name(s.c, mod_mask.get());
        put_on_cmdline(cstr::in_bytes(&name).to_bytes(), true);
    } else {
        // One character, its own buffer, without its composing characters.
        let mut ch = [0u8; MB_MAXCHAR];
        let j = encode_char(s.c.code(), &mut ch);
        put_on_cmdline(&ch[..j], true);
    }
    if Cc::current().one_key {
        Step::Leave
    } else {
        command_line_changed(s)
    }
}

/// The length of the character cluster at byte `i` of the command line: C's
/// `utfc_ptr2len(cmdbuff + i)`, 0 at the end.
pub(crate) fn cluster_len_at(cc: Cc, i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let line = cc.bytes();
    let at = usize::try_from(i).unwrap_or(0).min(line.len());
    let rest = &line[at..];
    // An ASCII character with no composing character after it: what nearly
    // every key typed on the command line is.
    match rest {
        [] => 0,
        [first, next, ..] if (1..0x80).contains(first) && *next < 0x80 => 1,
        [first] if (1..0x80).contains(first) => 1,
        _ => len_as_int(c_cluster_len(rest)),
    }
}
