//! The undo tree's own bookkeeping: freeing headers, branches and
//! entries, and the single-line `u_undoline` shadow buffer.
//!
//! Every header here is named by a [`UndoLink`] and reached through the
//! buffer's store (see [`super::store`]); the `*mut u_header_T` locals are
//! the borrow that lookup hands back, never an owner.

#![deny(unsafe_op_in_unsafe_fn)]

use super::store::{header_at, header_chain, header_free, link_of, store_release};
use super::*;

/// Marks `start` and everything reachable from it backwards as changed.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn u_unch_branch(buf: *mut buf_T, start: UndoLink) {
    // SAFETY: a live buffer, and nothing here frees a header.
    unsafe {
        for mut uh in header_chain(buf, start, |uh| uh.uh_prev) {
            uh.uh_flags |= UH_CHANGED;
            if uh.uh_alt_next.is_some() {
                u_unch_branch(buf, uh.uh_alt_next);
            }
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
    unsafe {
        let newhead = header_at(buf, (*buf).b_u_newhead);
        if newhead.is_null() || (*newhead).uh_entry.is_null() {
            iemsg(gettext(c"E439: Undo list corrupt".as_ptr()));
            return ptr::null_mut();
        }
        (*newhead).uh_entry
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
    unsafe {
        if u_get_headentry(buf).is_null() {
            return;
        }
        let newhead = header_at(buf, (*buf).b_u_newhead);
        let uep = (*newhead).uh_getbot_entry;
        if !uep.is_null() {
            let extra: linenr_T = (*buf).b_ml.ml_line_count - (*uep).ue_lcount;
            (*uep).ue_bot = (*uep).ue_top + (*uep).ue_size + 1 + extra;
            if (*uep).ue_bot < 1 || (*uep).ue_bot > (*buf).b_ml.ml_line_count {
                iemsg(gettext(c"E440: Undo line missing".as_ptr()));
                (*uep).ue_bot = (*uep).ue_top + 1;
            }
            (*newhead).uh_getbot_entry = ptr::null_mut();
        }
        (*buf).b_u_synced = true;
    }
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
    unsafe {
        if (*uhp).uh_alt_next.is_some() {
            u_freebranch(buf, header_at(buf, (*uhp).uh_alt_next), uhpp);
        }
        let alt_prev = header_at(buf, (*uhp).uh_alt_prev);
        if !alt_prev.is_null() {
            (*alt_prev).uh_alt_next = UndoLink::NONE;
        }
        let next = header_at(buf, (*uhp).uh_next);
        if next.is_null() {
            (*buf).b_u_oldhead = (*uhp).uh_prev;
        } else {
            (*next).uh_prev = (*uhp).uh_prev;
        }
        if (*uhp).uh_prev.is_none() {
            (*buf).b_u_newhead = (*uhp).uh_next;
        } else {
            // The alternate headers at `uh_prev` all claim this header's
            // successor.
            for mut uhap in header_chain(buf, (*uhp).uh_prev, |uh| uh.uh_alt_next) {
                uhap.uh_next = (*uhp).uh_next;
            }
        }
        u_freeentries(buf, uhp, uhpp);
    }
}

/// Frees a whole alternate branch, oldest header first.
///
/// # Safety
///
/// As [`u_freeheader`].
pub(crate) unsafe fn u_freebranch(buf: *mut buf_T, uhp: *mut u_header_T, uhpp: *mut UndoLink) {
    // SAFETY: a live buffer and a header it owns.
    unsafe {
        // Freeing the oldest header takes the whole tree with it, so let
        // `u_freeheader` do the unlinking rather than walking here.
        if link_of(uhp) == (*buf).b_u_oldhead {
            while (*buf).b_u_oldhead.is_some() {
                let oldhead = header_at(buf, (*buf).b_u_oldhead);
                u_freeheader(buf, oldhead, uhpp);
            }
            return;
        }
        let alt_prev = header_at(buf, (*uhp).uh_alt_prev);
        if !alt_prev.is_null() {
            (*alt_prev).uh_alt_next = UndoLink::NONE;
        }
        // Not `header_chain`: the step would have to read a header this loop
        // has already freed.
        let mut next = uhp;
        while !next.is_null() {
            let tofree = next;
            if (*tofree).uh_alt_next.is_some() {
                u_freebranch(buf, header_at(buf, (*tofree).uh_alt_next), uhpp);
            }
            next = header_at(buf, (*tofree).uh_prev);
            u_freeentries(buf, tofree, uhpp);
        }
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
    unsafe {
        let link = link_of(uhp);
        if (*buf).b_u_curhead == link {
            (*buf).b_u_curhead = UndoLink::NONE;
        }
        if (*buf).b_u_newhead == link {
            (*buf).b_u_newhead = UndoLink::NONE;
        }
        if !uhpp.is_null() && *uhpp == link {
            *uhpp = UndoLink::NONE;
        }
        let mut uep: *mut u_entry_T = (*uhp).uh_entry;
        while !uep.is_null() {
            let nuep: *mut u_entry_T = (*uep).ue_next;
            u_freeentry(uep, (*uep).ue_size as c_int);
            uep = nuep;
        }
        xfree((*uhp).uh_extmark.items as *mut c_void);
        (*uhp).uh_extmark.capacity = 0;
        (*uhp).uh_extmark.size = 0;
        (*uhp).uh_extmark.items = ptr::null_mut();
        header_free(buf, uhp);
        (*buf).b_u_numhead -= 1;
    }
}

/// Frees one entry and the `n` saved lines it holds.
///
/// # Safety
///
/// `uep` points at a live entry whose `ue_array` holds at least `n` strings.
pub(crate) unsafe fn u_freeentry(uep: *mut u_entry_T, mut n: c_int) {
    // SAFETY: a live entry with at least `n` lines, by the contract above.
    unsafe {
        while n > 0 {
            n -= 1;
            xfree(*(*uep).ue_array.offset(n as isize) as *mut c_void);
        }
        xfree((*uep).ue_array as *mut c_void);
        xfree(uep as *mut c_void);
    }
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
    unsafe {
        (*buf).b_u_curhead = UndoLink::NONE;
        (*buf).b_u_oldhead = UndoLink::NONE;
        (*buf).b_u_newhead = UndoLink::NONE;
        (*buf).b_u_synced = true;
        (*buf).b_u_numhead = 0;
        (*buf).b_u_line_ptr = ptr::null_mut();
        (*buf).b_u_line_lnum = 0;
    }
}

/// Frees every header the buffer's tree still reaches, and the shadow line.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_blockfree(buf: *mut buf_T) {
    // SAFETY: a live buffer; each pass frees the oldest header, and the
    // assert is the transpiled loop's own guard against not making progress.
    unsafe {
        while (*buf).b_u_oldhead.is_some() {
            let previous_oldhead = (*buf).b_u_oldhead;
            u_freeheader(buf, header_at(buf, previous_oldhead), ptr::null_mut());
            debug_assert!(
                (*buf).b_u_oldhead != previous_oldhead,
                "buf->b_u_oldhead != previous_oldhead"
            );
        }
        xfree((*buf).b_u_line_ptr as *mut c_void);
        store_release(buf);
    }
}

/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_clearallandblockfree(buf: *mut buf_T) {
    // SAFETY: a live buffer.
    unsafe {
        u_blockfree(buf);
        u_clearall(buf);
    }
}

/// Remembers one line so `U` can put it back.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn u_saveline(buf: *mut buf_T, lnum: linenr_T) {
    // SAFETY: a live buffer, and `lnum` is checked against its line count.
    unsafe {
        if lnum == (*buf).b_u_line_lnum {
            return;
        }
        if lnum < 1 || lnum > (*buf).b_ml.ml_line_count {
            return;
        }
        u_clearline(buf);
        (*buf).b_u_line_lnum = lnum;
        if (*curwin.get()).w_buffer == buf && (*curwin.get()).w_cursor.lnum == lnum {
            (*buf).b_u_line_colnr = (*curwin.get()).w_cursor.col;
        } else {
            (*buf).b_u_line_colnr = 0;
        }
        (*buf).b_u_line_ptr = u_save_line_buf(buf, lnum);
    }
}

/// Forgets the line `U` would have put back.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_clearline(buf: *mut buf_T) {
    // SAFETY: a live buffer; `b_u_line_ptr` is this module's own allocation.
    unsafe {
        if (*buf).b_u_line_ptr.is_null() {
            return;
        }
        xfree((*buf).b_u_line_ptr.cast());
        (*buf).b_u_line_ptr = ptr::null_mut();
        (*buf).b_u_line_lnum = 0;
    }
}

/// `U`: swap the current line against the one `u_saveline` kept.
///
/// # Safety
///
/// Called from the editor's main loop, with a current buffer and window.
pub unsafe fn u_undoline() {
    // SAFETY: a live current buffer and window.
    unsafe {
        if (*curbuf.get()).b_u_line_ptr.is_null()
            || (*curbuf.get()).b_u_line_lnum > (*curbuf.get()).b_ml.ml_line_count
        {
            beep_flush();
            return;
        }
        if u_savecommon(
            curbuf.get(),
            (*curbuf.get()).b_u_line_lnum - 1,
            (*curbuf.get()).b_u_line_lnum + 1,
            0,
            false,
        ) == FAIL
        {
            return;
        }
        let oldp: *mut c_char = u_save_line((*curbuf.get()).b_u_line_lnum);
        ml_replace(
            (*curbuf.get()).b_u_line_lnum,
            (*curbuf.get()).b_u_line_ptr,
            true,
        );
        extmark_splice_cols(
            curbuf.get(),
            (*curbuf.get()).b_u_line_lnum as c_int - 1,
            0,
            strlen(oldp) as colnr_T,
            strlen((*curbuf.get()).b_u_line_ptr) as colnr_T,
            kExtmarkUndo,
        );
        changed_bytes((*curbuf.get()).b_u_line_lnum, 0);
        xfree((*curbuf.get()).b_u_line_ptr as *mut c_void);
        (*curbuf.get()).b_u_line_ptr = oldp;
        let t: colnr_T = (*curbuf.get()).b_u_line_colnr;
        if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_u_line_lnum {
            (*curbuf.get()).b_u_line_colnr = (*curwin.get()).w_cursor.col;
        }
        (*curwin.get()).w_cursor.col = t;
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_u_line_lnum;
        check_cursor_col(curwin.get());
    }
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
