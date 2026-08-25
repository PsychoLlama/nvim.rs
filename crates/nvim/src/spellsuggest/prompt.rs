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

use crate::change::inserted_bytes;
use crate::charset::rl_mirror_ascii;
use crate::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, append_to_redobuff_literally, beep_flush,
    reset_redobuff,
};
use crate::input::prompt_for_input;
use crate::main::{
    IObuff, Rows, cmdline_row, cmdmsg_rl, curwin, e_no_spell, lines_left, mouse_row, msg_col,
    msg_row, msg_scroll, p_verbose,
};
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memline::ml_replace;
use crate::memory::{xfree, xmalloc, xmemcpyz, xstrdup, xstrlcpy};
use crate::message::{
    emsg, msg, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_puts, msg_start,
};
use crate::normal::{end_visual_mode, visual_active, visual_anchor};
use crate::options::kOptBoFlagSpell;
use crate::os::cshim::{gettext, memmove, strncmp};
use crate::search::FORWARD;
use crate::smsg_c;
use crate::spell::{
    SMT_ALL, check_need_cap, parse_spelllang, repl_from, repl_to, spell_iswordp_nmw, spell_move_to,
};
use crate::spellsuggest::collect::suggestions;
use crate::spellsuggest::{
    MAXWLEN, OK, SPS_BEST, SPS_DOUBLE, spell_find_cleanup, spell_find_suggest,
    spell_suggest_timeout, sps_flags, sps_limit, suggest_T, suginfo_T,
};
use crate::strings::{vim_snprintf, xstrnsave};
use crate::types::ui::kUIMessages;
use crate::types::{IOSIZE, NUL, colnr_T, int64_t, pos_T};
use crate::ui::{ui_has, vim_beep};
use crate::undo::u_save_cursor;
use ::libc::{strcat, strcpy, strlen};
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr};

/// The escape the redo buffer ends the change-word command with.
const ESC: c_int = 0x1b;

/// The shared scratch buffer messages are formatted into.
fn iobuff() -> *mut c_char {
    // `GlobalCell::ptr` takes the address without forming a reference.
    IObuff.ptr() as *mut c_char
}

/// `z=`: suggest replacements for the badly spelled word under or after
/// the cursor.
///
/// In Visual mode the highlighted text is the bad word. A non-zero `count`
/// picks that suggestion without asking.
///
/// # Safety
///
/// There must be a current window with a buffer.
pub(crate) unsafe fn spell_suggest(count: c_int) {
    // SAFETY: the caller guarantees the window; `curwin` is re-read after
    // the body because autocommands may have moved it.
    unsafe {
        let prev_cursor = (*curwin.get()).w_cursor;
        let msg_scroll_save = msg_scroll.get();

        // `z=` works with 'spell' off, but 'spelllang' has to be parsed
        // for it, which is what turning the option on does.
        let wo_spell_save = (*curwin.get()).w_onebuf_opt.wo_spell;
        if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
            parse_spelllang(curwin.get());
            (*curwin.get()).w_onebuf_opt.wo_spell = 1;
        }

        suggest_and_replace(count, prev_cursor, msg_scroll_save);

        // Every way out of the body comes through here.
        (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
    }
}

/// The body of `z=`, with `'spell'` already on.
///
/// # Safety
///
/// As [`spell_suggest`], and `'spell'` must be on.
unsafe fn suggest_and_replace(count: c_int, prev_cursor: pos_T, msg_scroll_save: c_int) {
    // SAFETY: the caller guarantees the window and its spell state; `line`
    // is owned here and outlives every pointer taken into it.
    unsafe {
        if *(*(*curwin.get()).w_s).b_p_spl as c_int == NUL {
            emsg(gettext(&raw const e_no_spell as *const c_char));
            return;
        }

        let Some(badlen) = move_to_bad_word(prev_cursor) else {
            return;
        };

        // Should the replacement start with a capital?
        let need_cap = check_need_cap(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            (*curwin.get()).w_cursor.col,
        );

        // Autocommands may free the line, so work from a copy.
        let line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as usize);
        spell_suggest_timeout.set(5000);

        // List at most as many as fit on the screen, or as `'spellsuggest'`
        // allows, whichever is smaller.
        let limit = sps_limit.get().min(Rows.get() - 2);
        let mut sug: suginfo_T = mem::zeroed();
        spell_find_suggest(
            line.offset((*curwin.get()).w_cursor.col as isize),
            badlen,
            &raw mut sug,
            limit,
            true,
            need_cap,
            true,
        );

        let mut selected = count;
        msg_ext_set_kind(c"confirm".as_ptr());
        if sug.su_ga.ga_len <= 0 {
            msg(gettext(c"No suggestions".as_ptr()), 0);
        } else if count > 0 {
            if count > sug.su_ga.ga_len {
                smsg_c!(
                    0,
                    gettext(c"Only %ld suggestions".as_ptr()),
                    sug.su_ga.ga_len as int64_t,
                );
            }
        } else {
            selected = ask_which_suggestion(&mut sug, msg_scroll_save);
        }

        if selected > 0 && selected <= sug.su_ga.ga_len && u_save_cursor() == OK {
            let stp = suggestions(&raw mut sug.su_ga)[selected as usize - 1];
            apply_suggestion(&sug, &stp, line);
        } else {
            (*curwin.get()).w_cursor = prev_cursor;
        }

        spell_find_cleanup(&raw mut sug);
        xfree(line as *mut c_void);
    }
}

/// Put the cursor on the word `z=` should work on.
///
/// Returns how much of the line the bad word covers, or 0 to let the spell
/// checker decide; `None` means there is nothing to suggest for, and the
/// beep has already been made.
///
/// # Safety
///
/// There must be a current window with a buffer and its spell state.
unsafe fn move_to_bad_word(prev_cursor: pos_T) -> Option<c_int> {
    // SAFETY: the caller guarantees the window; the scan below stays
    // between the start of the cursor line and its terminator.
    unsafe {
        if visual_active() {
            // The Visual selection is the bad word, but only within a
            // single line.
            if (*curwin.get()).w_cursor.lnum != visual_anchor().lnum {
                vim_beep(kOptBoFlagSpell as core::ffi::c_uint);
                return None;
            }
            let mut badlen = (*curwin.get()).w_cursor.col - visual_anchor().col;
            if badlen < 0 {
                badlen = -badlen;
            } else {
                (*curwin.get()).w_cursor.col = visual_anchor().col;
            }
            badlen += 1;
            end_visual_mode();
            // Leave out the NUL at the end of the line.
            return Some(badlen.min(get_cursor_line_len() - (*curwin.get()).w_cursor.col));
        }

        if spell_move_to(
            curwin.get(),
            FORWARD as c_int,
            SMT_ALL,
            true,
            ptr::null_mut(),
        ) != 0
            && (*curwin.get()).w_cursor.col <= prev_cursor.col
        {
            return Some(0);
        }

        // No bad word, or the one found starts after the cursor: take the
        // word under the cursor instead.
        (*curwin.get()).w_cursor = prev_cursor;
        let curline = get_cursor_line_ptr();
        let mut p = curline.offset((*curwin.get()).w_cursor.col as isize);
        // Back up to before the start of the word...
        while p > curline && spell_iswordp_nmw(p, curwin.get()) {
            p = p.sub(utf_head_off(curline, p.sub(1)) as usize + 1);
        }
        // ...then forward to its start.
        while *p as c_int != NUL && !spell_iswordp_nmw(p, curwin.get()) {
            p = p.add(utfc_ptr2len(p) as usize);
        }
        if !spell_iswordp_nmw(p, curwin.get()) {
            beep_flush(); // no word at all
            return None;
        }
        (*curwin.get()).w_cursor.col = p.offset_from(curline) as colnr_T;
        Some(0)
    }
}

/// Draw the numbered list of suggestions and ask which one to use.
///
/// Returns the number chosen, or 0 for none.
///
/// # Safety
///
/// `sug` must have been filled by `spell_find_suggest` and its bad word
/// must still point into a live line.
unsafe fn ask_which_suggestion(sug: &mut suginfo_T, msg_scroll_save: c_int) -> c_int {
    // SAFETY: the caller guarantees `sug`; every message is formatted into
    // `IObuff` with its own size.
    unsafe {
        // With 'rightleft' the list is drawn right to left.
        cmdmsg_rl.set((*curwin.get()).w_onebuf_opt.wo_rl != 0);

        msg_start();
        msg_row.set(Rows.get() - 1); // for when 'cmdheight' > 1
        lines_left.set(Rows.get()); // avoid the more-prompt

        let mut fmt = gettext(c"Change \"%.*s\" to:".as_ptr());
        if cmdmsg_rl.get() && strncmp(fmt, c"Change".as_ptr(), 6) == 0 {
            // And now the rabbit from the high hat: avoid showing the
            // untranslated message right-to-left.
            fmt = c":ot \"%.*s\" egnahC".as_ptr().cast_mut();
        }
        vim_snprintf(iobuff(), IOSIZE as usize, fmt, sug.su_badlen, sug.su_badptr);
        msg_puts(iobuff());
        msg_clr_eos();
        msg_putchar('\n' as c_int);

        msg_scroll.set(1);
        let last = sug.su_ga.ga_len - 1;
        let badlen = sug.su_badlen;
        let badptr = sug.su_badptr;
        for (i, stp) in suggestions(&raw mut sug.su_ga).iter().enumerate() {
            show_suggestion(i as c_int, stp, badlen, badptr);
            if !ui_has(kUIMessages) || (i as c_int) < last {
                msg_putchar('\n' as c_int);
            }
        }

        cmdmsg_rl.set(false);
        msg_col.set(0);

        let mut mouse_used = false;
        let mut selected = prompt_for_input(ptr::null_mut(), 0, false, &raw mut mouse_used);
        if mouse_used {
            selected = sug.su_ga.ga_len + 1 - (cmdline_row.get() - mouse_row.get());
        }

        lines_left.set(Rows.get()); // avoid the more-prompt
        msg_scroll.set(msg_scroll_save); // no delay for 'smd' in normal_cmd()
        selected
    }
}

/// Print one numbered suggestion, without its trailing newline.
///
/// `badlen` and `badptr` are the bad word this replaces.
///
/// # Safety
///
/// `stp` must be a live suggestion and `badptr` must point into a live
/// line.
unsafe fn show_suggestion(i: c_int, stp: &suggest_T, badlen: c_int, badptr: *mut c_char) {
    // SAFETY: the caller guarantees the suggestion and the line; `wcopy`
    // has room for the longest word plus what is appended to it.
    unsafe {
        // The suggestion may replace only part of the bad word; show the
        // rest of it too, as long as that does not get too long.
        let mut wcopy = [0 as c_char; MAXWLEN + 2];
        let wcopyp = wcopy.as_mut_ptr();
        xstrlcpy(wcopyp, stp.st_word, MAXWLEN + 1);
        let extra = badlen - stp.st_orglen;
        if extra > 0 && stp.st_wordlen + extra <= MAXWLEN as c_int {
            debug_assert!(!badptr.is_null());
            xmemcpyz(
                wcopyp.offset(stp.st_wordlen as isize) as *mut c_void,
                badptr.offset(stp.st_orglen as isize) as *const c_void,
                extra as usize,
            );
        }

        vim_snprintf(iobuff(), IOSIZE as usize, c"%2d".as_ptr(), i + 1);
        if cmdmsg_rl.get() {
            rl_mirror_ascii(iobuff(), ptr::null_mut());
        }
        msg_puts(iobuff());

        vim_snprintf(iobuff(), IOSIZE as usize, c" \"%s\"".as_ptr(), wcopyp);
        msg_puts(iobuff());

        // The word may replace more than the bad word does.
        if badlen < stp.st_orglen {
            vim_snprintf(
                iobuff(),
                IOSIZE as usize,
                gettext(c" < \"%.*s\"".as_ptr()),
                stp.st_orglen,
                badptr,
            );
            msg_puts(iobuff());
        }

        if p_verbose.get() > 0 {
            show_score(stp);
        }
    }
}

/// Append a suggestion's score, which `'verbose'` asks for.
///
/// # Safety
///
/// `stp` must be a live suggestion.
unsafe fn show_score(stp: &suggest_T) {
    // SAFETY: the caller guarantees the suggestion; the format strings and
    // `IObuff`'s size match.
    unsafe {
        if sps_flags.get() & (SPS_DOUBLE | SPS_BEST) != 0 {
            vim_snprintf(
                iobuff(),
                IOSIZE as usize,
                c" (%s%d - %d)".as_ptr(),
                if stp.st_salscore {
                    c"s ".as_ptr()
                } else {
                    c"".as_ptr()
                },
                stp.st_score,
                stp.st_altscore,
            );
        } else {
            vim_snprintf(iobuff(), IOSIZE as usize, c" (%d)".as_ptr(), stp.st_score);
        }
        if cmdmsg_rl.get() {
            // Mirror the numbers, but keep the leading space.
            rl_mirror_ascii(iobuff().add(1), ptr::null_mut());
        }
        msg_advance(30);
        msg_puts(iobuff());
    }
}

/// Put the chosen suggestion into the line, and record it for
/// `:spellrepall` and for redo.
///
/// # Safety
///
/// `sug` and `stp` must be live, `line` must be the owned copy of the
/// cursor line that `sug`'s bad word points into, and the undo state must
/// already have been saved.
unsafe fn apply_suggestion(sug: &suginfo_T, stp: &suggest_T, line: *mut c_char) {
    // SAFETY: the caller guarantees the pointers; the new line is sized
    // from the three pieces written into it and is handed to `ml_replace`,
    // which takes it over.
    unsafe {
        // What `:spellrepall` will repeat.
        xfree(repl_from.get() as *mut c_void);
        repl_from.set(ptr::null_mut());
        xfree(repl_to.get() as *mut c_void);
        repl_to.set(ptr::null_mut());

        if sug.su_badlen > stp.st_orglen {
            // Replacing less than the bad word: what is left of it goes on
            // the end of the replacement.
            repl_from.set(xstrnsave(sug.su_badptr, sug.su_badlen as usize));
            vim_snprintf(
                iobuff(),
                IOSIZE as usize,
                c"%s%.*s".as_ptr(),
                stp.st_word,
                sug.su_badlen - stp.st_orglen,
                sug.su_badptr.offset(stp.st_orglen as isize),
            );
            repl_to.set(xstrdup(iobuff()));
        } else {
            // Replacing the whole bad word, or more of the line than it
            // covers.
            repl_from.set(xstrnsave(sug.su_badptr, stp.st_orglen as usize));
            repl_to.set(xstrdup(stp.st_word));
        }

        // Build the new line: what came before the bad word, the
        // suggestion, and what came after what it replaces.
        let newline =
            xmalloc(strlen(line) as usize - stp.st_orglen as usize + stp.st_wordlen as usize + 1)
                as *mut c_char;
        let col = sug.su_badptr.offset_from(line) as c_int;
        memmove(newline as *mut c_void, line as *const c_void, col as usize);
        strcpy(newline.offset(col as isize), stp.st_word);
        strcat(newline, sug.su_badptr.offset(stp.st_orglen as isize));

        // Redo is a change-word command.
        reset_redobuff();
        append_to_redobuff(c"ciw".as_ptr());
        append_to_redobuff_literally(
            newline.offset(col as isize),
            stp.st_wordlen + sug.su_badlen - stp.st_orglen,
        );
        append_to_redobuff_char(ESC);

        // `newline` may be freed here.
        ml_replace((*curwin.get()).w_cursor.lnum, newline, false);
        (*curwin.get()).w_cursor.col = col as colnr_T;
        inserted_bytes(
            (*curwin.get()).w_cursor.lnum,
            col as colnr_T,
            stp.st_orglen,
            stp.st_wordlen,
        );
    }
}
