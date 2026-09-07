//! The undo tree's own bookkeeping: freeing headers, branches and
//! entries, and the single-line `u_undoline` shadow buffer.
//!
//! Every header here is named by a [`UndoLink`] and reached through the
//! buffer's store (see [`super::store`]); the `*mut UndoHeader` locals are
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
pub(crate) unsafe fn u_unch_branch(buffer: Buf, start: UndoLink) {
    // SAFETY: nothing here frees a header.
    for mut uh in header_chain(buffer, start, |uh| uh.uh_prev) {
        uh.uh_flags |= UH_CHANGED;
        if uh.uh_alt_next.is_some() {
            unsafe { u_unch_branch(buffer, uh.uh_alt_next) };
        }
    }
}

/// The newest header's entry list, complaining if the tree has come apart.
///
/// Safe: `b_u_newhead` is resolved through the store, so it names either a
/// live header or nothing.
pub(crate) fn u_get_headentry(buffer: Buf) -> *mut UndoEntry {
    let newhead = buffer.header(buffer.b_u_newhead);
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
pub(crate) fn u_getbot(mut buffer: Buf) {
    if u_get_headentry(buffer).is_null() {
        return;
    }
    let mut newhead = buffer
        .header(buffer.b_u_newhead)
        .expect("u_get_headentry proved it is there");
    let uep = newhead.uh_getbot_entry;
    if !uep.is_null() {
        // SAFETY: the newest header's own deferred entry, proved live above.
        let extra: LineNr = buffer.b_ml.ml_line_count - unsafe { (*uep).ue_lcount };
        unsafe { (*uep).ue_bot = (*uep).ue_top + (*uep).ue_size + 1 + extra };
        if unsafe { (*uep).ue_bot } < 1 || unsafe { (*uep).ue_bot } > buffer.b_ml.ml_line_count {
            iemsg(gettext(c"E440: Undo line missing"));
            unsafe { (*uep).ue_bot = (*uep).ue_top + 1 };
        }
        newhead.uh_getbot_entry = ptr::null_mut();
    }
    buffer.b_u_synced = true;
}

/// Unlinks one header from the tree and frees it, along with the alternate
/// branch hanging off it.
///
/// `uhpp`, when it is not NULL, is a link the caller is still holding: it is
/// cleared if it named the header that went away.
///
/// # Safety
///
/// `uhp` points at a header `buffer` owns, and `uhpp` is NULL or points at a
/// link the caller owns.
pub(crate) unsafe fn u_freeheader(mut buffer: Buf, uhp: *mut UndoHeader, uhpp: *mut UndoLink) {
    // SAFETY: a header the buffer owns; every link below is resolved through
    // the store, so a stale one reads as "nothing".
    let b = buffer;
    if let Some(alt) = b.header(unsafe { (*uhp).uh_alt_next }) {
        unsafe { u_freebranch(buffer, alt.raw(), uhpp) };
    }
    if let Some(mut alt_prev) = b.header(unsafe { (*uhp).uh_alt_prev }) {
        alt_prev.uh_alt_next = UndoLink::NONE;
    }
    match b.header(unsafe { (*uhp).uh_next }) {
        Some(mut next) => next.uh_prev = unsafe { (*uhp).uh_prev },
        None => buffer.b_u_oldhead = unsafe { (*uhp).uh_prev },
    }
    if unsafe { (*uhp).uh_prev.is_none() } {
        buffer.b_u_newhead = unsafe { (*uhp).uh_next };
    } else {
        // The alternate headers at `uh_prev` all claim this header's
        // successor.
        for mut uhap in unsafe { header_chain(buffer, (*uhp).uh_prev, |uh| uh.uh_alt_next) } {
            uhap.uh_next = unsafe { (*uhp).uh_next };
        }
    }
    unsafe { u_freeentries(buffer, uhp, uhpp) };
}

/// Frees a whole alternate branch, oldest header first.
///
/// # Safety
///
/// As [`u_freeheader`].
pub(crate) unsafe fn u_freebranch(buffer: Buf, uhp: *mut UndoHeader, uhpp: *mut UndoLink) {
    // SAFETY: a header the buffer owns.
    // Freeing the oldest header takes the whole tree with it, so let
    // `u_freeheader` do the unlinking rather than walking here.
    let b = buffer;
    if unsafe { Header::new(uhp) }
        .map(Header::link)
        .unwrap_or_default()
        == buffer.b_u_oldhead
    {
        while let Some(oldhead) = b.header(buffer.b_u_oldhead) {
            unsafe { u_freeheader(buffer, oldhead.raw(), uhpp) };
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
            unsafe { u_freebranch(buffer, alt.raw(), uhpp) };
        }
        next = b.header(tofree.uh_prev);
        unsafe { u_freeentries(buffer, tofree.raw(), uhpp) };
    }
}

/// Frees one header's entries, its extmark list and the header itself.
///
/// # Safety
///
/// As [`u_freeheader`].
pub(crate) unsafe fn u_freeentries(mut buffer: Buf, uhp: *mut UndoHeader, uhpp: *mut UndoLink) {
    // SAFETY: a header the buffer owns; the entry list is that header's and
    // is walked one node ahead of the free.
    let link = UndoLink::to_seq(unsafe { (*uhp).uh_seq });
    if buffer.b_u_curhead == link {
        buffer.b_u_curhead = UndoLink::NONE;
    }
    if buffer.b_u_newhead == link {
        buffer.b_u_newhead = UndoLink::NONE;
    }
    if !uhpp.is_null() && unsafe { *uhpp } == link {
        unsafe { *uhpp = UndoLink::NONE };
    }
    let mut uep: *mut UndoEntry = unsafe { (*uhp).uh_entry };
    while !uep.is_null() {
        let nuep: *mut UndoEntry = unsafe { (*uep).ue_next };
        unsafe { u_freeentry(uep, (*uep).ue_size as c_int) };
        uep = nuep;
    }
    unsafe { xfree((*uhp).uh_extmark.items as *mut c_void) };
    unsafe { (*uhp).uh_extmark.capacity = 0 };
    unsafe { (*uhp).uh_extmark.size = 0 };
    unsafe { (*uhp).uh_extmark.items = ptr::null_mut() };
    unsafe { header_free(buffer, uhp) };
    buffer.b_u_numhead -= 1;
}

/// Frees one entry and the `n` saved lines it holds.
///
/// # Safety
///
/// `uep` points at a live entry whose `ue_array` holds at least `n` strings.
pub(crate) unsafe fn u_freeentry(uep: *mut UndoEntry, mut n: c_int) {
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
pub fn u_clearall(mut buffer: Buf) {
    buffer.b_u_curhead = UndoLink::NONE;
    buffer.b_u_oldhead = UndoLink::NONE;
    buffer.b_u_newhead = UndoLink::NONE;
    buffer.b_u_synced = true;
    buffer.b_u_numhead = 0;
    buffer.b_u_line_ptr = ptr::null_mut();
    buffer.b_u_line_lnum = 0;
}

/// Frees every header the buffer's tree still reaches, and the shadow line.
///
/// Safe: every header freed here is one the buffer's own tree still holds.
pub fn u_blockfree(buffer: Buf) {
    let b = buffer;
    while let Some(oldhead) = b.header(buffer.b_u_oldhead) {
        let previous_oldhead = buffer.b_u_oldhead;
        // SAFETY: a header the tree still holds, and no link the caller owns.
        // Each pass frees the oldest header, and the assert is the transpiled
        // loop's own guard against not making progress.
        unsafe { u_freeheader(buffer, oldhead.raw(), ptr::null_mut()) };
        debug_assert!(
            buffer.b_u_oldhead != previous_oldhead,
            "buf->b_u_oldhead != previous_oldhead"
        );
    }
    // SAFETY: `b_u_line_ptr` is this module's own allocation.
    unsafe { xfree(buffer.b_u_line_ptr as *mut c_void) };
    store_release(buffer);
}

/// Safe: as [`u_blockfree`] and [`u_clearall`], which are the whole of it.
pub fn u_clearallandblockfree(buffer: Buf) {
    u_blockfree(buffer);
    u_clearall(buffer);
}

/// Remembers one line so `U` can put it back.
///
/// Safe: `lnum` is checked against the buffer's own line count.
pub(crate) fn u_saveline(mut buffer: Buf, lnum: LineNr) {
    if lnum == buffer.b_u_line_lnum {
        return;
    }
    if lnum < 1 || lnum > buffer.b_ml.ml_line_count {
        return;
    }
    u_clearline(buffer);
    buffer.b_u_line_lnum = lnum;
    if Win::current().w_buffer == buffer.raw() && Win::current().w_cursor.lnum == lnum {
        buffer.b_u_line_colnr = Win::current().w_cursor.col;
    } else {
        buffer.b_u_line_colnr = 0;
    }
    // SAFETY: `lnum` was checked against the buffer's line count above.
    buffer.b_u_line_ptr = unsafe { u_save_line_buf(buffer, lnum) };
}

/// Forgets the line `U` would have put back.
///
/// Safe: `b_u_line_ptr` is this module's own allocation.
pub fn u_clearline(mut buffer: Buf) {
    if buffer.b_u_line_ptr.is_null() {
        return;
    }
    // SAFETY: this module allocated it and nothing else holds it.
    unsafe { xfree(buffer.b_u_line_ptr.cast()) };
    buffer.b_u_line_ptr = ptr::null_mut();
    buffer.b_u_line_lnum = 0;
}

/// `U`: swap the current line against the one `u_saveline` kept.
///
/// # Safety
///
/// Called from the editor's main loop, with a current buffer and window.
pub unsafe fn u_undoline() {
    // SAFETY: a live current buffer and window.
    if Buf::current().b_u_line_ptr.is_null()
        || Buf::current().b_u_line_lnum > Buf::current().b_ml.ml_line_count
    {
        beep_flush();
        return;
    }
    // Bound first: rustfmt puts a call wider than 60 columns on one line per
    // argument, and every one of those lines is inside the region.
    let lnum = Buf::current().b_u_line_lnum;
    if u_savecommon(Buf::current(), lnum - 1, lnum + 1, 0, false).is_err() {
        return;
    }
    let oldp: *mut c_char = unsafe { u_save_line(Buf::current().b_u_line_lnum) };
    let (lnum, line) = (Buf::current().b_u_line_lnum, Buf::current().b_u_line_ptr);
    let _ = unsafe { ml_replace(lnum, line, true) };
    let oldp_len = unsafe { cstr::bytes_at(oldp) }.len();
    let ptr_len = unsafe { cstr::bytes_at(Buf::current().b_u_line_ptr) }.len();
    extmark_splice_cols(
        Buf::current(),
        Buf::current().b_u_line_lnum as c_int - 1,
        0,
        oldp_len as ColNr,
        ptr_len as ColNr,
        kExtmarkUndo,
    );
    unsafe { changed_bytes(Buf::current().b_u_line_lnum, 0) };
    unsafe { xfree(Buf::current().b_u_line_ptr as *mut c_void) };
    Buf::current().b_u_line_ptr = oldp;
    let t: ColNr = Buf::current().b_u_line_colnr;
    if Win::current().w_cursor.lnum == Buf::current().b_u_line_lnum {
        Buf::current().b_u_line_colnr = Win::current().w_cursor.col;
    }
    Win::current().w_cursor.col = t;
    Win::current().w_cursor.lnum = Buf::current().b_u_line_lnum;
    check_cursor_col(Win::current());
}

/// A fresh copy of line `lnum` of the current buffer.
///
/// # Safety
///
/// A live current buffer holding line `lnum`.
pub(crate) unsafe fn u_save_line(lnum: LineNr) -> *mut c_char {
    // SAFETY: a live current buffer holding that line, by the contract above.
    unsafe { u_save_line_buf(Buf::current(), lnum) }
}

/// A fresh copy of line `lnum` of `buffer`.
///
/// # Safety
///
/// `buffer` holds line `lnum`.
pub(crate) unsafe fn u_save_line_buf(buffer: Buf, lnum: LineNr) -> *mut c_char {
    // SAFETY: the buffer holds that line, by the contract above.
    unsafe { xstrdup(ml_get_buf(buffer, lnum)) }
}
