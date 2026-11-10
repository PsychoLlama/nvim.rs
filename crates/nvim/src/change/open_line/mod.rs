//! `open_line` -- the new line `o`, `O`, `<CR>` and an auto-wrap all make.
//!
//! One question, asked as a pipeline, which is how it is split:
//!
//! | phase | where |
//! | --- | --- |
//! | cut the current line in two at the cursor | [`Opening::cut`] |
//! | guess an indent from 'autoindent'/'smartindent' | [`Opening::guess_indent`], [`smart`] |
//! | decide and build the comment leader | [`Opening::find_leader`], [`Opening::leader`], [`comment`] |
//! | the new line's text, blanks skipped | [`Opening::new_line_text`] |
//! | put the new line in the buffer | [`Opening::put_new_line`], [`append_new_line`] |
//! | apply the indent, feeding the replace stack | [`apply_new_indent`] |
//! | shorten the old line and splice the extmarks | [`truncate_old_line`] |
//! | reindent with 'cindent'/'indentexpr'/'lisp' | [`reindent_new_line`] |
//!
//! Two things make it harder than that reads.
//!
//! **Virtual Replace mode does not add a line at all** unless the cursor is
//! on the last line: it starts *replacing* the next one. So the new text is
//! built as if a line were being opened, then taken back off, the original
//! line put back, and the text re-inserted character by character through
//! `ins_bytes` so that every replaced byte reaches the replace stack. That is
//! the `next_line` copy in [`Opening::cut`] and the end of
//! [`Opening::put_new_line`].
//!
//! **It is reentrant with `textformat.rs`**: `internal_format` calls it to
//! break a line, and `OPENLINE_FORMAT` is how it knows. The `did_do_comment`
//! out-parameter is set when a comment leader was deliberately put in front
//! of the new line, which is what stops the second half of a broken line from
//! starting a *new* comment.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;
use crate::extmark::EXTMARK_NOOP;
use crate::extmark::EXTMARK_UNDO;
use crate::guard::Suppress;
use crate::indent::Indenter;
use crate::indent::SIN_INSERT;
use crate::indent::SIN_NOMARK;
use crate::memline::{Lines, ml_append_terminated};
use crate::memory::XString;
use crate::narrow::len_as_int;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use self::comment::{LeaderContext, build_leader, indent_after_comment_end, plan_leader};
use self::smart::smart_indent;
use super::*;
use crate::ascii::NUL;
use crate::edit::buf_prompt_text_owned;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_flags, cmdmod_set_flags};
use crate::mbyte::char_at;
use crate::option::defs::FoFlag;
use crate::optionstr::OptString;
use crate::types::Failed;
use crate::types::Key;

mod comment;
mod smart;

/// Whether Replace mode -- but not *Virtual* Replace mode, which keeps the
/// replace stack itself -- is active.
///
/// Upstream's `REPLACE_NORMAL(s)`.
fn replace_normal(state: c_int) -> bool {
    state & REPLACE_FLAG != 0 && state & VREPLACE_FLAG == 0
}

/// How many bytes of white space start `text`.
fn white_run(text: &[u8]) -> usize {
    text.iter()
        .take_while(|&&b| ascii_iswhite(c_int::from(b)))
        .count()
}

/// A prompt line cannot have anything inserted above it, so `O` on one moves
/// the prompt down instead: the prompt text is taken off the current line and
/// prepended to `text`, what will become the new line.
///
/// Answers the new line, or `None` if this was not a prompt line. Freezes
/// the marks (`CmdModFlags::LOCKMARKS`) for the move, so that
/// `b_prompt_start` stays where it is.
fn move_prompt_down(text: &[u8]) -> Option<XString> {
    if !buf_is_prompt(current_buf())
        || Win::current().w_cursor.lnum != Buf::current().b_prompt_start.mark.lnum
    {
        return None;
    }
    let lnum = Win::current().w_cursor.lnum;
    let prompt = buf_prompt_text_owned(Buf::current());
    let prompt = prompt.as_bytes();
    let rest = {
        let mut lines = Buf::current().lines();
        let line = lines.line(lnum);
        if !line.starts_with(prompt) {
            return None;
        }
        // Take the prompt off the front of the line.
        XString::from_bytes(&line[prompt.len()..])
    };
    cmdmod_add_flags(CmdModFlags::LOCKMARKS);
    let _ = ml_replace_owned(lnum, rest);
    Some(concat(prompt, text))
}

/// Put `text` in the buffer as the new line.
///
/// Answers `Some(did_append)` -- false meaning Virtual Replace overwrote the
/// following line instead of adding one -- or `None` when `ml_append` failed.
/// The cursor is left on the line *above* the new one either way.
fn append_new_line(text: &XString, old_cursor: Pos) -> Option<bool> {
    if STATE.get() & VREPLACE_FLAG == 0 || old_cursor.lnum >= ORIG_LINE_COUNT.get() {
        let line = text.as_bytes_with_nul();
        if ml_append_terminated(Win::current().w_cursor.lnum, line, false).is_err() {
            return None;
        }
        // changed_lines() is postponed: calling it here would upset
        // marker folding.
        let below = Win::current().w_cursor.lnum + 1;
        let max = MAXLNUM;
        mark_adjust(below, max, 1, 0, EXTMARK_NOOP);
        return Some(true);
    }

    // Virtual Replace: start replacing the next line.
    Win::current().w_cursor.lnum += 1;
    if Win::current().w_cursor.lnum >= INSSTART.get().lnum + VR_LINES_CHANGED.get() {
        // NL to a new line, BS back, NL again: don't save the new line
        // for undo twice. Errors are ignored.
        let _ = u_save_cursor();
        VR_LINES_CHANGED.set(VR_LINES_CHANGED.get() + 1);
    }
    let _ = ml_replace_buf_text(Buf::current(), Win::current().w_cursor.lnum, text);
    changed_bytes(Win::current().w_cursor.lnum, ColNr::ZERO);
    Win::current().w_cursor.lnum -= 1;
    Some(false)
}

/// Indent the new line, and account for the columns that takes.
///
/// `less_cols` and `newcol` are adjusted in place. `saved_line` is the old
/// line, whose indent 'copyindent' copies. The cursor must be on the line
/// above the new one.
fn apply_new_indent(
    mut newindent: c_int,
    saved_line: &[u8],
    less_cols: &mut i32,
    newcol: &mut ColNr,
    no_si: bool,
) {
    Win::current().w_cursor.lnum += 1;
    if DID_SI.get() {
        let sw = get_sw_value(Buf::current());
        if p_sr() {
            newindent -= newindent % sw;
        }
        newindent += sw;
    }

    if Buf::current().b_p_ci != 0 {
        copy_indent(newindent, saved_line);
        // Keep 'preserveindent' on so that later fiddling with the line
        // does not undo the copy; restored at the end of `open_line`.
        Buf::current().b_p_pi = c_int::from(true);
    } else {
        set_indent(newindent, SIN_INSERT | SIN_NOMARK);
    }
    *less_cols -= Win::current().w_cursor.col.get();
    AI_COL.set(Win::current().w_cursor.col);

    // In Replace mode every character of the new indent needs a NUL on
    // the replace stack, for when BS deletes it.
    if replace_normal(STATE.get()) {
        for _ in 0..Win::current().w_cursor.col.get() {
            replace_push_nul();
        }
    }
    *newcol += Win::current().w_cursor.col.get();
    if no_si {
        DID_SI.set(false);
    }
}

/// Where [`truncate_old_line`] splices the extmarks of the line that was
/// split.
#[derive(Debug)]
struct Split {
    lnum: LineNr,
    mincol: ColNr,
    less_cols: i32,
    less_cols_off: i32,
}

/// Cut the old line off at the cursor and tell the extmark tree about both
/// halves of the split.
///
/// `saved_line` becomes the buffer's line. The cursor must be back on the
/// old line, and `did_append` describe what [`append_new_line`] did.
fn truncate_old_line(
    mut saved_line: Vec<u8>,
    flags: c_int,
    trunc_line: bool,
    split: Split,
    did_append: bool,
) {
    saved_line.truncate(Win::current().w_cursor.col.as_usize());
    // Remove trailing white space, unless the caller asked to keep it --
    // and only when the line being left was auto-indented, so that white
    // space the user typed on purpose survives.
    if trunc_line && flags & OPENLINE_KEEPTRAIL == 0 {
        truncate_spaces(&mut saved_line);
    }
    let new_len = ColNr::from_index(saved_line.len());
    let _ = ml_replace_owned(Win::current().w_cursor.lnum, XString::from_vec(saved_line));

    let mut cols_spliced = 0;
    if new_len < Win::current().w_cursor.col {
        // Trailing white space went as well as the split.
        let cb = Buf::current();
        let row = Win::current().w_cursor.lnum - 1;
        let gone = Win::current().w_cursor.col - new_len;
        extmark_splice_cols(
            cb,
            row.get(),
            new_len,
            ColNr::new(gone),
            ColNr::ZERO,
            EXTMARK_UNDO,
        );
        cols_spliced = gone;
    }

    if did_append {
        let Split {
            lnum,
            mincol,
            less_cols,
            less_cols_off,
        } = split;
        // Move the extmarks of the line the cursor is on; the
        // mark_adjust() in `append_new_line` took care of the lines below.
        let added = mincol - 1 + less_cols_off - less_cols;
        let cb = Buf::current();
        let row = lnum - 1;
        let at = mincol - 1 - cols_spliced;
        let off = less_cols_off;
        let old_b = off as BCount;
        let new_b = (added.get() + 1) as BCount;
        let undo = EXTMARK_UNDO;
        let (cur_lnum, cur_col) = (Win::current().w_cursor.lnum, Win::current().w_cursor.col);
        extmark_splice(
            cb,
            row.get(),
            at,
            0,
            ColNr::new(off),
            old_b,
            1,
            added,
            new_b,
            undo,
        );
        changed_lines(cb, cur_lnum, cur_col, cur_lnum + 1, 1, true);
        // Move marks that were after the break onto the new line.
        if flags & OPENLINE_MARKFIX != 0 {
            mark_col_adjust(cur_lnum, cur_col + off, 1, -less_cols, 0);
        }
    } else {
        changed_bytes(Win::current().w_cursor.lnum, Win::current().w_cursor.col);
    }
}

/// Reindent the new line with 'lisp', 'cindent' or 'indentexpr', whichever
/// applies; 'lisp' only when no comment leader was built.
///
/// Virtual Replace handles the replace stack itself, so `State` is faked to
/// plain Insert first to stop `change_indent()` from touching it. The cursor
/// must be on the new line.
fn reindent_new_line(has_leader: bool, do_cindent: bool) {
    let vreplace_mode = if STATE.get() & VREPLACE_FLAG != 0 {
        let saved = STATE.get();
        STATE.set(MODE_INSERT);
        saved
    } else {
        0
    };

    if !p_paste() {
        if !has_leader
            && !use_indentexpr_for_lisp()
            && Buf::current().b_p_lisp != 0
            && Buf::current().b_p_ai != 0
        {
            fixthisline(Indenter::Lisp);
            AI_COL.set(getwhitecols_curline());
        } else if do_cindent || (Buf::current().b_p_ai != 0 && use_indentexpr_for_lisp()) {
            do_c_expr_indent();
            AI_COL.set(getwhitecols_curline());
        }
    }

    if vreplace_mode != 0 {
        STATE.set(vreplace_mode);
    }
}

/// Add a new line below or above the current one.
///
/// `dir` is `FORWARD` (`o`, `<CR>`) or `BACKWARD` (`O`). `flags` is the
/// `OPENLINE_*` set. `second_line_indent` is the indent wanted after `CTRL-D`
/// in Insert mode -- or, with `OPENLINE_COM_LIST`, the *column* a 'formatlistpat'
/// item's text should line up at. `did_do_comment` is set to true when a
/// comment leader was deliberately put in front of the new line.
///
/// The caller takes care of undo. Virtual Replace may touch any number of
/// lines, so this calls `u_save_cursor()` again itself when it starts on a
/// new one. The cursor must be on a valid line of the current buffer.
///
/// `Err` only when `ml_append` failed.
pub fn open_line(
    dir: c_int,
    flags: c_int,
    second_line_indent: c_int,
    did_do_comment: Option<&mut bool>,
) -> Result<(), Failed> {
    let saved_pi = Buf::current().b_p_pi;
    let opening = Opening::cut(dir, flags, second_line_indent);
    let (newindent, no_si) = opening.guess_indent();
    let do_cindent = wants_cindent(dir, flags);
    let lead = opening.find_leader(do_cindent, did_do_comment);
    let mut layout = Layout {
        newindent,
        no_si,
        newcol: ColNr::ZERO,
        less_cols: 0,
        less_cols_off: 0,
        lead_len: lead.len,
    };
    let leader = opening.leader(lead, &mut layout);
    let has_leader = leader.is_some();
    let new_text = opening.new_line_text(leader, &mut layout);

    let old_cmod_flags = cmdmod_flags();
    let retval = opening.put_new_line(new_text, layout, has_leader, do_cindent);
    Buf::current().b_p_pi = saved_pi;
    cmdmod_set_flags(old_cmod_flags);
    retval
}

/// The line being split, and what the first look at it learned.
#[derive(Debug)]
struct Opening {
    dir: c_int,
    flags: c_int,
    second_line_indent: c_int,
    do_si: bool,
    /// Where the cursor was: the line split, and one past the column.
    lnum: LineNr,
    mincol: ColNr,
    /// A copy of the current line, so that it can be cut in two.
    saved_line: Vec<u8>,
    /// Virtual Replace's copy of the line it is about to start replacing.
    next_line: Option<XString>,
    /// What moves to the new line: the tail of the old one, from `extra`
    /// on (Insert state only, not Virtual Replace).
    extra: Option<usize>,
    extra_len: usize,
    /// The first character after the break, for 'smartindent'.
    first_char: c_int,
    /// Nothing was typed on the line being left: truncate it.
    trunc_line: bool,
}

/// The columns and indent the phases after the first adjust in turn.
#[derive(Debug)]
struct Layout {
    newindent: c_int,
    /// 'smartindent' asked for no extra indent after all.
    no_si: bool,
    /// The cursor column on the new line.
    newcol: ColNr,
    /// Columns the marks after the break move left by.
    less_cols: i32,
    /// Blanks skipped at the start of the moving tail.
    less_cols_off: i32,
    lead_len: c_int,
}

/// The comment leader the current line starts with (or ends in, after
/// code), as [`Opening::find_leader`] found it.
#[derive(Debug)]
struct Lead {
    len: c_int,
    flags: Option<ComFlags>,
    /// Where a line comment after code starts.
    comment_start: c_int,
}

/// Whether to reindent once the line is open: 'cindent' or 'indentexpr' and
/// the `o`/`O` entry of 'cinkeys'.
fn wants_cindent(dir: c_int, flags: c_int) -> bool {
    let key = if dir == FORWARD {
        Key::OPEN_FORWARD
    } else {
        Key::OPEN_BACKWARD
    };
    // The short circuits are upstream's: `in_cinkeys` reads 'cinkeys' and
    // the cursor line.
    !p_paste()
        && (Buf::current().b_p_cin != 0 || c_int::from(Buf::current().b_p_inde.first_byte()) != NUL)
        && in_cinkeys(key, b' ', linewhite(Win::current().w_cursor.lnum))
        && flags & OPENLINE_FORCE_INDENT == 0
}

impl Opening {
    /// Copy the cursor line and decide where it is cut.
    fn cut(dir: c_int, flags: c_int, second_line_indent: c_int) -> Opening {
        let do_si = may_do_si();
        let lnum = Win::current().w_cursor.lnum;
        let mincol = Win::current().w_cursor.col + 1;
        let mut saved_line = Lines::current().line(lnum).to_vec();
        let col = Win::current().w_cursor.col.as_usize().min(saved_line.len());
        let mut next_line = None;

        if STATE.get() & VREPLACE_FLAG != 0 {
            // Virtual Replace keeps a copy of the line it is about to start
            // replacing. The new line is built empty so that the indent and
            // leader machinery can do as it likes; what it produces is taken
            // off again at the end and inserted character by character over
            // the original.  -- webb.
            next_line = Some(if lnum < ORIG_LINE_COUNT.get() {
                XString::from_bytes(Lines::current().line(lnum + 1))
            } else {
                XString::new()
            });

            // A NL replaces the rest of the line, so everything past the
            // cursor goes on the replace stack. Twice, because BS over a NL
            // expects it.
            replace_push_nul();
            replace_push_nul();
            replace_push(&saved_line[col..]);
            saved_line.truncate(col);
        }

        // Until the leader is built the line is read as ending at `extra`
        // -- the C writes a NUL over the cut and puts the byte back
        // afterwards.
        let mut extra = None;
        let mut extra_len = 0;
        let mut first_char = NUL;
        if STATE.get() & MODE_INSERT != 0 && STATE.get() & VREPLACE_FLAG == 0 {
            extra = Some(col);
            let tail = &saved_line[col..];
            if do_si {
                // 'smartindent' wants the first character after the break.
                first_char = c_int::from(byte_at(tail, white_run(tail)));
            }
            extra_len = tail.len();
        }

        u_clearline(Buf::current()); // "U" cannot undo added lines
        DID_SI.set(false);
        AI_COL.set(ColNr::ZERO);

        // An auto-indent means nothing was typed on the line being left, so
        // it should be truncated. Also true when only a comment leader was
        // inserted automatically, which sets did_ai too.
        let trunc_line = dir == FORWARD && DID_AI.get();

        Opening {
            dir,
            flags,
            second_line_indent,
            do_si,
            lnum,
            mincol,
            saved_line,
            next_line,
            extra,
            extra_len,
            first_char,
            trunc_line,
        }
    }

    /// The old line up to the cut.
    fn head(&self) -> &[u8] {
        &self.saved_line[..self.extra.unwrap_or(self.saved_line.len())]
    }

    /// The new line's indent from 'autoindent'/'smartindent' (or the forced
    /// one), and whether 'smartindent' then asked for no extra indent.
    fn guess_indent(&self) -> (c_int, bool) {
        if self.flags & OPENLINE_FORCE_INDENT != 0 {
            return (self.second_line_indent, false);
        }
        if Buf::current().b_p_ai == 0 && !self.do_si {
            return (0, false);
        }
        let head = self.head();
        let mut newindent = Buf::current().indent_width(head);
        let mut no_si = false;
        if newindent == 0 && self.flags & OPENLINE_COM_LIST == 0 {
            newindent = self.second_line_indent; // CTRL-D in Insert mode
        }
        // Text moving to the next line that starts with `{` gets no extra
        // indent, so that a <CR> before the `{` of "if (cond) {" works.
        if !self.trunc_line
            && self.do_si
            && !head.is_empty()
            && (self.extra.is_none() || self.first_char != c_int::from(b'{'))
        {
            (newindent, no_si) = smart_indent(self.dir, self.flags, head, newindent);
        }
        if self.do_si {
            CAN_SI.set(true);
        }
        DID_AI.set(true);
        (newindent, no_si)
    }

    /// Does the current line start with a comment leader that should be
    /// repeated in front of the new line?
    fn find_leader(&self, do_cindent: bool, did_do_comment: Option<&mut bool>) -> Lead {
        END_COMMENT_PENDING.set(None);
        let mut lead = Lead {
            len: 0,
            flags: None,
            comment_start: 0,
        };
        if self.flags & OPENLINE_DO_COM == 0 {
            return lead;
        }
        let head = self.head();
        (lead.len, lead.flags) = get_leader_len(head, self.dir == BACKWARD, true);
        if lead.len == 0
            && Buf::current().b_p_cin != 0
            && do_cindent
            && self.dir == FORWARD
            && (!has_format_option(FoFlag::NO_OPEN_COMS) || self.flags & OPENLINE_FORMAT != 0)
        {
            // A line comment after code: `code(); // why`.
            if let Some(from) = check_linecomment(head) {
                lead.comment_start = len_as_int(from);
                let (len, tried) = get_leader_len(&head[from..], false, true);
                // One out-parameter for both scans upstream: a scan that
                // tried nothing leaves the first one's answer.
                lead.flags = tried.or(lead.flags);
                lead.len = len;
                if lead.len != 0 {
                    lead.len += lead.comment_start;
                    if let Some(did_do_comment) = did_do_comment {
                        *did_do_comment = true;
                    }
                }
            }
        }
        lead
    }

    /// Build the comment leader for the new line, if there is one to repeat.
    ///
    /// Updates the indent, the cursor column and the leader's length in
    /// `layout`; a comment that ends on this line may set the indent too.
    fn leader(&self, lead: Lead, layout: &mut Layout) -> Option<Vec<u8>> {
        let lead_flags = lead.flags.filter(|_| layout.lead_len > 0)?;
        let head = self.head();
        let plan = plan_leader(
            self.dir,
            layout.lead_len,
            lead_flags,
            head,
            self.extra.is_some(),
        );
        layout.lead_len = plan.lead_len;
        if layout.lead_len > 0 {
            let ctx = LeaderContext {
                comment_start: lead.comment_start,
                extra_len: self.extra_len,
                second_line_indent: self.second_line_indent,
            };
            let built = build_leader(plan, lead_flags, head, ctx, self.do_si, layout.newindent);
            layout.lead_len = built.lead_len;
            layout.newcol = built.newcol;
            layout.newindent = built.newindent;
            return Some(built.text);
        }
        if let Some(comment_end) = plan.comment_end {
            layout.newindent =
                indent_after_comment_end(comment_end, head, self.do_si, layout.newindent);
        }
        None
    }

    /// Where the text moving to the new line starts.
    ///
    /// With 'autoindent' or `OPENLINE_DELSPACES` the blanks after the cut
    /// are skipped. In Replace mode they go on the replace stack, preceded
    /// by a NUL, so BS can put them back. Only reached with `dir ==
    /// FORWARD`, in Insert or Replace state.
    fn skip_moving_blanks(&self, layout: &mut Layout) -> usize {
        let Some(mut from) = self.extra else {
            return self.saved_line.len();
        };
        let saved_line = &self.saved_line;
        if replace_normal(STATE.get()) {
            replace_push_nul(); // end of the extra blanks
        }
        if Buf::current().b_p_ai != 0 || self.flags & OPENLINE_DELSPACES != 0 {
            while matches!(byte_at(saved_line, from), b' ' | b'\t')
                && !utf_iscomposing_first(char_at(&saved_line[from + 1..]))
            {
                if replace_normal(STATE.get()) {
                    // The byte the loop just tested. Always ASCII.
                    replace_push(&saved_line[from..=from]);
                }
                from += 1;
                layout.less_cols_off += 1;
            }
        }
        // Columns for the marks, adjusted for the ones just removed.
        layout.less_cols = len_as_int(from);
        from
    }

    /// The new line's text: the leader, if any, then what moves down.
    fn new_line_text(&self, leader: Option<Vec<u8>>, layout: &mut Layout) -> XString {
        let tail = &self.saved_line[self.skip_moving_blanks(layout)..];
        let Some(mut text) = leader else {
            END_COMMENT_PENDING.set(None); // there was no leader after all
            return XString::from_bytes(tail); // an empty line when nothing moves
        };
        if self.flags & OPENLINE_COM_LIST != 0 && self.second_line_indent > 0 {
            // 'formatlistpat': pad after the comment leader so that the text
            // lines up with the first line's. The white space *before* the
            // leader is `set_indent`'s job later.
            let padding = self.second_line_indent - (layout.newindent + len_as_int(text.len()));
            if let Ok(pad) = usize::try_from(padding)
                && pad > 0
            {
                text.resize(text.len() + pad, b' ');
                layout.less_cols -= padding;
                layout.newcol += padding;
            }
        }
        text.extend_from_slice(tail);
        DID_AI.set(true); // so that truncating blanks works with comments
        layout.less_cols -= layout.lead_len;
        XString::from_vec(text)
    }

    /// Put `new_text` in the buffer, indent it, shorten the old line and
    /// leave the cursor on the new one. `Err` only when `ml_append` failed.
    fn put_new_line(
        mut self,
        mut new_text: XString,
        mut layout: Layout,
        has_leader: bool,
        do_cindent: bool,
    ) -> Result<(), Failed> {
        let splice = Suppress::splice();
        let old_cursor = Win::current().w_cursor;
        if self.dir == BACKWARD {
            if let Some(moved) = move_prompt_down(&new_text) {
                new_text = moved;
            }
            Win::current().w_cursor.lnum -= 1;
        }

        let mut did_append = append_new_line(&new_text, old_cursor).ok_or(Failed)?;

        let uncounted = Suppress::delete_count();
        if layout.newindent != 0 || DID_SI.get() {
            apply_new_indent(
                layout.newindent,
                &self.saved_line,
                &mut layout.less_cols,
                &mut layout.newcol,
                layout.no_si,
            );
        }
        drop(uncounted);

        // One NUL on the replace stack per character of the leader, for
        // when BS deletes it.
        if replace_normal(STATE.get()) {
            for _ in 0..layout.lead_len {
                replace_push_nul();
            }
        }

        Win::current().w_cursor = old_cursor;
        if self.dir == FORWARD {
            if self.trunc_line || STATE.get() & MODE_INSERT != 0 {
                let split = Split {
                    lnum: self.lnum,
                    mincol: self.mincol,
                    less_cols: layout.less_cols,
                    less_cols_off: layout.less_cols_off,
                };
                // The buffer owns the line now.
                let line = core::mem::take(&mut self.saved_line);
                truncate_old_line(line, self.flags, self.trunc_line, split, did_append);
                did_append = false;
            }
            // Put the cursor on the new line. `old_cursor`, not
            // `w_cursor`: a scroll above may have moved the latter.
            Win::current().w_cursor.lnum = old_cursor.lnum + 1;
        }
        if did_append {
            let cb = Buf::current();
            let at = Win::current().w_cursor.lnum;
            let extra = BCount::try_from(cb.lines().line_len(at).get()).unwrap_or(0);
            extmark_splice(
                cb,
                (at - 1).get(),
                ColNr::ZERO,
                0,
                ColNr::ZERO,
                0,
                1,
                ColNr::ZERO,
                1 + extra,
                EXTMARK_UNDO,
            );
            changed_lines(cb, at, ColNr::ZERO, at, 1, true);
        }
        drop(splice);

        Win::current().w_cursor.col = layout.newcol;
        Win::current().w_cursor.coladd = 0;

        reindent_new_line(has_leader, do_cindent);

        if STATE.get() & VREPLACE_FLAG != 0 {
            // Take what ended up on the new line back off, put the original
            // line back, and insert the new text character by character so
            // that each replaced byte reaches the replace stack.
            let at = Win::current().w_cursor.lnum;
            let new_text = Lines::current().line(at).to_vec();
            if let Some(next_line) = self.next_line.take() {
                let _ = ml_replace_owned(at, next_line);
            }
            Win::current().w_cursor.col = ColNr::ZERO;
            Win::current().w_cursor.coladd = 0;
            ins_bytes(&new_text); // calls changed_bytes()
        }
        Ok(())
    }
}
