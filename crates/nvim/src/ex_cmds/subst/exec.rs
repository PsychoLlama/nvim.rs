//! `do_sub` -- the substitute engine, and the state its stages hand around.
//!
//! Upstream is one 1,050-line function.  It is here as five named stages:
//! [`args`](super::args) reads the command line, [`substitute_range`] walks
//! the range, [`match_loop`] walks the matches within one line,
//! [`confirm`](super::confirm) is the `c` flag's dialogue and
//! [`replace`](super::replace) builds the new text and puts it in the buffer.
//! [`finish`] is what happens once the last line is done -- the cursor, the
//! marks, the report and the `'inccommand'` preview.
//!
//! [`Sub`] is the state those stages share.  Its comments are upstream's
//! description of how the new text is built up piece by piece, which is the
//! part of `:s` that is genuinely hard: [`Sub::old_line`] is the old text
//! unmodified (upstream's `sub_firstline`), `copied` how far it has been
//! copied over (`copycol`), `matchcol` where to look for the next match, and
//! [`Sub::new_line`] the text produced so far (`new_start`, and the
//! `new_start_len` its growth policy needed).
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::args::{SubSetup, parse_sub};
use super::confirm::{Confirm, ask_confirm};
use super::replace::{SubLine, build_replacement, commit_line};
use super::{do_sub_msg, global_need_beginline, show_sub, static_cstr_optval, subflags};
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::cursor::coladvance;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::{LineData, PreviewLines, SID_NONE, SubResult, print_line, re_multiline};
use crate::ex_cmds::{sub_nlines, sub_nsubs};
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::state::global_busy;
use crate::ex_eval::aborting;
use crate::fold::has_any_folding;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::highlight_group::syn_check_group;
use crate::mark::setpcmark;
use crate::mbyte::cluster_len;
use crate::memory::xfree;
use crate::message::e_interr;
use crate::message::{emsg, msg};
use crate::message_fmt::c_str;
use crate::r#move::changed_window_setting;
use crate::option::set_option_direct;
use crate::option::vars::{p_ch, p_cwh, p_icm};
use crate::options::kOptInccommand;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::pos::MAXCOL;
use crate::profile::profile_passed_limit;
use crate::regexp::{vim_regexec_multi, vim_regfree};
use crate::search::get_search_pat;
use crate::semsg;
use crate::types::ui::kUIMessages;
use crate::types::{
    CmdModFlags, ColNr, ExArg, Handle, LPos, LineNr, NUL, OptInt, OptionSetFlags, Pos, ProfTime,
    RegMMatch, int64_t, size_t,
};
use crate::ui::ui_has;
use crate::undo::u_save_cursor;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// The highlight group the preview marks matches with, looked up once.
static pre_hl_id: GlobalCell<c_int> = GlobalCell::new(0);

/// What the whole command was asked for, and never changes while it runs.
pub(super) struct SubArgs {
    /// The command's range, which the `'[`/`']` marks and the preview's
    /// "is this more than the current line" test are all it wanted the
    /// command block for.
    pub range: (LineNr, LineNr),
    /// When the substitute takes longer than this the preview gives up.
    pub timeout: ProfTime,
    /// The namespace to draw `'inccommand'` highlights in; `<= 0` means this
    /// is a real substitute, not a preview.
    pub cmdpreview_ns: c_int,
    pub cmdpreview_bufnr: Handle,
    /// Was a replacement given at all?  Without a closing delimiter a preview
    /// only highlights.
    pub has_second_delim: bool,
    /// Whether a pattern was given rather than reused.
    pub pat_given: bool,
    /// Vi quirk: a repeated `:s` after `$` leaves the cursor in the last
    /// column.
    pub endcolumn: bool,
    pub old_cursor: Pos,
    pub old_line_count: LineNr,
    /// `sub_nsubs` before this command, so that a `:global` can tell whether
    /// *this* `:s` did anything.
    pub start_nsubs: c_int,
    /// The user's `g` and `c` flags, put back for `:&&`.
    pub save_do_all: bool,
    pub save_do_ask: bool,
}

/// Everything the substitute's stages hand one another.
pub(super) struct Sub {
    /// The compiled pattern and its last match.
    pub regmatch: RegMMatch,
    /// The replacement text, owned.
    pub sub: *mut c_char,

    // --- the range ---
    /// The line where the start of the match was found.  Can be below the
    /// line searched, when there is a `\n` before a `\zs` in the pattern.
    pub lnum: LineNr,
    /// Last line of the range.  The `l` answer and a `\r` in the replacement
    /// both move it.
    pub line2: LineNr,
    pub got_quit: bool,
    pub got_match: bool,
    /// Whether undo has been saved for this command yet -- extmarks need it
    /// saved once, before the first change.
    pub did_save: bool,
    /// First changed line, and the line below the last changed one *after*
    /// the change.  Zero until something changes.
    pub first_line: LineNr,
    pub last_line: LineNr,
    pub preview_lines: PreviewLines,

    // --- the line being substituted ---
    /// How many lines the last regexp match spanned, or -1 when it has to be
    /// searched for again.
    pub nmatch: c_int,
    /// A copy of the first line of the match, unmodified.  Absent until the
    /// line it names is loaded.
    pub old_line: Option<SubLine>,
    /// The line in the buffer to look for a match in.  Differs from `lnum`
    /// when the pattern or the replacement contains line breaks.
    pub sub_firstlnum: LineNr,
    /// Offset into the old text from which text still has to be copied over.
    pub copied: usize,
    /// Column of the old text to look for the next match at: just after the
    /// previous match, or one further.
    pub matchcol: ColNr,
    /// Column just after the previous match, if any.  Equal to `matchcol`
    /// except for the first match and after skipping an empty one.
    pub prev_matchcol: ColNr,
    /// The new text, all that has been produced so far.  Absent until the
    /// first substitution on this line has been built.
    pub new_line: Option<SubLine>,
    /// Length of the substitution, including its NUL.
    pub sublen: c_int,
    pub did_sub: bool,
    /// Number of lines matched below `lnum`, waiting to be deleted.
    pub nmatch_tl: LineNr,
    /// Try again after joining lines.
    pub do_again: bool,
    pub skip_match: bool,
    /// Where the substitutions on this line started.
    pub lnum_start: LineNr,
    /// Per-match data, sent to `extmark_splice` in a batch once the line has
    /// been replaced.
    pub line_matches: Vec<LineData>,
}

impl Sub {
    /// The copy of the old line, which every read of it happens after it was
    /// loaded.
    pub(super) fn old_line(&self) -> &SubLine {
        self.old_line
            .as_ref()
            .expect("the old line is loaded before a match is looked for")
    }

    /// The line being rebuilt, which every read of it happens after the
    /// first substitution on the line was built.
    pub(super) fn new_line(&self) -> &SubLine {
        self.new_line
            .as_ref()
            .expect("a substitution has been built on this line")
    }

    /// [`Sub::new_line`], writable.
    pub(super) fn new_line_mut(&mut self) -> &mut SubLine {
        self.new_line
            .as_mut()
            .expect("a substitution has been built on this line")
    }

    /// Take a fresh copy of the buffer line `sub_firstlnum` names, so that it
    /// cannot be taken away by a screen update or a multi-line match.
    pub(super) fn load_firstline(&mut self) {
        self.old_line = Some(SubLine::from_line(self.sub_firstlnum));
    }

    /// After a multi-line match, continue in a copy of the *last* matched
    /// line -- upstream's `ADJUST_SUB_FIRSTLNUM`.
    pub(super) fn adjust_sub_firstlnum(&mut self) {
        if self.nmatch > 1 as c_int {
            self.sub_firstlnum += self.nmatch as LineNr - 1 as LineNr;
            self.load_firstline();
            // When going beyond the last line, stop substituting.
            if self.sub_firstlnum <= self.line2 {
                self.do_again = true;
            } else {
                subflags.with_mut(|flags| flags.do_all = false);
            }
        }
        if self.skip_match {
            // Already hit the end of the buffer: sub_firstlnum is one less
            // than it ought to be.
            self.old_line = Some(SubLine::default());
            self.copied = 0;
        }
    }
}

/// Does the replacement start with `\=`, so that it is an expression rather
/// than text?
///
/// # Safety
/// `sub` must be a live C string.
pub(super) unsafe fn is_expr_sub(sub: *const c_char) -> bool {
    // SAFETY: caller's contract -- reading the second byte is safe because
    // the first was not the terminator.
    unsafe { *sub as c_int == '\\' as c_int && *sub.add(1) as c_int == '=' as c_int }
}

/// Search for the pattern in the current buffer, from `lnum`:`col`.
///
/// # Safety
/// Main thread; `regmatch` must hold a compiled program.
pub(super) unsafe fn regexec_at(regmatch: *mut RegMMatch, lnum: LineNr, col: ColNr) -> c_int {
    let (win, buffer) = (Win::current(), Buf::current());
    let (timeout, timed_out) = (ptr::null_mut(), ptr::null_mut());
    unsafe { vim_regexec_multi(regmatch, Some(win), buffer, lnum, col, timeout, timed_out) }
}

/// Record a match for the `'inccommand'` preview, and how many lines it adds
/// to what the preview window will have to show.
fn push_preview(preview_lines: &mut PreviewLines, current_match: SubResult) {
    let match_lines = current_match.end.lnum - current_match.start.lnum + 1 as LineNr;
    let continues =
        preview_lines.subresults.last().map(|last| last.end.lnum) == Some(current_match.start.lnum);
    preview_lines.lines_needed += if continues {
        match_lines - 1 as LineNr
    } else {
        match_lines
    };
    preview_lines.subresults.push(current_match);
}

/// Stages 1 to 3 for one match: the empty-match rule, the `n` counter, the
/// `c` dialogue, the clamp to the end of the buffer, and then either the
/// preview's bookkeeping or the real replacement.
///
/// Returning is upstream's `goto skip`.
fn match_one(st: &mut Sub, args: &SubArgs, current_match: &mut SubResult) {
    // 1. A match of the empty string does not count, except for the first
    //    match.  This reproduces the strange vi behaviour, and also catches
    //    endless loops.
    if st.matchcol == st.prev_matchcol
        && st.regmatch.endpos[0].lnum == 0 as LineNr
        && st.matchcol == st.regmatch.endpos[0].col
    {
        let old = st.old_line().bytes();
        if st.matchcol as usize >= old.len() {
            // Already at the end of the line: don't look for a match in this
            // line again.
            st.skip_match = true;
        } else {
            // Search for a match at the next column.
            st.matchcol += cluster_len(&old[st.matchcol as usize..]) as ColNr;
        }
        // The match will be pushed to preview_lines: bring it into a proper
        // state first.
        current_match.start.col = st.matchcol;
        current_match.end.lnum = st.sub_firstlnum;
        current_match.end.col = st.matchcol;
        return;
    }

    // Normally we continue searching for a match just after the previous one.
    st.matchcol = st.regmatch.endpos[0].col;
    st.prev_matchcol = st.matchcol;

    // 2. With the "n" flag only increase the counter.  With "c", ask.
    if subflags.with(|flags| flags.do_count) {
        // For a multi-line match, put matchcol at the NUL at the end of the
        // line and set nmatch to one, so that we continue looking for a match
        // on the next line.  Avoids that ":s/\nB\@=//gc" gets stuck.
        if st.nmatch > 1 as c_int {
            st.matchcol = st.old_line().len() as ColNr;
            st.nmatch = 1 as c_int;
            st.skip_match = true;
        }
        sub_nsubs.set(sub_nsubs.get() + 1);
        st.did_sub = true;
        // Skip the substitution, unless an expression is used: then it is
        // evaluated in the sandbox.
        // SAFETY: `sub` is a live C string.
        if !unsafe { is_expr_sub(st.sub) } {
            return;
        }
    }

    if subflags.with(|flags| flags.do_ask) && args.cmdpreview_ns <= 0 as c_int {
        match ask_confirm(st) {
            Confirm::Replace => {}
            Confirm::Skip | Confirm::Quit => return,
        }
    }

    // Move the cursor to the start of the match, so that we can use
    // "\=col('.')".
    Win::current().w_cursor.col = st.regmatch.startpos[0].col;
    let line_count = Buf::current().b_ml.ml_line_count;

    // When the match included the "$" of the last line it may go beyond the
    // last line of the buffer.
    if st.nmatch as LineNr > line_count - st.sub_firstlnum + 1 as LineNr {
        st.nmatch = (line_count - st.sub_firstlnum + 1 as LineNr) as c_int;
        current_match.end.lnum = st.sub_firstlnum + st.nmatch as LineNr;
        st.skip_match = true;
        // Safety check.
        if st.nmatch < 0 as c_int {
            return;
        }
    }

    // Save the line numbers for the preview buffer.  If the pattern matches a
    // final newline the next line is shown too, but not highlighted --
    // intentional for now.
    if args.cmdpreview_ns > 0 as c_int && !args.has_second_delim {
        current_match.start.col = st.regmatch.startpos[0].col;
        if current_match.end.lnum == 0 as LineNr {
            current_match.end.lnum = st.sub_firstlnum + st.nmatch as LineNr - 1 as LineNr;
        }
        current_match.end.col = st.regmatch.endpos[0].col;
        st.adjust_sub_firstlnum();
        st.lnum += st.nmatch as LineNr - 1 as LineNr;
        return;
    }

    // 3. Substitute the string.  During an 'inccommand' preview only do this
    //    if there is a replacement pattern.
    if args.cmdpreview_ns <= 0 as c_int || args.has_second_delim {
        build_replacement(st, args, current_match);
    }
}

/// Have we done the last substitution on this line?
///
/// We already know we have when we are at the end of the line, except that a
/// pattern like `bar\|\nfoo` may match at the NUL.  `lnum` can be below
/// `line2` when there is a `\zs` in the pattern after a line break.
fn is_last_match(st: &Sub) -> bool {
    if st.skip_match || got_int.get() || st.got_quit || st.lnum > st.line2 {
        return true;
    }
    if !(subflags.with(|flags| flags.do_all) || st.do_again) {
        return true;
    }
    // Upstream only reads the byte at `matchcol` once the tests above have
    // all failed, and it must stay that way: after a multi-line match that
    // took the `goto skip` before the line was adjusted, `matchcol` is a
    // column of a *different* line and can be past this one's end -- which
    // here answers "at the end of the line" rather than reading out of
    // bounds, and is why the answer is a length comparison.
    // SAFETY: caller's contract -- the compiled program is live.
    st.matchcol as usize >= st.old_line().len()
        && st.nmatch <= 1 as c_int
        && !unsafe { re_multiline(st.regmatch.regprog) }
}

/// Loop until there is nothing more to replace on this line.
fn match_loop(st: &mut Sub, args: &SubArgs) {
    loop {
        let mut current_match = SubResult {
            start: LPos {
                lnum: 0 as LineNr,
                col: 0 as ColNr,
            },
            end: LPos {
                lnum: 0 as LineNr,
                col: 0 as ColNr,
            },
            pre_match: 0 as LineNr,
        };

        // Advance "lnum" to the line where the match starts.  The match does
        // not start in the first line when there is a line break before \zs.
        if st.regmatch.startpos[0].lnum > 0 as LineNr {
            current_match.pre_match = st.lnum;
            st.lnum += st.regmatch.startpos[0].lnum;
            st.sub_firstlnum += st.regmatch.startpos[0].lnum;
            st.nmatch -= st.regmatch.startpos[0].lnum as c_int;
            st.old_line = None;
        }

        // Now we are at the line where the pattern match starts.  If this is
        // not the first match on the line, the column is not known here.
        current_match.start.lnum = st.sub_firstlnum;

        // The match might be after the last line, for "\n\zs" matching at the
        // end of the last line.
        // SAFETY: the current buffer is live.
        if st.lnum > Buf::current().b_ml.ml_line_count {
            break;
        }
        if st.old_line.is_none() {
            st.load_firstline();
        }

        // Save the line number of the last change for the final cursor
        // position, just like Vi.
        // SAFETY: the current window is live.
        Win::current().w_cursor.lnum = st.lnum;
        st.do_again = false;

        match_one(st, args, &mut current_match);

        // 4. Find the next match, if "g" was given.  Guard against an endless
        //    loop with patterns that match the empty string, e.g. ":s/$/pat/g"
        //    or ":s/[a-z]* /(&)/g" -- but ":s/\n/#/" is fine.
        let lastone = is_last_match(st);
        st.nmatch = -1 as c_int;

        // Replace the line in the buffer when needed.  This is skipped when
        // there are more matches.  The nmatch_tl check is needed for when
        // multi-line matching must replace the lines before trying another
        // match, otherwise "\@<=" won't work; and when the match starts below
        // where we started searching we also need to replace the line first
        // (using \zs after \n).
        let no_more = if lastone || st.nmatch_tl > 0 as LineNr {
            true
        } else {
            // SAFETY: the program is compiled.
            st.nmatch = unsafe { regexec_at(&raw mut st.regmatch, st.sub_firstlnum, st.matchcol) };
            st.nmatch == 0 as c_int || st.regmatch.startpos[0].lnum > 0 as LineNr
        };

        if no_more {
            if st.new_line.is_some() {
                if !commit_line(st) {
                    break;
                }
            }
            if st.nmatch == -1 as c_int && !lastone {
                // SAFETY: the program is compiled.
                st.nmatch =
                    unsafe { regexec_at(&raw mut st.regmatch, st.sub_firstlnum, st.matchcol) };
            }

            // 5. Break if there isn't another match in this line.
            if st.nmatch <= 0 as c_int {
                // If the match found didn't start where we were searching, do
                // the next search in the line where we found the match.
                if st.nmatch == -1 as c_int {
                    st.lnum -= st.regmatch.startpos[0].lnum;
                }
                if args.cmdpreview_ns > 0 as c_int {
                    push_preview(&mut st.preview_lines, current_match);
                }
                break;
            }
        }
        if args.cmdpreview_ns > 0 as c_int {
            push_preview(&mut st.preview_lines, current_match);
        }
        line_breakcheck();
    }
}

/// Everything that happens for one line of the range that has a match.
fn substitute_line(st: &mut Sub, args: &SubArgs) {
    st.prev_matchcol = MAXCOL as ColNr;
    st.new_line = None;
    st.did_sub = false;
    st.nmatch_tl = 0 as LineNr;
    st.skip_match = false;
    st.lnum_start = 0 as LineNr;
    st.line_matches.clear();
    st.sub_firstlnum = st.lnum;
    st.copied = 0;
    st.matchcol = 0 as ColNr;

    // At the first match, remember the current cursor position.
    if !st.got_match {
        // SAFETY: main thread, live window.
        setpcmark();
        st.got_match = true;
    }

    match_loop(st, args);

    if st.did_sub {
        sub_nlines.set(sub_nlines.get() + 1);
    }
    // The rebuilt line is only still here when the substitution was
    // cancelled, and the old line is this line's.
    st.new_line = None;
    st.old_line = None;
    st.line_matches.clear();
}

/// Check for a match on each line of the range.  Under a preview, stop once
/// enough lines have been collected to fill the preview window.
fn substitute_range(st: &mut Sub, args: &SubArgs) {
    while st.lnum <= st.line2 && !st.got_quit {
        // SAFETY: main thread.
        if aborting() {
            break;
        }
        if args.cmdpreview_ns > 0 as c_int
            && st.preview_lines.lines_needed > p_cwh.get() as LineNr
            // SAFETY: the current window is live.
            && st.lnum > Win::current().w_botline
        {
            break;
        }
        // SAFETY: the program is compiled.
        st.nmatch = unsafe { regexec_at(&raw mut st.regmatch, st.lnum, 0 as ColNr) };
        if st.nmatch != 0 {
            substitute_line(st, args);
        }
        line_breakcheck();
        // SAFETY: a profile value we were handed.
        if profile_passed_limit(args.timeout) {
            st.got_quit = true;
        }
        st.lnum += 1;
    }
}

/// Report what happened, put the cursor and the marks where Vi would, and
/// draw the preview if this was one.
fn finish(st: &mut Sub, args: &SubArgs) -> c_int {
    // SAFETY: the current buffer is live.
    Buf::current().deleted_bytes2 = 0 as size_t;

    if st.first_line != 0 as LineNr {
        // Subtract the number of added lines from "last_line" to get the line
        // number before the change (the same as adding the number of deleted
        // lines).
        let added = Buf::current().b_ml.ml_line_count - args.old_line_count;
        changed_lines(
            Buf::current(),
            st.first_line,
            0 as ColNr,
            st.last_line - added,
            added,
            false,
        );
        let num_added = (st.last_line - st.first_line) as int64_t;
        let num_removed = num_added - added as int64_t;
        let buffer = Buf::current();
        buf_updates_send_changes(buffer, st.first_line, num_added, num_removed);
    }

    // May still be holding the copy of the line.
    st.old_line = None;

    // ":s/pat//n" doesn't move the cursor.
    if subflags.with(|flags| flags.do_count) {
        // SAFETY: the current window is live.
        Win::current().w_cursor = args.old_cursor;
    }

    if sub_nsubs.get() > args.start_nsubs {
        if !cmdmod_has(CmdModFlags::LOCKMARKS) {
            // Set the '[ and '] marks.
            // SAFETY: the current buffer is live.
            Buf::current().b_op_start.lnum = args.range.0;
            Buf::current().b_op_end.lnum = st.line2;
            Buf::current().b_op_end.col = 0 as ColNr;
            Buf::current().b_op_start.col = Buf::current().b_op_end.col;
        }

        if global_busy.get() == 0 {
            // When interactive, leave the cursor on the match.
            if !subflags.with(|flags| flags.do_ask) {
                // SAFETY: the current window is live.
                if args.endcolumn {
                    coladvance(Win::current(), MAXCOL as c_int);
                } else {
                    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
                }
            }
            // The report is only given for a real substitute, never for a
            // preview -- and upstream's `&&` means it is not even *computed*
            // for one.
            if args.cmdpreview_ns <= 0 as c_int
                && !do_sub_msg(subflags.with(|flags| flags.do_count))
                && subflags.with(|flags| flags.do_ask)
                && p_ch.get() > 0 as OptInt
            {
                msg(c"", 0 as c_int);
            }
        } else {
            global_need_beginline.set(true);
        }
        if subflags.with(|flags| flags.do_print) {
            print_line(
                Win::current().w_cursor.lnum,
                subflags.with(|flags| flags.do_number),
                subflags.with(|flags| flags.do_list),
                true,
            );
        }
    } else if global_busy.get() == 0 {
        if got_int.get() {
            // Interrupted.
            emsg(gettext(e_interr));
        } else if st.got_match {
            // Did find something, but nothing was substituted.
            if p_ch.get() > 0 as OptInt && !ui_has(kUIMessages) {
                msg(c"", 0 as c_int);
            }
        } else if subflags.with(|flags| flags.do_error) {
            // Nothing found. // SAFETY: the search pattern is a live C string.
            let arg0 = unsafe { c_str(get_search_pat()) };
            semsg!("E486: Pattern not found: {arg0}");
        }
    }

    // SAFETY: the current window is live.
    if subflags.with(|flags| flags.do_ask) && has_any_folding(Win::current()) != 0 {
        // The cursor position may require updating.
        changed_window_setting(Win::current());
    }

    // SAFETY: the compiled program and the replacement text are ours.
    unsafe { vim_regfree(st.regmatch.regprog) };
    unsafe { xfree(st.sub as *mut c_void) };

    // Restore the flag values: they can be used for ":&&".
    subflags.with_mut(|flags| {
        flags.do_all = args.save_do_all;
        flags.do_ask = args.save_do_ask;
    });

    // Show the 'inccommand' preview if there are matched lines.
    // SAFETY: main thread.
    if args.cmdpreview_ns <= 0 as c_int || aborting() {
        return 0 as c_int;
    }
    // SAFETY: a profile value we were handed.
    if st.got_quit || profile_passed_limit(args.timeout) {
        // Too slow: disable.
        set_option_direct(
            kOptInccommand,
            static_cstr_optval(c""),
            OptionSetFlags::NONE,
            SID_NONE,
        );
        return 0 as c_int;
    }
    // SAFETY: 'inccommand' is a live string option.
    if unsafe { *p_icm.get() } as c_int == NUL || !args.pat_given {
        return 0 as c_int;
    }
    if pre_hl_id.get() == 0 as c_int {
        // SAFETY: a literal group name and its length.
        pre_hl_id.set(unsafe { syn_check_group(c"Substitute".as_ptr(), 10 as size_t) });
    }
    show_sub(
        args.range,
        args.old_cursor,
        &st.preview_lines,
        pre_hl_id.get(),
        args.cmdpreview_ns,
        args.cmdpreview_bufnr,
    )
}

/// Perform a substitution from line `args.line1` to line `args.line2` using
/// the command in `args.arg`, which should be of the form
/// `/pattern/substitution/{flags}`.  The usual escapes are supported, as
/// described in the regexp docs.
///
/// `cmdpreview_ns` is the namespace to show 'inccommand' preview highlights
/// in; `<= 0` means no preview.  Returns 0, 1 or 2 -- see
/// `cmdpreview_may_show` for what they mean.
pub(crate) fn do_sub(
    args: &mut ExArg,
    timeout: ProfTime,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: Handle,
) -> c_int {
    if global_busy.get() == 0 {
        sub_nsubs.set(0 as c_int);
        sub_nlines.set(0 as LineNr);
    }
    let start_nsubs = sub_nsubs.get();
    let keeppatterns = cmdmod_has(CmdModFlags::KEEPPATTERNS);
    // SAFETY: the current window and buffer are live.
    let (old_cursor, old_line_count) = (Win::current().w_cursor, Buf::current().b_ml.ml_line_count);

    let Some(setup) = parse_sub(args, cmdpreview_ns, keeppatterns) else {
        return 0 as c_int;
    };
    let SubSetup {
        sub,
        regmatch,
        has_second_delim,
        endcolumn,
        pat_given,
        save_do_all,
        save_do_ask,
    } = setup;

    let (line1, line2) = (args.line1, args.line2);
    let args = SubArgs {
        range: (line1, line2),
        timeout,
        cmdpreview_ns,
        cmdpreview_bufnr,
        has_second_delim,
        pat_given,
        endcolumn,
        old_cursor,
        old_line_count,
        start_nsubs,
        save_do_all,
        save_do_ask,
    };
    let mut st = Sub {
        regmatch,
        sub,
        lnum: line1,
        line2,
        got_quit: false,
        got_match: false,
        did_save: false,
        first_line: 0 as LineNr,
        last_line: 0 as LineNr,
        preview_lines: PreviewLines::default(),
        nmatch: 0 as c_int,
        old_line: None,
        sub_firstlnum: 0 as LineNr,
        copied: 0,
        matchcol: 0 as ColNr,
        prev_matchcol: 0 as ColNr,
        new_line: None,
        sublen: 0 as c_int,
        did_sub: false,
        nmatch_tl: 0 as LineNr,
        do_again: false,
        skip_match: false,
        lnum_start: 0 as LineNr,
        line_matches: Vec::new(),
    };

    substitute_range(&mut st, &args);
    finish(&mut st, &args)
}

/// Required for undo to work for extmarks: save the cursor line once, before
/// the first change of the command.
pub(super) fn save_undo_once(st: &mut Sub) {
    if !st.did_save {
        let _ = u_save_cursor();
        st.did_save = true;
    }
}
