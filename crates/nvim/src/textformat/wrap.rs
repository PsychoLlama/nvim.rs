//! Auto-wrap: breaking the line being typed at 'textwidth'.
//!
//! [`internal_format`] is called from `insertchar` (`edit.rs`) for every
//! character that could take the line over the margin, and is reentrant with
//! it: it calls `open_line`, which runs the whole indent machinery and can
//! come back here. Two questions per break, in order -- *where* may this line
//! be broken ([`BreakSearch`]), and then what it takes to break it there.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::change::OPENLINE_COM_LIST;
use crate::change::OPENLINE_DELSPACES;
use crate::change::OPENLINE_DO_COM;
use crate::change::OPENLINE_FORMAT;
use crate::change::OPENLINE_KEEPTRAIL;
use crate::change::OPENLINE_MARKFIX;
use crate::indent::INDENT_SET;
use crate::indent::SIN_CHANGED;
use crate::mbyte::Codepoint;
use crate::memline::Lines;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;
use crate::change::{get_leader_len, ins_bytes, ins_str, open_line};
use crate::charset::char2cells;
use crate::cursor::{
    dec_cursor_gchar, gchar_cursor, get_cursor_line_len, inc_cursor, inc_cursor_gchar, pchar_cursor,
};
use crate::drawscreen::{UPD_VALID, redraw_curbuf_later};
use crate::edit::{INSCHAR_COM_LIST, INSCHAR_DO_COM, INSCHAR_FORMAT};
use crate::edit::{backspace_until_column, get_nolist_virtcol, set_can_cindent, undisplay_dollar};
use crate::getchar::state::GOT_INT;
use crate::indent::{change_indent, get_number_indent, set_indent};
use crate::mbyte::{utf_allow_break, utf_allow_break_before};
use crate::r#move::update_topline;
use crate::narrow::len_as_int;
use crate::os::input::line_breakcheck;
use crate::search::{FORWARD, check_linecomment};
use crate::state::VREPLACE_FLAG;
use crate::state::mode::{
    CAN_SI, CAN_SI_BACK, DID_AI, DID_SI, INSSTART, OLD_INDENT, REPLACE_OFFSET, STATE,
};
use crate::types::ColNr;

/// What one step of the backwards search for a break column decided.
///
/// These are upstream's three ways out of the search loop's body: `break`,
/// `continue`, and falling off the end -- which steps back one character
/// before going round again.
#[derive(Debug)]
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
#[derive(Debug)]
struct BreakSearch {
    /// The cursor column the search started from.
    startcol: c_int,
    /// The column 'textwidth' falls at, and so the last one worth breaking
    /// at: reaching it ends the search.
    wantcol: c_int,
    /// Bytes of comment leader that must not be broken inside.
    leader_len: i32,
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
    /// in front of the run of blanks this is the end of. The caller has just
    /// found `whitechar(cc)` true at the cursor.
    fn at_white(&mut self, mut cc: c_int) -> Step {
        // Remember where the blank just before the text is.
        let end_col = Win::current().w_cursor.col;

        // Walk back to the start of the run of blanks, counting them --
        // only "more than one" matters, for the `p` flag below.
        let mut wcc = 0;
        // The first test is the caller's, and is not asked again.
        let mut known_white = true;
        while Win::current().w_cursor.col > ColNr::ZERO
            && (::core::mem::take(&mut known_white) || whitechar(cc))
        {
            cc = dec_cursor_gchar().1;
            if wcc < 2 {
                wcc += 1;
            }
        }
        if Win::current().w_cursor.col == ColNr::ZERO && whitechar(cc) {
            return Step::Stop; // only spaces in front of the text
        }
        // 'formatoptions' `p`: don't break after a period followed by
        // fewer than two spaces -- that is an abbreviation, not a
        // sentence end.
        if has_format_option(FoFlag::PERIOD_ABBR) && cc == c_int::from(b'.') && wcc < 2 {
            return Step::Again;
        }
        // Don't break inside the comment leader.
        if Win::current().w_cursor.col < ColNr::new(self.leader_len) {
            return Step::Stop;
        }
        if has_format_option(FoFlag::ONE_LETTER) {
            // Don't break after a one-letter word.
            if Win::current().w_cursor.col == ColNr::ZERO {
                return Step::Stop; // a one-letter word at the start
            }
            // Don't break `#a b` when 'textwidth' is 2.
            if Win::current().w_cursor.col <= ColNr::new(self.leader_len) {
                return Step::Stop;
            }
            let col = Win::current().w_cursor.col;
            cc = dec_cursor_gchar().1;
            if whitechar(cc) {
                return Step::Again; // one letter: keep looking
            }
            Win::current().w_cursor.col = col;
        }
        inc_cursor();
        self.end_foundcol = end_col.get() + 1;
        self.foundcol = Win::current().w_cursor.col.get();
        if Win::current().w_cursor.col <= ColNr::new(self.wantcol) {
            return Step::Stop;
        }
        Step::Back
    }

    /// Upstream's `fo_multibyte` arm: a break may go straight between two
    /// characters, with no blank in sight, if the pair allows it.
    fn at_multibyte(&mut self, mut cc: c_int) -> Step {
        let mut col;
        // First try breaking *after* this character.
        if Win::current().w_cursor.col != ColNr::new(self.startcol) {
            // Don't break inside the comment leader.
            if Win::current().w_cursor.col < ColNr::new(self.leader_len) {
                return Step::Stop;
            }
            col = Win::current().w_cursor.col;
            let ncc = inc_cursor_gchar().1;
            let allow_break = utf_allow_break(cc, ncc);
            if Win::current().w_cursor.col != ColNr::new(self.skip_pos) && allow_break {
                self.foundcol = Win::current().w_cursor.col.get();
                self.end_foundcol = self.foundcol;
                if Win::current().w_cursor.col <= ColNr::new(self.wantcol) {
                    return Step::Stop;
                }
            }
            Win::current().w_cursor.col = col;
        }
        if Win::current().w_cursor.col == ColNr::ZERO {
            return Step::Stop;
        }

        // Then breaking *before* it.
        let mut ncc = cc;
        col = Win::current().w_cursor.col;
        cc = dec_cursor_gchar().1;
        if whitechar(cc) {
            return Step::Again; // break with a space instead
        }
        // Don't break inside the comment leader.
        if Win::current().w_cursor.col < ColNr::new(self.leader_len) {
            return Step::Stop;
        }
        Win::current().w_cursor.col = col;
        self.skip_pos = Win::current().w_cursor.col.get();

        let mut allow_break = utf_allow_break(cc, ncc);
        // Honour the line-break prohibition classes even here.
        if allow_break {
            self.foundcol = Win::current().w_cursor.col.get();
            self.end_foundcol = self.foundcol;
        }
        if Win::current().w_cursor.col <= ColNr::new(self.wantcol) {
            let ncc_allow_break = utf_allow_break_before(ncc);
            if allow_break {
                return Step::Stop;
            }
            if !ncc_allow_break && !self.fo_rigor_tw {
                // Let at most one punctuation character hang past
                // 'textwidth'.
                if Win::current().w_cursor.col == ColNr::new(self.startcol) {
                    // The character being inserted is itself unbreakable:
                    // put the check off until the next one.
                    self.foundcol = 0;
                    self.end_foundcol = 0;
                    return Step::Stop;
                }
                // Neither `cc` nor `ncc` is NUL here, so stepping forward
                // is safe.
                col = Win::current().w_cursor.col;
                cc = ncc;
                ncc = inc_cursor_gchar().1;
                // At end of line, the character being inserted is next.
                ncc = if ncc != NUL { ncc } else { self.c };
                allow_break = utf_allow_break(cc, ncc);
                if allow_break {
                    // Break only when this is not the end of the line.
                    self.foundcol = if ncc == NUL {
                        0
                    } else {
                        Win::current().w_cursor.col.get()
                    };
                    self.end_foundcol = self.foundcol;
                    return Step::Stop;
                }
                Win::current().w_cursor.col = col;
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
    fn run(&mut self, flags: c_int, fo_ins_blank: bool) {
        // The character the last step back landed on, read with the step.
        let mut landed: Option<c_int> = None;
        while (!fo_ins_blank && !has_format_option(FoFlag::INS_VI))
            || flags & INSCHAR_FORMAT != 0
            || Win::current().w_cursor.lnum != INSSTART.get().lnum
            || Win::current().w_cursor.col >= INSSTART.get().col
        {
            let under = landed.take();
            let cc = if Win::current().w_cursor.col == ColNr::new(self.startcol) && self.c != NUL {
                self.c
            } else {
                under.unwrap_or_else(|| gchar_cursor().get())
            };
            let step = if whitechar(cc) {
                self.at_white(cc)
            } else if (cc >= 0x100 || !utf_allow_break_before(cc)) && self.fo_multibyte {
                self.at_multibyte(cc)
            } else {
                Step::Back
            };
            match step {
                Step::Stop => return,
                Step::Again => continue,
                Step::Back => {}
            }
            if Win::current().w_cursor.col == ColNr::ZERO {
                return;
            }
            landed = Some(dec_cursor_gchar().1);
        }
    }
}

/// The comment leader of the current line, for the purpose of not breaking
/// inside one. Answers 0 when there is none.
///
/// With 'cindent', a leader that is not at the start of the line still counts
/// -- a line comment after code -- which is what the second lookup is for.
fn wrap_leader_len() -> i32 {
    let mut lines = Lines::current();
    let line = lines.line(Win::current().w_cursor.lnum);
    let mut leader_len = get_leader_len(line, false, true).0;
    if leader_len == 0
        && Buf::current().b_p_cin != 0
        && let Some(comment_start) = check_linecomment(line)
    {
        leader_len = get_leader_len(&line[comment_start..], false, true).0;
        if leader_len != 0 {
            leader_len += len_as_int(comment_start);
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
pub fn internal_format(
    textwidth: c_int,
    second_indent: c_int,
    flags: c_int,
    format_only: bool,
    c: c_int,
) {
    let mut win = Win::current();
    let mut wrap = Wrap {
        textwidth,
        second_indent,
        flags,
        c,
        fo_ins_blank: has_format_option(FoFlag::INS_BLANK),
        fo_multibyte: has_format_option(FoFlag::MBYTE_BREAK),
        fo_rigor_tw: has_format_option(FoFlag::RIGOROUS_TW),
        fo_white_par: has_format_option(FoFlag::WHITE_PAR),
        first_line: true,
        no_leader: false,
        do_comments: flags & INSCHAR_DO_COM != 0,
    };
    let has_lbr = win.w_onebuf_opt.wo_lbr;

    // So that `win_charsize()` counts correctly.
    win.w_onebuf_opt.wo_lbr = 0;

    // With 'autoindent' off, a space under the cursor must not be
    // deleted; stand an `x` in for it and put it back at the end.
    let mut save_char = None;
    if Buf::current().b_p_ai == 0 && STATE.get() & VREPLACE_FLAG == 0 {
        let cc = gchar_cursor();
        if let Some(white) = u8::try_from(cc)
            .ok()
            .filter(|&b| ascii_iswhite(c_int::from(b)))
        {
            save_char = Some(white);
            pchar_cursor(b'x');
        }
    }

    // Break lines until the current one is no longer too long.
    let mut haveto_redraw = false;
    while !GOT_INT.get() && wrap.break_once() {
        haveto_redraw = true;
    }

    if let Some(white) = save_char {
        // Put the space after the cursor back.
        pchar_cursor(white);
    }
    win.w_onebuf_opt.wo_lbr = has_lbr;

    if !format_only && haveto_redraw {
        update_topline(win);
        redraw_curbuf_later(UPD_VALID);
    }
}

/// One [`internal_format`] call: what it was asked, and what each break
/// leaves for the next.
#[derive(Debug)]
struct Wrap {
    textwidth: c_int,
    /// The second line's indent (or, with `INSCHAR_COM_LIST`, the leader
    /// length for `open_line`); a numbered list may set it on the first
    /// break.
    second_indent: c_int,
    flags: c_int,
    /// The character about to be inserted, or NUL.
    c: c_int,
    fo_ins_blank: bool,
    fo_multibyte: bool,
    fo_rigor_tw: bool,
    fo_white_par: bool,
    /// No line has been broken yet.
    first_line: bool,
    /// The line had no comment leader, so the lines broken off it start
    /// none either.
    no_leader: bool,
    do_comments: bool,
}

impl Wrap {
    /// Break the cursor line once if it is too long: whether it was, and a
    /// break column was found.
    fn break_once(&mut self) -> bool {
        let mut win = Win::current();
        let virtcol = get_nolist_virtcol()
            + char2cells(if self.c != NUL {
                Codepoint::new(self.c)
            } else {
                gchar_cursor()
            });
        if virtcol <= ColNr::new(self.textwidth) {
            return false;
        }

        let leader_len = self.comment_leader_len();
        if self.flags & INSCHAR_FORMAT == 0 && leader_len == 0 && !has_format_option(FoFlag::WRAP) {
            return false;
        }
        let startcol = win.w_cursor.col.get();
        if startcol == 0 {
            return false;
        }
        let Some(search) = self.find_break(startcol, leader_len) else {
            // No break column: the line has to stay long.
            win.w_cursor.col = ColNr::new(startcol);
            return false;
        };

        let (startcol, saved_text) = self.cut_at(&search, startcol);
        self.open_broken_line();
        if self.first_line {
            self.indent_second_line(leader_len);
            self.first_line = false;
        }

        if STATE.get() & VREPLACE_FLAG != 0 {
            // MODE_VREPLACE backspaced over the text being moved; put it
            // into the new line.
            ins_bytes(&saved_text);
        } else {
            // Keep the cursor off the NUL past the end: cindent may have
            // added or removed indent.
            win.w_cursor.col += startcol;
            let len = get_cursor_line_len();
            win.w_cursor.col = win.w_cursor.col.min(len);
        }

        set_can_cindent(true);
        // The cursor moved: don't autoindent or cindent now.
        DID_AI.set(false);
        DID_SI.set(false);
        CAN_SI.set(false);
        CAN_SI_BACK.set(false);
        line_breakcheck();
        true
    }

    /// The comment leader this break must not go inside, 0 for none; and
    /// whether the broken-off line repeats it.
    fn comment_leader_len(&mut self) -> i32 {
        if self.no_leader {
            self.do_comments = false;
        } else if self.flags & INSCHAR_FORMAT == 0 && has_format_option(FoFlag::WRAP_COMS) {
            self.do_comments = true;
        }
        let leader_len = if self.do_comments {
            wrap_leader_len()
        } else {
            0
        };
        // When this line does not start with a comment leader, don't start
        // one on a line broken off it either: otherwise a `%word` moved to
        // the next line makes every following line start `%`.
        if leader_len == 0 {
            self.no_leader = true;
        }
        leader_len
    }

    /// Search back from `startcol` for a column to break at, `None` when
    /// there is none. The cursor is left where the search stopped.
    fn find_break(&self, startcol: c_int, leader_len: i32) -> Option<BreakSearch> {
        let mut win = Win::current();
        // Find the column 'textwidth' falls at.
        win.coladvance(ColNr::new(self.textwidth));
        let wantcol = win.w_cursor.col.get();
        win.w_cursor.col = ColNr::new(startcol);

        let mut search = BreakSearch {
            startcol,
            wantcol,
            leader_len,
            c: self.c,
            foundcol: 0,
            end_foundcol: 0,
            skip_pos: 0,
            fo_multibyte: self.fo_multibyte,
            fo_rigor_tw: self.fo_rigor_tw,
        };
        search.run(self.flags, self.fo_ins_blank);
        (search.foundcol != 0).then_some(search)
    }

    /// Get the line ready to be split at `search`'s column: the replace
    /// stack's offset, the cursor after the break, and (Virtual Replace)
    /// the text that moves, which it backspaces over.
    ///
    /// Answers how many bytes past the break the cursor was -- where it
    /// goes back to on the new line -- and the saved text.
    fn cut_at(&self, search: &BreakSearch, mut startcol: c_int) -> (c_int, Vec<u8>) {
        let mut win = Win::current();
        let foundcol = search.foundcol;

        // The line is going to be broken; take any `$` off first.
        undisplay_dollar();

        // The replace stack needs the offset between the cursor and the
        // break. MODE_VREPLACE does not use it -- it backspaces over the
        // text instead.
        let mut orig_col = 0;
        if STATE.get() & VREPLACE_FLAG != 0 {
            orig_col = startcol; // where the backspacing will start
        } else {
            REPLACE_OFFSET.set(startcol - search.end_foundcol);
        }

        // Move `startcol` past the spaces that are about to be deleted and
        // the characters that stay on the top line.
        win.w_cursor.col = ColNr::new(foundcol);
        while {
            let cc = gchar_cursor();
            (whitechar(cc.get())) && (!self.fo_white_par || win.w_cursor.col < ColNr::new(startcol))
        } {
            inc_cursor();
        }
        startcol -= win.w_cursor.col.get();
        startcol = startcol.max(0);

        let mut saved_text = Vec::new();
        if STATE.get() & VREPLACE_FLAG != 0 {
            // MODE_VREPLACE backspaces over the text being wrapped, so save
            // a copy now -- the `startcol` bytes from the cursor -- to put
            // on the next line.
            saved_text = {
                let cursor = win.w_cursor;
                let mut lines = Lines::current();
                let line = lines.line(cursor.lnum);
                let rest = &line[usize::try_from(cursor.col).unwrap_or(0).min(line.len())..];
                rest[..usize::try_from(startcol).unwrap_or(0).min(rest.len())].to_vec()
            };
            win.w_cursor.col = ColNr::new(orig_col);
            if !self.fo_white_par {
                backspace_until_column(foundcol);
            }
        } else if !self.fo_white_par {
            // Put the cursor after the position to break at.
            win.w_cursor.col = ColNr::new(foundcol);
        }
        (startcol, saved_text)
    }

    /// The `OPENLINE_*` flags a break opens its new line with.
    fn open_line_flags(&self) -> c_int {
        OPENLINE_DELSPACES
            + OPENLINE_MARKFIX
            + if self.fo_white_par {
                OPENLINE_KEEPTRAIL
            } else {
                0
            }
            + if self.do_comments { OPENLINE_DO_COM } else { 0 }
            + OPENLINE_FORMAT
            + if self.flags & INSCHAR_COM_LIST != 0 {
                OPENLINE_COM_LIST
            } else {
                0
            }
    }

    /// Split the line at the cursor. Only insert and delete lines; don't
    /// really redraw the window.
    fn open_broken_line(&mut self) {
        let mut did_do_comment = false;
        let second_line_indent = if self.flags & INSCHAR_COM_LIST != 0 {
            self.second_indent
        } else {
            OLD_INDENT.get()
        };
        let _ = open_line(
            FORWARD as c_int,
            self.open_line_flags(),
            second_line_indent,
            Some(&mut did_do_comment),
        );
        if self.flags & INSCHAR_COM_LIST == 0 {
            OLD_INDENT.set(0);
        }
        // A comment leader was inserted, so a following line may get one
        // too.
        if did_do_comment {
            self.no_leader = false;
        }
        REPLACE_OFFSET.set(0);
    }

    /// After the first break, indent the new line as the paragraph's second
    /// line -- in Insert mode only.
    ///
    /// Auto-wrap of numbered lists. Outside Insert mode -- that is, from
    /// `format_lines` -- `INSCHAR_COM_LIST` is set and `open_line` has
    /// already done this.
    fn indent_second_line(&mut self, leader_len: i32) {
        if self.flags & INSCHAR_COM_LIST != 0 {
            return;
        }
        if self.second_indent < 0 && has_format_option(FoFlag::Q_NUMBER) {
            self.second_indent = get_number_indent(Win::current().w_cursor.lnum - 1);
        }
        let second_indent = self.second_indent;
        if second_indent < 0 {
            return;
        }
        if STATE.get() & VREPLACE_FLAG != 0 {
            change_indent(INDENT_SET, second_indent, 0, true);
        } else if leader_len > 0 && second_indent - leader_len > 0 {
            // A numbered list item that has a comment: `open_line` put the
            // leader in and left the cursor after it, so all that is missing
            // is the white space the number wants after it.
            let padding = second_indent - leader_len;
            for _ in 0..padding {
                ins_str(b" ");
            }
        } else {
            set_indent(second_indent, SIN_CHANGED);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(flags: c_int, fo_white_par: bool, do_comments: bool) -> Wrap {
        Wrap {
            textwidth: 20,
            second_indent: -1,
            flags,
            c: NUL,
            fo_ins_blank: false,
            fo_multibyte: false,
            fo_rigor_tw: false,
            fo_white_par,
            first_line: true,
            no_leader: false,
            do_comments,
        }
    }

    #[test]
    fn a_break_opens_its_line_with_what_the_call_and_options_ask() {
        let base = OPENLINE_DELSPACES | OPENLINE_MARKFIX | OPENLINE_FORMAT;
        assert_eq!(wrap(0, false, false).open_line_flags(), base);
        // 'formatoptions' `w`: trailing white space marks a paragraph, so
        // the line being left keeps it.
        assert_eq!(
            wrap(0, true, false).open_line_flags(),
            base | OPENLINE_KEEPTRAIL
        );
        assert_eq!(
            wrap(0, false, true).open_line_flags(),
            base | OPENLINE_DO_COM
        );
        // From `format_lines`: the list's leader length rides along.
        assert_eq!(
            wrap(INSCHAR_COM_LIST, false, false).open_line_flags(),
            base | OPENLINE_COM_LIST
        );
    }
}
