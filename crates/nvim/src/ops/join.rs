//! `J` and `gJ` -- joining lines.
//!
//! [`do_join`] builds the joined line in **one** allocation, which is why it
//! is two passes over the same lines. The first ([`measure_join`]) walks
//! forwards and only measures: how much of each line survives once
//! [`skip_comment`] has trimmed a comment leader, and how many spaces go in
//! each seam. The second ([`assemble_join`]) walks *backwards* and copies,
//! filling the buffer from its end -- which is also the order marks have to
//! move in, since each line's marks land at a column that depends on where the
//! lines after it ended up.
//!
//! The seam width is the fiddly part: normally one space, two after `.`, `?`
//! or `!` under 'joinspaces', none before `)` or after a TAB, none at all for
//! `gJ`, and none between two multi-byte characters under the
//! 'formatoptions' `M`/`B` rules -- which exist because CJK text has no spaces
//! between words and a join must not invent one.

#![forbid(unsafe_code)]

use crate::guard::Suppress;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::mbyte::char_at;
use crate::option::cpo_has;
use crate::types::{CpoFlag, Failed, FoFlag, NUL};

/// Where the text of `line` starts once its comment leader is skipped, and
/// whether the line ends *inside* an unclosed comment.
///
/// `process` false only answers the second question and leaves the start at
/// 0; that is how the caller decides whether the *next* line's leader may be
/// removed at all, since a leader is only noise when the line before it was a
/// comment too.
pub fn skip_comment(line: &[u8], process: bool, include_space: bool) -> (usize, bool) {
    let (leader_offset, comment_flags) = get_last_leader_offset(line);
    // A flag string walked to its `COM_END`, its colon, or its end, whichever
    // comes first.
    let end_or_colon = |flags: Option<ComFlags>| {
        c_int::from(flags.map_or(NUL as u8, |flags| flags.first_of(&[COM_END as u8, b':'])))
    };

    // Does the line end with an unclosed comment? It does unless the last
    // leader's flags carry COM_END.
    let is_comment = leader_offset != -1 && end_or_colon(comment_flags) != COM_END;
    if !process {
        return (0, is_comment);
    }

    let (lead_len, flags) = get_leader_len(line, false, include_space);
    if lead_len == 0 {
        return (0, is_comment);
    }

    // A colon means this is not the closing part of a three-part comment.
    // Those are left alone: removing them would be annoying.
    let flag = end_or_colon(flags.or(comment_flags));
    if flag == ':' as c_int || flag == NUL {
        return (usize::try_from(lead_len).unwrap_or(0), is_comment);
    }
    (0, is_comment)
}

/// What [`measure_join`] worked out and [`assemble_join`] replays.
struct JoinPlan {
    /// Spaces to put in front of line `t`; `count` entries.
    spaces: Vec<u8>,
    /// Bytes of comment leader skipped on line `t`, `count` entries; `None`
    /// when 'formatoptions' does not have `j`.
    comments: Option<Vec<c_int>>,
    /// Length of the joined line, spaces included.
    sumsize: c_int,
    /// Length of the *last* line joined, after skipping.
    currsize: c_int,
    /// Bytes skipped at the start of the last line joined: its leader and
    /// white space.
    curr: usize,
}

/// `J` and `gJ`: join `count` lines from the cursor.
///
/// `insert_space` is the difference between them: `J` puts a space in each
/// seam, `gJ` joins the text as it stands. `use_formatoptions` is false for
/// the callers -- backspace over a line break, and the charwise delete in
/// `op_delete` -- that must not have comment leaders removed underneath them.
/// `setmark` false leaves `'[`/`']` to the caller.
///
/// The cursor line plus `count - 1` must exist.
pub fn do_join(
    count: size_t,
    insert_space: bool,
    save_undo: bool,
    use_formatoptions: bool,
    setmark: bool,
) -> Result<(), Failed> {
    debug_assert!(count >= 1);
    let remove_comments = use_formatoptions && has_format_option(FoFlag::REMOVE_COMS);

    let above = Win::current().w_cursor.lnum - 1;
    let past = Win::current().w_cursor.lnum + count as LineNr;
    if save_undo {
        u_save(above, past)?;
    }

    // The per-line space counts are wanted twice: to size the one
    // allocation the joined line goes in, and to place each line in it.
    let mut plan = JoinPlan {
        spaces: vec![0; count],
        comments: remove_comments.then(|| vec![0; count]),
        sumsize: 0,
        currsize: 0,
        curr: 0,
    };

    measure_join(count, insert_space, setmark, &mut plan)?;
    assemble_join(count, insert_space, setmark, &mut plan);
    Ok(())
}

/// First pass: measure the joined line without moving anything.
///
/// Answers `Err` when the user interrupted it, which is why the walk calls
/// `line_breakcheck` -- a join can be over a very large count.
///
/// The cursor line plus `count - 1` must exist.
fn measure_join(
    count: size_t,
    insert_space: bool,
    setmark: bool,
    plan: &mut JoinPlan,
) -> Result<(), Failed> {
    // The last character of the line before, and the one before it: the
    // seam rules below are all about those two.
    let mut endcurr1 = NUL;
    let mut endcurr2 = NUL;
    let mut prev_was_comment = false;

    for t in 0..count {
        let mut lines = Lines::current();
        let line = lines.line(Win::current().w_cursor.lnum + t as LineNr);
        let mut curr = 0;

        if t == 0 && setmark && !cmdmod_has(CmdModFlags::LOCKMARKS) {
            let mut buf = Win::current().buffer();
            buf.b_op_start.lnum = Win::current().w_cursor.lnum;
            buf.b_op_start.col = line.len() as ColNr;
        }

        if let Some(comments) = plan.comments.as_mut() {
            // The leader is only noise when the line before was a comment
            // too; otherwise just ask whether *this* line is one.
            if t > 0 && prev_was_comment {
                let skipped;
                (skipped, prev_was_comment) = skip_comment(line, true, insert_space);
                comments[t] = skipped as c_int;
                curr = skipped;
            } else {
                prev_was_comment = skip_comment(line, false, insert_space).1;
            }
        }

        if insert_space && t > 0 {
            curr += skip::white(&line[curr..]);
            let at = c_int::from(byte_at(line, curr));
            let first = || char_at(&line[curr..]);
            if at != NUL
                && at != ')' as c_int
                && plan.sumsize != 0
                && endcurr1 != TAB
                // 'formatoptions' M: no space between two multi-byte
                // characters. B: no space if either side is a character
                // that eats one.
                && (!has_format_option(FoFlag::MBYTE_JOIN) || (first() < 0x100 && endcurr1 < 0x100))
                && (!has_format_option(FoFlag::MBYTE_JOIN2)
                    || (first() < 0x100 && !utf_eat_space(endcurr1))
                    || (endcurr1 < 0x100 && !utf_eat_space(first())))
            {
                if endcurr1 == ' ' as c_int {
                    // The line already ends in a space; look one further
                    // back for the 'joinspaces' test below.
                    endcurr1 = endcurr2;
                } else {
                    plan.spaces[t] += 1;
                }
                // 'joinspaces': two spaces after the end of a sentence.
                if p_js()
                    && (endcurr1 == '.' as c_int
                        || endcurr1 == '?' as c_int
                        || endcurr1 == '!' as c_int)
                {
                    plan.spaces[t] += 1;
                }
            }
        }

        let added = c_int::from(plan.spaces[t]);
        if t > 0 && curbuf_splice_pending.get() == 0 {
            let removed = curr as ColNr;
            let row = Win::current().w_cursor.lnum as c_int - 1;
            let (old, new) = ((removed + 1) as BCount, added as BCount);
            let op = kExtmarkUndo;
            extmark_splice(
                Buf::current(),
                row,
                plan.sumsize,
                1,
                removed,
                old,
                0,
                added,
                new,
                op,
            );
        }

        let text = &line[curr..];
        plan.curr = curr;
        plan.currsize = text.len() as c_int;
        plan.sumsize += plan.currsize + added;

        endcurr1 = NUL;
        endcurr2 = NUL;
        if insert_space && !text.is_empty() {
            // The last two characters, stepping back over whole ones.
            let back = |end: usize| end - (head_off(text, end - 1) + 1);
            let mut cend = back(text.len());
            endcurr1 = char_at(&text[cend..]);
            if cend > 0 {
                cend = back(cend);
                endcurr2 = char_at(&text[cend..]);
            }
        }

        line_breakcheck();
        if got_int.get() {
            return Err(Failed);
        }
    }
    Ok(())
}

/// Second pass: build the joined line and move the marks onto it.
///
/// Walks *backwards*, filling the one allocation from its end, because each
/// line's column offset in the result depends on everything after it. The
/// forward pass left `plan.curr`/`currsize` on the last line, which is where
/// this one starts.
///
/// `plan` must be as [`measure_join`] left it.
fn assemble_join(count: size_t, insert_space: bool, setmark: bool, plan: &mut JoinPlan) {
    let last = count - 1;
    // The column the last line starts at, for the cursor below.
    let col = plan.sumsize - plan.currsize - c_int::from(plan.spaces[last]);

    // `sumsize` is what `measure_join` counted for exactly the lines copied
    // back here; the terminator's room is reserved so that it costs no
    // second allocation.
    let sumsize = plan.sumsize as usize;
    let mut newp = Vec::with_capacity(sumsize + 1);
    newp.resize(sumsize, 0);
    let mut cend = sumsize;

    // The four edits below are one splice as far as extmarks and the
    // buffer-update RPC are concerned.
    let splice = Suppress::splice();

    let mut t = last;
    loop {
        let spaces_t = usize::from(plan.spaces[t]);
        {
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum + t as LineNr);
            let text = &line[plan.curr..];
            cend -= text.len();
            newp[cend..cend + text.len()].copy_from_slice(text);
        }
        cend -= spaces_t;
        newp[cend..cend + spaces_t].fill(b' ');
        // Where the mark adjustment below wants the line to have landed.
        let spaces_removed = plan.curr as c_int - spaces_t as c_int;
        let at = cend as ColNr - spaces_removed;

        // Marks move from each deleted line onto the joined one. Not Vi
        // compatible -- Vi deletes them -- but better. If more spaces are
        // deleted than added, a mark inside them moves no further than
        // what was added.
        let lnum = Win::current().w_cursor.lnum + t as LineNr;
        mark_col_adjust(lnum, 0, -(t as LineNr), at, spaces_removed);

        if t == 0 {
            break;
        }

        let mut lines = Lines::current();
        let line = lines.line(Win::current().w_cursor.lnum + t as LineNr - 1);
        plan.curr = plan
            .comments
            .as_ref()
            .map_or(0, |comments| comments[t - 1] as usize);
        if insert_space && t > 1 {
            plan.curr += skip::white(&line[plan.curr..]);
        }
        plan.currsize = (line.len() - plan.curr) as c_int;
        t -= 1;
    }

    let _ = ml_replace_owned(Win::current().w_cursor.lnum, XString::from_vec(newp));

    if setmark && !cmdmod_has(CmdModFlags::LOCKMARKS) {
        let mut buf = Win::current().buffer();
        buf.b_op_end.lnum = Win::current().w_cursor.lnum;
        buf.b_op_end.col = plan.sumsize;
    }

    // Only the first line's change is reported here; `del_lines` reports
    // the lines it deletes.
    let (lnum, next) = (
        Win::current().w_cursor.lnum,
        Win::current().w_cursor.lnum + 1,
    );
    changed_lines(Buf::current(), lnum, plan.currsize, next, 0, true);

    // Delete the following lines with the cursor moved there briefly.
    // `del_lines` may move it up again if the last line went, so the line
    // number is kept.
    let joined_lnum = Win::current().w_cursor.lnum;
    Win::current().w_cursor.lnum += 1;
    del_lines(count as LineNr - 1, false);
    Win::current().w_cursor.lnum = joined_lnum;
    drop(splice);
    Buf::current().deleted_bytes2 = 0;

    // 'cpoptions' `q`: Vi puts the cursor at the column of the *first*
    // join, Vim at the column of the last.
    Win::current().w_cursor.col = if cpo_has(CpoFlag::JOINCOL) {
        plan.currsize
    } else {
        col
    };
    check_cursor_col(Win::current());
    Win::current().w_cursor.coladd = 0;
    Win::current().w_set_curswant = true;
}
