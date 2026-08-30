//! The undo tree's own bookkeeping: freeing headers, branches and
//! entries, and the single-line `u_undoline` shadow buffer.
//!
//! Every header here is named by a [`UndoLink`] and reached through the
//! buffer's store (see [`super::store`]); the `*mut u_header_T` locals are
//! the borrow that lookup hands back, never an owner.

#![deny(unsafe_op_in_unsafe_fn)]

use super::store::{Header, header_chain, header_free, store_release};
use super::*;
use crate::cstr;
use crate::winlayer::Buf;
use crate::winlayer::Win;

/// Marks `start` and everything reachable from it backwards as changed.
///
/// # Safety
///
/// Nothing frees a header this walk has already visited.
pub(crate) unsafe fn u_unch_branch(buf: Buf, start: UndoLink) {
    // SAFETY: nothing here frees a header.
    for mut uh in unsafe { header_chain(buf, start, |uh| uh.uh_prev) } {
        uh.uh_flags |= UH_CHANGED;
        if uh.uh_alt_next.is_some() {
            unsafe { u_unch_branch(buf, uh.uh_alt_next) };
        }
    }
}

/// The newest header's entry list, complaining if the tree has come apart.
///
/// Safe: `b_u_newhead` is resolved through the store, so it names either a
/// live header or nothing.
pub(crate) fn u_get_headentry(buf: Buf) -> *mut u_entry_T {
    let newhead = buf.header(buf.b_u_newhead);
    match newhead.filter(|uh| !uh.uh_entry.is_null()) {
        Some(uh) => uh.uh_entry,
        None => {
            iemsg(gettext(c"E439: Undo list corrupt"));
            ptr::null_mut()
        }
    }
}

/// Fills in the `ue_bot` the newest header deferred, and marks the buffer
/// synced.
///
/// Safe: `u_get_headentry` proves the newest header and its entry list are
/// there before anything reads them.
pub(crate) fn u_getbot(mut buf: Buf) {
    if u_get_headentry(buf).is_null() {
        return;
    }
    let mut newhead = buf
        .header(buf.b_u_newhead)
        .expect("u_get_headentry proved it is there");
    let uep = newhead.uh_getbot_entry;
    if !uep.is_null() {
        // SAFETY: the newest header's own deferred entry, proved live above.
        let extra: linenr_T = buf.b_ml.ml_line_count - unsafe { (*uep).ue_lcount };
        unsafe { (*uep).ue_bot = (*uep).ue_top + (*uep).ue_size + 1 + extra };
        if unsafe { (*uep).ue_bot } < 1 || unsafe { (*uep).ue_bot } > buf.b_ml.ml_line_count {
            iemsg(gettext(c"E440: Undo line missing"));
            unsafe { (*uep).ue_bot = (*uep).ue_top + 1 };
        }
        newhead.uh_getbot_entry = ptr::null_mut();
    }
    buf.b_u_synced = true;
}

/// Unlinks one header from the tree and frees it, along with the alternate
/// branch hanging off it.
///
/// `uhpp`, when it is not NULL, is a link the caller is still holding: it is
/// cleared if it named the header that went away.
///
/// # Safety
///
/// `uhp` points at a header `buf` owns, and `uhpp` is NULL or points at a
/// link the caller owns.
pub(crate) unsafe fn u_freeheader(mut buf: Buf, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a header the buffer owns; every link below is resolved through
    // the store, so a stale one reads as "nothing".
    let b = buf;
    if let Some(alt) = b.header(unsafe { (*uhp).uh_alt_next }) {
        unsafe { u_freebranch(buf, alt.raw(), uhpp) };
    }
    if let Some(mut alt_prev) = b.header(unsafe { (*uhp).uh_alt_prev }) {
        alt_prev.uh_alt_next = UndoLink::NONE;
    }
    match b.header(unsafe { (*uhp).uh_next }) {
        Some(mut next) => next.uh_prev = unsafe { (*uhp).uh_prev },
        None => buf.b_u_oldhead = unsafe { (*uhp).uh_prev },
    }
    if unsafe { (*uhp).uh_prev.is_none() } {
        buf.b_u_newhead = unsafe { (*uhp).uh_next };
    } else {
        // The alternate headers at `uh_prev` all claim this header's
        // successor.
        for mut uhap in unsafe { header_chain(buf, (*uhp).uh_prev, |uh| uh.uh_alt_next) } {
            uhap.uh_next = unsafe { (*uhp).uh_next };
        }
    }
    unsafe { u_freeentries(buf, uhp, uhpp) };
}

/// Frees a whole alternate branch, oldest header first.
///
/// # Safety
///
/// As [`u_freeheader`].
pub(crate) unsafe fn u_freebranch(buf: Buf, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a header the buffer owns.
    // Freeing the oldest header takes the whole tree with it, so let
    // `u_freeheader` do the unlinking rather than walking here.
    let b = buf;
    if unsafe { Header::new(uhp) }
        .map(Header::link)
        .unwrap_or_default()
        == buf.b_u_oldhead
    {
        while let Some(oldhead) = b.header(buf.b_u_oldhead) {
            unsafe { u_freeheader(buf, oldhead.raw(), uhpp) };
        }
        return;
    }
    if let Some(mut alt_prev) = b.header(unsafe { (*uhp).uh_alt_prev }) {
        alt_prev.uh_alt_next = UndoLink::NONE;
    }
    // Not `header_chain`: the step would have to read a header this loop
    // has already freed.
    let mut next = unsafe { Header::new(uhp) };
    while let Some(tofree) = next {
        if let Some(alt) = b.header(tofree.uh_alt_next) {
            unsafe { u_freebranch(buf, alt.raw(), uhpp) };
        }
        next = b.header(tofree.uh_prev);
        unsafe { u_freeentries(buf, tofree.raw(), uhpp) };
    }
}

/// Frees one header's entries, its extmark list and the header itself.
///
/// # Safety
///
/// As [`u_freeheader`].
pub(crate) unsafe fn u_freeentries(mut buf: Buf, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a header the buffer owns; the entry list is that header's and
    // is walked one node ahead of the free.
    let link = UndoLink::to_seq(unsafe { (*uhp).uh_seq });
    if buf.b_u_curhead == link {
        buf.b_u_curhead = UndoLink::NONE;
    }
    if buf.b_u_newhead == link {
        buf.b_u_newhead = UndoLink::NONE;
    }
    if !uhpp.is_null() && unsafe { *uhpp } == link {
        unsafe { *uhpp = UndoLink::NONE };
    }
    let mut uep: *mut u_entry_T = unsafe { (*uhp).uh_entry };
    while !uep.is_null() {
        let nuep: *mut u_entry_T = unsafe { (*uep).ue_next };
        unsafe { u_freeentry(uep, (*uep).ue_size as c_int) };
        uep = nuep;
    }
    unsafe { xfree((*uhp).uh_extmark.items as *mut c_void) };
    unsafe { (*uhp).uh_extmark.capacity = 0 };
    unsafe { (*uhp).uh_extmark.size = 0 };
    unsafe { (*uhp).uh_extmark.items = ptr::null_mut() };
    unsafe { header_free(buf, uhp) };
    buf.b_u_numhead -= 1;
}

/// Frees one entry and the `n` saved lines it holds.
///
/// # Safety
///
/// `uep` points at a live entry whose `ue_array` holds at least `n` strings.
pub(crate) unsafe fn u_freeentry(uep: *mut u_entry_T, mut n: c_int) {
    // SAFETY: a live entry with at least `n` lines, by the contract above.
    while n > 0 {
        n -= 1;
        unsafe { xfree(*(*uep).ue_array.offset(n as isize) as *mut c_void) };
    }
    unsafe { xfree((*uep).ue_array as *mut c_void) };
    unsafe { xfree(uep as *mut c_void) };
}

/// Detaches the buffer's undo tree without freeing it.
///
/// The headers stay in the store; a command preview puts them back.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs.
pub fn u_clearall(mut buf: Buf) {
    buf.b_u_curhead = UndoLink::NONE;
    buf.b_u_oldhead = UndoLink::NONE;
    buf.b_u_newhead = UndoLink::NONE;
    buf.b_u_synced = true;
    buf.b_u_numhead = 0;
    buf.b_u_line_ptr = ptr::null_mut();
    buf.b_u_line_lnum = 0;
}

/// Frees every header the buffer's tree still reaches, and the shadow line.
///
/// Safe: every header freed here is one the buffer's own tree still holds.
pub fn u_blockfree(buf: Buf) {
    let b = buf;
    while let Some(oldhead) = b.header(buf.b_u_oldhead) {
        let previous_oldhead = buf.b_u_oldhead;
        // SAFETY: a header the tree still holds, and no link the caller owns.
        // Each pass frees the oldest header, and the assert is the transpiled
        // loop's own guard against not making progress.
        unsafe { u_freeheader(buf, oldhead.raw(), ptr::null_mut()) };
        debug_assert!(
            buf.b_u_oldhead != previous_oldhead,
            "buf->b_u_oldhead != previous_oldhead"
        );
    }
    // SAFETY: `b_u_line_ptr` is this module's own allocation.
    unsafe { xfree(buf.b_u_line_ptr as *mut c_void) };
    store_release(buf);
}

/// Safe: as [`u_blockfree`] and [`u_clearall`], which are the whole of it.
pub fn u_clearallandblockfree(buf: Buf) {
    u_blockfree(buf);
    u_clearall(buf);
}

/// Remembers one line so `U` can put it back.
///
/// Safe: `lnum` is checked against the buffer's own line count.
pub(crate) fn u_saveline(mut buf: Buf, lnum: linenr_T) {
    if lnum == buf.b_u_line_lnum {
        return;
    }
    if lnum < 1 || lnum > buf.b_ml.ml_line_count {
        return;
    }
    u_clearline(buf);
    buf.b_u_line_lnum = lnum;
    if cur_win().w_buffer == buf.raw() && cur_win().w_cursor.lnum == lnum {
        buf.b_u_line_colnr = cur_win().w_cursor.col;
    } else {
        buf.b_u_line_colnr = 0;
    }
    // SAFETY: `lnum` was checked against the buffer's line count above.
    buf.b_u_line_ptr = unsafe { u_save_line_buf(buf, lnum) };
}

/// Forgets the line `U` would have put back.
///
/// Safe: `b_u_line_ptr` is this module's own allocation.
pub fn u_clearline(mut buf: Buf) {
    if buf.b_u_line_ptr.is_null() {
        return;
    }
    // SAFETY: this module allocated it and nothing else holds it.
    unsafe { xfree(buf.b_u_line_ptr.cast()) };
    buf.b_u_line_ptr = ptr::null_mut();
    buf.b_u_line_lnum = 0;
}

/// `U`: swap the current line against the one `u_saveline` kept.
///
/// # Safety
///
/// Called from the editor's main loop, with a current buffer and window.
pub unsafe fn u_undoline() {
    // SAFETY: a live current buffer and window.
    if cur_buf().b_u_line_ptr.is_null() || cur_buf().b_u_line_lnum > cur_buf().b_ml.ml_line_count {
        beep_flush();
        return;
    }
    // Bound first: rustfmt puts a call wider than 60 columns on one line per
    // argument, and every one of those lines is inside the region.
    let lnum = cur_buf().b_u_line_lnum;
    if u_savecommon(cur_buf(), lnum - 1, lnum + 1, 0, false).is_err() {
        return;
    }
    let oldp: *mut c_char = unsafe { u_save_line(cur_buf().b_u_line_lnum) };
    let _ = unsafe { ml_replace(cur_buf().b_u_line_lnum, cur_buf().b_u_line_ptr, true) };
    let oldp_len = unsafe { cstr::bytes_at(oldp) }.len();
    let ptr_len = unsafe { cstr::bytes_at(cur_buf().b_u_line_ptr) }.len();
    unsafe {
        extmark_splice_cols(
            curbuf.get(),
            cur_buf().b_u_line_lnum as c_int - 1,
            0,
            oldp_len as colnr_T,
            ptr_len as colnr_T,
            kExtmarkUndo,
        )
    };
    unsafe { changed_bytes(cur_buf().b_u_line_lnum, 0) };
    unsafe { xfree(cur_buf().b_u_line_ptr as *mut c_void) };
    cur_buf().b_u_line_ptr = oldp;
    let t: colnr_T = cur_buf().b_u_line_colnr;
    if cur_win().w_cursor.lnum == cur_buf().b_u_line_lnum {
        cur_buf().b_u_line_colnr = cur_win().w_cursor.col;
    }
    cur_win().w_cursor.col = t;
    cur_win().w_cursor.lnum = cur_buf().b_u_line_lnum;
    check_cursor_col(unsafe { Win::current() });
}

/// A fresh copy of line `lnum` of the current buffer.
///
/// # Safety
///
/// A live current buffer holding line `lnum`.
pub(crate) unsafe fn u_save_line(lnum: linenr_T) -> *mut c_char {
    // SAFETY: a live current buffer holding that line, by the contract above.
    unsafe { u_save_line_buf(cur_buf(), lnum) }
}

/// A fresh copy of line `lnum` of `buf`.
///
/// # Safety
///
/// `buf` holds line `lnum`.
pub(crate) unsafe fn u_save_line_buf(buf: Buf, lnum: linenr_T) -> *mut c_char {
    // SAFETY: the buffer holds that line, by the contract above.
    unsafe { xstrdup(ml_get_buf(buf.raw(), lnum)) }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
