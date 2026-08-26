//! Auto-wrap: breaking the line being typed at 'textwidth'.
//!
//! [`internal_format`] is called from `insertchar` (`edit.rs`) for every
//! character that could take the line over the margin, and is reentrant with
//! it: it calls `open_line`, which runs the whole indent machinery and can
//! come back here. Two questions per break, in order -- *where* may this line
//! be broken ([`BreakSearch`]), and then what it takes to break it there.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::change::{get_leader_len, ins_bytes, ins_str, open_line};
use crate::charset::char2cells;
use crate::cursor::{
    dec_cursor, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len,
    get_cursor_pos_ptr, inc_cursor, pchar_cursor,
};
use crate::drawscreen::{UPD_VALID, redraw_curbuf_later};
use crate::edit::{backspace_until_column, get_nolist_virtcol, set_can_cindent, undisplay_dollar};
use crate::indent::{change_indent, get_number_indent, set_indent};
use crate::main::{
    Insstart, State, can_si, can_si_back, did_ai, did_si, got_int, old_indent, replace_offset,
};
use crate::mbyte::{utf_allow_break, utf_allow_break_before};
use crate::memory::xfree;
use crate::r#move::update_topline;
use crate::os::input::line_breakcheck;
use crate::pos::MAXCOL;
use crate::search::{FORWARD, check_linecomment};
use crate::state::VREPLACE_FLAG;
use crate::strings::xstrnsave;
use crate::types::{INSCHAR_COM_LIST, INSCHAR_DO_COM, INSCHAR_FORMAT, NUL, colnr_T, size_t};

/// What one step of the backwards search for a break column decided.
///
/// These are upstream's three ways out of the search loop's body: `break`,
/// `continue`, and falling off the end -- which steps back one character
/// before going round again.
enum Step {
    /// Stop searching; whatever `foundcol` holds is the answer.
    Stop,
    /// Go round again from the cursor's *current* position.
    Again,
    /// Step back one character, then go round again.
    Back,
}

/// The backwards walk from the cursor looking for somewhere to break.
///
/// The cursor itself is the walk's position -- upstream moves it and every
/// character test reads it -- so this holds only what the walk decides and
/// what it was told.
struct BreakSearch {
    /// The cursor column the search started from.
    startcol: c_int,
    /// The column 'textwidth' falls at, and so the last one worth breaking
    /// at: reaching it ends the search.
    wantcol: c_int,
    /// Bytes of comment leader that must not be broken inside.
    leader_len: colnr_T,
    /// The character about to be inserted, or NUL. It is not in the buffer
    /// yet, so at `startcol` the walk substitutes it.
    c: c_int,
    /// Where the line will be split: the first column of the run of white
    /// space, or of the multibyte character, that ends the last word that
    /// fits. Zero means "nowhere", which stops the wrap.
    foundcol: c_int,
    /// Where the text that stays on this line ends. It differs from
    /// `foundcol` for a *run* of blanks: the break happens before the run and
    /// the replace stack has to know where it ended.
    end_foundcol: c_int,
    /// A multibyte position already answered "no" once. Re-testing it would
    /// answer the same and cost another pair of cursor moves.
    skip_pos: c_int,
    /// 'formatoptions' `m`: allow a break between two multibyte characters.
    fo_multibyte: bool,
    /// 'formatoptions' `]`: respect 'textwidth' rigorously, rather than
    /// letting one punctuation character hang past it.
    fo_rigor_tw: bool,
}

impl BreakSearch {
    /// Upstream's `WHITECHAR(cc)` arm: `cc` is white space, so the break goes
    /// in front of the run of blanks this is the end of.
    ///
    /// # Safety
    /// There must be a current line and the cursor must be on it.
    unsafe fn at_white(&mut self, mut cc: c_int) -> Step {
        // Remember where the blank just before the text is.
        let end_col = cur_win().w_cursor.col;

        // Walk back to the start of the run of blanks, counting them --
        // only "more than one" matters, for the `p` flag below.
        let mut wcc = 0;
        while cur_win().w_cursor.col > 0 && unsafe { whitechar(cc) } {
            unsafe { dec_cursor() };
            cc = unsafe { gchar_cursor() };
            if wcc < 2 {
                wcc += 1;
            }
        }
        if cur_win().w_cursor.col == 0 && unsafe { whitechar(cc) } {
            return Step::Stop; // only spaces in front of the text
        }
        // 'formatoptions' `p`: don't break after a period followed by
        // fewer than two spaces -- that is an abbreviation, not a
        // sentence end.
        if unsafe { has_format_option(FoFlag::PERIOD_ABBR) } && cc == '.' as c_int && wcc < 2 {
            return Step::Again;
        }
        // Don't break inside the comment leader.
        if cur_win().w_cursor.col < self.leader_len {
            return Step::Stop;
        }
        if unsafe { has_format_option(FoFlag::ONE_LETTER) } {
            // Don't break after a one-letter word.
            if cur_win().w_cursor.col == 0 {
                return Step::Stop; // a one-letter word at the start
            }
            // Don't break `#a b` when 'textwidth' is 2.
            if cur_win().w_cursor.col <= self.leader_len {
                return Step::Stop;
            }
            let col = cur_win().w_cursor.col;
            unsafe { dec_cursor() };
            cc = unsafe { gchar_cursor() };
            if unsafe { whitechar(cc) } {
                return Step::Again; // one letter: keep looking
            }
            cur_win().w_cursor.col = col;
        }
        unsafe { inc_cursor() };
        self.end_foundcol = end_col as c_int + 1;
        self.foundcol = cur_win().w_cursor.col as c_int;
        if cur_win().w_cursor.col <= self.wantcol {
            return Step::Stop;
        }
        Step::Back
    }

    /// Upstream's `fo_multibyte` arm: a break may go straight between two
    /// characters, with no blank in sight, if the pair allows it.
    ///
    /// # Safety
    /// There must be a current line and the cursor must be on it.
    unsafe fn at_multibyte(&mut self, mut cc: c_int) -> Step {
        let mut col;
        // First try breaking *after* this character.
        if cur_win().w_cursor.col != self.startcol {
            // Don't break inside the comment leader.
            if cur_win().w_cursor.col < self.leader_len {
                return Step::Stop;
            }
            col = cur_win().w_cursor.col;
            unsafe { inc_cursor() };
            let ncc = unsafe { gchar_cursor() };
            let allow_break = utf_allow_break(cc, ncc);
            if cur_win().w_cursor.col != self.skip_pos && allow_break {
                self.foundcol = cur_win().w_cursor.col as c_int;
                self.end_foundcol = self.foundcol;
                if cur_win().w_cursor.col <= self.wantcol {
                    return Step::Stop;
                }
            }
            cur_win().w_cursor.col = col;
        }
        if cur_win().w_cursor.col == 0 {
            return Step::Stop;
        }

        // Then breaking *before* it.
        let mut ncc = cc;
        col = cur_win().w_cursor.col;
        unsafe { dec_cursor() };
        cc = unsafe { gchar_cursor() };
        if unsafe { whitechar(cc) } {
            return Step::Again; // break with a space instead
        }
        // Don't break inside the comment leader.
        if cur_win().w_cursor.col < self.leader_len {
            return Step::Stop;
        }
        cur_win().w_cursor.col = col;
        self.skip_pos = cur_win().w_cursor.col as c_int;

        let mut allow_break = utf_allow_break(cc, ncc);
        // Honour the line-break prohibition classes even here.
        if allow_break {
            self.foundcol = cur_win().w_cursor.col as c_int;
            self.end_foundcol = self.foundcol;
        }
        if cur_win().w_cursor.col <= self.wantcol {
            let ncc_allow_break = utf_allow_break_before(ncc);
            if allow_break {
                return Step::Stop;
            }
            if !ncc_allow_break && !self.fo_rigor_tw {
                // Let at most one punctuation character hang past
                // 'textwidth'.
                if cur_win().w_cursor.col == self.startcol {
                    // The character being inserted is itself unbreakable:
                    // put the check off until the next one.
                    self.foundcol = 0;
                    self.end_foundcol = 0;
                    return Step::Stop;
                }
                // Neither `cc` nor `ncc` is NUL here, so stepping forward
                // is safe.
                col = cur_win().w_cursor.col;
                unsafe { inc_cursor() };
                cc = ncc;
                ncc = unsafe { gchar_cursor() };
                // At end of line, the character being inserted is next.
                ncc = if ncc != NUL { ncc } else { self.c };
                allow_break = utf_allow_break(cc, ncc);
                if allow_break {
                    // Break only when this is not the end of the line.
                    self.foundcol = if ncc == NUL {
                        0
                    } else {
                        cur_win().w_cursor.col as c_int
                    };
                    self.end_foundcol = self.foundcol;
                    return Step::Stop;
                }
                cur_win().w_cursor.col = col;
            }
        }
        Step::Back
    }

    /// Walk back from `startcol` until a break column is found or there is
    /// nowhere left to look, leaving the answer in `foundcol`.
    ///
    /// `flags` and `fo_ins_blank` between them decide how far back the walk
    /// may go: outside an explicit format, 'formatoptions' `v`/`b` stop it at
    /// the first character the user actually typed in this insert.
    ///
    /// # Safety
    /// There must be a current line and the cursor must be on it.
    unsafe fn run(&mut self, flags: c_int, fo_ins_blank: bool) {
        while (!fo_ins_blank && !unsafe { has_format_option(FoFlag::INS_VI) })
            || flags & INSCHAR_FORMAT as c_int != 0
            || cur_win().w_cursor.lnum != Insstart.get().lnum
            || cur_win().w_cursor.col >= Insstart.get().col
        {
            let cc = if cur_win().w_cursor.col == self.startcol as colnr_T && self.c != NUL {
                self.c
            } else {
                unsafe { gchar_cursor() }
            };
            let step = if unsafe { whitechar(cc) } {
                unsafe { self.at_white(cc) }
            } else if (cc >= 0x100 || !utf_allow_break_before(cc)) && self.fo_multibyte {
                unsafe { self.at_multibyte(cc) }
            } else {
                Step::Back
            };
            match step {
                Step::Stop => return,
                Step::Again => continue,
                Step::Back => {}
            }
            if cur_win().w_cursor.col == 0 {
                return;
            }
            unsafe { dec_cursor() };
        }
    }
}

/// The comment leader of the current line, for the purpose of not breaking
/// inside one. Answers 0 when there is none.
///
/// With 'cindent', a leader that is not at the start of the line still counts
/// -- a line comment after code -- which is what the second lookup is for.
///
/// # Safety
/// There must be a current line.
unsafe fn wrap_leader_len() -> colnr_T {
    let line = unsafe { get_cursor_line_ptr() };
    let mut leader_len =
        unsafe { get_leader_len(line, ::core::ptr::null_mut::<*mut c_char>(), false, true) };
    if leader_len == 0 && cur_buf().b_p_cin != 0 {
        let comment_start = unsafe { check_linecomment(line) };
        if comment_start != MAXCOL {
            leader_len = unsafe {
                get_leader_len(
                    line.offset(comment_start as isize),
                    ::core::ptr::null_mut::<*mut c_char>(),
                    false,
                    true,
                )
            };
            if leader_len != 0 {
                leader_len += comment_start;
            }
        }
    }
    leader_len
}

/// Format the text at the current insert position: break the line at
/// `textwidth` as often as it takes for it to fit, then leave the cursor
/// where the typing should carry on.
///
/// `c` is the character about to be inserted, which is not in the buffer yet
/// and so is counted separately; it may be NUL. `second_indent` is the indent
/// for the second line of the paragraph, and with `INSCHAR_COM_LIST` in
/// `flags` it is instead the comment leader length handed to `open_line`.
/// `format_only` suppresses the redraw, for a caller that is going to do one.
///
/// # Safety
/// There must be a current line, and it must be modifiable. Reentrant with
/// `edit.rs` through `open_line`.
pub unsafe fn internal_format(
    textwidth: c_int,
    mut second_indent: c_int,
    flags: c_int,
    format_only: bool,
    c: c_int,
) {
    let mut win = cur_win();
    let mut save_char = NUL as c_char;
    let mut haveto_redraw = false;
    let fo_ins_blank = unsafe { has_format_option(FoFlag::INS_BLANK) };
    let fo_multibyte = unsafe { has_format_option(FoFlag::MBYTE_BREAK) };
    let fo_rigor_tw = unsafe { has_format_option(FoFlag::RIGOROUS_TW) };
    let fo_white_par = unsafe { has_format_option(FoFlag::WHITE_PAR) };
    let mut first_line = true;
    let mut no_leader = false;
    let mut do_comments = flags & INSCHAR_DO_COM as c_int != 0;
    let has_lbr = win.w_onebuf_opt.wo_lbr;

    // So that `win_charsize()` counts correctly.
    win.w_onebuf_opt.wo_lbr = 0;

    // With 'autoindent' off, a space under the cursor must not be
    // deleted; stand an `x` in for it and put it back at the end.
    if cur_buf().b_p_ai == 0 && State.get() & VREPLACE_FLAG == 0 {
        let cc = unsafe { gchar_cursor() };
        if ascii_iswhite(cc) {
            save_char = cc as c_char;
            unsafe { pchar_cursor('x' as c_char) };
        }
    }

    // Break lines until the current one is no longer too long.
    while !got_int.get() {
        let mut orig_col = 0;
        let mut did_do_comment = false;

        let virtcol = unsafe { get_nolist_virtcol() }
            + unsafe { char2cells(if c != NUL { c } else { gchar_cursor() }) };
        if virtcol <= textwidth {
            break;
        }

        if no_leader {
            do_comments = false;
        } else if flags & INSCHAR_FORMAT as c_int == 0
            && unsafe { has_format_option(FoFlag::WRAP_COMS) }
        {
            do_comments = true;
        }
        let leader_len = if do_comments {
            unsafe { wrap_leader_len() }
        } else {
            0
        };

        // When this line does not start with a comment leader, don't
        // start one on a line broken off it either: otherwise a `%word`
        // moved to the next line makes every following line start `%`.
        if leader_len == 0 {
            no_leader = true;
        }
        if flags & INSCHAR_FORMAT as c_int == 0
            && leader_len == 0
            && !unsafe { has_format_option(FoFlag::WRAP) }
        {
            break;
        }
        let mut startcol = win.w_cursor.col as c_int;
        if startcol == 0 {
            break;
        }

        // Find the column 'textwidth' falls at.
        win.coladvance(textwidth);
        let wantcol = win.w_cursor.col as c_int;
        win.w_cursor.col = startcol as colnr_T;

        let mut search = BreakSearch {
            startcol,
            wantcol,
            leader_len,
            c,
            foundcol: 0,
            end_foundcol: 0,
            skip_pos: 0,
            fo_multibyte,
            fo_rigor_tw,
        };
        unsafe { search.run(flags, fo_ins_blank) };
        if search.foundcol == 0 {
            // No break column: the line has to stay long.
            win.w_cursor.col = startcol as colnr_T;
            break;
        }
        let foundcol = search.foundcol;

        // The line is going to be broken; take any `$` off first.
        unsafe { undisplay_dollar() };

        // The replace stack needs the offset between the cursor and the
        // break. MODE_VREPLACE does not use it -- it backspaces over the
        // text instead.
        if State.get() & VREPLACE_FLAG != 0 {
            orig_col = startcol; // where the backspacing will start
        } else {
            replace_offset.set(startcol - search.end_foundcol);
        }

        // Move `startcol` past the spaces that are about to be deleted
        // and the characters that stay on the top line.
        win.w_cursor.col = foundcol as colnr_T;
        while {
            let cc = unsafe { gchar_cursor() };
            (unsafe { whitechar(cc) }) && (!fo_white_par || win.w_cursor.col < startcol as colnr_T)
        } {
            unsafe { inc_cursor() };
        }
        startcol -= win.w_cursor.col as c_int;
        startcol = startcol.max(0);

        let mut saved_text = ::core::ptr::null_mut::<c_char>();
        if State.get() & VREPLACE_FLAG != 0 {
            // MODE_VREPLACE backspaces over the text being wrapped, so
            // save a copy now to put on the next line.
            saved_text = unsafe { xstrnsave(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t) };
            win.w_cursor.col = orig_col as colnr_T;
            unsafe { *saved_text.offset(startcol as isize) = NUL as c_char };
            if !fo_white_par {
                unsafe { backspace_until_column(foundcol) };
            }
        } else if !fo_white_par {
            // Put the cursor after the position to break at.
            win.w_cursor.col = foundcol as colnr_T;
        }

        // Split the line just before the margin. Only insert and delete
        // lines; don't really redraw the window.
        unsafe {
            open_line(
                FORWARD as c_int,
                (OPENLINE_DELSPACES
                    + OPENLINE_MARKFIX
                    + if fo_white_par { OPENLINE_KEEPTRAIL } else { 0 }
                    + if do_comments { OPENLINE_DO_COM } else { 0 }
                    + OPENLINE_FORMAT
                    + if flags & INSCHAR_COM_LIST as c_int != 0 {
                        OPENLINE_COM_LIST
                    } else {
                        0
                    }) as c_int,
                if flags & INSCHAR_COM_LIST as c_int != 0 {
                    second_indent
                } else {
                    old_indent.get()
                },
                &raw mut did_do_comment,
            )
        };
        if flags & INSCHAR_COM_LIST as c_int == 0 {
            old_indent.set(0);
        }
        // A comment leader was inserted, so a following line may get one
        // too.
        if did_do_comment {
            no_leader = false;
        }
        replace_offset.set(0);

        if first_line {
            if flags & INSCHAR_COM_LIST as c_int == 0 {
                // Auto-wrap of numbered lists. Outside Insert mode --
                // that is, from `format_lines` -- `INSCHAR_COM_LIST` is
                // set and `open_line` above has already done this.
                if second_indent < 0 && unsafe { has_format_option(FoFlag::Q_NUMBER) } {
                    second_indent = unsafe { get_number_indent(win.w_cursor.lnum - 1) };
                }
                if second_indent >= 0 {
                    if State.get() & VREPLACE_FLAG != 0 {
                        unsafe { change_indent(INDENT_SET as c_int, second_indent, 0, true) };
                    } else if leader_len > 0 && second_indent - leader_len > 0 {
                        // A numbered list item that has a comment:
                        // `open_line` put the leader in and left the
                        // cursor after it, so all that is missing is the
                        // white space the number wants after it.
                        let padding = second_indent - leader_len;
                        for _ in 0..padding {
                            unsafe { ins_str(c" ".as_ptr() as *mut c_char, 1) };
                        }
                    } else {
                        unsafe { set_indent(second_indent, SIN_CHANGED as c_int) };
                    }
                }
            }
            first_line = false;
        }

        if State.get() & VREPLACE_FLAG != 0 {
            // MODE_VREPLACE backspaced over the text being moved; put it
            // into the new line.
            unsafe { ins_bytes(saved_text) };
            unsafe { xfree(saved_text as *mut ::core::ffi::c_void) };
        } else {
            // Keep the cursor off the NUL past the end: cindent may have
            // added or removed indent.
            win.w_cursor.col += startcol as colnr_T;
            let len = unsafe { get_cursor_line_len() };
            win.w_cursor.col = win.w_cursor.col.min(len);
        }

        haveto_redraw = true;
        set_can_cindent(true);
        // The cursor moved: don't autoindent or cindent now.
        did_ai.set(false);
        did_si.set(false);
        can_si.set(false);
        can_si_back.set(false);
        line_breakcheck();
    }

    if save_char as c_int != NUL {
        // Put the space after the cursor back.
        unsafe { pchar_cursor(save_char) };
    }
    win.w_onebuf_opt.wo_lbr = has_lbr;

    if !format_only && haveto_redraw {
        unsafe { update_topline(win.raw()) };
        unsafe { redraw_curbuf_later(UPD_VALID) };
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
