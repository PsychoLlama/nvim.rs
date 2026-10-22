//! Applying one header: the part of undo that actually changes text.
//!
//! A header holds a list of entries, and an entry is "lines `top+1`..`bot-1`
//! of the buffer used to be these lines instead". Applying it swaps the two —
//! the saved lines go into the buffer and the buffer's go into the entry — so
//! the very same header applied again moves back, which is what makes undo
//! and redo one function.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::super::*;
use crate::edit::BeginlineOpts;
use crate::memline::{MlFlags, ml_append_flags, ml_replace_buf_owned};
use crate::memory::XString;
use crate::pos::MAXLNUM;
use crate::winlayer::{Buf, Win};

// ---------------------------------------------------------------------------
// Applying one header

/// Where the cursor should land once every entry in a header has been
/// applied.
struct CursorPick {
    /// The earliest line an entry actually changed: `MAXLNUM` until one
    /// does, and `-1` once the header's own saved cursor claimed the
    /// position, which no later entry may override.
    line: LineNr,
    /// The position itself.
    pos: Pos,
}

impl CursorPick {
    /// Decides where the cursor should end up from what this entry is about
    /// to change. Nothing moves yet — the lines are not there.
    ///
    /// `saved_cursor` is the header's own; `last` says the entry is the
    /// last of the header's list, the one applied last.
    fn consider(
        &mut self,
        saved_cursor: Pos,
        saved: &[XString],
        last: bool,
        top: LineNr,
        oldsize: LineNr,
        newsize: LineNr,
    ) {
        // If the header's saved cursor falls inside this entry it wins: that
        // is what puts the cursor back where it was after a "gwap".
        let saved_line = saved_cursor.lnum;
        if saved_line >= top && saved_line <= top + newsize + 1 {
            self.pos = saved_cursor;
            self.line = -1;
            return;
        }
        if top >= self.line {
            return;
        }
        // Otherwise the first line that really differs, so that undoing an
        // auto-format does not land on the line before it.
        let mut same: LineNr = 0;
        let mut lines = Buf::current().lines();
        for (was, lnum) in saved
            .iter()
            .zip(top + 1..)
            .take(usize::try_from(oldsize).unwrap_or(0))
        {
            if **was != *lines.line(lnum) {
                break;
            }
            same += 1;
        }
        if same == newsize && self.line == MAXLNUM && last {
            self.line = top;
        } else if same < newsize {
            self.line = top + same;
        } else {
            return;
        }
        self.pos.lnum = self.line + 1;
    }
}

/// Applies the header at `b_u_curhead`: every entry's saved lines replace the
/// buffer lines they cover, and those lines take their place in the entry, so
/// that applying the same header again moves back.
///
/// The entries are taken out of the header while they are applied and put
/// back afterwards: the memline calls in between reach buffer-update
/// callbacks, and nothing of the store is borrowed across them. The header
/// is looked up by its link again at every step after.
pub(crate) fn u_undoredo(undo: bool, do_buf_event: bool) {
    let mut buf = Buf::current();
    let link = buf.b_u_curhead;
    let Some(header) = buf.header_mut(link) else {
        return;
    };
    let old_flags = header.flags;
    let saved_cursor = header.cursor;
    let mut entries = core::mem::take(&mut header.entries);
    let getbot = header.getbot.take();

    // Autocommands must not see the undo structures: they are inconsistent
    // until the end.
    block_autocmds();

    let new_flags = (if buf.b_changed != 0 { UH_CHANGED } else { 0 })
        | (if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
            UH_EMPTYBUF
        } else {
            0
        })
        | (old_flags & UH_RELOAD);
    setpcmark();

    // The marks and visual area from before the move; they go into the header
    // at the end, swapped with the ones it was carrying.
    drop_named_mark_extras(&mut buf.b_namedm);
    let saved_marks = buf.b_namedm.each_ref().map(SavedMark::of);
    let saved_visual = buf.b_visual;
    buf.b_op_start.lnum = buf.b_ml.ml_line_count;
    buf.b_op_start.col = 0;
    buf.b_op_end.lnum = 0;
    buf.b_op_end.col = 0;

    let mut pick = CursorPick {
        line: MAXLNUM,
        pos: Win::current().w_cursor,
    };
    // Newest first, which is the end of the vector; the list comes back in
    // the reverse of the order it is applied, which is the order the next
    // move wants it in.
    let count = entries.len();
    for (applied, at) in (0..count).rev().enumerate() {
        if !apply_entry(
            buf,
            saved_cursor,
            &mut entries[at],
            at == 0,
            &mut pick,
            do_buf_event,
        ) {
            // Upstream had relinked the entries it applied into a list of
            // their own by now, and the header kept only the first of them
            // -- or all of them, when the first one was the bad one.
            if let Some(header) = buf.header_mut(link) {
                if applied == 0 {
                    header.entries = entries;
                    header.getbot = getbot;
                } else {
                    header.getbot = getbot.filter(|&g| g == count - 1).map(|_| 0);
                    header.entries = entries.drain(count - 1..).collect();
                }
            }
            unblock_autocmds();
            iemsg(gettext(c"E438: u_undo: line numbers wrong"));
            changed(buf); // don't want UNCHANGED now
            return;
        }
    }
    entries.reverse();
    let getbot = getbot.map(|g| count - 1 - g);

    // Keep the '[ and '] marks inside the buffer.
    let last_line = buf.b_ml.ml_line_count;
    buf.b_op_start.lnum = buf.b_op_start.lnum.min(last_line);
    buf.b_op_end.lnum = buf.b_op_end.lnum.min(last_line);

    // Undo replays the header's extmark moves backwards; redo replays them in
    // the order they were recorded. The list is taken out for the replay,
    // which records nothing (`kExtmarkNoUndo`), and put back after it.
    let marks = buf
        .header_mut(link)
        .map(|uh| core::mem::take(&mut uh.extmarks))
        .unwrap_or_default();
    if undo {
        marks
            .iter()
            .rev()
            .for_each(|&mark| extmark_apply_undo(mark, true));
    } else {
        marks
            .iter()
            .for_each(|&mark| extmark_apply_undo(mark, false));
    }
    let reload = old_flags & UH_RELOAD != 0;
    if let Some(header) = buf.header_mut(link) {
        let mut marks = marks;
        marks.append(&mut header.extmarks);
        header.extmarks = marks;
    }
    if reload {
        // Upstream TODO(bfredl): crude. With 'undoreload' there is enough
        // information to send a buffer-reloading on_lines/on_bytes event.
        buf_updates_unload(buf, true);
    }

    // The cursor goes where the entries decided; check the line exists.
    let mut win = Win::current();
    win.w_cursor = pick.pos;
    check_cursor_lnum(win);

    if let Some(header) = buf.header_mut(link) {
        header.entries = entries;
        header.getbot = getbot;
        header.flags = new_flags;
    }
    if old_flags & UH_EMPTYBUF != 0 && buf_is_empty(buf) {
        buf.b_ml.ml_flags |= MlFlags::EMPTY;
    }
    if old_flags & UH_CHANGED != 0 {
        changed(buf);
    } else {
        unchanged(buf, false, true);
    }
    // Those two bumped changedtick again, so the watchers need an event
    // carrying just its new value.
    if do_buf_event {
        buf_updates_changedtick(buf);
    }

    swap_marks(buf, link, &saved_marks, saved_visual);
    if let Some((cursor, cursor_vcol)) = buf.header(link).map(|uh| (uh.cursor, uh.cursor_vcol)) {
        place_cursor(buf, win, cursor, cursor_vcol);
    }

    // Where "g-" and ":earlier 10s" resume from. After an undo we are below
    // the change just undone, but it is recorded as just *above* it so that
    // ":earlier 1s" works.
    let Some((seq, next, save_nr, time)) = buf
        .header(link)
        .map(|uh| (uh.seq, uh.next, uh.save_nr, uh.time))
    else {
        unblock_autocmds();
        return;
    };
    buf.b_u_seq_cur = if undo {
        buf.header(next).map_or(0, |uh| uh.seq)
    } else {
        seq
    };
    // Where ":earlier 1f" and ":later 1f" resume from.
    if save_nr != 0 {
        buf.b_u_save_nr_cur = if undo { save_nr - 1 } else { save_nr };
    }
    // Several changes can share a timestamp; the one that moved wins.
    buf.b_u_time_cur = time;
    unblock_autocmds();
}

/// Swaps one entry's saved lines with the buffer lines it covers.
///
/// Answers `false` — having changed nothing — when the entry's line numbers
/// do not fit the buffer. The entry is the caller's own, out of its header.
fn apply_entry(
    mut buffer: Buf,
    saved_cursor: Pos,
    entry: &mut UndoEntry,
    last: bool,
    pick: &mut CursorPick,
    do_buf_event: bool,
) -> bool {
    let (top, saved_bot, newsize) = (entry.top, entry.bot, entry.size());
    // Zero is the sentinel for "to the end of the buffer" — `u_savecommon`
    // writes it whenever the change ran past the last line — and it is
    // resolved here, against the buffer as it stands now.
    let bot = if saved_bot == 0 {
        buffer.b_ml.ml_line_count + 1
    } else {
        saved_bot
    };
    if top > buffer.b_ml.ml_line_count || top >= bot || bot > buffer.b_ml.ml_line_count + 1 {
        return false;
    }
    let oldsize = bot - top - 1; // lines the entry covers now

    pick.consider(saved_cursor, &entry.lines, last, top, oldsize, newsize);

    // Take the covered lines out, backwards: cheaper in most cases.
    let mut emptied = false;
    let mut taken: Vec<XString> = Vec::with_capacity(usize::try_from(oldsize).unwrap_or(0));
    for i in (0..oldsize).rev() {
        taken.push(line_copy(Buf::current(), top + 1 + i));
        // Deleting the buffer's last line leaves a dummy empty one behind,
        // which the insert below has to replace.
        if buffer.b_ml.ml_line_count == 1 {
            emptied = true;
        }
        let _ = ml_delete(top + 1 + i);
    }
    taken.reverse();
    // Make sure the cursor is on a line that still exists.
    check_cursor_lnum(Win::current());

    // Put the entry's saved lines in between top and bot.
    for (line, i) in core::mem::take(&mut entry.lines).into_iter().zip(0..) {
        if emptied && top + i == 0 {
            // Over the dummy line the delete left, without a copy.
            let _ = ml_replace_buf_owned(Buf::current(), 1, line);
        } else {
            let _ = ml_append_flags(top + i, line.as_bytes_with_nul(), 0);
        }
    }

    if oldsize != newsize {
        let delta = newsize - oldsize;
        mark_adjust(top + 1, top + oldsize, MAXLNUM, delta, kExtmarkNOOP);
        if buffer.b_op_start.lnum > top + oldsize {
            buffer.b_op_start.lnum += delta;
        }
        if buffer.b_op_end.lnum > top + oldsize {
            buffer.b_op_end.lnum += delta;
        }
    }
    if oldsize > 0 || newsize > 0 {
        changed_lines(buffer, top + 1, 0, bot, newsize - oldsize, do_buf_event);
        // The next line's start may have gained or lost a SpellCap, so
        // schedule it for redrawing just in case.
        if spell_check_window(Win::current()) && bot <= buffer.b_ml.ml_line_count {
            redraw_win_line(Win::current(), bot);
        }
    }

    // The '[ mark, then the '] mark.
    buffer.b_op_start.lnum = buffer.b_op_start.lnum.min(top + 1);
    if newsize == 0 && top + 1 > buffer.b_op_end.lnum {
        buffer.b_op_end.lnum = top + 1;
    } else if top + newsize > buffer.b_op_end.lnum {
        buffer.b_op_end.lnum = top + newsize;
    }
    u_newcount.set(u_newcount.get() + newsize);
    u_oldcount.set(u_oldcount.get() + oldsize);

    // The entry now describes the lines it displaced, ready to put them back.
    entry.lines = taken;
    entry.bot = top + newsize + 1;
    true
}

/// Gives the buffer the named marks (and Visual area) the header was
/// carrying, and the header the ones the buffer had, so that it can put them
/// back on the next move.
fn swap_marks(
    mut buffer: Buf,
    link: UndoLink,
    saved: &[SavedMark; NMARKS as usize],
    saved_visual: VisualInfo,
) {
    let b: &mut Buffer = &mut buffer;
    let Some(header) = b
        .b_u_store
        .as_deref_mut()
        .and_then(|store| store.get_mut(link))
    else {
        return;
    };
    for (i, saved) in saved.iter().enumerate() {
        if header.named_marks[i].mark.lnum != 0 {
            // The buffer's mark is replaced, and its payload goes with it.
            b.b_namedm[i] = header.named_marks[i].to_mark();
        }
        if saved.mark.lnum != 0 {
            header.named_marks[i] = *saved;
        } else {
            // Only the line number is cleared, as the C clears it: the rest
            // of the mark is left as it was.
            header.named_marks[i].mark.lnum = 0;
        }
    }
    if header.visual.vi_start.lnum != 0 {
        b.b_visual = header.visual;
        header.visual = saved_visual;
    }
}

/// Puts the cursor where the header says it was, or on the first line the
/// move actually changed.
fn place_cursor(buffer: Buf, mut win: Win, cursor: Pos, cursor_vcol: ColNr) {
    // Off by exactly one line: put it back where the change started, which is
    // what the "o" command wants. Otherwise it goes to the first undone line.
    if cursor.lnum + 1 == win.w_cursor.lnum && win.w_cursor.lnum > 1 {
        win.w_cursor.lnum -= 1;
    }
    if win.w_cursor.lnum > buffer.b_ml.ml_line_count {
        // Past the end, which happens after undoing lines added at the end of
        // the file. `check_cursor` below moves it to the last line, so all
        // that is left is to put it in the first column.
        win.w_cursor.col = 0;
        win.w_cursor.coladd = 0;
    } else if cursor.lnum == win.w_cursor.lnum {
        win.w_cursor.col = cursor.col;
        if virtual_active(win) && cursor_vcol >= 0 {
            coladvance(win, cursor_vcol);
        } else {
            win.w_cursor.coladd = 0;
        }
    } else {
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    // Make sure the cursor is on an existing line and column.
    check_cursor(win);
}
