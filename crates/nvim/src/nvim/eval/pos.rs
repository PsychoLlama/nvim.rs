//! Turning an expression into a buffer position.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::buflist_findnr;
use crate::src::nvim::eval::typval::{
    tv_get_string_chk, tv_list_find, tv_list_find_nr, tv_list_len,
};
use crate::src::nvim::eval::{FAIL, NUL, OK, kMarkAll};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{VIsual, VIsual_active, curbuf, curwin};
use crate::src::nvim::mark::mark_get;
use crate::src::nvim::mbyte::{mb_charlen, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::r#move::{check_cursor_moved, update_topline, validate_botline_win};
use crate::src::nvim::os::libc::strcmp;
use crate::src::nvim::types::{
    VAR_LIST, VAR_STRING, buf_T, colnr_T, fmark_T, linenr_T, list_T, listitem_T, pos_T, typval_T,
    uint8_t, win_T,
};

/// The character index of byte index `byteidx` in a buffer line.
///
/// # Safety
/// `buf` must be null or valid.
pub unsafe fn buf_byteidx_to_charidx(buf: *mut buf_T, mut lnum: linenr_T, byteidx: c_int) -> c_int {
    unsafe {
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return -1;
        }
        if lnum > (*buf).b_ml.ml_line_count {
            lnum = (*buf).b_ml.ml_line_count;
        }
        let str = ml_get_buf(buf, lnum);
        if *str as c_int == NUL {
            return 0;
        }

        let mut t = str;
        let mut count = 0;
        while *t as c_int != NUL && t <= str.offset(byteidx as isize) {
            t = t.offset(utfc_ptr2len(t) as isize);
            count += 1;
        }
        // A byte index exactly at the terminator counts the position past
        // the last character, unless it is index zero on an empty line.
        if *t as c_int == NUL && byteidx != 0 && t == str.offset(byteidx as isize) {
            count += 1;
        }
        count - 1
    }
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
    unsafe {
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return -1;
        }
        if lnum > (*buf).b_ml.ml_line_count {
            lnum = (*buf).b_ml.ml_line_count;
        }
        let str = ml_get_buf(buf, lnum);
        let mut t = str;
        // The decrement is inside the condition, so a `charidx` of 0 or 1
        // both answer byte 0.
        while *t as c_int != NUL && {
            charidx -= 1;
            charidx > 0
        } {
            t = t.offset(utfc_ptr2len(t) as isize);
        }
        t.offset_from(str) as c_int
    }
}

/// Resolve a position expression — a `[lnum, col]` List, `.`, `v`, `'m`,
/// `w0`, `w$` or `$` — against the window `wp`.
///
/// The answer points at one shared static, so a caller must be done with it
/// before asking again. That is upstream's design and several callers rely
/// on writing through it.
///
/// # Safety
/// `tv`, `ret_fnum` and `wp` must be valid.
pub unsafe fn var2fpos(
    tv: *const typval_T,
    dollar_lnum: bool,
    ret_fnum: *mut c_int,
    charcol: bool,
    wp: *mut win_T,
) -> *mut pos_T {
    /// The one position every answer is written into.
    static POS: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });

    unsafe {
        let pos = POS.ptr();
        let bp: *mut buf_T = (*wp).w_buffer;

        // `[lnum, col]`, `[lnum, col, off]`.
        if (*tv).v_type == VAR_LIST {
            let l: *mut list_T = (*tv).vval.v_list;
            if l.is_null() {
                return null_mut();
            }
            let mut error = false;
            (*pos).lnum = tv_list_find_nr(l, 0, &raw mut error) as linenr_T;
            if error || (*pos).lnum <= 0 || (*pos).lnum > (*bp).b_ml.ml_line_count {
                return null_mut();
            }
            (*pos).col = tv_list_find_nr(l, 1, &raw mut error) as colnr_T;
            if error {
                return null_mut();
            }

            let len = if charcol {
                mb_charlen(ml_get_buf(bp, (*pos).lnum))
            } else {
                ml_get_buf_len(bp, (*pos).lnum) as c_int
            };
            // The column may be spelled `"$"`, meaning end of line.
            let li: *mut listitem_T = tv_list_find(l, 1);
            if !li.is_null()
                && (*li).li_tv.v_type == VAR_STRING
                && !(*li).li_tv.vval.v_string.is_null()
                && strcmp((*li).li_tv.vval.v_string, c"$".as_ptr()) == 0
            {
                (*pos).col = len + 1;
            }
            if (*pos).col == 0 || (*pos).col > len + 1 {
                return null_mut();
            }
            (*pos).col -= 1;

            (*pos).coladd = tv_list_find_nr(l, 2, &raw mut error) as colnr_T;
            if error {
                (*pos).coladd = 0;
            }
            return pos;
        }

        let name = tv_get_string_chk(tv);
        if name.is_null() {
            return null_mut();
        }

        // A zero line number is the "nothing matched yet" marker for the
        // three forms below.
        (*pos).lnum = 0;
        if *name.add(0) == b'.' as c_char {
            POS.set((*wp).w_cursor);
        } else if *name.add(0) == b'v' as c_char && *name.add(1) as c_int == NUL {
            // The other end of the Visual selection — but only in the
            // window that owns it.
            if VIsual_active.get() && wp == curwin.get() {
                POS.set(VIsual.get());
            } else {
                POS.set((*wp).w_cursor);
            }
        } else if *name.add(0) == b'\'' as c_char {
            let mname = *name.add(1) as uint8_t as c_int;
            let fm: *const fmark_T = mark_get(bp, wp, null_mut::<fmark_T>(), kMarkAll, mname);
            if fm.is_null() || (*fm).mark.lnum <= 0 {
                return null_mut();
            }
            POS.set((*fm).mark);
            // Only the file marks carry a buffer of their own.
            if (mname >= b'A' as c_int && mname <= b'Z' as c_int) || ascii_isdigit(mname) {
                *ret_fnum = (*fm).fnum;
            }
        }

        if (*pos).lnum != 0 {
            if charcol {
                (*pos).col = buf_byteidx_to_charidx(bp, (*pos).lnum, (*pos).col) as colnr_T;
            }
            return pos;
        }

        (*pos).coladd = 0;
        if *name.add(0) == b'w' as c_char && dollar_lnum {
            check_cursor_moved(wp);
            (*pos).col = 0;
            if *name.add(1) == b'0' as c_char {
                update_topline(wp);
                (*pos).lnum = (*wp).w_topline.max(1);
                return pos;
            }
            if *name.add(1) == b'$' as c_char {
                validate_botline_win(wp);
                (*pos).lnum = if (*wp).w_botline > 0 {
                    (*wp).w_botline - 1
                } else {
                    0
                };
                return pos;
            }
        } else if *name.add(0) == b'$' as c_char {
            // `$` is the last line where a line number is wanted, and the
            // end of the current line where a column is.
            if dollar_lnum {
                (*pos).lnum = (*bp).b_ml.ml_line_count;
                (*pos).col = 0;
            } else {
                (*pos).lnum = (*wp).w_cursor.lnum;
                (*pos).col = if charcol {
                    mb_charlen(ml_get_buf(bp, (*wp).w_cursor.lnum))
                } else {
                    ml_get_buf_len(bp, (*wp).w_cursor.lnum)
                };
            }
            return pos;
        }
        null_mut()
    }
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
) -> c_int {
    unsafe {
        if (*arg).v_type != VAR_LIST {
            return FAIL;
        }
        let l: *mut list_T = (*arg).vval.v_list;
        if l.is_null() {
            return FAIL;
        }
        // Without a buffer number the List is 2..4 items, with one 3..5.
        let least = if fnump.is_null() { 2 } else { 3 };
        let most = if fnump.is_null() { 4 } else { 5 };
        if tv_list_len(l) < least || tv_list_len(l) > most {
            return FAIL;
        }

        let mut i = 0;
        if !fnump.is_null() {
            let mut n = tv_list_find_nr(l, i, null_mut()) as c_int;
            i += 1;
            if n < 0 {
                return FAIL;
            }
            if n == 0 {
                n = (*curbuf.get()).handle as c_int; // buffer 0 is "current"
            }
            *fnump = n;
        }

        let n = tv_list_find_nr(l, i, null_mut()) as c_int;
        i += 1;
        if n < 0 {
            return FAIL;
        }
        (*posp).lnum = n as linenr_T;

        let mut n = tv_list_find_nr(l, i, null_mut()) as c_int;
        i += 1;
        if n < 0 {
            return FAIL;
        }
        if charcol {
            let buf = buflist_findnr(if fnump.is_null() {
                (*curbuf.get()).handle as c_int
            } else {
                *fnump
            });
            if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
                return FAIL;
            }
            let lnum = if (*posp).lnum == 0 {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*posp).lnum
            };
            n = buf_charidx_to_byteidx(buf, lnum, n) + 1;
        }
        (*posp).col = n as colnr_T;

        // A missing or negative offset is no offset.
        let off = tv_list_find_nr(l, i, null_mut()) as c_int;
        (*posp).coladd = if off < 0 { 0 } else { off as colnr_T };

        if !curswantp.is_null() {
            *curswantp = tv_list_find_nr(l, i + 1, null_mut()) as colnr_T;
        }
        OK
    }
}
