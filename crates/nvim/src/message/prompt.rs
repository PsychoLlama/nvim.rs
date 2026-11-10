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
use crate::event_queue::process_events_in;
use crate::guard::Keys;
use crate::keycodes::SpecialKey;
use crate::log::logmsg;
use crate::register::RegName;
use crate::types::{Key, KeyKind};
use crate::winlayer::Win;
use core::ffi::c_int;

/// The pager's and hit-enter prompt's own letters, matched as keys beside the
/// named ones.
const KEY_SPACE: Key = Key::ascii(b' ');
const KEY_COLON: Key = Key::ascii(b':');
const KEY_QUESTION: Key = Key::ascii(b'?');
const KEY_SLASH: Key = Key::ascii(b'/');
const KEY_UPPER_G: Key = Key::ascii(b'G');
const KEY_B: Key = Key::ascii(b'b');
const KEY_D: Key = Key::ascii(b'd');
const KEY_F: Key = Key::ascii(b'f');
const KEY_G: Key = Key::ascii(b'g');
const KEY_J: Key = Key::ascii(b'j');
const KEY_K: Key = Key::ascii(b'k');
const KEY_Q: Key = Key::ascii(b'q');
const KEY_U: Key = Key::ascii(b'u');

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
        crate::input::prompt_for_key(c"Press any key to continue", HLF_M);
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
    let c = if QUIT_MORE.get() {
        QUIT_MORE.set(false);
        GOT_INT.set(false);
        Key::CAR // just pretend CR was hit
    } else if EXMODE_ACTIVE.get() {
        msg_str(c" "); // make sure the cursor is on the right line
        GOT_INT.set(false);
        Key::CAR // no need for a return in ex mode
    } else if !stuff_empty() {
        // With stuffed characters the next one would dismiss the prompt
        // immediately and have to be put back, so don't show it at all.
        Key::CAR
    } else {
        prompt_hit_return()
    };
    REDIR_OFF.set(false);
    finish_wait_return(c, old_state, redraw);
}

/// Show the hit-return prompt (or pause, without 'more''s `hit-enter`
/// flag) and answer the key that dismissed it.
fn prompt_hit_return() -> Key {
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

    if MSG_FLAGS.get() & OPT_MOPT_FLAG_HIT_ENTER == 0 {
        // Wait to allow the user to verify the output.
        do_sleep(int64_t::from(MSG_WAIT.get()), true);
        return Key::CAR;
    }

    hit_return_msg(true);
    let c = loop {
        // Remember "got_int": if it is set vgetc().code() probably
        // answers CTRL-C, and we need to loop then.
        let had_got_int = GOT_INT.get();
        let c = hit_return_scroll(read_hit_return_key(had_got_int));
        if !((had_got_int && c == Key::CTRL_C) || ignored_at_hit_return(c)) {
            break c;
        }
    };
    os_breakcheck();

    // Avoid that the mouse-up event causes Visual mode to start.
    if matches!(
        c.kind(),
        KeyKind::Special(
            SpecialKey::Leftmouse
                | SpecialKey::Middlemouse
                | SpecialKey::Rightmouse
                | SpecialKey::X1mouse
                | SpecialKey::X2mouse
        )
    ) {
        jump_to_mouse(MOUSE_SETPOS as c_int, None, 0);
    } else if !matches!(c, Key::CAR | Key::NL | KEY_SPACE | Key::CTRL_C | KEY_Q) {
        // Put the character back in the typeahead buffer. Not the
        // stuff buffer, because lmaps wouldn't work.
        ins_char_typebuf(VGETC_CHAR.get(), VGETC_MOD_MASK.get(), true);
        DO_REDRAW.set(true); // need a redraw even though there is typeahead
    }
    c
}

/// One key from the user at the hit-return prompt, unmapped and
/// unrecorded: the character goes back into the typeahead buffer and is
/// mapped and recorded then, once it is added after the prompt.
fn read_hit_return_key(had_got_int: bool) -> Key {
    let raw_key = Keys::unmapped_with_codes();
    let save_reg_recording = REG_RECORDING.get();
    let save_scriptout = SCRIPTOUT.take();
    REG_RECORDING.set(RegName::UNNAMED);
    let c = safe_vgetc();
    if had_got_int && GLOBAL_BUSY.get() == 0 {
        GOT_INT.set(false);
    }
    drop(raw_key);
    REG_RECORDING.set(save_reg_recording);
    SCRIPTOUT.set(save_scriptout);
    c
}

/// With 'more', a scroll key at the hit-return prompt pages back through
/// the messages (or is ignored when they fit), and a scroll-down key is
/// ignored when messages fill the screen, so one 'j' too many does not make
/// them disappear. Answers the key the prompt goes on with.
fn hit_return_scroll(c: Key) -> Key {
    if !p_more() {
        return c;
    }
    if matches!(
        c.kind(),
        KeyKind::Special(SpecialKey::Up | SpecialKey::Pageup)
            | KeyKind::Char(KEY_B | Key::CTRL_B | KEY_K | KEY_U | KEY_G)
    ) {
        let mut c = c;
        if MSG_SCROLLED.get() > ROWS.get() {
            // scroll back to show older messages
            do_more_prompt(c);
        } else {
            MSG_DIDOUT.set(false);
            c = Key::from(SpecialKey::Ignore);
            MSG_COL.set(0);
        }
        if QUIT_MORE.get() {
            QUIT_MORE.set(false);
            GOT_INT.set(false);
            Key::CAR // just pretend CR was hit
        } else {
            if c != SpecialKey::Ignore {
                hit_return_msg(false);
            }
            Key::from(SpecialKey::Ignore)
        }
    } else if MSG_SCROLLED.get() > ROWS.get() - 2
        && matches!(
            c.kind(),
            KeyKind::Special(SpecialKey::Down | SpecialKey::Pagedown)
                | KeyKind::Char(KEY_J | KEY_D | KEY_F | Key::CTRL_F)
        )
    {
        Key::from(SpecialKey::Ignore)
    } else {
        c
    }
}

/// Keys the hit-return prompt waits through: `<Ignore>` and mouse moves,
/// drags and releases.
fn ignored_at_hit_return(c: Key) -> bool {
    matches!(
        c.kind(),
        KeyKind::Special(
            SpecialKey::Ignore
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
                | SpecialKey::Mousemove
        )
    )
}

/// After the hit-return prompt was dismissed with `c`: restore the state
/// it was entered in and queue the redraw `redraw` asks for.
fn finish_wait_return(c: Key, old_state: c_int, redraw: c_int) {
    // If the user hits ':', '?' or '/' we get a command line from the next
    // line. It draws over the prompt, so move the cmdline row down to
    // where the prompt was and skip the redraw this tail would queue.
    if c == KEY_COLON || c == KEY_QUESTION || c == KEY_SLASH {
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
        msg_put_ascii(b'\n'); // start on a new line
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
pub(crate) fn do_more_prompt(typed_char: Key) -> bool {
    let mut used_typed_char = typed_char;
    let old_state = STATE.get();
    let mut retval = false;

    // We get called recursively when a timer callback outputs a message.
    // In that case don't show another prompt. Also don't take over a
    // hit-return prompt nobody asked us to.
    let no_need_more = HEADLESS_MODE.get() && !EMBEDDED_MODE.get() && ui_active() == 0;
    if no_need_more
        || MORE_PROMPT_BUSY.get()
        || (STATE.get() == MODE_HITRETURN && typed_char.is_nul())
    {
        return false;
    }
    MORE_PROMPT_BUSY.set(true);

    let mut pager = Pager {
        mp_last: None,
        to_redraw: false,
    };
    if typed_char == KEY_UPPER_G {
        // "g<" -- find the first line on the last page.
        pager.mp_last = page_above(msg_sb_start(sb_last()));
    }

    STATE.set(MODE_ASKMORE);
    setmouse();
    if typed_char.is_nul() {
        msg_moremsg(false);
    }

    loop {
        let c = if !used_typed_char.is_nul() {
            let c = used_typed_char;
            used_typed_char = Key::NUL;
            c
        } else {
            get_keystroke(Some(&RESIZE_EVENTS))
        };

        let mut toscroll = 0;
        match MoreKey::of(c, ROWS.get()) {
            MoreKey::Scroll(lines) => toscroll = lines,
            MoreKey::ToEnd => {
                toscroll = 999999;
                LINES_LEFT.set(999999);
            }
            MoreKey::Command => {
                // Start an Ex command on the next line.
                if CONFIRM_MSG_USED.get() == 0 {
                    typeahead_noflush(b':');
                    CMDLINE_ROW.set(ROWS.get() - 1);
                    SKIP_REDRAW.set(true);
                    NEED_WAIT_RETURN.set(false);
                }
                retval |= quit_more_prompt();
            }
            MoreKey::Quit => retval |= quit_more_prompt(),
            MoreKey::Event => {
                // Process the event on the main loop's queue.
                process_events_in(&RESIZE_EVENTS);
                pager.to_redraw = true;
            }
            MoreKey::Other => {
                // Any other key: show the full prompt and ask again.
                msg_moremsg(true);
                continue;
            }
        }

        debug_assert!(toscroll == 0 || !pager.to_redraw);
        if toscroll == 0 && !pager.to_redraw {
            break;
        }

        if toscroll < 0 || pager.to_redraw {
            pager.scroll_back(toscroll);
        } else if pager.scroll_forward(toscroll) {
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

/// What a key typed at the `--More--` prompt asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoreKey {
    /// Scroll this many lines: back when negative.
    Scroll(c_int),
    /// `G`: to the newest text, and let it all through.
    ToEnd,
    /// `:`: quit, and start an Ex command unless a dialog is up.
    Command,
    /// `q`, CTRL-C, `<Esc>`: quit.
    Quit,
    /// An event arrived: process it and redraw.
    Event,
    /// Anything else: show the full prompt and ask again.
    Other,
}

impl MoreKey {
    /// `key`'s meaning on a screen of `rows` lines.
    fn of(key: Key, rows: c_int) -> MoreKey {
        match key.kind() {
            KeyKind::Special(SpecialKey::Bs | SpecialKey::Up)
            | KeyKind::Char(Key::CTRL_H | KEY_K) => MoreKey::Scroll(-1),
            KeyKind::Special(SpecialKey::Down)
            | KeyKind::Char(Key::CTRL_M | Key::CTRL_J | KEY_J) => MoreKey::Scroll(1),
            KeyKind::Char(KEY_U) => MoreKey::Scroll(-(rows / 2)),
            KeyKind::Char(KEY_D) => MoreKey::Scroll(rows / 2),
            KeyKind::Special(SpecialKey::Pageup) | KeyKind::Char(KEY_B | Key::CTRL_B) => {
                MoreKey::Scroll(-(rows - 1))
            }
            KeyKind::Special(SpecialKey::Pagedown | SpecialKey::Leftmouse)
            | KeyKind::Char(KEY_SPACE | KEY_F | Key::CTRL_F) => MoreKey::Scroll(rows - 1),
            KeyKind::Char(KEY_G) => MoreKey::Scroll(-999999),
            KeyKind::Char(KEY_UPPER_G) => MoreKey::ToEnd,
            KeyKind::Char(KEY_COLON) => MoreKey::Command,
            KeyKind::Char(KEY_Q | Key::CTRL_C | Key::ESC) => MoreKey::Quit,
            KeyKind::Special(SpecialKey::Event) => MoreKey::Event,
            _ => MoreKey::Other,
        }
    }
}

/// Quit the pager: answer a `:confirm` dialog (true), or interrupt the
/// output.
fn quit_more_prompt() -> bool {
    let answered = CONFIRM_MSG_USED.get() != 0;
    if !answered {
        GOT_INT.set(true);
        QUIT_MORE.set(true);
    }
    LINES_LEFT.set(ROWS.get() - 1);
    answered
}

/// The line `rows - 2` lines above `from` (or the oldest one, when there
/// are fewer): the top of a screen whose last text line is `from`.
fn page_above(from: Option<SbPos>) -> Option<SbPos> {
    let mut mp = from;
    let mut i = 0;
    while i < ROWS.get() - 2
        && let Some(prev) = mp.and_then(sb_prev)
    {
        mp = msg_sb_start(Some(prev));
        i += 1;
    }
    mp
}

/// Where the pager stands in the scrollback.
struct Pager {
    /// The chunk that starts the line below the screen's last, once the
    /// pager has scrolled back; `None` while it shows the newest text.
    mp_last: Option<SbPos>,
    /// An event asked for the whole screen to be shown again.
    to_redraw: bool,
}

impl Pager {
    /// The start of the line the screen shows last.
    fn last_shown(&self) -> Option<SbPos> {
        match self.mp_last {
            None => msg_sb_start(sb_last()),
            Some(last) => msg_sb_start(sb_prev(last)),
        }
    }

    /// Scroll back `-toscroll` lines, or redisplay the screen. Nothing
    /// happens at the oldest line unless a redraw was asked for.
    fn scroll_back(&mut self, toscroll: c_int) {
        // Find the line at the top of the screen, and the one `toscroll`
        // lines above it.
        let mut mp = page_above(self.last_shown());
        if !mp.is_some_and(|top| sb_prev(top).is_some() || self.to_redraw) {
            return;
        }
        // Scroll back to the previous message.
        let mut i = 0;
        while i > toscroll {
            let Some(prev) = mp.and_then(sb_prev) else {
                break;
            };
            mp = msg_sb_start(Some(prev));
            self.mp_last = self.last_shown();
            i -= 1;
        }

        if toscroll == -1 && !self.to_redraw {
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
            self.to_redraw = false;
        }
    }

    /// Scroll forwards `toscroll` lines. Answers true when the scrollback
    /// ran out first: the rest of the output goes through as `lines_left`
    /// and the prompt quits.
    fn scroll_forward(&mut self, mut toscroll: c_int) -> bool {
        if CMDLINE_ROW.get() >= ROWS.get() && !ui_has(UI_MESSAGES) {
            msg_scroll_up(true, false);
            MSG_SCROLLED.set(MSG_SCROLLED.get() + 1);
        }
        while toscroll > 0
            && let Some(line) = self.mp_last
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
            self.mp_last = disp_sb_line(ROWS.get() - 2, line);
            toscroll -= 1;
        }
        if toscroll > 0 {
            // Displayed the requested number of lines: quit the prompt.
            LINES_LEFT.set(toscroll);
            return true;
        }
        false
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_prompt_keys_scroll_by_the_screen() {
        let rows = 24;
        let cases = [
            (Key::from(SpecialKey::Up), MoreKey::Scroll(-1)),
            (KEY_K, MoreKey::Scroll(-1)),
            (Key::CTRL_H, MoreKey::Scroll(-1)),
            (Key::CTRL_M, MoreKey::Scroll(1)),
            (KEY_J, MoreKey::Scroll(1)),
            (KEY_U, MoreKey::Scroll(-12)),
            (KEY_D, MoreKey::Scroll(12)),
            (KEY_B, MoreKey::Scroll(-23)),
            (Key::from(SpecialKey::Pagedown), MoreKey::Scroll(23)),
            (KEY_SPACE, MoreKey::Scroll(23)),
            (Key::from(SpecialKey::Leftmouse), MoreKey::Scroll(23)),
            (KEY_G, MoreKey::Scroll(-999999)),
            (KEY_UPPER_G, MoreKey::ToEnd),
            (KEY_COLON, MoreKey::Command),
            (KEY_Q, MoreKey::Quit),
            (Key::ESC, MoreKey::Quit),
            (Key::CTRL_C, MoreKey::Quit),
            (Key::from(SpecialKey::Event), MoreKey::Event),
            (Key::ascii(b'x'), MoreKey::Other),
            (Key::from(SpecialKey::Ignore), MoreKey::Other),
        ];
        for (key, want) in cases {
            assert_eq!(MoreKey::of(key, rows), want, "{key:?}");
        }
    }

    #[test]
    fn the_hit_return_prompt_waits_through_mouse_motion_only() {
        for key in [
            SpecialKey::Ignore,
            SpecialKey::Leftdrag,
            SpecialKey::Mousemove,
            SpecialKey::Mouseup,
        ] {
            assert!(ignored_at_hit_return(Key::from(key)), "{key:?}");
        }
        for key in [
            Key::from(SpecialKey::Leftmouse),
            Key::CAR,
            KEY_COLON,
            Key::CTRL_C,
        ] {
            assert!(!ignored_at_hit_return(key), "{key:?}");
        }
    }
}
