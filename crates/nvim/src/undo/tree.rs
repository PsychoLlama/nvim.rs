//! The undo tree's own bookkeeping: freeing headers, branches and
//! entries, and the single-line `u_undoline` shadow buffer.
//!
//! Every header here is named by a [`UndoLink`] and reached through the
//! buffer's store (see [`super::store`]); the `*mut u_header_T` locals are
//! the borrow that lookup hands back, never an owner.

#![deny(unsafe_op_in_unsafe_fn)]

use super::store::{Header, header_chain, header_free, store_release};
use super::*;
use crate::winlayer::Buf;
use crate::winlayer::Win;

/// Marks `start` and everything reachable from it backwards as changed.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn u_unch_branch(buf: *mut buf_T, start: UndoLink) {
    // SAFETY: a live buffer, and nothing here frees a header.
    for mut uh in unsafe { header_chain(buf, start, |uh| uh.uh_prev) } {
        uh.uh_flags |= UH_CHANGED;
        if uh.uh_alt_next.is_some() {
            unsafe { u_unch_branch(buf, uh.uh_alt_next) };
        }
    }
}

/// The newest header's entry list, complaining if the tree has come apart.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn u_get_headentry(buf: *mut buf_T) -> *mut u_entry_T {
    // SAFETY: a live buffer; `b_u_newhead` is resolved through the store, so
    // it is either a live header or nothing.
    // SAFETY: a live buffer, by the contract above.
    let newhead = unsafe { Buf::new(buf) }.header(unsafe { (*buf).b_u_newhead });
    match newhead.filter(|uh| !uh.uh_entry.is_null()) {
        Some(uh) => uh.uh_entry,
        None => {
            // SAFETY: a NUL-terminated literal.
            unsafe { iemsg(gettext(c"E439: Undo list corrupt".as_ptr())) };
            ptr::null_mut()
        }
    }
}

/// Fills in the `ue_bot` the newest header deferred, and marks the buffer
/// synced.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn u_getbot(buf: *mut buf_T) {
    // SAFETY: a live buffer, and `u_get_headentry` proved the newest header
    // and its entry list are there.
    if unsafe { u_get_headentry(buf) }.is_null() {
        return;
    }
    let mut newhead = unsafe { Buf::new(buf) }
        .header(unsafe { (*buf).b_u_newhead })
        .expect("u_get_headentry proved it is there");
    let uep = newhead.uh_getbot_entry;
    if !uep.is_null() {
        let extra: linenr_T = unsafe { (*buf).b_ml.ml_line_count } - unsafe { (*uep).ue_lcount };
        unsafe { (*uep).ue_bot = (*uep).ue_top + (*uep).ue_size + 1 + extra };
        if unsafe { (*uep).ue_bot } < 1
            || unsafe { (*uep).ue_bot } > unsafe { (*buf).b_ml.ml_line_count }
        {
            unsafe { iemsg(gettext(c"E440: Undo line missing".as_ptr())) };
            unsafe { (*uep).ue_bot = (*uep).ue_top + 1 };
        }
        newhead.uh_getbot_entry = ptr::null_mut();
    }
    unsafe { (*buf).b_u_synced = true };
}

/// Unlinks one header from the tree and frees it, along with the alternate
/// branch hanging off it.
///
/// `uhpp`, when it is not NULL, is a link the caller is still holding: it is
/// cleared if it named the header that went away.
///
/// # Safety
///
/// `buf` points at a live buffer, `uhp` at a header it owns, and `uhpp` is
/// NULL or points at a link the caller owns.
pub(crate) unsafe fn u_freeheader(buf: *mut buf_T, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a live buffer and a header it owns; every link below is
    // resolved through the store, so a stale one reads as "nothing".
    let b = unsafe { Buf::new(buf) };
    if let Some(alt) = b.header(unsafe { (*uhp).uh_alt_next }) {
        unsafe { u_freebranch(buf, alt.raw(), uhpp) };
    }
    if let Some(mut alt_prev) = b.header(unsafe { (*uhp).uh_alt_prev }) {
        alt_prev.uh_alt_next = UndoLink::NONE;
    }
    match b.header(unsafe { (*uhp).uh_next }) {
        Some(mut next) => next.uh_prev = unsafe { (*uhp).uh_prev },
        None => unsafe { (*buf).b_u_oldhead = (*uhp).uh_prev },
    }
    if unsafe { (*uhp).uh_prev.is_none() } {
        unsafe { (*buf).b_u_newhead = (*uhp).uh_next };
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
pub(crate) unsafe fn u_freebranch(buf: *mut buf_T, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a live buffer and a header it owns.
    // Freeing the oldest header takes the whole tree with it, so let
    // `u_freeheader` do the unlinking rather than walking here.
    let b = unsafe { Buf::new(buf) };
    if unsafe { Header::new(uhp) }
        .map(Header::link)
        .unwrap_or_default()
        == unsafe { (*buf).b_u_oldhead }
    {
        while let Some(oldhead) = b.header(unsafe { (*buf).b_u_oldhead }) {
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
pub(crate) unsafe fn u_freeentries(buf: *mut buf_T, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a live buffer and a header it owns; the entry list is that
    // header's and is walked one node ahead of the free.
    let link = UndoLink::to_seq(unsafe { (*uhp).uh_seq });
    if unsafe { (*buf).b_u_curhead } == link {
        unsafe { (*buf).b_u_curhead = UndoLink::NONE };
    }
    if unsafe { (*buf).b_u_newhead } == link {
        unsafe { (*buf).b_u_newhead = UndoLink::NONE };
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
    unsafe { (*buf).b_u_numhead -= 1 };
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
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_clearall(buf: *mut buf_T) {
    // SAFETY: a live buffer.
    unsafe { (*buf).b_u_curhead = UndoLink::NONE };
    unsafe { (*buf).b_u_oldhead = UndoLink::NONE };
    unsafe { (*buf).b_u_newhead = UndoLink::NONE };
    unsafe { (*buf).b_u_synced = true };
    unsafe { (*buf).b_u_numhead = 0 };
    unsafe { (*buf).b_u_line_ptr = ptr::null_mut() };
    unsafe { (*buf).b_u_line_lnum = 0 };
}

/// Frees every header the buffer's tree still reaches, and the shadow line.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_blockfree(buf: *mut buf_T) {
    // SAFETY: a live buffer; each pass frees the oldest header, and the
    // assert is the transpiled loop's own guard against not making progress.
    let b = unsafe { Buf::new(buf) };
    while let Some(oldhead) = b.header(unsafe { (*buf).b_u_oldhead }) {
        let previous_oldhead = unsafe { (*buf).b_u_oldhead };
        unsafe { u_freeheader(buf, oldhead.raw(), ptr::null_mut()) };
        debug_assert!(
            unsafe { (*buf).b_u_oldhead } != previous_oldhead,
            "buf->b_u_oldhead != previous_oldhead"
        );
    }
    unsafe { xfree((*buf).b_u_line_ptr as *mut c_void) };
    unsafe { store_release(buf) };
}

/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_clearallandblockfree(buf: *mut buf_T) {
    // SAFETY: a live buffer.
    unsafe { u_blockfree(buf) };
    unsafe { u_clearall(buf) };
}

/// Remembers one line so `U` can put it back.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn u_saveline(buf: *mut buf_T, lnum: linenr_T) {
    // SAFETY: a live buffer, and `lnum` is checked against its line count.
    if lnum == unsafe { (*buf).b_u_line_lnum } {
        return;
    }
    if lnum < 1 || lnum > unsafe { (*buf).b_ml.ml_line_count } {
        return;
    }
    unsafe { u_clearline(buf) };
    unsafe { (*buf).b_u_line_lnum = lnum };
    if cur_win().w_buffer == buf && cur_win().w_cursor.lnum == lnum {
        unsafe { (*buf).b_u_line_colnr = cur_win().w_cursor.col };
    } else {
        unsafe { (*buf).b_u_line_colnr = 0 };
    }
    unsafe { (*buf).b_u_line_ptr = u_save_line_buf(buf, lnum) };
}

/// Forgets the line `U` would have put back.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_clearline(buf: *mut buf_T) {
    // SAFETY: a live buffer; `b_u_line_ptr` is this module's own allocation.
    if unsafe { (*buf).b_u_line_ptr.is_null() } {
        return;
    }
    unsafe { xfree((*buf).b_u_line_ptr.cast()) };
    unsafe { (*buf).b_u_line_ptr = ptr::null_mut() };
    unsafe { (*buf).b_u_line_lnum = 0 };
}

/// `U`: swap the current line against the one `u_saveline` kept.
///
/// # Safety
///
/// Called from the editor's main loop, with a current buffer and window.
pub unsafe fn u_undoline() {
    // SAFETY: a live current buffer and window.
    if cur_buf().b_u_line_ptr.is_null() || cur_buf().b_u_line_lnum > cur_buf().b_ml.ml_line_count {
        unsafe { beep_flush() };
        return;
    }
    // Bound first: rustfmt puts a call wider than 60 columns on one line per
    // argument, and every one of those lines is inside the region.
    let lnum = cur_buf().b_u_line_lnum;
    if unsafe { u_savecommon(curbuf.get(), lnum - 1, lnum + 1, 0, false) } == FAIL {
        return;
    }
    let oldp: *mut c_char = unsafe { u_save_line(cur_buf().b_u_line_lnum) };
    unsafe { ml_replace(cur_buf().b_u_line_lnum, cur_buf().b_u_line_ptr, true) };
    unsafe {
        extmark_splice_cols(
            curbuf.get(),
            cur_buf().b_u_line_lnum as c_int - 1,
            0,
            strlen(oldp) as colnr_T,
            strlen(cur_buf().b_u_line_ptr) as colnr_T,
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
    unsafe { check_cursor_col(curwin.get()) };
}

/// A fresh copy of line `lnum` of the current buffer.
///
/// # Safety
///
/// A live current buffer holding line `lnum`.
pub(crate) unsafe fn u_save_line(lnum: linenr_T) -> *mut c_char {
    // SAFETY: a live current buffer, by the contract above.
    unsafe { u_save_line_buf(curbuf.get(), lnum) }
}

/// A fresh copy of line `lnum` of `buf`.
///
/// # Safety
///
/// `buf` points at a live buffer holding line `lnum`.
pub(crate) unsafe fn u_save_line_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut c_char {
    // SAFETY: a live buffer holding that line, by the contract above.
    unsafe { xstrdup(ml_get_buf(buf, lnum)) }
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
