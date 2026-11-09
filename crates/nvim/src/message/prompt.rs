//! The two prompts: hit-enter and the `--More--` pager.
//!
//! [`wait_return`] is what puts `Press ENTER or type command to continue` up
//! after a message that scrolled; [`do_more_prompt`] is the pager the
//! `'more'` option raises when the message area fills. Both run their own key
//! loop, which is why almost nothing else in this module blocks.
//!
//! The two disagree about what "headless" means, and the difference decides
//! which of them a test harness can reach: `do_more_prompt` bails only when
//! there is no embedded UI *and* no UI at all, so an `--embed --headless`
//! child does run the pager, while `wait_return` bails without an attached
//! UI whatever else is true. See docket O-B13-2.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::ascii::NUL;
use crate::event_queue::process_events_in;
use crate::guard::Keys;
use crate::keycodes::{CTRL_B, CTRL_C, CTRL_F, NotAKey, SpecialKey};
use crate::log::logmsg;
use crate::strings::has_char;
use crate::types::Key;
use crate::winlayer::Win;
use core::ffi::c_int;

/// The pager's and hit-enter prompt's own keys, as `c_int` so they can share
/// a `match` with the `K_*` codes -- which are negative, so a `u8` match
/// would alias them onto the ASCII range.
const KEY_SPACE: c_int = Key::ascii(b' ').code();
const KEY_COLON: c_int = Key::ascii(b':').code();
const KEY_UPPER_G: c_int = Key::ascii(b'G').code();
const KEY_B: c_int = Key::ascii(b'b').code();
const KEY_D: c_int = Key::ascii(b'd').code();
const KEY_F: c_int = Key::ascii(b'f').code();
const KEY_G: c_int = Key::ascii(b'g').code();
const KEY_J: c_int = Key::ascii(b'j').code();
const KEY_K: c_int = Key::ascii(b'k').code();
const KEY_Q: c_int = Key::ascii(b'q').code();
const KEY_U: c_int = Key::ascii(b'u').code();

/// Call this after prompting the user: avoids a hit-return message and a
/// delay.
pub fn msg_end_prompt() {
    NEED_WAIT_RETURN.set(false);
    EMSG_ON_DISPLAY.set(false);
    CMDLINE_ROW.set(MSG_ROW.get());
    MSG_COL.set(0);
    msg_clr_eos();
    LINES_LEFT.set(-1);
}

/// Wait for the user to hit a key, normally Enter.
///
/// `redraw` is 1 to redraw the whole screen `UPD_NOT_VALID`, 0 for a normal
/// redraw, and -1 for none at all.
pub fn wait_return(redraw: c_int) {
    if redraw == 1 {
        redraw_all_later(UPD_NOT_VALID);
    }

    if ui_has(UI_MESSAGES) {
        prompt_for_input(Some(c"Press any key to continue"), HLF_M, true, None);
        return;
    }

    // If using ":silent cmd", don't wait for a return.  Also don't set
    // need_wait_return to do it later.
    if MSG_SILENT.get() != 0 {
        return;
    }
    if HEADLESS_MODE.get() && ui_active() == 0 {
        return;
    }
    // Inside vgetc().code() we can't wait for a typed character at all.
    if VGETC_BUSY.get() > 0 {
        return;
    }
    NEED_WAIT_RETURN.set(true);
    if NO_WAIT_RETURN.get() != 0 {
        // With the global command (and some others) we only need one
        // return at the end. Adjust cmdline_row so the next message does
        // not overwrite the last one.
        if !EXMODE_ACTIVE.get() {
            CMDLINE_ROW.set(MSG_ROW.get());
        }
        return;
    }

    REDIR_OFF.set(true); // don't redirect this message
    let old_state = STATE.get();
    let mut c;
    if QUIT_MORE.get() {
        c = CAR; // just pretend CR was hit
        QUIT_MORE.set(false);
        GOT_INT.set(false);
    } else if EXMODE_ACTIVE.get() {
        msg_str(c" "); // make sure the cursor is on the right line
        c = CAR; // no need for a return in ex mode
        GOT_INT.set(false);
    } else if !stuff_empty() {
        // With stuffed characters the next one would dismiss the prompt
        // immediately and have to be put back, so don't show it at all.
        c = CAR;
    } else {
        STATE.set(MODE_HITRETURN);
        setmouse();
        CMDLINE_ROW.set(MSG_ROW.get());
        // Avoid the sequence where the user types ":" at the hit-return
        // prompt to start an Ex command but the file-changed dialog gets
        // in the way.
        if NEED_CHECK_TIMESTAMPS.get() {
            check_timestamps(false);
        }

        // With 'cmdheight' zero we need to scroll the first line of
        // msg_grid onto the screen.
        if p_ch() == 0 && !ui_has(UI_MESSAGES) && MSG_SCROLLED.get() == 0 {
            msg_grid_validate();
            msg_scroll_up(false, true);
            MSG_SCROLLED.set(MSG_SCROLLED.get() + 1);
            CMDLINE_ROW.set(ROWS.get() - 1);
        }

        if MSG_FLAGS.get() & OPT_MOPT_FLAG_HIT_ENTER != 0 {
            hit_return_msg(true);
            loop {
                // Remember "got_int": if it is set vgetc().code() probably
                // answers CTRL-C, and we need to loop then.
                let had_got_int = GOT_INT.get();

                // Don't do mappings here: the character goes back into the
                // typeahead buffer. Recording is disabled for the same
                // reason -- the character is recorded later, when it is
                // added to the typebuf after the loop.
                let raw_key = Keys::unmapped_with_codes();
                let save_reg_recording = REG_RECORDING.get();
                let save_scriptout = SCRIPTOUT.take();
                REG_RECORDING.set(0);
                c = safe_vgetc().code();
                if had_got_int && GLOBAL_BUSY.get() == 0 {
                    GOT_INT.set(false);
                }
                drop(raw_key);
                REG_RECORDING.set(save_reg_recording);
                SCRIPTOUT.set(save_scriptout);

                if p_more() {
                    // Allow scrolling back in the messages. Also accept
                    // scroll-down commands when messages fill the screen,
                    // so one 'j' too many does not make them disappear.
                    if matches!(
                        SpecialKey::try_from(c),
                        Ok(SpecialKey::Up | SpecialKey::Pageup)
                            | Err(NotAKey(KEY_B | CTRL_B | KEY_K | KEY_U | KEY_G))
                    ) {
                        if MSG_SCROLLED.get() > ROWS.get() {
                            // scroll back to show older messages
                            do_more_prompt(c);
                        } else {
                            MSG_DIDOUT.set(false);
                            c = SpecialKey::Ignore.code();
                            MSG_COL.set(0);
                        }
                        if QUIT_MORE.get() {
                            c = CAR; // just pretend CR was hit
                            QUIT_MORE.set(false);
                            GOT_INT.set(false);
                        } else if c != SpecialKey::Ignore.code() {
                            c = SpecialKey::Ignore.code();
                            hit_return_msg(false);
                        }
                    } else if MSG_SCROLLED.get() > ROWS.get() - 2
                        && matches!(
                            SpecialKey::try_from(c),
                            Ok(SpecialKey::Down | SpecialKey::Pagedown)
                                | Err(NotAKey(KEY_J | KEY_D | KEY_F | CTRL_F))
                        )
                    {
                        c = SpecialKey::Ignore.code();
                    }
                }

                let ignored = matches!(
                    SpecialKey::try_from(c),
                    Ok(SpecialKey::Ignore
                        | SpecialKey::Leftdrag
                        | SpecialKey::Leftrelease
                        | SpecialKey::Middledrag
                        | SpecialKey::Middlerelease
                        | SpecialKey::Rightdrag
                        | SpecialKey::Rightrelease
                        | SpecialKey::Mouseleft
                        | SpecialKey::Mouseright
                        | SpecialKey::Mousedown
                        | SpecialKey::Mouseup
                        | SpecialKey::Mousemove)
                );
                if !((had_got_int && c == CTRL_C) || ignored) {
                    break;
                }
            }
            os_breakcheck();

            // Avoid that the mouse-up event causes Visual mode to start.
            if matches!(
                SpecialKey::try_from(c),
                Ok(SpecialKey::Leftmouse
                    | SpecialKey::Middlemouse
                    | SpecialKey::Rightmouse
                    | SpecialKey::X1mouse
                    | SpecialKey::X2mouse)
            ) {
                jump_to_mouse(MOUSE_SETPOS as c_int, None, 0);
            } else if !has_char(c"\r\n ", c) && c != CTRL_C && c != KEY_Q {
                // Put the character back in the typeahead buffer. Not the
                // stuff buffer, because lmaps wouldn't work.
                ins_char_typebuf(VGETC_CHAR.get(), VGETC_MOD_MASK.get(), true);
                DO_REDRAW.set(true); // need a redraw even though there is typeahead
            }
        } else {
            c = CAR;
            // Wait to allow the user to verify the output.
            do_sleep(int64_t::from(MSG_WAIT.get()), true);
        }
    }
    REDIR_OFF.set(false);

    // If the user hits ':', '?' or '/' we get a command line from the next
    // line. It draws over the prompt, so move the cmdline row down to
    // where the prompt was and skip the redraw this tail would queue.
    if c == KEY_COLON || c == c_int::from(b'?') || c == c_int::from(b'/') {
        if !EXMODE_ACTIVE.get() {
            CMDLINE_ROW.set(MSG_ROW.get());
        }
        SKIP_REDRAW.set(true); // skip redraw once
        DO_REDRAW.set(false);
    }

    // If the screen size changed, screen_resize() redraws the screen.
    // Otherwise the screen is only redrawn if 'redraw' is set and no ':'
    // was typed.
    let tmp_state = STATE.get();
    STATE.set(old_state); // restore State before screen_resize()
    setmouse();
    msg_check();
    NEED_WAIT_RETURN.set(false);
    DID_WAIT_RETURN.set(true);
    EMSG_ON_DISPLAY.set(false); // can delete error message now
    LINES_LEFT.set(-1); // reset lines_left at next msg_start()
    reset_last_sourcing();
    let room = (ROWS.get() - CMDLINE_ROW.get() - 1) * COLUMNS.get() + SC_COL.get();
    if KEEP_MSG.with(|kept| {
        kept.as_ref()
            .is_some_and(|kept| vim_strsize_cstr(kept.as_cstr()) >= room)
    }) {
        // Don't redisplay the message, it's too long.
        KEEP_MSG.set(None);
    }

    if tmp_state == MODE_SETWSIZE {
        // got resize event while in vgetc().code()
        ui_refresh();
    } else if !SKIP_REDRAW.get() && (redraw == 1 || (MSG_SCROLLED.get() != 0 && redraw != -1)) {
        redraw_later(Win::current(), UPD_VALID);
    }
}

/// Write the hit-return prompt.
///
/// `newline_sb` is set when starting a new line should add it to the
/// scrollback.
pub(crate) fn hit_return_msg(newline_sb: bool) {
    let save_p_more = p_more();
    if !newline_sb {
        P_MORE.set(false);
    }
    if MSG_DIDOUT.get() {
        msg_putchar(c_int::from(b'\n')); // start on a new line
    }
    P_MORE.set(false); // don't want to see this message when scrolling back
    if GOT_INT.get() {
        msg_str(gettext(c"Interrupt: "));
    }
    let prompt = gettext(c"Press ENTER or type command to continue");
    msg_str_hl(prompt, HLF_R, false);
    if msg_use_printf() == 0 {
        msg_clr_eos();
    }
    P_MORE.set(save_p_more);
}

/// The `--More--` pager: page back and forth through the scrollback.
///
/// `typed_char` is the key that got us here, or NUL to prompt for one.
/// Answers true when the user answered a `:confirm` dialog rather than
/// scrolling.
pub(crate) fn do_more_prompt(typed_char: c_int) -> bool {
    let mut used_typed_char = typed_char;
    let old_state = STATE.get();
    let mut retval = false;
    let mut to_redraw = false;
    // The chunk that starts the line below the screen's last, once the
    // pager has scrolled back; `None` while it shows the newest text.
    let mut mp_last: Option<SbPos> = None;

    // We get called recursively when a timer callback outputs a message.
    // In that case don't show another prompt. Also don't take over a
    // hit-return prompt nobody asked us to.
    let no_need_more = HEADLESS_MODE.get() && !EMBEDDED_MODE.get() && ui_active() == 0;
    if no_need_more || MORE_PROMPT_BUSY.get() || (STATE.get() == MODE_HITRETURN && typed_char == 0)
    {
        return false;
    }
    MORE_PROMPT_BUSY.set(true);

    if typed_char == KEY_UPPER_G {
        // "g<" -- find the first line on the last page.
        mp_last = msg_sb_start(sb_last());
        let mut i = 0;
        while i < ROWS.get() - 2
            && let Some(prev) = mp_last.and_then(sb_prev)
        {
            mp_last = msg_sb_start(Some(prev));
            i += 1;
        }
    }

    STATE.set(MODE_ASKMORE);
    setmouse();
    if typed_char == NUL {
        msg_moremsg(false);
    }

    'more: loop {
        let c = if used_typed_char != NUL {
            let c = used_typed_char;
            used_typed_char = NUL;
            c
        } else {
            get_keystroke(Some(&RESIZE_EVENTS))
        };

        let mut toscroll = 0;
        'scroll: {
            match SpecialKey::try_from(c) {
                Ok(SpecialKey::Bs | SpecialKey::Up) | Err(NotAKey(BS | KEY_K)) => {
                    toscroll = -1;
                    break 'scroll;
                }
                Ok(SpecialKey::Down) | Err(NotAKey(CAR | NL | KEY_J)) => {
                    toscroll = 1;
                    break 'scroll;
                }
                Err(NotAKey(KEY_U)) => {
                    toscroll = -(ROWS.get() / 2);
                    break 'scroll;
                }
                Err(NotAKey(KEY_D)) => {
                    toscroll = ROWS.get() / 2;
                    break 'scroll;
                }
                Ok(SpecialKey::Pageup) | Err(NotAKey(KEY_B | CTRL_B)) => {
                    toscroll = -(ROWS.get() - 1);
                    break 'scroll;
                }
                Ok(SpecialKey::Pagedown | SpecialKey::Leftmouse)
                | Err(NotAKey(KEY_SPACE | KEY_F | CTRL_F)) => {
                    toscroll = ROWS.get() - 1;
                    break 'scroll;
                }
                Err(NotAKey(KEY_G)) => {
                    toscroll = -999999;
                    break 'scroll;
                }
                Err(NotAKey(KEY_UPPER_G)) => {
                    toscroll = 999999;
                    LINES_LEFT.set(999999);
                    break 'scroll;
                }
                Err(NotAKey(KEY_COLON)) => {
                    // Start an Ex command on the next line.
                    if CONFIRM_MSG_USED.get() == 0 {
                        typeahead_noflush(b':');
                        CMDLINE_ROW.set(ROWS.get() - 1);
                        SKIP_REDRAW.set(true);
                        NEED_WAIT_RETURN.set(false);
                    }
                    // falls through to the quit tail
                }
                Err(NotAKey(KEY_Q | CTRL_C | ESC)) => {
                    // falls through to the quit tail
                }
                Ok(SpecialKey::Event) => {
                    // Process the event on the main loop's queue.
                    process_events_in(&RESIZE_EVENTS);
                    to_redraw = true;
                    break 'scroll;
                }
                _ => {
                    // Any other key: show the full prompt and ask again.
                    msg_moremsg(true);
                    continue 'more;
                }
            }
            // Quit: answer a :confirm dialog, or interrupt the output.
            if CONFIRM_MSG_USED.get() != 0 {
                retval = true;
            } else {
                GOT_INT.set(true);
                QUIT_MORE.set(true);
            }
            LINES_LEFT.set(ROWS.get() - 1);
        }

        debug_assert!(toscroll == 0 || !to_redraw);
        if toscroll == 0 && !to_redraw {
            break;
        }

        if toscroll < 0 || to_redraw {
            // Find the line at the top of the screen, and the one
            // `toscroll` lines above it.
            let mut mp = match mp_last {
                None => msg_sb_start(sb_last()),
                Some(last) => msg_sb_start(sb_prev(last)),
            };
            let mut i = 0;
            while i < ROWS.get() - 2
                && let Some(prev) = mp.and_then(sb_prev)
            {
                mp = msg_sb_start(Some(prev));
                i += 1;
            }

            if let Some(top) = mp
                && (sb_prev(top).is_some() || to_redraw)
            {
                // Scroll back to the previous message.
                let mut i = 0;
                while i > toscroll {
                    let Some(prev) = mp.and_then(sb_prev) else {
                        break;
                    };
                    mp = msg_sb_start(Some(prev));
                    mp_last = match mp_last {
                        None => msg_sb_start(sb_last()),
                        Some(last) => msg_sb_start(sb_prev(last)),
                    };
                    i -= 1;
                }

                if toscroll == -1 && !to_redraw {
                    // Display a line at the top, scrolling the rest down.
                    grid_ins_lines(msg_grid_ref(), 0, 1, ROWS.get(), 0, COLUMNS.get());
                    clear_msg_area(0, 1, 0, COLUMNS.get());
                    if let Some(top) = mp {
                        disp_sb_line(0, top);
                    }
                } else {
                    // Redisplay the whole screen.
                    clear_msg_area(0, ROWS.get(), 0, COLUMNS.get());
                    let mut i = 0;
                    while let Some(line) = mp
                        && i < ROWS.get() - 1
                    {
                        mp = disp_sb_line(i, line);
                        MSG_SCROLLED.set(MSG_SCROLLED.get() + 1);
                        i += 1;
                    }
                    to_redraw = false;
                }
                toscroll = 0;
            }
        } else {
            // Scroll forwards.
            if CMDLINE_ROW.get() >= ROWS.get() && !ui_has(UI_MESSAGES) {
                msg_scroll_up(true, false);
                MSG_SCROLLED.set(MSG_SCROLLED.get() + 1);
            }
            while toscroll > 0
                && let Some(line) = mp_last
            {
                // A throttled scroll here would be undone by the flush, so
                // discount it instead.
                if msg_do_throttle() && !msg_grid_ref().throttled {
                    MSG_SCROLLED_AT_FLUSH.set(MSG_SCROLLED_AT_FLUSH.get() - 1);
                    MSG_GRID_SCROLL_DISCOUNT.set(MSG_GRID_SCROLL_DISCOUNT.get() + 1);
                }
                msg_scroll_up(true, false);
                inc_msg_scrolled();
                clear_msg_area(ROWS.get() - 2, ROWS.get() - 1, 0, COLUMNS.get());
                mp_last = disp_sb_line(ROWS.get() - 2, line);
                toscroll -= 1;
            }
        }

        if toscroll > 0 {
            // Displayed the requested number of lines: quit the prompt.
            LINES_LEFT.set(toscroll);
            break;
        }
        clear_msg_area(ROWS.get() - 1, ROWS.get(), 0, COLUMNS.get());
        msg_moremsg(false);
    }

    // Clear the --More-- message.
    clear_msg_area(ROWS.get() - 1, ROWS.get(), 0, COLUMNS.get());
    REDRAW_CMDLINE.set(true);
    CLEAR_CMDLINE.set(false);
    MODE_DISPLAYED.set(false);

    STATE.set(old_state);
    setmouse();
    if QUIT_MORE.get() {
        MSG_ROW.set(ROWS.get() - 1);
        MSG_COL.set(0);
    }

    MORE_PROMPT_BUSY.set(false);
    retval
}

/// Write the `--More--` prompt, with its key legend when `full` is set.
pub(crate) fn msg_moremsg(full: bool) {
    let attr = hl_combine_attr(hl_attr(HLF_MSG as c_int), hl_attr(HLF_M as c_int));
    grid_line_start(msg_grid_view(), ROWS.get() - 1);
    let mut len = grid_line_puts(0, gettext(c"-- More --").to_bytes(), attr);
    if full {
        let keys = c" SPACE/d/j: screen/page/line down, b/u/k: up, q: quit ";
        let help = gettext(keys);
        len += grid_line_puts(len, help.to_bytes(), attr);
    }
    grid_line_cursor_goto(len);
    grid_line_flush();
}

/// The screen was cleared under a prompt: write it again.
pub fn repeat_message() {
    if ui_has(UI_MESSAGES) {
        return;
    }
    if STATE.get() == MODE_ASKMORE {
        msg_moremsg(true); // display --MORE-- message again
        MSG_ROW.set(ROWS.get() - 1);
    } else if STATE.get() & MODE_CMDLINE != 0 && CONFIRM_MSG.with(Option::is_some) {
        display_confirm_msg(); // display ":confirm" message again
        MSG_ROW.set(ROWS.get() - 1);
    } else if STATE.get() == MODE_EXTERNCMD {
        ui_cursor_goto(MSG_ROW.get(), MSG_COL.get()); // put cursor back
    } else if STATE.get() == MODE_HITRETURN || STATE.get() == MODE_SETWSIZE {
        if MSG_ROW.get() == ROWS.get() - 1 {
            // Avoid drawing the "hit-enter" prompt below the last line.
            MSG_DIDOUT.set(false);
            MSG_COL.set(0);
            msg_clr_eos();
        }
        hit_return_msg(false);
        MSG_ROW.set(ROWS.get() - 1);
    }
}

/// Give the user time to see a message, unless the UI shows them itself.
pub fn msg_delay(ms: uint64_t, ignoreinput: bool) {
    if ui_has(UI_MESSAGES) {
        return;
    }
    // Under the test harness a real delay would just be slow.
    let ms = if NVIM_TESTING.get() { 100 } else { ms };
    let note = if NVIM_TESTING.get() {
        " (skipped by NVIM_TEST)"
    } else {
        ""
    };
    logmsg!(LOGLVL_DBG, c"msg_delay", 4047, "{ms} ms{note}");
    ui_flush();
    os_delay(ms, ignoreinput);
}

/// Pause after an error message, so it is not scrolled away unseen.
pub fn msg_check_for_delay(check_msg_scroll: bool) {
    if (EMSG_ON_DISPLAY.get() || (check_msg_scroll && MSG_SCROLL.get() != 0))
        && !DID_WAIT_RETURN.get()
        && EMSG_SILENT.get() == 0
        && !IN_ASSERT_FAILS.get()
        && !ui_has(UI_MESSAGES)
    {
        msg_delay(1006, true);
        EMSG_ON_DISPLAY.set(false);
        if check_msg_scroll {
            MSG_SCROLL.set(0);
        }
    }
}
