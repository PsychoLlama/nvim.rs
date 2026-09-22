//! The `z=` command: ask which suggestion to use, and make the change.
//!
//! This is the only interactive part of the spell-suggestion machinery.
//! Everything else in this module tree produces a list; here that list is
//! shown, numbered, and one entry of it is put into the buffer.
//!
//! A run is four steps: decide which word is the bad one, ask
//! [`spell_find_suggest`] what could replace it, get a number out of the
//! user, and edit the line. The number can also come in as `z=`'s count,
//! in which case nothing is shown and nothing is asked.
//!
//! # Two states that have to be put back
//!
//! `z=` works even where `'spell'` is off: it turns the option on, which
//! is what loads `'spelllang'`, and has to turn it back off again on every
//! way out — including the three early ones. [`spell_suggest`] is
//! therefore a wrapper whose only job is that restore, around
//! [`suggest_and_replace`], which is free to return where the C used a
//! `goto`.
//!
//! `'more'` is suppressed the same way while the list is drawn, and
//! restored from the value read before anything was printed.
//!
//! # What the replacement leaves behind
//!
//! Besides the changed line, an accepted suggestion sets `repl_from` and
//! `repl_to` so that `:spellrepall` can repeat it over the whole buffer,
//! and fills the redo buffer with `ciw{word}<Esc>` so that `.` can.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::change::inserted_bytes;
use crate::cstr;
use crate::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::drawscreen::state::cmdline_row;
use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, append_to_redobuff_literally, beep_flush,
    reset_redobuff,
};
use crate::input::prompt_for_input;
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memline::ml_replace;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::e_no_spell;
use crate::message::state::{cmdmsg_rl, lines_left, msg_col, msg_row, msg_scroll};
use crate::message::{
    emsg, msg, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_start,
};
use crate::message_fmt::{msg_bytes, msg_text};
use crate::mouse::state::mouse_row;
use crate::normal::{end_visual_mode, visual_active, visual_anchor};
use crate::option::vars::p_verbose;
use crate::options::kOptBoFlagSpell;
use crate::optionstr::LocalOptStr;
use crate::os::cshim::gettext;
use crate::search::FORWARD;
use crate::spell::{
    SMT_ALL, check_need_cap, parse_spelllang, repl_from, repl_to, spell_iswordp_nmw, spell_move_to,
};
use crate::spellsuggest::{
    MAXWLEN, SPS_BEST, SPS_DOUBLE, Sug, SugInfo, Suggest, spell_find_cleanup, spell_find_suggest,
    spell_suggest_timeout, sps_flags, sps_limit,
};
use crate::strings::xstrnsave;
use crate::types::ui::kUIMessages;
use crate::types::{ColNr, IOSIZE, NUL, Pos, int64_t};
use crate::ui::state::Rows;
use crate::ui::{ui_has, vim_beep};
use crate::undo::u_save_cursor;
use crate::winlayer::Win;
use crate::{smsg, tr};
use ::libc::{strcat, strcpy};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// The escape the redo buffer ends the change-word command with.
const ESC: c_int = 0x1b;

/// `z=`: suggest replacements for the badly spelled word under or after
/// the cursor.
///
/// In Visual mode the highlighted text is the bad word. A non-zero `count`
/// picks that suggestion without asking.
pub(crate) fn spell_suggest(count: c_int) {
    // SAFETY: the caller guarantees the window; `curwin` is re-read after
    // the body because autocommands may have moved it.
    let prev_cursor = Win::current().w_cursor;
    let msg_scroll_save = msg_scroll.get();

    // `z=` works with 'spell' off, but 'spelllang' has to be parsed
    // for it, which is what turning the option on does.
    let wo_spell_save = Win::current().w_onebuf_opt.wo_spell;
    if Win::current().w_onebuf_opt.wo_spell == 0 {
        let _ = parse_spelllang(Win::current());
        Win::current().w_onebuf_opt.wo_spell = 1;
    }

    suggest_and_replace(count, prev_cursor, msg_scroll_save);

    // Every way out of the body comes through here.
    Win::current().w_onebuf_opt.wo_spell = wo_spell_save;
}

/// The body of `z=`, with `'spell'` already on.
fn suggest_and_replace(count: c_int, prev_cursor: Pos, msg_scroll_save: c_int) {
    // SAFETY: the caller guarantees the window and its spell state; `line`
    // is owned here and outlives every pointer taken into it.
    if unsafe { (*Win::current().w_s).b_p_spl.first_byte() } as c_int == NUL {
        emsg(gettext(e_no_spell));
        return;
    }

    let Some(badlen) = move_to_bad_word(prev_cursor) else {
        return;
    };

    // Should the replacement start with a capital?
    //
    // SAFETY: `curwin` is the window whose cursor position is passed with
    // it, and the caller guarantees its spell state.
    let lnum = Win::current().w_cursor.lnum;
    let col = Win::current().w_cursor.col;
    let need_cap = check_need_cap(Win::current(), lnum, col);

    // Autocommands may free the line, so work from a copy.
    let line = unsafe { xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as usize) };
    spell_suggest_timeout.set(5000);

    // List at most as many as fit on the screen, or as `'spellsuggest'`
    // allows, whichever is smaller.
    let limit = sps_limit.get().min(Rows.get() - 2);
    let mut sug = SugInfo::new();
    // SAFETY: `sug` is this frame's own, live for the whole call.
    let su = unsafe { Sug::new(&raw mut sug) };
    // SAFETY: `line` is the copy of the cursor line taken above, so the
    // cursor's column is inside it, and `su` is the live `sug`.
    let badword = unsafe { line.offset(Win::current().w_cursor.col as isize) };
    unsafe { spell_find_suggest(badword, badlen, su, limit, true, need_cap, true) };

    let mut selected = count;
    msg_ext_set_kind(c"confirm");
    if sug.su_ga.len() as c_int <= 0 {
        msg(gettext(c"No suggestions"), 0);
    } else if count > 0 {
        if count > sug.su_ga.len() as c_int {
            let found = sug.su_ga.len() as c_int as int64_t;
            smsg!(0, "Only {} suggestions", found);
        }
    } else {
        selected = unsafe { ask_which_suggestion(&mut sug, msg_scroll_save) };
    }

    if selected > 0 && selected <= sug.su_ga.len() as c_int && u_save_cursor().is_ok() {
        let stp = &sug.su_ga[selected as usize - 1];
        unsafe { apply_suggestion(&sug, stp, line) };
    } else {
        Win::current().w_cursor = prev_cursor;
    }

    unsafe { spell_find_cleanup(su) };
    unsafe { xfree(line as *mut c_void) };
}

/// Put the cursor on the word `z=` should work on.
///
/// Returns how much of the line the bad word covers, or 0 to let the spell
/// checker decide; `None` means there is nothing to suggest for, and the
/// beep has already been made.
fn move_to_bad_word(prev_cursor: Pos) -> Option<c_int> {
    if visual_active() {
        // The Visual selection is the bad word, but only within a
        // single line.
        if Win::current().w_cursor.lnum != visual_anchor().lnum {
            vim_beep(kOptBoFlagSpell as core::ffi::c_uint);
            return None;
        }
        let mut badlen = Win::current().w_cursor.col - visual_anchor().col;
        if badlen < 0 {
            badlen = -badlen;
        } else {
            Win::current().w_cursor.col = visual_anchor().col;
        }
        badlen += 1;
        end_visual_mode();
        // Leave out the NUL at the end of the line.
        return Some(badlen.min(get_cursor_line_len() - Win::current().w_cursor.col));
    }

    // SAFETY: `curwin` is set from startup to exit and the caller
    // guarantees its spell state; a null `attrp` asks for no attribute.
    let win = Win::current();
    let moved = unsafe { spell_move_to(win, FORWARD as c_int, SMT_ALL, true, ptr::null_mut()) };
    if moved != 0 && Win::current().w_cursor.col <= prev_cursor.col {
        return Some(0);
    }

    // No bad word, or the one found starts after the cursor: take the
    // word under the cursor instead.
    Win::current().w_cursor = prev_cursor;
    let curline = get_cursor_line_ptr();
    let mut p = unsafe { curline.offset(Win::current().w_cursor.col as isize) };
    // Back up to before the start of the word...
    while p > curline && unsafe { spell_iswordp_nmw(p, Win::current()) } {
        p = unsafe { p.sub(utf_head_off(curline, p.sub(1)) as usize + 1) };
    }
    // ...then forward to its start.
    while unsafe { *p } as c_int != NUL && !unsafe { spell_iswordp_nmw(p, Win::current()) } {
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    if !unsafe { spell_iswordp_nmw(p, Win::current()) } {
        beep_flush(); // no word at all
        return None;
    }
    Win::current().w_cursor.col = unsafe { p.offset_from(curline) } as ColNr;
    Some(0)
}

/// Draw the numbered list of suggestions and ask which one to use.
///
/// Returns the number chosen, or 0 for none.
///
/// # Safety
///
/// `sug` must have been filled by `spell_find_suggest` and its bad word
/// must still point into a live line.
unsafe fn ask_which_suggestion(sug: &mut SugInfo, msg_scroll_save: c_int) -> c_int {
    // With 'rightleft' the list is drawn right to left.
    cmdmsg_rl.set(Win::current().w_onebuf_opt.wo_rl != 0);

    msg_start();
    msg_row.set(Rows.get() - 1); // for when 'cmdheight' > 1
    lines_left.set(Rows.get()); // avoid the more-prompt

    // SAFETY: the caller guarantees that the bad word still points into a
    // live line, of which it is `su_badlen` bytes.
    let bad = msg_bytes(unsafe { cstr::prefix_at(sug.su_badptr, sug.su_badlen as usize) });
    let asked = tr!("Change \"{bad}\" to:");
    let asked = if cmdmsg_rl.get() && asked.starts_with("Change") {
        // And now the rabbit from the high hat: avoid showing the
        // untranslated message right-to-left. Upstream tests the
        // *template* for that prefix; the rendering carries it just as
        // faithfully, since the bad word never starts the message.
        format!(":ot \"{bad}\" egnahC")
    } else {
        asked
    };
    msg_text(asked);
    msg_clr_eos();
    msg_putchar('\n' as c_int);

    msg_scroll.set(1);
    let last = sug.su_ga.len() as c_int - 1;
    let badlen = sug.su_badlen;
    let badptr = sug.su_badptr;
    for (i, stp) in sug.su_ga.iter().enumerate() {
        unsafe { show_suggestion(i as c_int, stp, badlen, badptr) };
        if !ui_has(kUIMessages) || (i as c_int) < last {
            msg_putchar('\n' as c_int);
        }
    }

    cmdmsg_rl.set(false);
    msg_col.set(0);

    let mut mouse_used = false;
    let mut selected = unsafe { prompt_for_input(None, 0, false, &raw mut mouse_used) };
    if mouse_used {
        selected = sug.su_ga.len() as c_int + 1 - (cmdline_row.get() - mouse_row.get());
    }

    lines_left.set(Rows.get()); // avoid the more-prompt
    msg_scroll.set(msg_scroll_save); // no delay for 'smd' in normal_cmd()
    selected
}

/// Print one numbered suggestion, without its trailing newline.
///
/// `badlen` and `badptr` are the bad word this replaces.
///
/// # Safety
///
/// `stp` must be a live suggestion and `badptr` must point into a live
/// line.
unsafe fn show_suggestion(i: c_int, stp: &Suggest, badlen: c_int, badptr: *mut c_char) {
    // The suggestion may replace only part of the bad word; show the
    // rest of it too, as long as that does not get too long.
    let mut word = stp.word_bytes().to_vec();
    word.truncate(MAXWLEN);
    let extra = badlen - stp.st_orglen;
    if extra > 0 && stp.st_wordlen + extra <= MAXWLEN as c_int {
        debug_assert!(!badptr.is_null());
        debug_assert_eq!(word.len(), stp.st_wordlen as usize);
        // SAFETY: the caller guarantees the line, and `badlen` exceeding
        // `st_orglen` by `extra` says those bytes of the bad word follow
        // what the suggestion replaces.
        let rest = unsafe { cstr::slice_at(badptr.add(stp.st_orglen as usize), extra as usize) };
        word.extend_from_slice(rest);
    }

    let mut number = format!("{:2}", i + 1);
    if cmdmsg_rl.get() {
        // Mirrored for 'rightleft'. The number is generated ASCII, so
        // reversing its characters is reversing its bytes.
        number = number.chars().rev().collect();
    }
    msg_text(number);

    msg_text(format!(" \"{}\"", msg_bytes(&word)));

    // The word may replace more than the bad word does.
    if badlen < stp.st_orglen {
        // SAFETY: as above, for the bytes the suggestion replaces.
        let replaced = msg_bytes(unsafe { cstr::prefix_at(badptr, stp.st_orglen as usize) });
        msg_text(tr!(" < \"{replaced}\""));
    }

    if p_verbose() > 0 {
        show_score(stp);
    }
}

/// Append a suggestion's score, which `'verbose'` asks for.
fn show_score(stp: &Suggest) {
    let mut shown = if sps_flags.get() & (SPS_DOUBLE | SPS_BEST) != 0 {
        let salscore = if stp.st_salscore { "s " } else { "" };
        format!(" ({}{} - {})", salscore, stp.st_score, stp.st_altscore)
    } else {
        format!(" ({})", stp.st_score)
    };
    if cmdmsg_rl.get() {
        // Mirror the numbers, but keep the leading space. All of it is
        // generated ASCII, so reversing characters reverses bytes.
        let mirrored: String = shown[1..].chars().rev().collect();
        shown.truncate(1);
        shown.push_str(&mirrored);
    }
    msg_advance(30);
    msg_text(shown);
}

/// Put the chosen suggestion into the line, and record it for
/// `:spellrepall` and for redo.
///
/// # Safety
///
/// `sug` and `stp` must be live, `line` must be the owned copy of the
/// cursor line that `sug`'s bad word points into, and the undo state must
/// already have been saved.
unsafe fn apply_suggestion(sug: &SugInfo, stp: &Suggest, line: *mut c_char) {
    // SAFETY: the caller guarantees the pointers; the new line is sized
    // from the three pieces written into it and is handed to `ml_replace`,
    // which takes it over.
    // What `:spellrepall` will repeat.
    unsafe { xfree(repl_from.get() as *mut c_void) };
    repl_from.set(ptr::null_mut());
    unsafe { xfree(repl_to.get() as *mut c_void) };
    repl_to.set(ptr::null_mut());

    if sug.su_badlen > stp.st_orglen {
        // Replacing less than the bad word: what is left of it goes on
        // the end of the replacement.
        repl_from.set(unsafe { xstrnsave(sug.su_badptr, sug.su_badlen as usize) });
        // SAFETY: `su_badptr` points into the line and the suggestion
        // replaces `st_orglen` of its bytes, so what is left of the bad
        // word starts there.
        let rest = unsafe {
            cstr::prefix_at(
                sug.su_badptr.add(stp.st_orglen as usize),
                (sug.su_badlen - stp.st_orglen) as usize,
            )
        };
        let mut repl = stp.word_bytes().to_vec();
        repl.extend_from_slice(rest);
        // `IOSIZE` was upstream's scratch buffer, and it truncated there.
        repl.truncate(IOSIZE as usize - 1);
        let repl = cstr::owned(&repl);
        repl_to.set(unsafe { xstrdup(repl.as_ptr()) });
    } else {
        // Replacing the whole bad word, or more of the line than it
        // covers.
        repl_from.set(unsafe { xstrnsave(sug.su_badptr, stp.st_orglen as usize) });
        repl_to.set(unsafe { xstrdup(stp.word()) });
    }

    // Build the new line: what came before the bad word, the
    // suggestion, and what came after what it replaces.
    let size =
        unsafe { cstr::bytes_at(line) }.len() - stp.st_orglen as usize + stp.st_wordlen as usize;
    let newline = unsafe { xmalloc(size + 1) } as *mut c_char;
    let col = unsafe { sug.su_badptr.offset_from(line) } as c_int;
    unsafe { newline.cast::<u8>().copy_from(line.cast(), col as usize) };
    unsafe { strcpy(newline.offset(col as isize), stp.word()) };
    unsafe { strcat(newline, sug.su_badptr.offset(stp.st_orglen as isize)) };

    // Redo is a change-word command.
    reset_redobuff();
    unsafe { append_to_redobuff(c"ciw".as_ptr()) };
    // SAFETY: the replacement starts at `col` in `newline` and is that many
    // bytes long, all of them inside the buffer sized above.
    let replaced = unsafe { newline.offset(col as isize) };
    let replaced_len = stp.st_wordlen + sug.su_badlen - stp.st_orglen;
    unsafe { append_to_redobuff_literally(replaced, replaced_len) };
    append_to_redobuff_char(ESC);

    // `newline` may be freed here.
    let _ = unsafe { ml_replace(Win::current().w_cursor.lnum, newline, false) };
    Win::current().w_cursor.col = col as ColNr;
    // SAFETY: the cursor is on the line just replaced.
    let lnum = Win::current().w_cursor.lnum;
    inserted_bytes(lnum, col as ColNr, stp.st_orglen, stp.st_wordlen);
}
