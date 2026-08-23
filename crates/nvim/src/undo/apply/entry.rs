//! Applying one header: the part of undo that actually changes text.
//!
//! A header holds a list of entries, and an entry is "lines `top+1`..`bot-1`
//! of the buffer used to be these lines instead". Applying it swaps the two —
//! the saved lines go into the buffer and the buffer's go into the entry — so
//! the very same header applied again moves back, which is what makes undo
//! and redo one function.

#![deny(unsafe_op_in_unsafe_fn)]

use super::super::store::Header;
use super::super::*;
use crate::edit::BeginlineOpts;
use crate::memline::MlFlags;
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
    line: linenr_T,
    /// The position itself.
    pos: pos_T,
}

impl CursorPick {
    /// Decides where the cursor should end up from what this entry is about
    /// to change. Nothing moves yet — the lines are not there.
    ///
    /// # Safety
    ///
    /// A live current buffer, and `uep` an entry of `curhead` whose
    /// `ue_array` holds `newsize` lines.
    unsafe fn consider(
        &mut self,
        curhead: Header,
        uep: *mut u_entry_T,
        top: linenr_T,
        oldsize: linenr_T,
        newsize: linenr_T,
    ) {
        // If the header's saved cursor falls inside this entry it wins: that
        // is what puts the cursor back where it was after a "gwap".
        let saved = curhead.uh_cursor.lnum;
        if saved >= top && saved <= top + newsize + 1 {
            self.pos = curhead.uh_cursor;
            self.line = -1;
            return;
        }
        if top >= self.line {
            return;
        }
        // Otherwise the first line that really differs, so that undoing an
        // auto-format does not land on the line before it.
        let mut same: linenr_T = 0;
        // SAFETY: a live entry with `newsize` lines, and every line compared
        // is one the buffer still holds, by the contract above.
        unsafe {
            while same < newsize && same < oldsize {
                let was = *(*uep).ue_array.offset(same as isize);
                if strcmp(was, ml_get(top + 1 + same)) != 0 {
                    break;
                }
                same += 1;
            }
            if same == newsize && self.line == MAXLNUM as linenr_T && (*uep).ue_next.is_null() {
                self.line = top;
            } else if same < newsize {
                self.line = top + same;
            } else {
                return;
            }
        }
        self.pos.lnum = self.line + 1;
    }
}

/// Applies the header at `b_u_curhead`: every entry's saved lines replace the
/// buffer lines they cover, and those lines take their place in the entry, so
/// that applying the same header again moves back.
///
/// # Safety
///
/// A live current buffer and window.
pub(crate) unsafe fn u_undoredo(undo: bool, do_buf_event: bool) {
    // SAFETY: a live current buffer and window, by the contract above.
    let mut buf = unsafe { Buf::current() };
    let Some(mut curhead) = buf.header(buf.b_u_curhead) else {
        return;
    };

    // Autocommands must not see the undo structures: they are inconsistent
    // until the end.
    // SAFETY: nothing here holds a borrow of editor state.
    unsafe { block_autocmds() };

    let old_flags = curhead.uh_flags;
    let new_flags = (if buf.b_changed != 0 { UH_CHANGED } else { 0 })
        | (if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
            UH_EMPTYBUF
        } else {
            0
        })
        | (old_flags & UH_RELOAD);
    // SAFETY: a live current buffer and window.
    unsafe { setpcmark() };

    // The marks and visual area from before the move; they go into the header
    // at the end, swapped with the ones it was carrying.
    // SAFETY: this module's own allocations, dropped exactly once.
    unsafe { zero_fmark_additional_data(&mut buf.b_namedm) };
    let saved_marks = buf.b_namedm;
    let saved_visual = buf.b_visual;
    buf.b_op_start.lnum = buf.b_ml.ml_line_count;
    buf.b_op_start.col = 0;
    buf.b_op_end.lnum = 0;
    buf.b_op_end.col = 0;

    // SAFETY: a live current window.
    let mut pick = CursorPick {
        line: MAXLNUM as linenr_T,
        pos: unsafe { Win::current() }.w_cursor,
    };
    // The entries come back in the reverse of the order they are applied,
    // which is the order the next move wants them in.
    let mut newlist: *mut u_entry_T = ptr::null_mut();
    let mut uep = curhead.uh_entry;
    while !uep.is_null() {
        // SAFETY: an entry of a live header, and a live current buffer.
        if !unsafe { apply_entry(buf, curhead, uep, &mut pick, do_buf_event) } {
            // SAFETY: as above; the entry was left untouched.
            unsafe {
                unblock_autocmds();
                iemsg(gettext(c"E438: u_undo: line numbers wrong".as_ptr()));
                changed(buf.raw()); // don't want UNCHANGED now
            }
            return;
        }
        // SAFETY: a live entry, and `newlist` is one or NULL.
        unsafe {
            let next = (*uep).ue_next;
            (*uep).ue_next = newlist;
            newlist = uep;
            uep = next;
        }
    }

    // Keep the '[ and '] marks inside the buffer.
    let last_line = buf.b_ml.ml_line_count;
    buf.b_op_start.lnum = buf.b_op_start.lnum.min(last_line);
    buf.b_op_end.lnum = buf.b_op_end.lnum.min(last_line);

    // Undo replays the header's extmark moves backwards; redo replays them in
    // the order they were recorded. `extmark_apply_undo` never touches the
    // header's own vector, so its length is read once.
    let marks = curhead.uh_extmark;
    for i in 0..marks.size {
        let i = if undo { marks.size - 1 - i } else { i };
        // SAFETY: `i` is in bounds of a vector the header owns.
        unsafe { extmark_apply_undo(*marks.items.add(i), undo) };
    }
    if curhead.uh_flags & UH_RELOAD != 0 {
        // Upstream TODO(bfredl): crude. With 'undoreload' there is enough
        // information to send a buffer-reloading on_lines/on_bytes event.
        // SAFETY: a live buffer.
        unsafe { buf_updates_unload(buf.raw(), true) };
    }

    // The cursor goes where the entries decided; check the line exists.
    // SAFETY: a live current window.
    let mut win = unsafe { Win::current() };
    win.w_cursor = pick.pos;
    // SAFETY: a live window.
    unsafe { check_cursor_lnum(win.raw()) };

    curhead.uh_entry = newlist;
    curhead.uh_flags = new_flags;
    // SAFETY: a live buffer.
    if old_flags & UH_EMPTYBUF != 0 && unsafe { buf_is_empty(buf.raw()) } {
        buf.b_ml.ml_flags |= MlFlags::EMPTY;
    }
    // SAFETY: a live buffer.
    unsafe {
        if old_flags & UH_CHANGED != 0 {
            changed(buf.raw());
        } else {
            unchanged(buf.raw(), false, true);
        }
        // Those two bumped changedtick again, so the watchers need an event
        // carrying just its new value.
        if do_buf_event {
            buf_updates_changedtick(buf.raw());
        }
    }

    // SAFETY: a live buffer and a live header.
    unsafe { swap_marks(buf, curhead, &saved_marks) };
    if curhead.uh_visual.vi_start.lnum != 0 {
        buf.b_visual = curhead.uh_visual;
        curhead.uh_visual = saved_visual;
    }
    // SAFETY: a live buffer, window and header.
    unsafe { place_cursor(buf, win, curhead) };

    // Where "g-" and ":earlier 10s" resume from. After an undo we are below
    // the change just undone, but it is recorded as just *above* it so that
    // ":earlier 1s" works.
    buf.b_u_seq_cur = if undo {
        buf.header(curhead.uh_next).map_or(0, |uh| uh.uh_seq)
    } else {
        curhead.uh_seq
    };
    // Where ":earlier 1f" and ":later 1f" resume from.
    if curhead.uh_save_nr != 0 {
        buf.b_u_save_nr_cur = if undo {
            curhead.uh_save_nr - 1
        } else {
            curhead.uh_save_nr
        };
    }
    // Several changes can share a timestamp; the one that moved wins.
    buf.b_u_time_cur = curhead.uh_time;
    // SAFETY: nothing here holds a borrow of editor state.
    unsafe { unblock_autocmds() };
}

/// Swaps one entry's saved lines with the buffer lines it covers.
///
/// Answers `false` — having changed nothing — when the entry's line numbers
/// do not fit the buffer.
///
/// # Safety
///
/// A live current buffer and window, and `uep` an entry of `curhead`.
unsafe fn apply_entry(
    mut buf: Buf,
    curhead: Header,
    uep: *mut u_entry_T,
    pick: &mut CursorPick,
    do_buf_event: bool,
) -> bool {
    // SAFETY: a live entry, by the contract above.
    let (top, saved_bot, newsize) = unsafe { ((*uep).ue_top, (*uep).ue_bot, (*uep).ue_size) };
    // Zero is the sentinel for "to the end of the buffer" — `u_savecommon`
    // writes it whenever the change ran past the last line — and it is
    // resolved here, against the buffer as it stands now.
    let bot = if saved_bot == 0 {
        buf.b_ml.ml_line_count + 1
    } else {
        saved_bot
    };
    if top > buf.b_ml.ml_line_count || top >= bot || bot > buf.b_ml.ml_line_count + 1 {
        return false;
    }
    let oldsize = bot - top - 1; // lines the entry covers now

    // SAFETY: a live entry of a live header, and a live current buffer.
    unsafe { pick.consider(curhead, uep, top, oldsize, newsize) };

    // Take the covered lines out, backwards: cheaper in most cases.
    let mut emptied = false;
    let mut taken: *mut *mut c_char = ptr::null_mut();
    if oldsize > 0 {
        // SAFETY: `taken` is `oldsize` pointers long and every line between
        // `top + 1` and `bot - 1` exists, by the bounds check above.
        unsafe {
            taken = xmalloc(size_of::<*mut c_char>() * oldsize as size_t) as *mut *mut c_char;
            for i in (0..oldsize).rev() {
                *taken.offset(i as isize) = u_save_line(top + 1 + i);
                // Deleting the buffer's last line leaves a dummy empty one
                // behind, which the insert below has to replace.
                if buf.b_ml.ml_line_count == 1 {
                    emptied = true;
                }
                ml_delete(top + 1 + i);
            }
        }
    }
    // Make sure the cursor is on a line that still exists.
    // SAFETY: a live current window.
    unsafe { check_cursor_lnum(curwin.get()) };

    // Put the entry's saved lines in between top and bot.
    if newsize != 0 {
        // SAFETY: `ue_array` is `newsize` lines long, and each is a
        // NUL-terminated allocation the entry owns and hands over here.
        unsafe {
            for i in 0..newsize {
                let line = *(*uep).ue_array.offset(i as isize);
                if emptied && top + i == 0 {
                    ml_replace(1, line, true);
                } else {
                    ml_append_flags(top + i, line, 0, 0);
                }
                xfree(line as *mut c_void);
            }
            xfree((*uep).ue_array as *mut c_void);
        }
    }

    if oldsize != newsize {
        // SAFETY: a live current buffer.
        unsafe {
            mark_adjust(
                top + 1,
                top + oldsize,
                MAXLNUM as linenr_T,
                newsize - oldsize,
                kExtmarkNOOP,
            );
        }
        if buf.b_op_start.lnum > top + oldsize {
            buf.b_op_start.lnum += newsize - oldsize;
        }
        if buf.b_op_end.lnum > top + oldsize {
            buf.b_op_end.lnum += newsize - oldsize;
        }
    }
    if oldsize > 0 || newsize > 0 {
        // SAFETY: a live buffer and window.
        unsafe {
            changed_lines(buf.raw(), top + 1, 0, bot, newsize - oldsize, do_buf_event);
            // The next line's start may have gained or lost a SpellCap, so
            // schedule it for redrawing just in case.
            if spell_check_window(curwin.get()) && bot <= buf.b_ml.ml_line_count {
                redraw_win_line(curwin.get(), bot);
            }
        }
    }

    // The '[ mark, then the '] mark.
    buf.b_op_start.lnum = buf.b_op_start.lnum.min(top + 1);
    if newsize == 0 && top + 1 > buf.b_op_end.lnum {
        buf.b_op_end.lnum = top + 1;
    } else if top + newsize > buf.b_op_end.lnum {
        buf.b_op_end.lnum = top + newsize;
    }
    u_newcount.set(u_newcount.get() + newsize);
    u_oldcount.set(u_oldcount.get() + oldsize);

    // The entry now describes the lines it displaced, ready to put them back.
    // SAFETY: a live entry, which has just given up its old array.
    unsafe {
        (*uep).ue_size = oldsize;
        (*uep).ue_array = taken;
        (*uep).ue_bot = top + newsize + 1;
    }
    true
}

/// Gives the buffer the named marks the header was carrying, and the header
/// the ones the buffer had, so that it can put them back on the next move.
///
/// # Safety
///
/// A live buffer and a live header.
unsafe fn swap_marks(mut buf: Buf, mut curhead: Header, saved: &[fmark_T; NMARKS as usize]) {
    for (i, &saved) in saved.iter().enumerate() {
        if curhead.uh_namedm[i].mark.lnum != 0 {
            // SAFETY: a mark the buffer owns and is about to drop.
            unsafe { free_fmark(buf.b_namedm[i]) };
            buf.b_namedm[i] = curhead.uh_namedm[i];
        }
        if saved.mark.lnum != 0 {
            curhead.uh_namedm[i] = saved;
        } else {
            // Only the line number is cleared, as the C clears it: the rest
            // of the mark is left as it was.
            curhead.uh_namedm[i].mark.lnum = 0;
        }
    }
}

/// Puts the cursor where the header says it was, or on the first line the
/// move actually changed.
///
/// # Safety
///
/// A live buffer, window and header.
unsafe fn place_cursor(buf: Buf, mut win: Win, curhead: Header) {
    // Off by exactly one line: put it back where the change started, which is
    // what the "o" command wants. Otherwise it goes to the first undone line.
    if curhead.uh_cursor.lnum + 1 == win.w_cursor.lnum && win.w_cursor.lnum > 1 {
        win.w_cursor.lnum -= 1;
    }
    if win.w_cursor.lnum > buf.b_ml.ml_line_count {
        // Past the end, which happens after undoing lines added at the end of
        // the file. `check_cursor` below moves it to the last line, so all
        // that is left is to put it in the first column.
        win.w_cursor.col = 0;
        win.w_cursor.coladd = 0;
    } else if curhead.uh_cursor.lnum == win.w_cursor.lnum {
        win.w_cursor.col = curhead.uh_cursor.col;
        // SAFETY: a live window, by the contract above.
        if unsafe { virtual_active(win.raw()) } && curhead.uh_cursor_vcol >= 0 {
            // SAFETY: as above.
            unsafe { coladvance(win.raw(), curhead.uh_cursor_vcol) };
        } else {
            win.w_cursor.coladd = 0;
        }
    } else {
        // SAFETY: a live current window.
        unsafe { beginline(BeginlineOpts::SOL | BeginlineOpts::FIX) };
    }
    // Make sure the cursor is on an existing line and column.
    // SAFETY: a live window.
    unsafe { check_cursor(win.raw()) };
}
