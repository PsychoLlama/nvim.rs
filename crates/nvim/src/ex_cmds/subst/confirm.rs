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

#![forbid(unsafe_code)]

use super::SUB_FLAGS;
use super::exec::Sub;
use crate::drawscreen::{
    UPD_SOME_VALID, number_width, redraw_later, show_cursor_info_later, update_screen,
};
use crate::ex_cmds::say;
use crate::ex_cmds::{ESC, print_line_no_prefix};
use crate::ex_docmd::state::ex_normal_busy;
use crate::ex_getln::{getcmdline_prompt_text, gotocmdline};
use crate::guard::{Allow, Suppress};
use crate::highlight_group::HLF_R;
use crate::input::prompt_for_input;
use crate::keycodes::{Ctrl_C, Ctrl_E, Ctrl_Y};
use crate::memline::{Lines, ml_replace_buf_text};
use crate::memory::XString;
use crate::message::state::{msg_didout, need_wait_return};
use crate::mouse::setmouse;
use crate::r#move::{
    do_check_cursorbind, scrolldown_clamp, scrollup_clamp, update_topline, validate_cursor,
};
use crate::option::cpo_has;
use crate::option::vars::{P_LZ, p_lz};
use crate::os::cshim::gettext;
use crate::plines::getvcol;
use crate::search::state::{highlight_match, search_match_endcol, search_match_lines};
use crate::state::mode::{State, exmode_active};
use crate::types::ui::kUIMessages;
use crate::types::{Callback, ColNr, CpoFlag, ExpandContext, IOSIZE, LineNr, NUL};
use crate::ui::ui_has;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

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
    cpo_has(CpoFlag::UNDO)
}

/// Ex mode's prompt: print the line, then a row of `^` under the match.
fn prompt_exmode(st: &Sub) -> c_int {
    print_line_no_prefix(
        st.lnum,
        SUB_FLAGS.with(|flags| flags.do_number),
        SUB_FLAGS.with(|flags| flags.do_list),
    );

    // The cursor is at the start of the match; moving it to the last byte
    // of the match and back is what gives the match's screen columns.
    let mut cursor = Win::current().w_cursor;
    let (mut sc, _, _) = getvcol(Win::current(), &mut cursor);
    Win::current().w_cursor = cursor;
    Win::current().w_cursor.col = (st.regmatch.endpos[0].col - 1 as c_int).max(0 as c_int);
    let mut cursor = Win::current().w_cursor;
    let (_, _, mut ec) = getvcol(Win::current(), &mut cursor);
    Win::current().w_cursor = cursor;
    Win::current().w_cursor.col = st.regmatch.startpos[0].col;
    if SUB_FLAGS.with(|flags| flags.do_number) || Win::current().w_onebuf_opt.wo_nu != 0 {
        let numw = number_width(Win::current()) + 1 as c_int;
        sc += numw;
        ec += numw;
    }

    // Spaces up to the match, then a caret under every column of it.
    // Upstream sizes an `xmallocz` from `ec - sc + 1`, which is a `size_t`:
    // a match that somehow ended before it started asked for a memset of
    // four billion bytes.  A `resize` past the end of a shorter buffer
    // truncates instead.
    let mut prompt = vec![b' '; sc.max(0) as usize];
    prompt.resize(ec.max(0) as usize + 1, b'^');
    let prompt = XString::from_vec(prompt);
    let resp = getcmdline_prompt_text(
        -1 as c_int,
        prompt.as_cstr(),
        0 as c_int,
        ExpandContext::Nothing,
        no_callback(),
        false,
    );
    if !ui_has(kUIMessages) {
        say::putchar('\n' as c_int);
    }
    // getcmdline_prompt() answers NULL when there is no command line to
    // return.
    let typed = resp.map_or(NUL, |resp| c_int::from(resp.first().copied().unwrap_or(0)));

    // When ":normal" runs out of characters we get an empty line.  Use "q" to
    // get out of the loop.
    if ex_normal_busy.get() != 0 && typed == NUL {
        'q' as c_int
    } else {
        typed
    }
}

/// The screen prompt: highlight the match, ask, then put everything back.
fn prompt_visual(st: &Sub) -> c_int {
    let mut orig_line: Option<Vec<u8>> = None;
    let mut len_change = 0 as c_int;
    let save_p_lz = p_lz();
    let save_p_fen = Win::current().w_onebuf_opt.wo_fen;
    Win::current().w_onebuf_opt.wo_fen = 0;

    // Invert the matched string; the inversion is removed afterwards.
    let redraw = Allow::redraw();
    // Avoid calling update_screen() in vgetorpeek().
    P_LZ.set(false);

    if st.new_line.is_some() {
        // There already was a substitution and we would like to show it, but
        // we cannot really update the line -- that would change what matches.
        // Replace it temporarily and change it back afterwards.
        let saved = Lines::current().line(st.lnum).to_vec();
        // The line as it would look: what has been rebuilt so far, and the
        // rest of the old text after it.
        let mut shown = st.new_line().bytes().to_vec();
        shown.extend_from_slice(&st.old_line().bytes()[st.copied..]);
        // Position the cursor relative to the end of the line: the
        // previous substitute may have inserted or deleted characters
        // before it.
        len_change = shown.len() as c_int - saved.len() as c_int;
        Win::current().w_cursor.col += len_change;
        let _ = ml_replace_buf_text(Buf::current(), st.lnum, &shown);
        orig_line = Some(saved);
    }

    search_match_lines.set(st.regmatch.endpos[0].lnum - st.regmatch.startpos[0].lnum);
    search_match_endcol.set(st.regmatch.endpos[0].col + len_change);
    if search_match_lines.get() == 0 as LineNr && search_match_endcol.get() == 0 as ColNr {
        // Highlight at least one character for /^/.
        search_match_endcol.set(1 as ColNr);
    }
    highlight_match.set(true);

    update_topline(Win::current());
    validate_cursor(Win::current());
    redraw_later(Win::current(), UPD_SOME_VALID);
    show_cursor_info_later(true);
    let _ = update_screen();
    redraw_later(Win::current(), UPD_SOME_VALID);
    Win::current().w_onebuf_opt.wo_fen = save_p_fen;

    let replacement = st.sub.as_cstr();
    let fmt = gettext(c"replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)");
    // Cut where upstream's `IObuff` cut it.
    let prompt = crate::formatted!(IOSIZE as usize, fmt.to_bytes(), replacement);
    let prompt = XString::from_bytes(&prompt);
    let typed = prompt_for_input(Some(prompt.as_cstr()), HLF_R, true, None);
    highlight_match.set(false);

    msg_didout.set(false); // don't scroll up
    gotocmdline(true);
    P_LZ.set(save_p_lz);
    drop(redraw);

    // Restore the line.
    if let Some(orig_line) = orig_line {
        let _ = ml_replace_buf_text(Buf::current(), st.lnum, &orig_line);
    }
    typed
}

/// Ask about this match, looping over the answers that only scroll.
pub(super) fn ask_confirm(st: &mut Sub) -> Confirm {
    let mut typed = 0 as c_int;
    let save_state = State.get();
    Win::current().w_cursor.col = st.regmatch.startpos[0].col;
    if Win::current().w_onebuf_opt.wo_crb != 0 {
        do_check_cursorbind();
    }
    // Held for the whole prompt: `'cpoptions'` is read once, where the C
    // read it again at the release and would have gone out of step with
    // itself had the prompt's own `CTRL-R =` changed the option.
    let no_sync = cpo_no_undo_sync().then(Suppress::undo_sync);

    // Loop until 'y', 'n', 'q', CTRL-E or CTRL-Y is typed.
    while SUB_FLAGS.with(|flags| flags.do_ask) {
        typed = if exmode_active.get() {
            prompt_exmode(st)
        } else {
            prompt_visual(st)
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
            SUB_FLAGS.with_mut(|flags| flags.do_all = false);
            st.line2 = st.lnum;
            break;
        }
        if typed == 'a' as c_int {
            SUB_FLAGS.with_mut(|flags| flags.do_ask = false);
            break;
        }
        if typed == Ctrl_E {
            scrollup_clamp();
        } else if typed == Ctrl_Y {
            scrolldown_clamp();
        }
    }
    State.set(save_state);
    setmouse();
    drop(no_sync);

    if typed == 'n' as c_int {
        // For a multi-line match, put matchcol at the NUL at the end of the
        // line and set nmatch to one, so that we continue looking for a match
        // on the next line.  Avoids that ":%s/\nB\@=//gc" and ":%s/\n/,\r/gc"
        // get stuck when pressing 'n'.
        if st.nmatch > 1 as c_int {
            st.matchcol = st.old_line().len() as ColNr;
            st.skip_match = true;
        }
        return Confirm::Skip;
    }
    if st.got_quit {
        return Confirm::Quit;
    }
    Confirm::Replace
}
