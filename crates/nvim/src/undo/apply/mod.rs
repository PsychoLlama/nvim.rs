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
//! store ([`Buf::header`](crate::winlayer::Buf::header)); a
//! [`Header`](super::store::Header) is the borrow that lookup hands back,
//! never an owner.

#![deny(unsafe_op_in_unsafe_fn)]

mod entry;
mod time;

use super::*;
use crate::drawscreen::UPD_NOT_VALID;
use crate::memline::MlFlags;
use crate::option::cpo_has;
use crate::smsg_keep_c;
use crate::winlayer::{Buf, windows};
use core::ffi::CStr;

pub(crate) use entry::u_undoredo;
pub use time::undo_time;

// ---------------------------------------------------------------------------
// One step: `u`, CTRL-R

/// `u` — undo, or, with `'cpoptions'` containing `u`, repeat the previous
/// undo-or-redo the other way, as the original vi did.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_undo(count: c_int) {
    // SAFETY: a live current buffer and window, by the contract above.
    unsafe {
        let count = count_after_sync(count);
        if cpo_has(CpoFlag::UNDO) {
            undo_undoes.set(!undo_undoes.get());
        } else {
            undo_undoes.set(true);
        }
        u_doit(count, false, true);
    }
}

/// CTRL-R — redo, or, with `'cpoptions'` containing `u`, repeat the previous
/// undo-or-redo.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_redo(count: c_int) {
    if !cpo_has(CpoFlag::UNDO) {
        undo_undoes.set(false);
    }
    // SAFETY: a live current buffer and window, by the contract above.
    unsafe { u_doit(count, false, true) };
}

/// Undo, then delete the branch that was undone, so that the change is gone
/// rather than reachable again with CTRL-R. Moves the cursor as a plain undo
/// would. Answers whether anything was undone.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_undo_and_forget(count: c_int, do_buf_event: bool) -> bool {
    // SAFETY: a live current buffer and window, by the contract above.
    let count = unsafe { count_after_sync(count) };
    undo_undoes.set(true);
    // SAFETY: as above.
    unsafe { u_doit(count, true, do_buf_event) };

    // SAFETY: as above.
    let mut buf = unsafe { Buf::current() };
    let Some(mut forgotten) = buf.header(buf.b_u_curhead) else {
        return false; // nothing was undone
    };

    // Drop the header just undone and put the next alternate branch, if there
    // is one, in its place; without one the tree is back at a leaf.
    buf.b_u_newhead = forgotten.uh_next;
    buf.b_u_curhead = forgotten.uh_alt_next;
    if let Some(mut curhead) = buf.header(buf.b_u_curhead) {
        forgotten.uh_alt_next = UndoLink::NONE;
        curhead.uh_alt_prev = forgotten.uh_alt_prev;
        buf.b_u_seq_cur = buf.header(curhead.uh_next).map_or(0, |uh| uh.uh_seq);
    } else if let Some(newhead) = buf.header(buf.b_u_newhead) {
        buf.b_u_seq_cur = newhead.uh_seq;
    }
    if let Some(mut alt_prev) = buf.header(forgotten.uh_alt_prev) {
        alt_prev.uh_alt_next = buf.b_u_curhead;
    }
    if let Some(mut newhead) = buf.header(buf.b_u_newhead) {
        newhead.uh_prev = buf.b_u_curhead;
    }
    if buf.b_u_seq_last == forgotten.uh_seq {
        buf.b_u_seq_last -= 1;
    }
    // SAFETY: a live buffer and a header it owns, and no link still names it.
    unsafe { u_freebranch(buf.raw(), forgotten.raw(), ptr::null_mut()) };
    true
}

/// Syncs a pending change before undoing it, and answers the count to use:
/// an unsynced buffer means we are inside a macro, where vi undoes exactly
/// one change however the command was counted. Twice in one macro and the
/// result stops being vi-compatible either way.
///
/// # Safety
///
/// A live current buffer.
unsafe fn count_after_sync(count: c_int) -> c_int {
    // SAFETY: a live current buffer, by the contract above.
    if unsafe { Buf::current() }.b_u_synced {
        return count;
    }
    // SAFETY: as above.
    unsafe { u_sync(true) };
    1
}

/// Undoes or redoes — whichever `undo_undoes` says — `startcount` times.
///
/// # Safety
///
/// A live current buffer and window.
pub(crate) unsafe fn u_doit(startcount: c_int, quiet: bool, do_buf_event: bool) {
    // SAFETY: a live current buffer, by the contract above.
    if !unsafe { undo_allowed(curbuf.get()) } {
        return;
    }
    u_newcount.set(0);
    // SAFETY: as above.
    let empty = unsafe { Buf::current() }.b_ml.ml_flags.has(MlFlags::EMPTY);
    u_oldcount.set(if empty { -1 } else { 0 });
    // SAFETY: a NUL-terminated literal.
    unsafe { msg_ext_set_kind(c"undo".as_ptr()) };

    let mut count = startcount;
    let mut first = true;
    while count != 0 {
        count -= 1;
        // The change warning goes first so that FileChangedRO fires before
        // anything else: it may reload the file, and that moves
        // `b_u_curhead` and more.
        // SAFETY: a live current buffer.
        unsafe { change_warning(curbuf.get(), 0) };
        // SAFETY: as above — and the reload may have replaced it.
        let mut buf = unsafe { Buf::current() };
        if undo_undoes.get() {
            if buf.b_u_curhead.is_none() {
                buf.b_u_curhead = buf.b_u_newhead; // the first undo
            // SAFETY: a live buffer.
            } else if unsafe { get_undolevel(buf.raw()) } > 0 {
                // Multi-level undo: on to the next one.
                let next = buf
                    .header(buf.b_u_curhead)
                    .map_or(UndoLink::NONE, |uh| uh.uh_next);
                buf.b_u_curhead = next;
            }
            if buf.b_u_numhead == 0 || buf.b_u_curhead.is_none() {
                // Nothing to undo: park `b_u_curhead` at the end.
                buf.b_u_curhead = buf.b_u_oldhead;
                // SAFETY: nothing here holds a borrow of editor state.
                unsafe { beep_flush() };
                if first {
                    // SAFETY: a NUL-terminated literal.
                    unsafe { msg(gettext(c"Already at oldest change".as_ptr()), 0) };
                    return;
                }
                break;
            }
            // SAFETY: a live current buffer and window, and `b_u_curhead`
            // names a header.
            unsafe { u_undoredo(true, do_buf_event) };
        } else {
            // SAFETY: a live buffer.
            if buf.b_u_curhead.is_none() || unsafe { get_undolevel(buf.raw()) } <= 0 {
                // SAFETY: nothing here holds a borrow of editor state.
                unsafe { beep_flush() };
                if first {
                    // SAFETY: a NUL-terminated literal.
                    unsafe { msg(gettext(c"Already at newest change".as_ptr()), 0) };
                    return;
                }
                break;
            }
            // SAFETY: as for the undo arm above.
            unsafe { u_undoredo(false, do_buf_event) };
            // Advance for the next redo, and mark the end of the redoable
            // changes with `b_u_newhead`.
            if let Some(curhead) = buf.header(buf.b_u_curhead) {
                if curhead.uh_prev.is_none() {
                    buf.b_u_newhead = buf.b_u_curhead;
                }
                buf.b_u_curhead = curhead.uh_prev;
            }
        }
        first = false;
    }
    // SAFETY: a live current buffer and window.
    unsafe { u_undo_end(undo_undoes.get(), false, quiet) };
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
///
/// # Safety
///
/// A live current buffer and window.
pub(crate) unsafe fn u_undo_end(did_undo: bool, absolute: bool, quiet: bool) {
    if fdo_flags.get() & kOptFdoFlagUndo as c_uint != 0 && KeyTyped.get() {
        // SAFETY: a live current window, by the contract above.
        unsafe { fold_open_cursor() };
    }
    // No messages until :global has finished, and none while 'lazyredraw'
    // holds them back.
    // SAFETY: nothing here holds a borrow of editor state.
    if quiet || global_busy.get() != 0 || !unsafe { messaging() } {
        return;
    }

    // SAFETY: a live current buffer.
    let buf = unsafe { Buf::current() };
    if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
        u_newcount.set(u_newcount.get() - 1);
    }
    u_oldcount.set(u_oldcount.get() - u_newcount.get());
    let (count, what) = undo_report(u_oldcount.get(), u_newcount.get());
    u_oldcount.set(count);

    // Which header's timestamp and sequence number to name. For ":undo N" an
    // "after #N" message reads better than a "before" one.
    let (target, did_undo) = match buf.header(buf.b_u_curhead) {
        Some(curhead) if absolute && curhead.uh_next.is_some() => {
            (buf.header(curhead.uh_next), false)
        }
        Some(curhead) if did_undo => (Some(curhead), true),
        Some(curhead) => (buf.header(curhead.uh_next), did_undo),
        None => (buf.header(buf.b_u_newhead), did_undo),
    };
    // Already the empty string when there is no header to date.
    let mut when: [c_char; 80] = [0; 80];
    if let Some(uhp) = target {
        // SAFETY: an 80-byte buffer, and its length says so.
        unsafe { undo_fmt_time(when.as_mut_ptr(), when.len(), uhp.uh_time) };
    }

    // What is concealed may have changed, so every window on this buffer with
    // 'conceallevel' set has to be redrawn in full.
    for wp in windows() {
        if wp.buffer() == buf && wp.w_onebuf_opt.wo_cole > 0 {
            wp.redraw_later(UPD_NOT_VALID);
        }
    }
    if VIsual_active.get() {
        // SAFETY: a live current buffer, and the cell lends the position for
        // the length of the call.
        VIsual.with_mut(|visual| unsafe { check_pos(buf.raw(), visual) });
    }

    // SAFETY: a format string and the arguments it names, and `when` is
    // NUL-terminated.
    unsafe {
        smsg_keep_c!(
            0,
            gettext(c"%ld %s; %s #%ld  %s".as_ptr()),
            int64_t::from(count).abs(),
            gettext(what.as_ptr()),
            if did_undo {
                gettext(c"before".as_ptr())
            } else {
                gettext(c"after".as_ptr())
            },
            target.map_or(0, |uhp| int64_t::from(uhp.uh_seq)),
            when.as_mut_ptr(),
        );
    }
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
