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

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::find_buf;
use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::state::{global_busy, listcmd_busy};
use crate::getchar::state::got_int;
use crate::memory::{xfree, xstrdup};
use crate::message::{
    message_filtered, msg_display, msg_ext_set_kind, msg_putchar, msg_str, msg_title,
};
use crate::option::vars::jop_flags;
use crate::os::cshim::{gettext, snprintf};
use crate::os::input::os_breakcheck;
use crate::pos::equalpos;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use super::show::*;
use super::store::{UNSET_XFMARK, Xfmark};
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
        // SAFETY: the list is full, so entry 0 is live and its allocations
        // are the list's to free.
        unsafe { free_xfmark(win.jump(0).read()) };
        // SAFETY: source and destination are inside `[XFileMark; 100]` and the
        // length is the constant `JUMPLISTSIZE - 1`, so the move ends exactly
        // at the array's last element. Raising it writes past the array.
        let list = unsafe { &raw mut (*win.raw()).w_jumplist }.cast::<XFileMark>();
        let into = list.cast::<u8>();
        unsafe {
            into.copy_from(
                list.offset(1).cast(),
                ((JUMPLISTSIZE - 1) as size_t).wrapping_mul(size_of::<XFileMark>()),
            )
        };
    }
    // One PAST the newest entry: see the module docs.
    win.w_jumplistidx = win.w_jumplistlen;
    let view = mark_view_make_at(win, win.w_pcmark);
    // `place`, not `replace`: upstream reaches this through `SET_XFMARK`
    // rather than `RESET_XFMARK`, so the record that is about to be
    // overwritten is NOT freed here. In the clamp path above it is the
    // duplicate the `memmove` left in the last slot, whose allocations the
    // slot below it now owns.
    win.jump(win.w_jumplistlen - 1)
        .place(win.w_pcmark, buf.handle as c_int, view);
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
/// Returns mark, NULL if out of jumplist bounds.
///
/// # Safety
/// `win` must be a live window and the editor's globals must be live.
pub unsafe fn get_jumplist(mut win: Win, mut count: c_int) -> *mut FileMark {
    cleanup_jumplist(win, true);
    if win.w_jumplistlen == 0 {
        return ptr::null_mut();
    }
    loop {
        if win.w_jumplistidx + count < 0 || win.w_jumplistidx + count >= win.w_jumplistlen {
            return ptr::null_mut();
        }
        // Stepping off the one-past-the-end state first records where the
        // user is *now*, so that `<C-i>` can come back to it.
        if win.w_jumplistidx == win.w_jumplistlen {
            setpcmark();
            win.w_jumplistidx -= 1;
            if win.w_jumplistidx + count < 0 {
                return ptr::null_mut();
            }
        }
        win.w_jumplistidx += count;
        let jump = win.jump(win.w_jumplistidx);
        if jump.fmark().fnum() == 0 {
            // SAFETY: the entry is live and its name, if any, is a C string.
            unsafe { fname2fnum(jump.raw()) };
        }
        let here = Buf::current().handle;
        // An entry whose buffer no longer exists is skipped rather than
        // jumped to, and the step continues in the same direction.
        if jump.fmark().fnum() == here || find_buf(jump.fmark().fnum()).is_some() {
            return jump.fmark().raw();
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
/// Returns mark, NULL if out of bounds.
pub fn get_changelist(buffer: Buf, win: Win, count: c_int) -> *mut FileMark {
    let (buf, mut win) = (buffer, win);
    if buf.b_changelistlen == 0 {
        return ptr::null_mut();
    }
    let at = win.w_changelistidx;
    // Walking off either end CLAMPS rather than failing — but only once: a
    // second `g;` from the oldest change is what reports "at start of
    // changelist".
    let n = if at + count < 0 {
        if at == 0 {
            return ptr::null_mut();
        }
        0
    } else if at + count >= buf.b_changelistlen {
        if at == buf.b_changelistlen - 1 {
            return ptr::null_mut();
        }
        buf.b_changelistlen - 1
    } else {
        at + count
    };
    win.w_changelistidx = n;
    let change = buf.change(n);
    // The entries carry no buffer of their own, so the answer is stamped with
    // the CURRENT buffer rather than with `buf`.
    change.set_fnum(Buf::current().handle as c_int);
    change.raw()
}

/// Remove every jump list entry referring to a given buffer.
/// This function will also adjust the current jump list index.
pub fn mark_jumplist_forget_file(mut window: Win, fnum: c_int) {
    // SAFETY: the caller promised a live window.
    // Backwards, so removing an entry cannot skip the one after it.
    for i in (0..window.w_jumplistlen).rev() {
        if window.jump(i).fmark().fnum() != fnum {
            continue;
        }
        // SAFETY: `i` is inside the list, so the entry is live and its
        // allocations are the list's to free.
        unsafe { free_xfmark(window.jump(i).read()) };
        if window.w_jumplistidx > i {
            window.w_jumplistidx -= 1;
        }
        window.w_jumplistlen -= 1;
        // SAFETY: source and destination are inside `[XFileMark; 100]` and the
        // length is what is left above `i`, so the move stays in the array.
        let list = unsafe { &raw mut (*window.raw()).w_jumplist }.cast::<XFileMark>();
        unsafe {
            (list.offset(i as isize)).cast::<u8>().copy_from(
                (list.offset(i as isize + 1)).cast(),
                size_t::try_from(window.w_jumplistlen - i)
                    .unwrap_or(0)
                    .wrapping_mul(size_of::<XFileMark>()),
            )
        };
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
        for jump in window.jumps() {
            if jump.fmark().fnum() == 0 && jump.fmark().lnum() != 0 {
                // SAFETY: the entry is live and its name is a C string.
                unsafe { fname2fnum(jump.raw()) };
            }
        }
    }

    // Compact in place: `from` reads, `to` writes, and an entry is dropped
    // when a LATER entry names the same line of the same buffer — the newest
    // of a run of duplicates is the one kept.
    let mut to: c_int = 0;
    let mut from: c_int = 0;
    while from < window.w_jumplistlen {
        if window.w_jumplistidx == from {
            window.w_jumplistidx = to;
        }
        let here = window.jump(from).fmark();
        let mut i = from + 1;
        while i < window.w_jumplistlen {
            let other = window.jump(i).fmark();
            if other.fnum() == here.fnum() && here.fnum() != 0 && other.lnum() == here.lnum() {
                break;
            }
            i += 1;
        }
        // No later duplicate: keep. A duplicate further along than the very
        // next entry: keep it only under 'jumpoptions' "stack", where the
        // list is a history rather than a set. The adjacent duplicate a line
        // deletion just created: always drop.
        let mustfree = if i >= window.w_jumplistlen {
            false
        } else if i > from + 1 {
            jop_flags.get() & kOptJopFlagStack as c_uint == 0
        } else {
            true
        };
        if mustfree {
            // Only the NAME is freed, not `additional_data`: the record is
            // about to be overwritten by a later entry that owns its own.
            // SAFETY: the name is this entry's to free.
            unsafe { xfree(window.jump(from).fname().cast()) };
        } else {
            if to != from {
                let entry = window.jump(from).read();
                window.jump(to).write(entry);
            }
            to += 1;
        }
        from += 1;
    }
    if window.w_jumplistidx == window.w_jumplistlen {
        window.w_jumplistidx = to;
    }
    window.w_jumplistlen = to;

    // Standing one past the end, on an entry that names the line the cursor
    // is already on, means the newest jump is where we are: drop it, so
    // `<C-o>` goes somewhere.
    if !loadfiles || window.w_jumplistlen == 0 || window.w_jumplistidx != window.w_jumplistlen {
        return;
    }
    let last = window.jump(window.w_jumplistlen - 1);
    let here = Buf::current().handle;
    if last.fmark().fnum() == here && last.fmark().lnum() == window.w_cursor.lnum {
        // SAFETY: the name is this entry's to free.
        unsafe { xfree(last.fname().cast()) };
        window.w_jumplistlen -= 1;
        window.w_jumplistidx -= 1;
    }
}

/// Copy the jumplist from window "from" to window "to".
pub fn copy_jumplist(from: Win, to: Win) {
    // SAFETY: the caller promised two live windows.
    let (from, mut to) = (from, to);
    for i in 0..from.w_jumplistlen {
        let entry = from.jump(i).read();
        to.jump(i).write(entry.clone());
        // The file name is owned per entry, so the copy gets its own.
        if !entry.fname.is_null() {
            // SAFETY: `fname` is a NUL-terminated string owned by `from`.
            to.jump(i).set_fname(unsafe { xstrdup(entry.fname) });
        }
    }
    to.w_jumplistlen = from.w_jumplistlen;
    to.w_jumplistidx = from.w_jumplistidx;
}

/// Free items in the jumplist of window "wp".
///
/// # Safety
/// `window` must be a live window whose jump list entries own their allocations.
pub unsafe fn free_jumplist(mut window: Win) {
    for jump in window.jumps() {
        // SAFETY: the entry is live and its allocations are the list's.
        unsafe { free_xfmark(jump.read()) };
    }
    window.w_jumplistlen = 0;
}

/// print the jumplist
pub fn ex_jumps(_excmd: &mut ExArg) {
    let mut row = [0 as c_char; IOSIZE as usize];
    let win = Win::current();
    cleanup_jumplist(win, true);
    msg_ext_set_kind(c"list_cmd");
    msg_title(gettext(c"\n jump line  col file/text"));
    let mut i: c_int = 0;
    while i < win.w_jumplistlen && !got_int.get() {
        let jump = win.jump(i);
        if jump.fmark().lnum() != 0 {
            // SAFETY: the entry is live; `fm_getname` answers an allocation.
            let mut name = unsafe { fm_getname(jump.fmark().raw(), 16) };
            // The entry the index stands on is shown even when its file has
            // gone, so `:jumps` never silently loses the `>` row.
            if name.is_null() && i == win.w_jumplistidx {
                // SAFETY: a `'static` C string.
                name = unsafe { xstrdup(c"-invalid-".as_ptr()) };
            }
            // SAFETY: `name` is a NUL-terminated allocation or null, owned
            // here; every path below frees it exactly once.
            if name.is_null() || message_filtered(unsafe { cstr::at(name) }) {
                unsafe { xfree(name.cast()) };
            } else {
                msg_putchar('\n' as c_int);
                if got_int.get() {
                    unsafe { xfree(name.cast()) };
                    break;
                }
                let here = Buf::current().handle;
                unsafe {
                    snprintf(
                        row.as_mut_ptr(),
                        IOSIZE as size_t,
                        c"%c %2d %5d %4d ".as_ptr(),
                        if i == win.w_jumplistidx {
                            '>' as c_int
                        } else {
                            ' ' as c_int
                        },
                        (i - win.w_jumplistidx).abs(),
                        jump.fmark().lnum(),
                        jump.fmark().col(),
                    )
                };
                msg_display(cstr::in_chars(&row), 0, false);
                let attr = if jump.fmark().fnum() == here {
                    HLF_D
                } else {
                    0
                };
                msg_display(unsafe { cstr::at(name) }, attr, false);
                unsafe { xfree(name.cast()) };
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
    // SAFETY: as above.
    unsafe { free_jumplist(win) };
    win.w_jumplistlen = 0;
    win.w_jumplistidx = 0;
}

/// print the changelist
pub fn ex_changes(_excmd: &mut ExArg) {
    let mut row = [0 as c_char; IOSIZE as usize];
    let (buf, win) = (Buf::current(), Win::current());
    // SAFETY: as above.
    msg_ext_set_kind(c"list_cmd");
    msg_title(gettext(c"\nchange line  col text"));
    let mut i: c_int = 0;
    while i < buf.b_changelistlen && !got_int.get() {
        let change = buf.change(i);
        if change.lnum() != 0 {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                break;
            }
            // SAFETY: `row` is `IOSIZE` bytes of live storage and the
            // format string matches the four arguments.
            unsafe {
                snprintf(
                    row.as_mut_ptr(),
                    IOSIZE as size_t,
                    c"%c %3d %5d %4d ".as_ptr(),
                    if i == win.w_changelistidx {
                        '>' as c_int
                    } else {
                        ' ' as c_int
                    },
                    (i - win.w_changelistidx).abs(),
                    change.lnum(),
                    change.col(),
                )
            };
            msg_display(cstr::in_chars(&row), 0, false);
            let name = unsafe { mark_line(change.pos(), 17) };
            msg_display(unsafe { cstr::at(name) }, HLF_D, false);
            unsafe { xfree(name.cast()) };
            os_breakcheck();
        }
        i += 1;
    }
    if win.w_changelistidx == buf.b_changelistlen {
        msg_str(c"\n>");
    }
}

/// Iterate over jumplist items
///
/// @warning No jumplist-editing functions must be called while iteration is in
///          progress.
///
/// `iter` — Iterator. Pass NULL to start iteration.
/// `win` — Window for which jump list is processed.
/// `fm` — Item definition.
///
/// Returns pointer that needs to be passed to next `mark_jumplist_iter` call or
///         NULL if iteration is over.
///
/// # Safety
/// `win` must be a live window, `fm` must point at a live, writable
/// `XFileMark`, and `iter` must be null or a value a previous call answered for
/// the same window.
pub unsafe fn mark_jumplist_iter(
    iter: *const c_void,
    win: Win,
    fm: *mut XFileMark,
) -> *const c_void {
    // SAFETY: the caller promised a live out-parameter.
    let out = unsafe { Xfmark::new(fm) };
    if iter.is_null() && win.w_jumplistlen == 0 {
        out.write(UNSET_XFMARK);
        return ptr::null();
    }
    // SAFETY: `iter` is null or an entry of this window's list.
    let entry = unsafe {
        if iter.is_null() {
            win.jump(0)
        } else {
            Xfmark::new(iter.cast_mut().cast())
        }
    };
    out.write(entry.read());
    if entry == win.jump(win.w_jumplistlen - 1) {
        return ptr::null();
    }
    // The entry after this one; the test above has already established that
    // it is inside the list.
    entry.raw().wrapping_add(1).cast()
}
