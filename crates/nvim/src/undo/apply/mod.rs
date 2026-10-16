//! Walking the tree: `:undo`, `:redo`, `:earlier`/`:later`, and the
//! entry-swapping that actually changes the buffer.
//!
//! Three layers, and only the middle one touches text:
//!
//! - [`u_undo`], [`u_redo`] and [`undo_time`] choose *which* header to move
//!   to — one step along the current branch here, or, in [`time`], the header
//!   nearest a target sequence number, timestamp or file write anywhere in
//!   the tree;
//! - [`u_undoredo`] ([`entry`]) applies one header, swapping every entry's
//!   saved lines with the buffer lines they cover so that applying the same
//!   header again moves back;
//! - [`u_undo_end`] reports what happened.
//!
//! Every header is named by an [`UndoLink`] and reached through the buffer's
//! store ([`Buf::header`](crate::winlayer::Buf::header)), looked up afresh
//! at each step.

#![forbid(unsafe_code)]

mod entry;
mod time;

use super::*;
use crate::drawscreen::UPD_NOT_VALID;
use crate::memline::MlFlags;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::normal::{visual_active, with_visual_anchor};
use crate::option::cpo_has;
use crate::smsg_keep;
use crate::winlayer::{Buf, windows};
use core::ffi::CStr;

pub(crate) use entry::u_undoredo;
pub use time::undo_time;

// ---------------------------------------------------------------------------
// One step: `u`, CTRL-R

/// `u` — undo, or, with `'cpoptions'` containing `u`, repeat the previous
/// undo-or-redo the other way, as the original vi did.
pub fn u_undo(count: c_int) {
    let count = count_after_sync(count);
    if cpo_has(CpoFlag::UNDO) {
        undo_undoes.set(!undo_undoes.get());
    } else {
        undo_undoes.set(true);
    }
    u_doit(count, false, true);
}

/// CTRL-R — redo, or, with `'cpoptions'` containing `u`, repeat the previous
/// undo-or-redo.
pub fn u_redo(count: c_int) {
    if !cpo_has(CpoFlag::UNDO) {
        undo_undoes.set(false);
    }
    u_doit(count, false, true);
}

/// Undo, then delete the branch that was undone, so that the change is gone
/// rather than reachable again with CTRL-R. Moves the cursor as a plain undo
/// would. Answers whether anything was undone.
pub fn u_undo_and_forget(count: c_int, do_buf_event: bool) -> bool {
    let count = count_after_sync(count);
    undo_undoes.set(true);
    u_doit(count, true, do_buf_event);

    let mut buf = Buf::current();
    let forgotten = buf.b_u_curhead;
    if !buf.has_header(forgotten) {
        return false; // nothing was undone
    }

    // Drop the header just undone and put the next alternate branch, if there
    // is one, in its place; without one the tree is back at a leaf.
    buf.b_u_newhead = buf.header_link(forgotten, |uh| uh.next);
    buf.b_u_curhead = buf.header_link(forgotten, |uh| uh.alt_next);
    let current = buf.b_u_curhead;
    if buf.has_header(current) {
        if let Some(uh) = buf.header_mut(forgotten) {
            uh.alt_next = UndoLink::NONE;
        }
        let alt_prev = buf.header_link(forgotten, |uh| uh.alt_prev);
        if let Some(uh) = buf.header_mut(current) {
            uh.alt_prev = alt_prev;
        }
        let above = buf.header_link(current, |uh| uh.next);
        buf.b_u_seq_cur = buf.header(above).map_or(0, |uh| uh.seq);
    } else if let Some(seq) = buf.header(buf.b_u_newhead).map(|uh| uh.seq) {
        buf.b_u_seq_cur = seq;
    }
    let alt_prev = buf.header_link(forgotten, |uh| uh.alt_prev);
    if let Some(uh) = buf.header_mut(alt_prev) {
        uh.alt_next = current;
    }
    let newhead = buf.b_u_newhead;
    if let Some(uh) = buf.header_mut(newhead) {
        uh.prev = current;
    }
    if buf
        .header(forgotten)
        .is_some_and(|uh| uh.seq == buf.b_u_seq_last)
    {
        buf.b_u_seq_last -= 1;
    }
    // No link still names it.
    u_freebranch(buf, forgotten, None);
    true
}

/// Syncs a pending change before undoing it, and answers the count to use:
/// an unsynced buffer means we are inside a macro, where vi undoes exactly
/// one change however the command was counted. Twice in one macro and the
/// result stops being vi-compatible either way.
fn count_after_sync(count: c_int) -> c_int {
    if Buf::current().b_u_synced {
        return count;
    }
    u_sync(true);
    1
}

/// Undoes or redoes — whichever `undo_undoes` says — `startcount` times.
pub(crate) fn u_doit(startcount: c_int, quiet: bool, do_buf_event: bool) {
    if !undo_allowed(Buf::current()) {
        return;
    }
    u_newcount.set(0);
    let empty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);
    u_oldcount.set(if empty { -1 } else { 0 });
    msg_ext_set_kind(c"undo");

    let mut count = startcount;
    let mut first = true;
    while count != 0 {
        count -= 1;
        // The change warning goes first so that FileChangedRO fires before
        // anything else: it may reload the file, and that moves
        // `b_u_curhead` and more.
        change_warning(Buf::current(), 0);
        let mut buf = Buf::current();
        if undo_undoes.get() {
            if buf.b_u_curhead.is_none() {
                buf.b_u_curhead = buf.b_u_newhead; // the first undo
            } else if get_undolevel(buf) > 0 {
                // Multi-level undo: on to the next one.
                buf.b_u_curhead = buf.header_link(buf.b_u_curhead, |uh| uh.next);
            }
            if buf.b_u_numhead == 0 || buf.b_u_curhead.is_none() {
                // Nothing to undo: park `b_u_curhead` at the end.
                buf.b_u_curhead = buf.b_u_oldhead;
                beep_flush();
                if first {
                    msg(gettext(c"Already at oldest change"), 0);
                    return;
                }
                break;
            }
            u_undoredo(true, do_buf_event);
        } else {
            if buf.b_u_curhead.is_none() || get_undolevel(buf) <= 0 {
                beep_flush();
                if first {
                    msg(gettext(c"Already at newest change"), 0);
                    return;
                }
                break;
            }
            u_undoredo(false, do_buf_event);
            // Advance for the next redo, and mark the end of the redoable
            // changes with `b_u_newhead`.
            if let Some(prev) = buf.header(buf.b_u_curhead).map(|uh| uh.prev) {
                if prev.is_none() {
                    buf.b_u_newhead = buf.b_u_curhead;
                }
                buf.b_u_curhead = prev;
            }
        }
        first = false;
    }
    u_undo_end(undo_undoes.get(), false, quiet);
}

// ---------------------------------------------------------------------------
// Reporting

/// How much to report and the word for it: the number of lines gained or
/// lost, or — when the move left the line count alone — the number of
/// changes, which is a guess but better than nothing.
fn undo_report(oldcount: c_int, newcount: c_int) -> (c_int, &'static CStr) {
    match oldcount {
        -1 => (oldcount, c"more line"),
        n if n < 0 => (oldcount, c"more lines"),
        1 => (oldcount, c"line less"),
        n if n > 1 => (oldcount, c"fewer lines"),
        _ if newcount == 1 => (newcount, c"change"),
        _ => (newcount, c"changes"),
    }
}

/// Reports what a move through the tree did, and redraws what has to be.
pub(crate) fn u_undo_end(did_undo: bool, absolute: bool, quiet: bool) {
    if fdo_flags.get() & kOptFdoFlagUndo as c_uint != 0 && KeyTyped.get() {
        fold_open_cursor();
    }
    // No messages until :global has finished, and none while 'lazyredraw'
    // holds them back.
    if quiet || global_busy.get() != 0 || !messaging() {
        return;
    }

    let buf = Buf::current();
    if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
        u_newcount.set(u_newcount.get() - 1);
    }
    u_oldcount.set(u_oldcount.get() - u_newcount.get());
    let (count, what) = undo_report(u_oldcount.get(), u_newcount.get());
    u_oldcount.set(count);

    // Which header's timestamp and sequence number to name. For ":undo N" an
    // "after #N" message reads better than a "before" one.
    let current = buf.b_u_curhead;
    let above = buf.header_link(current, |uh| uh.next);
    let (target, did_undo) = if !buf.has_header(current) {
        (buf.b_u_newhead, did_undo)
    } else if absolute && above.is_some() {
        (above, false)
    } else if did_undo {
        (current, true)
    } else {
        (above, did_undo)
    };
    let target = buf.header(target).map(|uh| (uh.seq, uh.time));
    // Already the empty string when there is no header to date.
    let when = target.map_or_else(XString::new, |(_, time)| undo_time_text(time, 80));

    // What is concealed may have changed, so every window on this buffer with
    // 'conceallevel' set has to be redrawn in full.
    for wp in windows() {
        if wp.buffer() == buf && wp.w_onebuf_opt.wo_cole > 0 {
            wp.redraw_later(UPD_NOT_VALID);
        }
    }
    if visual_active() {
        with_visual_anchor(|visual| check_pos(buf, visual));
    }

    let amount = int64_t::from(count).abs();
    let what = gettext(what).to_string_lossy();
    let which = if did_undo { c"before" } else { c"after" };
    let side = gettext(which).to_string_lossy();
    let seq = target.map_or(0, |(seq, _)| int64_t::from(seq));
    let when = msg_cstr(when.as_cstr());
    smsg_keep!(0, "{amount} {what}; {side} #{seq}  {when}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_names_lines_when_the_count_changed_and_changes_otherwise() {
        assert_eq!(undo_report(-1, 0), (-1, c"more line"));
        assert_eq!(undo_report(-3, 0), (-3, c"more lines"));
        assert_eq!(undo_report(1, 0), (1, c"line less"));
        assert_eq!(undo_report(4, 0), (4, c"fewer lines"));
        assert_eq!(undo_report(0, 1), (1, c"change"));
        assert_eq!(undo_report(0, 3), (3, c"changes"));
    }
}
