//! The jump list and the change list.
//!
//! Both are fixed arrays with a length and an index: the jump list is a
//! window's `[XFileMark; JUMPLISTSIZE]` and the change list a buffer's
//! `[FileMark; JUMPLISTSIZE]`. The index is where `<C-o>`/`<C-i>` and `g;`/`g,`
//! currently stand, and `idx == len` is a **legal** one-past-the-end state —
//! it is what makes `:jumps` print its trailing bare `>` row and the first
//! `<C-o>` reach the newest entry rather than the one before it.
//!
//! The change list's *append* is not here: it lives in `change/splice.rs`,
//! with the `b_new_change` gate and the column-distance dedup. This module
//! only walks the list.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::find_buf;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::state::{global_busy, listcmd_busy};
use crate::getchar::state::got_int;
use crate::memory::XString;
use crate::message::{
    message_filtered, msg_display, msg_ext_set_kind, msg_putchar, msg_str, msg_title,
};
use crate::option::vars::jop_flags;
use crate::os::cshim::gettext;
use crate::os::input::os_breakcheck;
use crate::pos::equalpos;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_uint};

use super::show::*;
use super::store::slot;
use super::*;
use crate::highlight_group::HLF_D;
use crate::types::CmdModFlags;

/// Set the previous context mark to the current position and add it to the
/// jump list.
///
/// Safe: the only promise is that the editor exists, which
/// `Win::current()`/`Buf::current()` carry.
pub fn setpcmark() {
    // `:keepjumps` and the two "the editor is driving itself" flags suppress
    // the push entirely — a `:global` that visits three hundred lines must
    // not fill the jump list with them.
    if global_busy.get() != 0 || listcmd_busy.get() || cmdmod_has(CmdModFlags::KEEPJUMPS) {
        return;
    }
    let (mut win, buf) = (Win::current(), Buf::current());
    win.w_prev_pcmark = win.w_pcmark;
    win.w_pcmark = win.w_cursor;
    if win.w_pcmark.lnum == 0 {
        win.w_pcmark.lnum = 1;
    }
    // With 'jumpoptions' "stack", a new jump truncates everything the user
    // had walked back past, the way a browser's history does.
    if jop_flags.get() & kOptJopFlagStack as c_uint != 0
        && win.w_jumplistidx < win.w_jumplistlen - 1
    {
        win.w_jumplistlen = win.w_jumplistidx + 1;
    }
    win.w_jumplistlen += 1;
    if win.w_jumplistlen > JUMPLISTSIZE {
        win.w_jumplistlen = JUMPLISTSIZE;
        // The oldest entry goes: everything moves down one, and the oldest
        // lands in the last slot, which the new entry below replaces.
        win.w_jumplist.rotate_left(1);
    }
    // One PAST the newest entry: see the module docs.
    win.w_jumplistidx = win.w_jumplistlen;
    let view = mark_view_make_at(win, win.w_pcmark);
    let newest = slot(win.w_jumplistlen - 1);
    win.w_jumplist[newest] = XFileMark {
        fmark: fresh_mark(win.w_pcmark, buf.handle, view),
        fname: None,
    };
}

/// To change context, call setpcmark(), then move the current position to
/// where ever, then call checkpcmark().  This ensures that the previous
/// context will only be changed if the cursor moved to a different line.
/// If pcmark was deleted (with "dG") the previous mark is restored.
pub fn checkpcmark() {
    let mut win = Win::current();
    if win.w_prev_pcmark.lnum != 0
        && (equalpos(win.w_pcmark, win.w_cursor) || win.w_pcmark.lnum == 0)
    {
        win.w_pcmark = win.w_prev_pcmark;
    }
    win.w_prev_pcmark.lnum = 0;
}

/// Get mark in "count" position in the |jumplist| relative to the current index.
///
/// If the mark is in a different buffer, it will be skipped unless the buffer exists.
///
/// @note cleanup_jumplist() is run, which removes duplicate marks, and
///       changes win->w_jumplistidx.
/// `win` — window to get jumplist from.
/// `count` — count to move may be negative.
///
/// Returns a copy of the mark, `None` if out of jumplist bounds.
pub fn get_jumplist(mut win: Win, mut count: c_int) -> Option<FileMark> {
    cleanup_jumplist(win, true);
    if win.w_jumplistlen == 0 {
        return None;
    }
    loop {
        if win.w_jumplistidx + count < 0 || win.w_jumplistidx + count >= win.w_jumplistlen {
            return None;
        }
        // Stepping off the one-past-the-end state first records where the
        // user is *now*, so that `<C-i>` can come back to it.
        if win.w_jumplistidx == win.w_jumplistlen {
            setpcmark();
            win.w_jumplistidx -= 1;
            if win.w_jumplistidx + count < 0 {
                return None;
            }
        }
        win.w_jumplistidx += count;
        let at = slot(win.w_jumplistidx);
        if win.w_jumplist[at].fmark.fnum == 0
            && let Some(fname) = win.w_jumplist[at].fname.clone()
        {
            // Loading the file runs autocommands; the entry is read again
            // after, by its index.
            fname2fnum(fname.as_cstr());
        }
        let fnum = win.w_jumplist[at].fmark.fnum;
        // An entry whose buffer no longer exists is skipped rather than
        // jumped to, and the step continues in the same direction.
        if fnum == Buf::current().handle || find_buf(fnum).is_some() {
            return Some(win.w_jumplist[at].fmark.clone());
        }
        count += if count < 0 { -1 } else { 1 };
    }
}

/// Get mark in "count" position in the |changelist| relative to the current index.
///
/// @note  Changes the win->w_changelistidx.
/// `win` — window to get jumplist from.
/// `count` — count to move may be negative.
///
/// Returns a copy of the mark, `None` if out of bounds.
pub fn get_changelist(mut buffer: Buf, win: Win, count: c_int) -> Option<FileMark> {
    let mut win = win;
    if buffer.b_changelistlen == 0 {
        return None;
    }
    let at = win.w_changelistidx;
    // Walking off either end CLAMPS rather than failing — but only once: a
    // second `g;` from the oldest change is what reports "at start of
    // changelist".
    let n = if at + count < 0 {
        if at == 0 {
            return None;
        }
        0
    } else if at + count >= buffer.b_changelistlen {
        if at == buffer.b_changelistlen - 1 {
            return None;
        }
        buffer.b_changelistlen - 1
    } else {
        at + count
    };
    win.w_changelistidx = n;
    // The entries carry no buffer of their own, so the entry is stamped with
    // the CURRENT buffer rather than with `buffer`, in place.
    let change = &mut buffer.b_changelist[slot(n)];
    change.fnum = Buf::current().handle;
    Some(change.clone())
}

/// Remove every jump list entry referring to a given buffer.
/// This function will also adjust the current jump list index.
pub fn mark_jumplist_forget_file(mut window: Win, fnum: c_int) {
    // Backwards, so removing an entry cannot skip the one after it.
    for i in (0..window.w_jumplistlen).rev() {
        let at = slot(i);
        if window.w_jumplist[at].fmark.fnum != fnum {
            continue;
        }
        if window.w_jumplistidx > i {
            window.w_jumplistidx -= 1;
        }
        let len = slot(window.w_jumplistlen);
        window.w_jumplistlen -= 1;
        // Close the gap: the entries above move down one, and the removed
        // one goes to the slot past the end, released there.
        let list = &mut window.w_jumplist[at..len];
        list.rotate_left(1);
        if let Some(gone) = list.last_mut() {
            *gone = XFileMark::UNSET;
        }
    }
}

/// When deleting lines, this may create duplicate marks in the
/// jumplist. They will be removed here for the specified window.
/// When "loadfiles" is true first ensure entries have the "fnum" field set
/// (this may be a bit slow).
pub fn cleanup_jumplist(mut window: Win, loadfiles: bool) {
    if loadfiles {
        // Every entry that still names its file by name gets its buffer
        // loaded, so that the duplicate test below can compare buffers.
        // The length is re-read every step: loading a file runs
        // autocommands.
        let mut i = 0;
        while i < window.w_jumplistlen {
            let jump = &window.w_jumplist[slot(i)];
            if jump.fmark.fnum == 0
                && jump.fmark.mark.lnum != 0
                && let Some(fname) = jump.fname.clone()
            {
                fname2fnum(fname.as_cstr());
            }
            i += 1;
        }
    }

    // Compact in place: `from` reads, `to` writes, and an entry is dropped
    // when a LATER entry names the same line of the same buffer — the newest
    // of a run of duplicates is the one kept.
    let stack = jop_flags.get() & kOptJopFlagStack as c_uint != 0;
    let w: &mut Window = &mut window;
    let len = slot(w.w_jumplistlen);
    let mut to = 0;
    for from in 0..len {
        if w.w_jumplistidx == c_int::try_from(from).unwrap_or(c_int::MAX) {
            w.w_jumplistidx = c_int::try_from(to).unwrap_or(c_int::MAX);
        }
        let here = &w.w_jumplist[from].fmark;
        let later = w.w_jumplist[from + 1..len].iter().position(|other| {
            other.fmark.fnum == here.fnum
                && here.fnum != 0
                && other.fmark.mark.lnum == here.mark.lnum
        });
        // No later duplicate: keep. A duplicate further along than the very
        // next entry: keep it only under 'jumpoptions' "stack", where the
        // list is a history rather than a set. The adjacent duplicate a line
        // deletion just created: always drop.
        let drop = match later {
            None => false,
            Some(0) => true,
            Some(_) => !stack,
        };
        if drop {
            w.w_jumplist[from] = XFileMark::UNSET;
        } else {
            if to != from {
                w.w_jumplist[to] = core::mem::replace(&mut w.w_jumplist[from], XFileMark::UNSET);
            }
            to += 1;
        }
    }
    let to = c_int::try_from(to).unwrap_or(c_int::MAX);
    if w.w_jumplistidx == w.w_jumplistlen {
        w.w_jumplistidx = to;
    }
    w.w_jumplistlen = to;

    // Standing one past the end, on an entry that names the line the cursor
    // is already on, means the newest jump is where we are: drop it, so
    // `<C-o>` goes somewhere.
    if !loadfiles || w.w_jumplistlen == 0 || w.w_jumplistidx != w.w_jumplistlen {
        return;
    }
    let last = slot(w.w_jumplistlen - 1);
    let here = Buf::current().handle;
    if w.w_jumplist[last].fmark.fnum == here
        && w.w_jumplist[last].fmark.mark.lnum == w.w_cursor.lnum
    {
        w.w_jumplist[last] = XFileMark::UNSET;
        w.w_jumplistlen -= 1;
        w.w_jumplistidx -= 1;
    }
}

/// Copy the jumplist from window "from" to window "to".
pub fn copy_jumplist(from: Win, mut to: Win) {
    for i in 0..slot(from.w_jumplistlen) {
        let entry = from.w_jumplist[i].clone();
        to.w_jumplist[i] = entry;
    }
    to.w_jumplistlen = from.w_jumplistlen;
    to.w_jumplistidx = from.w_jumplistidx;
}

/// Free items in the jumplist of window "wp".
///
/// Every slot, not only those in use: an entry past the end (one a
/// 'jumpoptions' "stack" jump cut off, say) still owns what it holds.
pub fn free_jumplist(mut window: Win) {
    window.w_jumplist.fill(XFileMark::UNSET);
    window.w_jumplistlen = 0;
}

/// One `:jumps`/`:changes` row's numbers, as `fmt` lays them out.
fn list_row(fmt: &[u8], current: bool, distance: c_int, pos: Pos) -> XString {
    let marker = if current { '>' as c_int } else { ' ' as c_int };
    let row = crate::formatted!(IOSIZE as usize, fmt, marker, distance, pos.lnum, pos.col);
    XString::from_vec(row)
}

/// print the jumplist
pub fn ex_jumps(_excmd: &mut ExArg) {
    let win = Win::current();
    cleanup_jumplist(win, true);
    msg_ext_set_kind(c"list_cmd");
    msg_title(gettext(c"\n jump line  col file/text"));
    let mut i: c_int = 0;
    while i < win.w_jumplistlen && !got_int.get() {
        let fmark = win.w_jumplist[slot(i)].fmark.clone();
        if fmark.mark.lnum != 0 {
            // The entry the index stands on is shown even when its file has
            // gone, so `:jumps` never silently loses the `>` row.
            let name = fm_getname(&fmark, 16)
                .or_else(|| (i == win.w_jumplistidx).then(|| XString::from_cstr(c"-invalid-")));
            if let Some(name) = name
                && !message_filtered(name.as_cstr())
            {
                msg_putchar('\n' as c_int);
                if got_int.get() {
                    break;
                }
                let here = Buf::current().handle;
                let row = list_row(
                    b"%c %2d %5d %4d ",
                    i == win.w_jumplistidx,
                    (i - win.w_jumplistidx).abs(),
                    fmark.mark,
                );
                msg_display(row.as_cstr(), 0, false);
                let attr = if fmark.fnum == here { HLF_D } else { 0 };
                msg_display(name.as_cstr(), attr, false);
                os_breakcheck();
            }
        }
        i += 1;
    }
    if win.w_jumplistidx == win.w_jumplistlen {
        // The bare `>` row: the index is one past the end, so there is no
        // entry to draw it on.
        msg_str(c"\n>");
    }
}

pub fn ex_clearjumps(_excmd: &mut ExArg) {
    let mut win = Win::current();
    free_jumplist(win);
    win.w_jumplistlen = 0;
    win.w_jumplistidx = 0;
}

/// print the changelist
pub fn ex_changes(_excmd: &mut ExArg) {
    let (buf, win) = (Buf::current(), Win::current());
    msg_ext_set_kind(c"list_cmd");
    msg_title(gettext(c"\nchange line  col text"));
    let mut i: c_int = 0;
    while i < buf.b_changelistlen && !got_int.get() {
        let pos = buf.b_changelist[slot(i)].mark;
        if pos.lnum != 0 {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                break;
            }
            let row = list_row(
                b"%c %3d %5d %4d ",
                i == win.w_changelistidx,
                (i - win.w_changelistidx).abs(),
                pos,
            );
            msg_display(row.as_cstr(), 0, false);
            let text = mark_line(pos, 17);
            msg_display(text.as_cstr(), HLF_D, false);
            os_breakcheck();
        }
        i += 1;
    }
    if win.w_changelistidx == buf.b_changelistlen {
        msg_str(c"\n>");
    }
}
