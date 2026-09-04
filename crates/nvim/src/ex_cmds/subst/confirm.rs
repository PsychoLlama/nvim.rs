//! The `c` flag: ask before each substitution.
//!
//! Two prompts, because Ex mode has no screen to highlight on.  In Ex mode
//! the line is printed and a row of `^` under the match is used as the
//! prompt; otherwise the match is highlighted in the buffer -- which means
//! temporarily putting the *partly substituted* line back, since the earlier
//! substitutions on this line have not reached the buffer yet, and then
//! putting the original back before returning.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::exec::Sub;
use super::subflags;
use crate::cstr;
use crate::drawscreen::{
    UPD_SOME_VALID, number_width, redraw_later, show_cursor_info_later, update_screen,
};
use crate::ex_cmds::cur_win;
use crate::ex_cmds::{ESC, print_line_no_prefix};
use crate::ex_getln::{getcmdline_prompt, gotocmdline};
use crate::guard::{Allow, Suppress};
use crate::highlight_group::HLF_R;
use crate::input::prompt_for_input;
use crate::keycodes::{Ctrl_C, Ctrl_E, Ctrl_Y};
use crate::main::{
    State, curwin, ex_normal_busy, exmode_active, highlight_match, msg_didout, need_wait_return,
    p_lz, search_match_endcol, search_match_lines,
};
use crate::memline::{ml_get, ml_get_len, ml_replace};
use crate::memory::{xfree, xmallocz, xstrdup};
use crate::message::msg_putchar;
use crate::mouse::setmouse;
use crate::r#move::{
    do_check_cursorbind, scrolldown_clamp, scrollup_clamp, update_topline, validate_cursor,
};
use crate::option::cpo_has;
use crate::os::cshim::{gettext, snprintf};
use crate::plines::getvcol;
use crate::strings::{concat_str, xstrnsave};
use crate::types::ui::kUIMessages;
use crate::types::{Callback, CpoFlag, ExpandContext, IOSIZE, NUL, colnr_T, linenr_T, size_t};
use crate::ui::ui_has;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// What the prompt decided for this match.
pub(super) enum Confirm {
    /// Replace it -- `y`, and also `a` and `l`, which change the flags first.
    Replace,
    /// Leave it: `n`.
    Skip,
    /// Stop the whole substitute: `q`, `<Esc>` or `CTRL-C`.
    Quit,
}

/// An empty callback, for a prompt that has no completion function.
fn no_callback() -> Callback {
    Callback::None
}

/// Does 'cpoptions' contain `u`?  Then undo is not synced while asking.
///
/// Re-read after the prompt, as upstream does: the user can change the option
/// from the command line the prompt runs.
fn cpo_no_undo_sync() -> bool {
    // SAFETY: 'cpoptions' is a live string option.
    cpo_has(CpoFlag::UNDO)
}

/// Ex mode's prompt: print the line, then a row of `^` under the match.
///
/// # Safety
/// Main thread; `st` must describe a live match on `st.lnum`.
unsafe fn prompt_exmode(st: &Sub) -> c_int {
    // SAFETY: caller's contract.
    unsafe {
        print_line_no_prefix(
            st.lnum,
            subflags.with(|flags| flags.do_number),
            subflags.with(|flags| flags.do_list),
        )
    };

    let mut sc = 0 as colnr_T;
    let mut ec = 0 as colnr_T;
    // SAFETY: the cursor is at the start of the match; moving it to the last
    // byte of the match and back is what gives the match's screen columns.
    unsafe {
        getvcol(
            Win::new(curwin.get()),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut sc,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    cur_win().w_cursor.col = (st.regmatch.endpos[0].col - 1 as c_int).max(0 as c_int);
    unsafe {
        getvcol(
            Win::new(curwin.get()),
            &raw mut (*curwin.get()).w_cursor,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut ec,
        )
    };
    cur_win().w_cursor.col = st.regmatch.startpos[0].col;
    if subflags.with(|flags| flags.do_number) || cur_win().w_onebuf_opt.wo_nu != 0 {
        let numw = unsafe { number_width(curwin.get()) } + 1 as c_int;
        sc += numw;
        ec += numw;
    }

    // SAFETY: the prompt is `ec + 1` bytes of spaces and carets inside an
    // `ec + 2` byte zeroed allocation.
    let typed = unsafe {
        let prompt = xmallocz((ec as size_t).wrapping_add(1 as size_t)) as *mut c_char;
        prompt.cast::<u8>().write_bytes(b' ', sc as size_t);
        (prompt.offset(sc as isize))
            .cast::<u8>()
            .write_bytes(b'^', ((ec - sc) as size_t).wrapping_add(1 as size_t));
        let resp = getcmdline_prompt(
            -1 as c_int,
            prompt,
            0 as c_int,
            ExpandContext::Nothing,
            ptr::null(),
            no_callback(),
            false,
            ptr::null_mut(),
        );
        if !ui_has(kUIMessages) {
            msg_putchar('\n' as c_int);
        }
        xfree(prompt as *mut c_void);
        if resp.is_null() {
            // getcmdline_prompt() answers NULL when there is no command line
            // to return.
            NUL
        } else {
            let typed = *resp as u8 as c_int;
            xfree(resp as *mut c_void);
            typed
        }
    };

    // When ":normal" runs out of characters we get an empty line.  Use "q" to
    // get out of the loop.
    if ex_normal_busy.get() != 0 && typed == NUL {
        'q' as c_int
    } else {
        typed
    }
}

/// The screen prompt: highlight the match, ask, then put everything back.
///
/// # Safety
/// Main thread; `st` must describe a live match on `st.lnum`.
unsafe fn prompt_visual(st: &Sub) -> c_int {
    let mut orig_line: *mut c_char = ptr::null_mut();
    let mut len_change = 0 as c_int;
    let save_p_lz = p_lz.get();
    // SAFETY: the current window is live.
    let save_p_fen = cur_win().w_onebuf_opt.wo_fen;
    // SAFETY: as above.
    cur_win().w_onebuf_opt.wo_fen = 0;

    // Invert the matched string; the inversion is removed afterwards.
    let redraw = Allow::redraw();
    // Avoid calling update_screen() in vgetorpeek().
    p_lz.set(0);

    if !st.new_start.is_null() {
        // There already was a substitution and we would like to show it, but
        // we cannot really update the line -- that would change what matches.
        // Replace it temporarily and change it back afterwards.
        // SAFETY: `lnum` is a line of the buffer and the pieces are live.
        orig_line = unsafe { xstrnsave(ml_get(st.lnum), ml_get_len(st.lnum) as size_t) };
        let new_line =
            unsafe { concat_str(st.new_start, st.sub_firstline.add(st.copycol as usize)) };
        // Position the cursor relative to the end of the line: the
        // previous substitute may have inserted or deleted characters
        // before it.
        len_change = unsafe { cstr::bytes_at(new_line) }.len() as c_int
            - unsafe { cstr::bytes_at(orig_line) }.len() as c_int;
        cur_win().w_cursor.col += len_change;
        let _ = unsafe { ml_replace(st.lnum, new_line, false) };
    }

    search_match_lines.set(st.regmatch.endpos[0].lnum - st.regmatch.startpos[0].lnum);
    search_match_endcol.set(st.regmatch.endpos[0].col + len_change);
    if search_match_lines.get() == 0 as linenr_T && search_match_endcol.get() == 0 as colnr_T {
        // Highlight at least one character for /^/.
        search_match_endcol.set(1 as colnr_T);
    }
    highlight_match.set(true);

    // SAFETY: the current window is live.
    update_topline(cur_win());
    validate_cursor(cur_win());
    unsafe { redraw_later(curwin.get(), UPD_SOME_VALID) };
    unsafe { show_cursor_info_later(true) };
    let _ = unsafe { update_screen() };
    unsafe { redraw_later(curwin.get(), UPD_SOME_VALID) };
    cur_win().w_onebuf_opt.wo_fen = save_p_fen;

    let mut ask = [0 as c_char; IOSIZE as usize];
    // SAFETY: `ask` is `IOSIZE` bytes, the format takes one string, and the
    // prompt is a fresh copy of it.
    let typed = unsafe {
        let iobuff = ask.as_mut_ptr();
        snprintf(
            iobuff,
            IOSIZE as size_t,
            gettext(c"replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)")
                .as_ptr(),
            st.sub,
        );
        let prompt = xstrdup(iobuff);
        let typed = prompt_for_input(prompt, HLF_R, true, ptr::null_mut());
        highlight_match.set(false);
        xfree(prompt as *mut c_void);
        typed
    };

    msg_didout.set(false); // don't scroll up
    // SAFETY: message state.
    unsafe { gotocmdline(true) };
    p_lz.set(save_p_lz);
    drop(redraw);

    // Restore the line.
    if !orig_line.is_null() {
        // SAFETY: `lnum` is a line of the buffer and `orig_line` its saved
        // text, which `ml_replace` takes ownership of.
        let _ = unsafe { ml_replace(st.lnum, orig_line, false) };
    }
    typed
}

/// Ask about this match, looping over the answers that only scroll.
///
/// # Safety
/// Main thread; `st` must describe a live match on `st.lnum`.
pub(super) unsafe fn ask_confirm(st: &mut Sub) -> Confirm {
    let mut typed = 0 as c_int;
    let save_state = State.get();
    // SAFETY: the current window is live.
    cur_win().w_cursor.col = st.regmatch.startpos[0].col;
    if cur_win().w_onebuf_opt.wo_crb != 0 {
        unsafe { do_check_cursorbind() };
    }
    // Held for the whole prompt: `'cpoptions'` is read once, where the C
    // read it again at the release and would have gone out of step with
    // itself had the prompt's own `CTRL-R =` changed the option.
    let no_sync = cpo_no_undo_sync().then(Suppress::undo_sync);

    // Loop until 'y', 'n', 'q', CTRL-E or CTRL-Y is typed.
    while subflags.with(|flags| flags.do_ask) {
        // SAFETY: caller's contract.
        typed = unsafe {
            if exmode_active.get() {
                prompt_exmode(st)
            } else {
                prompt_visual(st)
            }
        };

        need_wait_return.set(false); // no hit-return prompt
        if typed == 'q' as c_int || typed == ESC || typed == Ctrl_C {
            st.got_quit = true;
            break;
        }
        if typed == 'n' as c_int || typed == 'y' as c_int {
            break;
        }
        if typed == 'l' as c_int {
            // Last: replace and then stop.
            subflags.with_mut(|flags| flags.do_all = false);
            st.line2 = st.lnum;
            break;
        }
        if typed == 'a' as c_int {
            subflags.with_mut(|flags| flags.do_ask = false);
            break;
        }
        if typed == Ctrl_E {
            // SAFETY: the current window is live.
            unsafe { scrollup_clamp() };
        } else if typed == Ctrl_Y {
            // SAFETY: as above.
            unsafe { scrolldown_clamp() };
        }
    }
    State.set(save_state);
    // SAFETY: main thread.
    setmouse();
    drop(no_sync);

    if typed == 'n' as c_int {
        // For a multi-line match, put matchcol at the NUL at the end of the
        // line and set nmatch to one, so that we continue looking for a match
        // on the next line.  Avoids that ":%s/\nB\@=//gc" and ":%s/\n/,\r/gc"
        // get stuck when pressing 'n'.
        if st.nmatch > 1 as c_int {
            // SAFETY: the copied line is NUL-terminated.
            st.matchcol = unsafe { cstr::bytes_at(st.sub_firstline) }.len() as colnr_T;
            st.skip_match = true;
        }
        return Confirm::Skip;
    }
    if st.got_quit {
        return Confirm::Quit;
    }
    Confirm::Replace
}
