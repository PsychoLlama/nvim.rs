//! Turning an expression into a buffer position.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::eval::Tv;
use crate::eval::window::{cur_buf, cur_win};
use crate::winlayer::{Buf, Live};
use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

use crate::ascii::ascii_isdigit;
use crate::buffer::find_buf;
use crate::eval::kMarkAll;
use crate::eval::typval::{NumBuf, tv_list_find, tv_list_find_nr, tv_list_len};
use crate::mark::mark_get;
use crate::mbyte::{mb_charlen, utfc_ptr2len};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::r#move::{check_cursor_moved, update_topline, validate_botline_win};
use crate::normal::{visual_active, visual_anchor};
use crate::types::{
    Failed, NUL, VAR_LIST, VAR_STRING, buf_T, colnr_T, fmark_T, linenr_T, list_T, listitem_T,
    pos_T, typval_T, uint8_t, win_T,
};
use crate::winlayer::Win;

/// The character index of byte index `byteidx` in a buffer line.
///
/// # Safety
/// `buf` must be null or valid.
pub unsafe fn buf_byteidx_to_charidx(buf: *mut buf_T, mut lnum: linenr_T, byteidx: c_int) -> c_int {
    // SAFETY: the caller's promise -- `buf` is null or a live buffer.
    let Some(buf) = (unsafe { Buf::from_raw(buf) }) else {
        return -1;
    };
    if buf.b_ml.ml_mfp.is_null() {
        return -1;
    }
    if lnum > buf.line_count() {
        lnum = buf.line_count();
    }
    // SAFETY: `lnum` is a line of the buffer, clamped just above.
    let str = unsafe { ml_get_buf(buf.raw(), lnum) };
    // SAFETY: the line is NUL-terminated, so its first byte is readable.
    if unsafe { *str } as c_int == NUL {
        return 0;
    }

    // SAFETY: `byteidx` is a byte index into the line, so the bound is
    // inside it or one past its end.
    let bound = unsafe { str.offset(byteidx as isize) };
    let mut t = str;
    let mut count = 0;
    // SAFETY: `t` walks the NUL-terminated line and stops at the
    // terminator, so every read is inside it.
    while unsafe { *t } as c_int != NUL && t <= bound {
        // SAFETY: `t` is on a character of the line.
        t = unsafe { t.offset(utfc_ptr2len(t) as isize) };
        count += 1;
    }
    // A byte index exactly at the terminator counts the position past
    // the last character, unless it is index zero on an empty line.
    // SAFETY: `t` is inside the line.
    if unsafe { *t } as c_int == NUL && byteidx != 0 && t == bound {
        count += 1;
    }
    count - 1
}

/// The byte index of character index `charidx` in a buffer line.
///
/// # Safety
/// `buf` must be null or valid.
pub unsafe fn buf_charidx_to_byteidx(
    buf: *mut buf_T,
    mut lnum: linenr_T,
    mut charidx: c_int,
) -> c_int {
    // SAFETY: the caller's promise -- `buf` is null or a live buffer.
    let Some(buf) = (unsafe { Buf::from_raw(buf) }) else {
        return -1;
    };
    if buf.b_ml.ml_mfp.is_null() {
        return -1;
    }
    if lnum > buf.line_count() {
        lnum = buf.line_count();
    }
    // SAFETY: `lnum` is a line of the buffer, clamped just above.
    let str = unsafe { ml_get_buf(buf.raw(), lnum) };
    let mut t = str;
    // The decrement is inside the condition, so a `charidx` of 0 or 1
    // both answer byte 0.
    // SAFETY: `t` walks the NUL-terminated line and stops at the
    // terminator.
    while unsafe { *t } as c_int != NUL && {
        charidx -= 1;
        charidx > 0
    } {
        // SAFETY: `t` is on a character of the line.
        t = unsafe { t.offset(utfc_ptr2len(t) as isize) };
    }
    // SAFETY: both cursors are into the one line.
    unsafe { t.offset_from(str) as c_int }
}

/// Resolve a position expression — a `[lnum, col]` List, `.`, `v`, `'m`,
/// `w0`, `w$` or `$` — against the window `wp`.
///
/// # Safety
/// `tv`, `ret_fnum` and `wp` must be valid.
pub unsafe fn var2fpos(
    tv: *const typval_T,
    dollar_lnum: bool,
    ret_fnum: *mut c_int,
    charcol: bool,
    wp: *mut win_T,
) -> Option<pos_T> {
    let mut numbuf = NumBuf::new();
    // The record a `'m` lookup answers into: a motion mark has no store of
    // its own, so it is computed straight into this frame's slot.
    let mut slot = fmark_T::UNSET;
    // SAFETY: the caller's promise -- a live window, and a typval that is
    // only read through here, which is what makes casting its `const` away
    // sound. Nothing below holds either across a call that could close the
    // window: `wp` is the caller's and outlives this frame.
    let (wp, tv) = unsafe { (Win::new(wp), Tv::new(tv.cast_mut())) };
    let mut pos = pos_T::default();
    let bp = wp.buffer();

    // `[lnum, col]`, `[lnum, col, off]`.
    if tv.v_type == VAR_LIST {
        let l: *mut list_T = tv.list_or_null();
        if l.is_null() {
            return None;
        }
        let mut error = false;
        // SAFETY: `l` is a live List and `error` is this frame's.
        pos.lnum = unsafe { tv_list_find_nr(l, 0, &raw mut error) } as linenr_T;
        if error || pos.lnum <= 0 || pos.lnum > bp.line_count() {
            return None;
        }
        // SAFETY: as above.
        pos.col = unsafe { tv_list_find_nr(l, 1, &raw mut error) } as colnr_T;
        if error {
            return None;
        }

        // SAFETY (both arms): `pos.lnum` is a line of the buffer, checked
        // above, and a buffer line is NUL-terminated.
        let len = if charcol {
            unsafe { mb_charlen(ml_get_buf(bp.raw(), pos.lnum)) }
        } else {
            unsafe { ml_get_buf_len(bp.raw(), pos.lnum) as c_int }
        };
        // The column may be spelled `"$"`, meaning end of line.
        // SAFETY: `l` is a live List.
        let li: *mut listitem_T = unsafe { tv_list_find(l, 1) };
        // SAFETY: a non-null item holds a typval, and `VAR_STRING` says
        // `v_string` is its live member.
        let dollar = !li.is_null()
            && unsafe { (*li).li_tv.v_type } == VAR_STRING
            && !unsafe { (*li).li_tv.string_or_null() }.is_null()
            && unsafe { cstr::bytes_at((*li).li_tv.string_or_null()) == b"$" };
        if dollar {
            pos.col = len + 1;
        }
        if pos.col == 0 || pos.col > len + 1 {
            return None;
        }
        pos.col -= 1;

        // SAFETY: `l` is a live List and `error` is this frame's.
        pos.coladd = unsafe { tv_list_find_nr(l, 2, &raw mut error) } as colnr_T;
        if error {
            pos.coladd = 0;
        }
        return Some(pos);
    }

    // SAFETY: `tv` is the caller's typval and `numbuf` outlives the name.
    let name = unsafe { numbuf.string_chk(tv.raw()) };
    if name.is_null() {
        return None;
    }

    // A zero line number is the "nothing matched yet" marker for the
    // three forms below.
    // SAFETY: `name` is NUL-terminated, so its first byte is readable and,
    // while that is not the terminator, so is the second.
    let first = unsafe { *name };
    if first == b'.' as c_char {
        pos = wp.w_cursor;
    } else if first == b'v' as c_char && unsafe { *name.add(1) } as c_int == NUL {
        // The other end of the Visual selection — but only in the
        // window that owns it.
        if visual_active() && wp.is_current() {
            pos = visual_anchor();
        } else {
            pos = wp.w_cursor;
        }
    } else if first == b'\'' as c_char {
        // SAFETY: `first` is not the terminator, so the second byte is
        // still inside the name.
        let mname = unsafe { *name.add(1) } as uint8_t as c_int;
        // SAFETY: the buffer and the window are live, and `slot` is this
        // frame's record.
        let fm: *const fmark_T =
            unsafe { mark_get(bp.raw(), wp.raw(), &raw mut slot, kMarkAll, mname) };
        // SAFETY: a non-null answer is a live record.
        if fm.is_null() || unsafe { (*fm).mark.lnum } <= 0 {
            return None;
        }
        // SAFETY: as above.
        pos = unsafe { (*fm).mark };
        // Only the file marks carry a buffer of their own.
        if (mname >= b'A' as c_int && mname <= b'Z' as c_int) || ascii_isdigit(mname) {
            // SAFETY: `fm` is live and `ret_fnum` is the caller's.
            unsafe { *ret_fnum = (*fm).fnum };
        }
    }

    if pos.lnum != 0 {
        if charcol {
            // SAFETY: the buffer is live.
            pos.col = unsafe { buf_byteidx_to_charidx(bp.raw(), pos.lnum, pos.col) } as colnr_T;
        }
        return Some(pos);
    }

    pos.coladd = 0;
    if first == b'w' as c_char && dollar_lnum {
        check_cursor_moved(wp);
        pos.col = 0;
        // SAFETY: `first` is not the terminator, so the second byte is
        // still inside the name.
        let second = unsafe { *name.add(1) };
        if second == b'0' as c_char {
            update_topline(wp);
            pos.lnum = wp.w_topline.max(1);
            return Some(pos);
        }
        if second == b'$' as c_char {
            validate_botline_win(wp);
            pos.lnum = if wp.w_botline > 0 {
                wp.w_botline - 1
            } else {
                0
            };
            return Some(pos);
        }
    } else if first == b'$' as c_char {
        // `$` is the last line where a line number is wanted, and the
        // end of the current line where a column is.
        if dollar_lnum {
            pos.lnum = bp.line_count();
            pos.col = 0;
        } else {
            let lnum = wp.w_cursor.lnum;
            pos.lnum = lnum;
            // SAFETY (both arms): the cursor is on a line of the buffer.
            pos.col = if charcol {
                unsafe { mb_charlen(ml_get_buf(bp.raw(), lnum)) }
            } else {
                unsafe { ml_get_buf_len(bp.raw(), lnum) }
            };
        }
        return Some(pos);
    }
    None
}

/// Read a `[lnum, col]` List — optionally with a leading buffer number and
/// a trailing offset and 'curswant' — into `posp`.
///
/// # Safety
/// `arg` and `posp` must be valid; `fnump` and `curswantp` null or valid.
pub unsafe fn list2fpos(
    arg: *mut typval_T,
    posp: *mut pos_T,
    fnump: *mut c_int,
    curswantp: *mut colnr_T,
    charcol: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- both outlive the call.
    let (arg, mut posp) = unsafe { (Tv::new(arg), Live::<pos_T>::new(posp)) };
    if arg.v_type != VAR_LIST {
        return Err(Failed);
    }
    let l: *mut list_T = arg.list_or_null();
    if l.is_null() {
        return Err(Failed);
    }
    // Without a buffer number the List is 2..4 items, with one 3..5.
    let least = if fnump.is_null() { 2 } else { 3 };
    let most = if fnump.is_null() { 4 } else { 5 };
    // SAFETY: `l` is a live List.
    let n_items = unsafe { tv_list_len(l) };
    if n_items < least || n_items > most {
        return Err(Failed);
    }

    let mut i = 0;
    if !fnump.is_null() {
        // SAFETY: `l` is a live List; a null `error` means "do not report".
        let mut n = unsafe { tv_list_find_nr(l, i, null_mut()) } as c_int;
        i += 1;
        if n < 0 {
            return Err(Failed);
        }
        if n == 0 {
            n = cur_buf().handle as c_int; // buffer 0 is "current"
        }
        // SAFETY: the caller's promise -- a non-null `fnump` is valid.
        unsafe { *fnump = n };
    }

    // SAFETY: `l` is a live List.
    let n = unsafe { tv_list_find_nr(l, i, null_mut()) } as c_int;
    i += 1;
    if n < 0 {
        return Err(Failed);
    }
    posp.lnum = n as linenr_T;

    // SAFETY: as above.
    let mut n = unsafe { tv_list_find_nr(l, i, null_mut()) } as c_int;
    i += 1;
    if n < 0 {
        return Err(Failed);
    }
    if charcol {
        // SAFETY: the caller's promise -- a non-null `fnump` is valid, and
        // it was written above.
        let handle = if fnump.is_null() {
            cur_buf().handle as c_int
        } else {
            unsafe { *fnump }
        };
        let Some(mut buf) = find_buf(handle).filter(|b| !b.b_ml.ml_mfp.is_null()) else {
            return Err(Failed);
        };
        let lnum = if posp.lnum == 0 {
            cur_win().w_cursor.lnum
        } else {
            posp.lnum
        };
        // SAFETY: `buf` is a live buffer with a memline.
        n = unsafe { buf_charidx_to_byteidx(buf.raw(), lnum, n) } + 1;
    }
    posp.col = n as colnr_T;

    // A missing or negative offset is no offset.
    // SAFETY: `l` is a live List.
    let off = unsafe { tv_list_find_nr(l, i, null_mut()) } as c_int;
    posp.coladd = if off < 0 { 0 } else { off as colnr_T };

    if !curswantp.is_null() {
        // SAFETY: `l` is a live List, and a non-null `curswantp` is valid.
        unsafe { *curswantp = tv_list_find_nr(l, i + 1, null_mut()) as colnr_T };
    }
    Ok(())
}
