//! The undo tree's own bookkeeping: freeing headers, branches and
//! entries, and the single-line `u_undoline` shadow buffer.

use super::*;

pub(crate) unsafe extern "C" fn u_unch_branch(mut uhp: *mut u_header_T) {
    let mut uh: *mut u_header_T = uhp;
    while !uh.is_null() {
        (*uh).uh_flags |= UH_CHANGED as c_int;
        if !(*uh).uh_alt_next.ptr.is_null() {
            u_unch_branch((*uh).uh_alt_next.ptr);
        }
        uh = (*uh).uh_prev.ptr;
    }
}
pub(crate) unsafe extern "C" fn u_get_headentry(mut buf: *mut buf_T) -> *mut u_entry_T {
    if (*buf).b_u_newhead.is_null() || (*(*buf).b_u_newhead).uh_entry.is_null() {
        iemsg(gettext(c"E439: Undo list corrupt".as_ptr()));
        return ptr::null_mut();
    }
    return (*(*buf).b_u_newhead).uh_entry;
}
pub(crate) unsafe extern "C" fn u_getbot(mut buf: *mut buf_T) {
    let mut uep: *mut u_entry_T = u_get_headentry(buf);
    if uep.is_null() {
        return;
    }
    uep = (*(*buf).b_u_newhead).uh_getbot_entry;
    if !uep.is_null() {
        let mut extra: linenr_T = (*buf).b_ml.ml_line_count - (*uep).ue_lcount;
        (*uep).ue_bot = (*uep).ue_top + (*uep).ue_size + 1 as linenr_T + extra;
        if (*uep).ue_bot < 1 as linenr_T || (*uep).ue_bot > (*buf).b_ml.ml_line_count {
            iemsg(gettext(c"E440: Undo line missing".as_ptr()));
            (*uep).ue_bot = (*uep).ue_top + 1 as linenr_T;
        }
        (*(*buf).b_u_newhead).uh_getbot_entry = ptr::null_mut();
    }
    (*buf).b_u_synced = true;
}
pub(crate) unsafe extern "C" fn u_freeheader(
    mut buf: *mut buf_T,
    mut uhp: *mut u_header_T,
    mut uhpp: *mut *mut u_header_T,
) {
    if !(*uhp).uh_alt_next.ptr.is_null() {
        u_freebranch(buf, (*uhp).uh_alt_next.ptr, uhpp);
    }
    if !(*uhp).uh_alt_prev.ptr.is_null() {
        (*(*uhp).uh_alt_prev.ptr).uh_alt_next.ptr = ptr::null_mut();
    }
    if (*uhp).uh_next.ptr.is_null() {
        (*buf).b_u_oldhead = (*uhp).uh_prev.ptr;
    } else {
        (*(*uhp).uh_next.ptr).uh_prev.ptr = (*uhp).uh_prev.ptr;
    }
    if (*uhp).uh_prev.ptr.is_null() {
        (*buf).b_u_newhead = (*uhp).uh_next.ptr;
    } else {
        let mut uhap: *mut u_header_T = (*uhp).uh_prev.ptr;
        while !uhap.is_null() {
            (*uhap).uh_next.ptr = (*uhp).uh_next.ptr;
            uhap = (*uhap).uh_alt_next.ptr;
        }
    }
    u_freeentries(buf, uhp, uhpp);
}
pub(crate) unsafe extern "C" fn u_freebranch(
    mut buf: *mut buf_T,
    mut uhp: *mut u_header_T,
    mut uhpp: *mut *mut u_header_T,
) {
    if uhp == (*buf).b_u_oldhead {
        while !(*buf).b_u_oldhead.is_null() {
            u_freeheader(buf, (*buf).b_u_oldhead, uhpp);
        }
        return;
    }
    if !(*uhp).uh_alt_prev.ptr.is_null() {
        (*(*uhp).uh_alt_prev.ptr).uh_alt_next.ptr = ptr::null_mut();
    }
    let mut next: *mut u_header_T = uhp;
    while !next.is_null() {
        let mut tofree: *mut u_header_T = next;
        if !(*tofree).uh_alt_next.ptr.is_null() {
            u_freebranch(buf, (*tofree).uh_alt_next.ptr, uhpp);
        }
        next = (*tofree).uh_prev.ptr;
        u_freeentries(buf, tofree, uhpp);
    }
}
pub(crate) unsafe extern "C" fn u_freeentries(
    mut buf: *mut buf_T,
    mut uhp: *mut u_header_T,
    mut uhpp: *mut *mut u_header_T,
) {
    if (*buf).b_u_curhead == uhp {
        (*buf).b_u_curhead = ptr::null_mut();
    }
    if (*buf).b_u_newhead == uhp {
        (*buf).b_u_newhead = ptr::null_mut();
    }
    if !uhpp.is_null() && uhp == *uhpp {
        *uhpp = ptr::null_mut();
    }
    let mut nuep: *mut u_entry_T = ptr::null_mut();
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    while !uep.is_null() {
        nuep = (*uep).ue_next;
        u_freeentry(uep, (*uep).ue_size as c_int);
        uep = nuep;
    }
    xfree((*uhp).uh_extmark.items as *mut c_void);
    (*uhp).uh_extmark.capacity = 0 as size_t;
    (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
    (*uhp).uh_extmark.items = ptr::null_mut();
    xfree(uhp as *mut c_void);
    (*buf).b_u_numhead -= 1;
}
pub(crate) unsafe extern "C" fn u_freeentry(mut uep: *mut u_entry_T, mut n: c_int) {
    while n > 0 {
        n -= 1;
        xfree(*(*uep).ue_array.offset(n as isize) as *mut c_void);
    }
    xfree((*uep).ue_array as *mut c_void);
    xfree(uep as *mut c_void);
}
pub unsafe extern "C" fn u_clearall(mut buf: *mut buf_T) {
    (*buf).b_u_curhead = ptr::null_mut();
    (*buf).b_u_oldhead = (*buf).b_u_curhead;
    (*buf).b_u_newhead = (*buf).b_u_oldhead;
    (*buf).b_u_synced = true;
    (*buf).b_u_numhead = 0;
    (*buf).b_u_line_ptr = ptr::null_mut();
    (*buf).b_u_line_lnum = 0 as linenr_T;
}
pub unsafe extern "C" fn u_blockfree(mut buf: *mut buf_T) {
    while !(*buf).b_u_oldhead.is_null() {
        let mut previous_oldhead: *mut u_header_T = (*buf).b_u_oldhead;
        u_freeheader(buf, (*buf).b_u_oldhead, ptr::null_mut());
        assert!(
            (*buf).b_u_oldhead != previous_oldhead,
            "buf->b_u_oldhead != previous_oldhead"
        );
    }
    xfree((*buf).b_u_line_ptr as *mut c_void);
}
pub unsafe extern "C" fn u_clearallandblockfree(mut buf: *mut buf_T) {
    u_blockfree(buf);
    u_clearall(buf);
}
pub(crate) unsafe extern "C" fn u_saveline(mut buf: *mut buf_T, mut lnum: linenr_T) {
    if lnum == (*buf).b_u_line_lnum {
        return;
    }
    if lnum < 1 as linenr_T || lnum > (*buf).b_ml.ml_line_count {
        return;
    }
    u_clearline(buf);
    (*buf).b_u_line_lnum = lnum;
    if (*curwin.get()).w_buffer == buf && (*curwin.get()).w_cursor.lnum == lnum {
        (*buf).b_u_line_colnr = (*curwin.get()).w_cursor.col;
    } else {
        (*buf).b_u_line_colnr = 0 as colnr_T;
    }
    (*buf).b_u_line_ptr = u_save_line_buf(buf, lnum);
}
pub unsafe extern "C" fn u_clearline(mut buf: *mut buf_T) {
    if (*buf).b_u_line_ptr.is_null() {
        return;
    }
    xfree((*buf).b_u_line_ptr.cast());
    (*buf).b_u_line_ptr = ptr::null_mut();
    (*buf).b_u_line_lnum = 0 as linenr_T;
}
pub unsafe extern "C" fn u_undoline() {
    if (*curbuf.get()).b_u_line_ptr.is_null()
        || (*curbuf.get()).b_u_line_lnum > (*curbuf.get()).b_ml.ml_line_count
    {
        beep_flush();
        return;
    }
    if u_savecommon(
        curbuf.get(),
        (*curbuf.get()).b_u_line_lnum - 1 as linenr_T,
        (*curbuf.get()).b_u_line_lnum + 1 as linenr_T,
        0 as linenr_T,
        false,
    ) == FAIL
    {
        return;
    }
    let mut oldp: *mut c_char = u_save_line((*curbuf.get()).b_u_line_lnum);
    ml_replace(
        (*curbuf.get()).b_u_line_lnum,
        (*curbuf.get()).b_u_line_ptr,
        true,
    );
    extmark_splice_cols(
        curbuf.get(),
        (*curbuf.get()).b_u_line_lnum as c_int - 1,
        0 as colnr_T,
        strlen(oldp) as colnr_T,
        strlen((*curbuf.get()).b_u_line_ptr) as colnr_T,
        kExtmarkUndo,
    );
    changed_bytes((*curbuf.get()).b_u_line_lnum, 0 as colnr_T);
    xfree((*curbuf.get()).b_u_line_ptr as *mut c_void);
    (*curbuf.get()).b_u_line_ptr = oldp;
    let mut t: colnr_T = (*curbuf.get()).b_u_line_colnr;
    if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_u_line_lnum {
        (*curbuf.get()).b_u_line_colnr = (*curwin.get()).w_cursor.col;
    }
    (*curwin.get()).w_cursor.col = t;
    (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_u_line_lnum;
    check_cursor_col(curwin.get());
}
pub(crate) unsafe extern "C" fn u_save_line(mut lnum: linenr_T) -> *mut c_char {
    return u_save_line_buf(curbuf.get(), lnum);
}
pub(crate) unsafe extern "C" fn u_save_line_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
) -> *mut c_char {
    return xstrdup(ml_get_buf(buf, lnum));
}
