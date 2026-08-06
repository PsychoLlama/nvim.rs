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

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::{
    Ctrl_B, Ctrl_C, Ctrl_F, K_IGNORE, K_LEFTDRAG, K_LEFTMOUSE, K_LEFTRELEASE, K_MIDDLEDRAG,
    K_MIDDLEMOUSE, K_MIDDLERELEASE, K_MOUSEDOWN, K_MOUSELEFT, K_MOUSEMOVE, K_MOUSERIGHT, K_MOUSEUP,
    K_RIGHTDRAG, K_RIGHTMOUSE, K_RIGHTRELEASE, K_X1MOUSE, K_X2MOUSE,
};
use core::ffi::c_int;
use core::ptr;

/// The pager's and hit-enter prompt's own keys, as `c_int` so they can share
/// a `match` with the `K_*` codes -- which are negative, so a `u8` match
/// would alias them onto the ASCII range.
const KEY_SPACE: c_int = b' ' as c_int;
const KEY_COLON: c_int = b':' as c_int;
const KEY_UPPER_G: c_int = b'G' as c_int;
const KEY_B: c_int = b'b' as c_int;
const KEY_D: c_int = b'd' as c_int;
const KEY_F: c_int = b'f' as c_int;
const KEY_G: c_int = b'g' as c_int;
const KEY_J: c_int = b'j' as c_int;
const KEY_K: c_int = b'k' as c_int;
const KEY_Q: c_int = b'q' as c_int;
const KEY_U: c_int = b'u' as c_int;

/// Call this after prompting the user: avoids a hit-return message and a
/// delay.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_end_prompt() {
    unsafe {
        need_wait_return.set(false);
        emsg_on_display.set(false);
        cmdline_row.set(msg_row.get());
        msg_col.set(0);
        msg_clr_eos();
        lines_left.set(-1);
    }
}

/// Wait for the user to hit a key, normally Enter.
///
/// `redraw` is 1 to redraw the whole screen `UPD_NOT_VALID`, 0 for a normal
/// redraw, and -1 for none at all.
///
/// # Safety
/// Only that the editor is in a state where it can read a key.
pub unsafe fn wait_return(redraw: c_int) {
    unsafe {
        if redraw == true_0 {
            redraw_all_later(UPD_NOT_VALID);
        }

        if ui_has(kUIMessages) {
            prompt_for_input(
                c"Press any key to continue".as_ptr().cast_mut(),
                HLF_M,
                true,
                ptr::null_mut(),
            );
            return;
        }

        // If using ":silent cmd", don't wait for a return.  Also don't set
        // need_wait_return to do it later.
        if msg_silent.get() != 0 {
            return;
        }
        if headless_mode.get() && ui_active() == 0 {
            return;
        }
        // Inside vgetc() we can't wait for a typed character at all.
        if vgetc_busy.get() > 0 {
            return;
        }
        need_wait_return.set(true);
        if no_wait_return.get() != 0 {
            // With the global command (and some others) we only need one
            // return at the end. Adjust cmdline_row so the next message does
            // not overwrite the last one.
            if !exmode_active.get() {
                cmdline_row.set(msg_row.get());
            }
            return;
        }

        redir_off.set(true); // don't redirect this message
        let old_state = State.get();
        let mut c;
        if quit_more.get() {
            c = CAR; // just pretend CR was hit
            quit_more.set(false);
            got_int.set(false);
        } else if exmode_active.get() {
            msg_puts(c" ".as_ptr()); // make sure the cursor is on the right line
            c = CAR; // no need for a return in ex mode
            got_int.set(false);
        } else if !stuff_empty() {
            // With stuffed characters the next one would dismiss the prompt
            // immediately and have to be put back, so don't show it at all.
            c = CAR;
        } else {
            State.set(MODE_HITRETURN);
            setmouse();
            cmdline_row.set(msg_row.get());
            // Avoid the sequence where the user types ":" at the hit-return
            // prompt to start an Ex command but the file-changed dialog gets
            // in the way.
            if need_check_timestamps.get() {
                check_timestamps(false_0);
            }

            // With 'cmdheight' zero we need to scroll the first line of
            // msg_grid onto the screen.
            if p_ch.get() == 0 && !ui_has(kUIMessages) && msg_scrolled.get() == 0 {
                msg_grid_validate();
                msg_scroll_up(false, true);
                msg_scrolled.set(msg_scrolled.get() + 1);
                cmdline_row.set(Rows.get() - 1);
            }

            if msg_flags.get() & kOptMoptFlagHitEnter as c_int != 0 {
                hit_return_msg(true);
                loop {
                    // Remember "got_int": if it is set vgetc() probably
                    // answers CTRL-C, and we need to loop then.
                    let had_got_int = got_int.get();

                    // Don't do mappings here: the character goes back into the
                    // typeahead buffer. Recording is disabled for the same
                    // reason -- the character is recorded later, when it is
                    // added to the typebuf after the loop.
                    no_mapping.set(no_mapping.get() + 1);
                    allow_keys.set(allow_keys.get() + 1);
                    let save_reg_recording = reg_recording.get();
                    let save_scriptout = scriptout.get();
                    reg_recording.set(0);
                    scriptout.set(ptr::null_mut());
                    c = safe_vgetc();
                    if had_got_int && global_busy.get() == 0 {
                        got_int.set(false);
                    }
                    no_mapping.set(no_mapping.get() - 1);
                    allow_keys.set(allow_keys.get() - 1);
                    reg_recording.set(save_reg_recording);
                    scriptout.set(save_scriptout);

                    if p_more.get() != 0 {
                        // Allow scrolling back in the messages. Also accept
                        // scroll-down commands when messages fill the screen,
                        // so one 'j' too many does not make them disappear.
                        if matches!(c, KEY_B | Ctrl_B | KEY_K | KEY_U | KEY_G | K_UP | K_PAGEUP) {
                            if msg_scrolled.get() > Rows.get() {
                                // scroll back to show older messages
                                do_more_prompt(c);
                            } else {
                                msg_didout.set(false);
                                c = K_IGNORE;
                                msg_col.set(0);
                            }
                            if quit_more.get() {
                                c = CAR; // just pretend CR was hit
                                quit_more.set(false);
                                got_int.set(false);
                            } else if c != K_IGNORE {
                                c = K_IGNORE;
                                hit_return_msg(false);
                            }
                        } else if msg_scrolled.get() > Rows.get() - 2
                            && matches!(c, KEY_J | KEY_D | KEY_F | Ctrl_F | K_DOWN | K_PAGEDOWN)
                        {
                            c = K_IGNORE;
                        }
                    }

                    let ignored = matches!(
                        c,
                        K_IGNORE
                            | K_LEFTDRAG
                            | K_LEFTRELEASE
                            | K_MIDDLEDRAG
                            | K_MIDDLERELEASE
                            | K_RIGHTDRAG
                            | K_RIGHTRELEASE
                            | K_MOUSELEFT
                            | K_MOUSERIGHT
                            | K_MOUSEDOWN
                            | K_MOUSEUP
                            | K_MOUSEMOVE
                    );
                    if !((had_got_int && c == Ctrl_C) || ignored) {
                        break;
                    }
                }
                os_breakcheck();

                // Avoid that the mouse-up event causes Visual mode to start.
                if matches!(
                    c,
                    K_LEFTMOUSE | K_MIDDLEMOUSE | K_RIGHTMOUSE | K_X1MOUSE | K_X2MOUSE
                ) {
                    jump_to_mouse(MOUSE_SETPOS as c_int, ptr::null_mut(), 0);
                } else if vim_strchr(c"\r\n ".as_ptr(), c).is_null() && c != Ctrl_C && c != KEY_Q {
                    // Put the character back in the typeahead buffer. Not the
                    // stuff buffer, because lmaps wouldn't work.
                    ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true);
                    do_redraw.set(true); // need a redraw even though there is typeahead
                }
            } else {
                c = CAR;
                // Wait to allow the user to verify the output.
                do_sleep(msg_wait.get() as int64_t, true);
            }
        }
        redir_off.set(false);

        // If the user hits ':', '?' or '/' we get a command line from the next
        // line. It draws over the prompt, so move the cmdline row down to
        // where the prompt was and skip the redraw this tail would queue.
        if c == KEY_COLON || c == b'?' as c_int || c == b'/' as c_int {
            if !exmode_active.get() {
                cmdline_row.set(msg_row.get());
            }
            skip_redraw.set(true); // skip redraw once
            do_redraw.set(false);
        }

        // If the screen size changed, screen_resize() redraws the screen.
        // Otherwise the screen is only redrawn if 'redraw' is set and no ':'
        // was typed.
        let tmp_state = State.get();
        State.set(old_state); // restore State before screen_resize()
        setmouse();
        msg_check();
        need_wait_return.set(false);
        did_wait_return.set(true);
        emsg_on_display.set(false); // can delete error message now
        lines_left.set(-1); // reset lines_left at next msg_start()
        reset_last_sourcing();
        if !keep_msg.get().is_null()
            && vim_strsize(keep_msg.get())
                >= (Rows.get() - cmdline_row.get() - 1) * Columns.get() + sc_col.get()
        {
            // Don't redisplay the message, it's too long.
            xfree(keep_msg.get().cast());
            keep_msg.set(ptr::null_mut());
        }

        if tmp_state == MODE_SETWSIZE {
            // got resize event while in vgetc()
            ui_refresh();
        } else if !skip_redraw.get()
            && (redraw == true_0 || (msg_scrolled.get() != 0 && redraw != -1))
        {
            redraw_later(curwin.get(), UPD_VALID);
        }
    }
}

/// Write the hit-return prompt.
///
/// `newline_sb` is set when starting a new line should add it to the
/// scrollback.
///
/// # Safety
/// Only that the grids are initialised.
pub(crate) unsafe fn hit_return_msg(newline_sb: bool) {
    unsafe {
        let save_p_more = p_more.get();
        if !newline_sb {
            p_more.set(false_0);
        }
        if msg_didout.get() {
            msg_putchar(b'\n' as c_int); // start on a new line
        }
        p_more.set(false_0); // don't want to see this message when scrolling back
        if got_int.get() {
            msg_puts(gettext(c"Interrupt: ".as_ptr()));
        }
        msg_puts_hl(
            gettext(c"Press ENTER or type command to continue".as_ptr()),
            HLF_R,
            false,
        );
        if msg_use_printf() == 0 {
            msg_clr_eos();
        }
        p_more.set(save_p_more);
    }
}

/// The `--More--` pager: page back and forth through the scrollback.
///
/// `typed_char` is the key that got us here, or NUL to prompt for one.
/// Answers true when the user answered a `:confirm` dialog rather than
/// scrolling.
///
/// # Safety
/// Only that the scrollback list and the grids are well formed.
pub(crate) unsafe fn do_more_prompt(typed_char: c_int) -> bool {
    unsafe {
        static entered: GlobalCell<bool> = GlobalCell::new(false);
        let mut used_typed_char = typed_char;
        let old_state = State.get();
        let mut retval = false;
        let mut to_redraw = false;
        let mut mp_last: *mut msgchunk_T = ptr::null_mut();

        // We get called recursively when a timer callback outputs a message.
        // In that case don't show another prompt. Also don't take over a
        // hit-return prompt nobody asked us to.
        let no_need_more = headless_mode.get() && !embedded_mode.get() && ui_active() == 0;
        if no_need_more || entered.get() || (State.get() == MODE_HITRETURN && typed_char == 0) {
            return false;
        }
        entered.set(true);

        if typed_char == KEY_UPPER_G {
            // "g<" -- find the first line on the last page.
            mp_last = msg_sb_start(last_msgchunk.get());
            let mut i = 0;
            while i < Rows.get() - 2 && !mp_last.is_null() && !(*mp_last).sb_prev.is_null() {
                mp_last = msg_sb_start((*mp_last).sb_prev);
                i += 1;
            }
        }

        State.set(MODE_ASKMORE);
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
                get_keystroke(resize_events.get())
            };

            let mut toscroll = 0;
            'scroll: {
                match c {
                    BS | K_BS | KEY_K | K_UP => {
                        toscroll = -1;
                        break 'scroll;
                    }
                    CAR | NL | KEY_J | K_DOWN => {
                        toscroll = 1;
                        break 'scroll;
                    }
                    KEY_U => {
                        toscroll = -(Rows.get() / 2);
                        break 'scroll;
                    }
                    KEY_D => {
                        toscroll = Rows.get() / 2;
                        break 'scroll;
                    }
                    KEY_B | Ctrl_B | K_PAGEUP => {
                        toscroll = -(Rows.get() - 1);
                        break 'scroll;
                    }
                    KEY_SPACE | KEY_F | Ctrl_F | K_PAGEDOWN | K_LEFTMOUSE => {
                        toscroll = Rows.get() - 1;
                        break 'scroll;
                    }
                    KEY_G => {
                        toscroll = -999999;
                        break 'scroll;
                    }
                    KEY_UPPER_G => {
                        toscroll = 999999;
                        lines_left.set(999999);
                        break 'scroll;
                    }
                    KEY_COLON => {
                        // Start an Ex command on the next line.
                        if confirm_msg_used.get() == 0 {
                            typeahead_noflush(KEY_COLON);
                            cmdline_row.set(Rows.get() - 1);
                            skip_redraw.set(true);
                            need_wait_return.set(false);
                        }
                        // falls through to the quit tail
                    }
                    KEY_Q | Ctrl_C | ESC => {
                        // falls through to the quit tail
                    }
                    K_EVENT => {
                        // Process the event on the main loop's queue.
                        multiqueue_process_events(resize_events.get());
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
                if confirm_msg_used.get() != 0 {
                    retval = true;
                } else {
                    got_int.set(true);
                    quit_more.set(true);
                }
                lines_left.set(Rows.get() - 1);
            }

            debug_assert!(toscroll == 0 || !to_redraw);
            if toscroll == 0 && !to_redraw {
                break;
            }

            if toscroll < 0 || to_redraw {
                // Find the line at the top of the screen, and the one
                // `toscroll` lines above it.
                let mut mp = if mp_last.is_null() {
                    msg_sb_start(last_msgchunk.get())
                } else if !(*mp_last).sb_prev.is_null() {
                    msg_sb_start((*mp_last).sb_prev)
                } else {
                    ptr::null_mut()
                };
                let mut i = 0;
                while i < Rows.get() - 2 && !mp.is_null() && !(*mp).sb_prev.is_null() {
                    mp = msg_sb_start((*mp).sb_prev);
                    i += 1;
                }

                if !mp.is_null() && (!(*mp).sb_prev.is_null() || to_redraw) {
                    // Scroll back to the previous message.
                    let mut i = 0;
                    while i > toscroll {
                        if mp.is_null() || (*mp).sb_prev.is_null() {
                            break;
                        }
                        mp = msg_sb_start((*mp).sb_prev);
                        mp_last = if mp_last.is_null() {
                            msg_sb_start(last_msgchunk.get())
                        } else {
                            msg_sb_start((*mp_last).sb_prev)
                        };
                        i -= 1;
                    }

                    if toscroll == -1 && !to_redraw {
                        // Display a line at the top, scrolling the rest down.
                        grid_ins_lines(msg_grid.ptr(), 0, 1, Rows.get(), 0, Columns.get());
                        grid_clear(
                            msg_grid_adj.ptr(),
                            0,
                            1,
                            0,
                            Columns.get(),
                            hl_attr(HLF_MSG as c_int),
                        );
                        disp_sb_line(0, mp);
                    } else {
                        // Redisplay the whole screen.
                        grid_clear(
                            msg_grid_adj.ptr(),
                            0,
                            Rows.get(),
                            0,
                            Columns.get(),
                            hl_attr(HLF_MSG as c_int),
                        );
                        let mut i = 0;
                        while !mp.is_null() && i < Rows.get() - 1 {
                            mp = disp_sb_line(i, mp);
                            msg_scrolled.set(msg_scrolled.get() + 1);
                            i += 1;
                        }
                        to_redraw = false;
                    }
                    toscroll = 0;
                }
            } else {
                // Scroll forwards.
                if cmdline_row.get() >= Rows.get() && !ui_has(kUIMessages) {
                    msg_scroll_up(true, false);
                    msg_scrolled.set(msg_scrolled.get() + 1);
                }
                while toscroll > 0 && !mp_last.is_null() {
                    // A throttled scroll here would be undone by the flush, so
                    // discount it instead.
                    if msg_do_throttle() && !(*msg_grid.ptr()).throttled {
                        msg_scrolled_at_flush.set(msg_scrolled_at_flush.get() - 1);
                        msg_grid_scroll_discount.set(msg_grid_scroll_discount.get() + 1);
                    }
                    msg_scroll_up(true, false);
                    inc_msg_scrolled();
                    grid_clear(
                        msg_grid_adj.ptr(),
                        Rows.get() - 2,
                        Rows.get() - 1,
                        0,
                        Columns.get(),
                        hl_attr(HLF_MSG as c_int),
                    );
                    mp_last = disp_sb_line(Rows.get() - 2, mp_last);
                    toscroll -= 1;
                }
            }

            if toscroll > 0 {
                // Displayed the requested number of lines: quit the prompt.
                lines_left.set(toscroll);
                break;
            }
            grid_clear(
                msg_grid_adj.ptr(),
                Rows.get() - 1,
                Rows.get(),
                0,
                Columns.get(),
                hl_attr(HLF_MSG as c_int),
            );
            msg_moremsg(false);
        }

        // Clear the --More-- message.
        grid_clear(
            msg_grid_adj.ptr(),
            Rows.get() - 1,
            Rows.get(),
            0,
            Columns.get(),
            hl_attr(HLF_MSG as c_int),
        );
        redraw_cmdline.set(true);
        clear_cmdline.set(false);
        mode_displayed.set(false);

        State.set(old_state);
        setmouse();
        if quit_more.get() {
            msg_row.set(Rows.get() - 1);
            msg_col.set(0);
        }

        entered.set(false);
        retval
    }
}

/// Write the `--More--` prompt, with its key legend when `full` is set.
///
/// # Safety
/// Only that the grids are initialised.
pub(crate) unsafe fn msg_moremsg(full: bool) {
    unsafe {
        let attr = hl_combine_attr(hl_attr(HLF_MSG as c_int), hl_attr(HLF_M as c_int));
        grid_line_start(msg_grid_adj.ptr(), Rows.get() - 1);
        let mut len = grid_line_puts(0, gettext(c"-- More --".as_ptr()), -1, attr);
        if full {
            len += grid_line_puts(
                len,
                gettext(c" SPACE/d/j: screen/page/line down, b/u/k: up, q: quit ".as_ptr()),
                -1,
                attr,
            );
        }
        grid_line_cursor_goto(len);
        grid_line_flush();
    }
}

/// The screen was cleared under a prompt: write it again.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn repeat_message() {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        if State.get() == MODE_ASKMORE {
            msg_moremsg(true); // display --MORE-- message again
            msg_row.set(Rows.get() - 1);
        } else if State.get() & MODE_CMDLINE != 0 && !confirm_msg.get().is_null() {
            display_confirm_msg(); // display ":confirm" message again
            msg_row.set(Rows.get() - 1);
        } else if State.get() == MODE_EXTERNCMD {
            ui_cursor_goto(msg_row.get(), msg_col.get()); // put cursor back
        } else if State.get() == MODE_HITRETURN || State.get() == MODE_SETWSIZE {
            if msg_row.get() == Rows.get() - 1 {
                // Avoid drawing the "hit-enter" prompt below the last line.
                msg_didout.set(false);
                msg_col.set(0);
                msg_clr_eos();
            }
            hit_return_msg(false);
            msg_row.set(Rows.get() - 1);
        }
    }
}

/// Give the user time to see a message, unless the UI shows them itself.
///
/// # Safety
/// Only that the editor can pump the event loop here.
pub unsafe fn msg_delay(ms: uint64_t, ignoreinput: bool) {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        // Under the test harness a real delay would just be slow.
        let ms = if nvim_testing.get() { 100 } else { ms };
        logmsg(
            LOGLVL_DBG,
            ptr::null(),
            c"msg_delay".as_ptr(),
            4047,
            true,
            c"%lu ms%s".as_ptr(),
            ms,
            if nvim_testing.get() {
                c" (skipped by NVIM_TEST)".as_ptr()
            } else {
                c"".as_ptr()
            },
        );
        ui_flush();
        os_delay(ms, ignoreinput);
    }
}

/// Pause after an error message, so it is not scrolled away unseen.
///
/// # Safety
/// As [`msg_delay`].
pub unsafe fn msg_check_for_delay(check_msg_scroll: bool) {
    unsafe {
        if (emsg_on_display.get() || (check_msg_scroll && msg_scroll.get() != 0))
            && !did_wait_return.get()
            && emsg_silent.get() == 0
            && !in_assert_fails.get()
            && !ui_has(kUIMessages)
        {
            msg_delay(1006, true);
            emsg_on_display.set(false);
            if check_msg_scroll {
                msg_scroll.set(false_0);
            }
        }
    }
}
