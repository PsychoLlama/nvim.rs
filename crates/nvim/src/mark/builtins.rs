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
use crate::eval::typval::{tv_dict_alloc, tv_list_alloc};
use crate::memory::xfree;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};

use super::store::GlobalMarks;
use super::*;
use crate::pos::MAXCOL;
use crate::types::Failed;
use crate::types::kListLenMayKnow;

/// Add information about mark `mname` to list `l`.
fn add_mark(
    l: &mut List,
    mname: &CStr,
    pos: Pos,
    bufnr: c_int,
    fname: Option<&CStr>,
) -> Result<(), Failed> {
    // An unset mark is omitted rather than reported at line 0: the list is
    // "the marks that exist", which is what makes it usable without a filter.
    if pos.lnum <= 0 {
        return Ok(());
    }
    let d_held = tv_dict_alloc();
    l.push_dict(Some(d_held.clone()));
    let lpos = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    lpos.edit().push_number(VarNumber::from(bufnr));
    lpos.edit().push_number(VarNumber::from(pos.lnum));
    // 1-BASED, unlike `:marks` and unlike the store. `MAXCOL` — which is
    // what a linewise `'>` carries — is passed through rather than
    // incremented, so it stays recognisable.
    lpos.edit()
        .push_number(VarNumber::from(if pos.col < MAXCOL {
            pos.col + 1
        } else {
            MAXCOL
        }));
    lpos.edit().push_number(VarNumber::from(pos.coladd));
    let d = d_held.edit();
    if d.add_str(b"mark", Some(mname)).is_err()
        || d.add_list(b"pos", Some(lpos)).is_err()
        || fname.is_some_and(|fname| d.add_str(b"file", Some(fname)).is_err())
    {
        return Err(Failed);
    }
    Ok(())
}

/// Get information about marks local to a buffer.
///
/// `buffer` — Buffer to get the marks from
/// `l` — List to store marks
pub fn get_buf_local_marks(buffer: Buf, l: &mut List) {
    let (buf, win, cur) = (buffer, Win::current(), Buf::current());
    let handle = buf.handle as c_int;
    let mut mname: [u8; 3] = *b"' \0";
    for i in 0..NMARKS {
        mname[1] = u8::try_from('a' as c_int + i).expect("mark name is one ASCII byte");
        let name = CStr::from_bytes_with_nul(&mname).expect("a two-byte mark name");
        let _ = add_mark(l, name, buf.named_mark(i).pos(), handle, None);
    }
    // The context mark is the WINDOW's and is reported against the CURRENT
    // buffer, which is why it is the one row here that does not use `handle`.
    let _ = add_mark(l, c"''", win.w_pcmark, cur.handle as c_int, None);
    let positions: [(&CStr, Pos); 7] = [
        (c"'\"", buf.last_cursor().pos()),
        (c"'[", buf.b_op_start),
        (c"']", buf.b_op_end),
        (c"'^", buf.last_insert().pos()),
        (c"'.", buf.last_change().pos()),
        (c"'<", buf.b_visual.vi_start),
        (c"'>", buf.b_visual.vi_end),
    ];
    for (name, pos) in positions {
        let _ = add_mark(l, name, pos, handle, None);
    }
}

/// Get information about global marks ('A' to 'Z' and '0' to '9')
///
/// `l` — List to store global marks
pub fn get_global_marks(l: &mut List) {
    let mut mname: [u8; 3] = *b"' \0";
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
        mname[1] = u8::try_from(if i >= NMARKS {
            i - NMARKS + '0' as c_int
        } else {
            i + 'A' as c_int
        })
        .expect("mark name is one ASCII byte");
        let mark_text = CStr::from_bytes_with_nul(&mname).expect("a two-byte mark name");
        // SAFETY: `name` is a NUL-terminated string, the buffer's name just
        // allocated or the slot's own, live for the call.
        let file = unsafe { cstr::at(name) };
        let _ = add_mark(l, mark_text, mark.fmark().pos(), fnum, Some(file));
        if fnum != 0 {
            // SAFETY: `buflist_nr2name` answered an allocation nothing else
            // holds, and `file` is not used past here.
            unsafe { xfree(name.cast()) };
        }
    }
}
