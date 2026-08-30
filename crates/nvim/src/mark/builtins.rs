//! `getmarklist()`, in both its shapes.
//!
//! This is a second, independent surface over the same slots `:marks` prints
//! and `getpos()` reads, and the three do not agree by construction: the
//! column here is 1-based (`getpos()`'s convention), `:marks` prints the
//! internal 0-based one, and a mark that is not set is simply absent rather
//! than reported at line 0.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::buflist_nr2name;
use crate::eval::typval::{
    tv_dict_add_list, tv_dict_add_str, tv_dict_alloc, tv_list_alloc, tv_list_append_dict,
    tv_list_append_number,
};
use crate::main::c_bytes;
use crate::memory::xfree;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::ptr;

use super::store::{GlobalMarks, mark_name};
use super::*;
use crate::pos::MAXCOL;
use crate::types::Failed;
use crate::types::kListLenMayKnow;

/// Add information about mark 'mname' to list 'l'
///
/// # Safety
/// `l` must be a live list, `mname` and `fname` NUL-terminated strings (or
/// null, for `fname`), and `pos` a live position.
pub(super) unsafe fn add_mark(
    l: *mut list_T,
    mname: *const c_char,
    pos: *const pos_T,
    bufnr: c_int,
    fname: *const c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller promised a live position.
    let pos = unsafe { *pos };
    // An unset mark is omitted rather than reported at line 0: the list is
    // "the marks that exist", which is what makes it usable without a filter.
    if pos.lnum <= 0 {
        return Ok(());
    }
    // SAFETY: the caller promised a live list and NUL-terminated strings; the
    // dict and the position list are handed to `l`, which owns them from
    // `tv_list_append_dict` on.
    let d = unsafe { tv_dict_alloc() };
    unsafe { tv_list_append_dict(l, d) };
    let lpos = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
    unsafe { tv_list_append_number(lpos, varnumber_T::from(bufnr)) };
    unsafe { tv_list_append_number(lpos, varnumber_T::from(pos.lnum)) };
    // 1-BASED, unlike `:marks` and unlike the store. `MAXCOL` — which is
    // what a linewise `'>` carries — is passed through rather than
    // incremented, so it stays recognisable.
    unsafe {
        tv_list_append_number(
            lpos,
            varnumber_T::from(if pos.col < MAXCOL {
                pos.col + 1
            } else {
                MAXCOL
            }),
        )
    };
    unsafe { tv_list_append_number(lpos, varnumber_T::from(pos.coladd)) };
    if unsafe { tv_dict_add_str(d, c"mark".as_ptr(), c"mark".count_bytes(), mname) }.is_err()
        || unsafe { tv_dict_add_list(d, c"pos".as_ptr(), c"pos".count_bytes(), lpos) }.is_err()
        || (!fname.is_null()
            && unsafe { tv_dict_add_str(d, c"file".as_ptr(), c"file".count_bytes(), fname) }
                .is_err())
    {
        return Err(Failed);
    }
    Ok(())
}

/// Get information about marks local to a buffer.
///
/// `buf` — Buffer to get the marks from
/// `l` — List to store marks
///
/// # Safety
/// `buf` must be a live buffer, `l` a live list, and the editor's globals must
/// be live.
pub unsafe fn get_buf_local_marks(buf: *const buf_T, l: *mut list_T) {
    // SAFETY: the caller promised a live buffer; `curwin`/`curbuf` are live
    // from startup to exit.
    let (buf, win, cur) = unsafe { (Buf::new(buf.cast_mut()), Win::current(), Buf::current()) };
    let handle = buf.handle as c_int;
    let mut mname: [c_char; 3] = c_bytes(b"' \0");
    for i in 0..NMARKS {
        mname[1] = mark_name('a' as c_int + i);
        // SAFETY: `mname` is NUL-terminated and lives for the call, and the
        // mark handle names a live position.
        let _ = unsafe {
            add_mark(
                l,
                mname.as_ptr(),
                buf.named_mark(i).pos_raw(),
                handle,
                ptr::null(),
            )
        };
    }
    // The context mark is the WINDOW's and is reported against the CURRENT
    // buffer, which is why it is the one row here that does not use `handle`.
    // SAFETY: as above.
    let _ = unsafe {
        add_mark(
            l,
            c"''".as_ptr(),
            &raw const (*win.raw()).w_pcmark,
            cur.handle as c_int,
            ptr::null(),
        )
    };
    let positions: [(&core::ffi::CStr, *const pos_T); 7] = [
        (c"'\"", buf.last_cursor().pos_raw()),
        (c"'[", &raw const buf.b_op_start),
        (c"']", &raw const buf.b_op_end),
        (c"'^", buf.last_insert().pos_raw()),
        (c"'.", buf.last_change().pos_raw()),
        (c"'<", &raw const buf.b_visual.vi_start),
        (c"'>", &raw const buf.b_visual.vi_end),
    ];
    for (name, pos) in positions {
        // SAFETY: every position above is a field of the live buffer or of a
        // mark store inside it.
        let _ = unsafe { add_mark(l, name.as_ptr(), pos, handle, ptr::null()) };
    }
}

/// Get information about global marks ('A' to 'Z' and '0' to '9')
///
/// `l` — List to store global marks
///
/// # Safety
/// `l` must be a live list and the editor's globals must be live.
pub unsafe fn get_global_marks(l: *mut list_T) {
    let mut mname: [c_char; 3] = c_bytes(b"' \0");
    for (i, mark) in GlobalMarks::indexed() {
        let fnum = mark.fmark().fnum();
        // A slot whose buffer is loaded reports the buffer's name (allocated
        // here); one that came out of the shada file reports the name it
        // still carries, which belongs to the slot and must not be freed.
        let name = if fnum != 0 {
            buflist_nr2name(fnum, 1, 1)
        } else {
            mark.fname()
        };
        if name.is_null() {
            continue;
        }
        mname[1] = mark_name(if i >= NMARKS {
            i - NMARKS + '0' as c_int
        } else {
            i + 'A' as c_int
        });
        // SAFETY: `mname` and `name` are NUL-terminated and live for the
        // call, and the slot names a live position.
        let _ = unsafe { add_mark(l, mname.as_ptr(), mark.fmark().pos_raw(), fnum, name) };
        if fnum != 0 {
            // SAFETY: `buflist_nr2name` answered an allocation nothing else
            // holds.
            unsafe { xfree(name.cast()) };
        }
    }
}
