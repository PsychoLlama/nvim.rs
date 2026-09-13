//! `getmarklist()`, in both its shapes.
//!
//! This is a second, independent surface over the same slots `:marks` prints
//! and `getpos()` reads, and the three do not agree by construction: the
//! column here is 1-based (`getpos()`'s convention), `:marks` prints the
//! internal 0-based one, and a mark that is not set is simply absent rather
//! than reported at line 0.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::buflist_nr2name;
use crate::cstr::c_bytes;
use crate::eval::typval::{tv_dict_alloc, tv_list_alloc};
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
    l: *mut List,
    mname: *const c_char,
    pos: *const Pos,
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
    // `List::push_dict` on.
    let d_held = tv_dict_alloc();
    let d = d_held.as_ptr();
    unsafe { (*l).push_dict(Some(d_held)) };
    let held = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    let lpos = held.as_ptr();
    unsafe { (*lpos).push_number(VarNumber::from(bufnr)) };
    unsafe { (*lpos).push_number(VarNumber::from(pos.lnum)) };
    // 1-BASED, unlike `:marks` and unlike the store. `MAXCOL` — which is
    // what a linewise `'>` carries — is passed through rather than
    // incremented, so it stays recognisable.
    unsafe {
        (*lpos).push_number(VarNumber::from(if pos.col < MAXCOL {
            pos.col + 1
        } else {
            MAXCOL
        }))
    };
    unsafe { (*lpos).push_number(VarNumber::from(pos.coladd)) };
    if unsafe { (*d).add_str(b"mark", mname) }.is_err()
        || unsafe { (*d).add_list(b"pos", Some(held)) }.is_err()
        || (!fname.is_null() && unsafe { (*d).add_str(b"file", fname) }.is_err())
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
pub unsafe fn get_buf_local_marks(buffer: Buf, l: *mut List) {
    let (buf, win, cur) = (buffer, Win::current(), Buf::current());
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
    let positions: [(&core::ffi::CStr, *const Pos); 7] = [
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
pub unsafe fn get_global_marks(l: *mut List) {
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
