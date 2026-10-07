//! Turning an expression into a buffer position.

#![forbid(unsafe_code)]

use crate::winlayer::Buf;
use core::ffi::c_int;

use crate::ascii::ascii_isdigit;
use crate::buffer::find_buf;
use crate::eval::kMarkAll;
use crate::eval::typval::{NumBuf, list_find_nr, list_len};
use crate::mark::mark_lookup;
use crate::mbyte::{char_count, cluster_len};
use crate::memline::{Lines, ml_get_buf_len};
use crate::r#move::{check_cursor_moved, update_topline, validate_botline_win};
use crate::normal::{visual_active, visual_anchor};
use crate::types::{ColNr, Failed, LineNr, Pos, TypVal, VAR_LIST};
use crate::winlayer::Win;

/// How many characters line `lnum` of `buffer` holds.
fn line_char_count(buffer: Buf, lnum: LineNr) -> c_int {
    let count = char_count(Lines::in_buffer(buffer).line(lnum));
    c_int::try_from(count).unwrap_or(c_int::MAX)
}

/// The character index of byte index `byteidx` in a buffer line.
pub fn buf_byteidx_to_charidx(buffer: Option<Buf>, mut lnum: LineNr, byteidx: c_int) -> c_int {
    let Some(buf) = buffer else {
        return -1;
    };
    if buf.b_ml.ml_mfp.is_null() {
        return -1;
    }
    if lnum > buf.line_count() {
        lnum = buf.line_count();
    }
    let mut lines = Lines::in_buffer(buf);
    let line = lines.line(lnum);
    if line.is_empty() {
        return 0;
    }

    // The walk stops at the end of the line or past the byte index,
    // whichever comes first; a memline line holds no NUL of its own, so its
    // end is where the C met the terminator.
    // A negative index is before the line, so the walk does not start.
    let bound = usize::try_from(byteidx).ok();
    let mut t = 0usize;
    let mut count = 0;
    while t < line.len() && bound.is_some_and(|bound| t <= bound) {
        t += cluster_len(&line[t..]);
        count += 1;
    }
    // A byte index exactly at the terminator counts the position past
    // the last character, unless it is index zero on an empty line.
    if t == line.len() && byteidx != 0 && bound == Some(t) {
        count += 1;
    }
    count - 1
}

/// The byte index of character index `charidx` in a buffer line.
pub fn buf_charidx_to_byteidx(buffer: Option<Buf>, mut lnum: LineNr, mut charidx: c_int) -> c_int {
    let Some(buf) = buffer else {
        return -1;
    };
    if buf.b_ml.ml_mfp.is_null() {
        return -1;
    }
    if lnum > buf.line_count() {
        lnum = buf.line_count();
    }
    let mut lines = Lines::in_buffer(buf);
    let line = lines.line(lnum);
    let mut t = 0usize;
    // The decrement is inside the condition, so a `charidx` of 0 or 1
    // both answer byte 0.
    while t < line.len() && {
        charidx -= 1;
        charidx > 0
    } {
        t += cluster_len(&line[t..]);
    }
    c_int::try_from(t).unwrap_or(c_int::MAX)
}

/// Resolve a position expression — a `[lnum, col]` List, `.`, `v`, `'m`,
/// `w0`, `w$` or `$` — against the window `wp`.
///
/// A file mark (`'A`–`'Z`, `'0`–`'9`) stores the number of its buffer in
/// `ret_fnum`; every other form leaves it alone.
pub fn var2fpos(
    tv: &TypVal,
    dollar_lnum: bool,
    ret_fnum: &mut c_int,
    charcol: bool,
    window: Win,
) -> Option<Pos> {
    let mut numbuf = NumBuf::new();
    let wp = window;
    let mut pos = Pos::default();
    let bp = wp.buffer();

    // `[lnum, col]`, `[lnum, col, off]`.
    if tv.v_type() == VAR_LIST {
        let l = tv.list_ref()?;
        let mut error = false;
        pos.lnum = list_find_nr(Some(l), 0, Some(&mut error)) as LineNr;
        if error || pos.lnum <= 0 || pos.lnum > bp.line_count() {
            return None;
        }
        pos.col = list_find_nr(Some(l), 1, Some(&mut error)) as ColNr;
        if error {
            return None;
        }

        let len = if charcol {
            line_char_count(bp, pos.lnum)
        } else {
            ml_get_buf_len(bp, pos.lnum) as c_int
        };
        // The column may be spelled `"$"`, meaning end of line.
        let li = l.items().get(1);
        let dollar = li.is_some_and(|li| li.li_tv.string_bytes() == b"$");
        if dollar {
            pos.col = len + 1;
        }
        if pos.col == 0 || pos.col > len + 1 {
            return None;
        }
        pos.col -= 1;

        pos.coladd = list_find_nr(Some(l), 2, Some(&mut error)) as ColNr;
        if error {
            pos.coladd = 0;
        }
        return Some(pos);
    }

    let name = numbuf.bytes_chk(tv)?;

    // A zero line number is the "nothing matched yet" marker for the
    // three forms below. Past the end of the name reads as its terminator.
    let first = name.first().copied().unwrap_or(0);
    let second = name.get(1).copied().unwrap_or(0);
    if first == b'.' {
        pos = wp.w_cursor;
    } else if first == b'v' && second == 0 {
        // The other end of the Visual selection — but only in the
        // window that owns it.
        if visual_active() && wp.is_current() {
            pos = visual_anchor();
        } else {
            pos = wp.w_cursor;
        }
    } else if first == b'\'' {
        let mname = c_int::from(second);
        let fm = mark_lookup(bp, wp, kMarkAll, mname)?;
        if fm.mark.lnum <= 0 {
            return None;
        }
        pos = fm.mark;
        // Only the file marks carry a buffer of their own.
        if (mname >= b'A' as c_int && mname <= b'Z' as c_int) || ascii_isdigit(mname) {
            *ret_fnum = fm.fnum;
        }
    }

    if pos.lnum != 0 {
        if charcol {
            pos.col = buf_byteidx_to_charidx(Some(bp), pos.lnum, pos.col) as ColNr;
        }
        return Some(pos);
    }

    pos.coladd = 0;
    if first == b'w' && dollar_lnum {
        check_cursor_moved(wp);
        pos.col = 0;
        if second == b'0' {
            update_topline(wp);
            pos.lnum = wp.w_topline.max(1);
            return Some(pos);
        }
        if second == b'$' {
            validate_botline_win(wp);
            pos.lnum = if wp.w_botline > 0 {
                wp.w_botline - 1
            } else {
                0
            };
            return Some(pos);
        }
    } else if first == b'$' {
        // `$` is the last line where a line number is wanted, and the
        // end of the current line where a column is.
        if dollar_lnum {
            pos.lnum = bp.line_count();
            pos.col = 0;
        } else {
            let lnum = wp.w_cursor.lnum;
            pos.lnum = lnum;
            pos.col = if charcol {
                line_char_count(bp, lnum)
            } else {
                ml_get_buf_len(bp, lnum)
            };
        }
        return Some(pos);
    }
    None
}

/// Read a `[lnum, col]` List — optionally with a leading buffer number and
/// a trailing offset and 'curswant' — into `pos`.
///
/// The buffer number is read only when `fnum` is given, and 'curswant' only
/// when `curswant` is; both are written only as far as the List parsed.
pub fn list2fpos(
    arg: &TypVal,
    pos: &mut Pos,
    mut fnum: Option<&mut c_int>,
    curswant: Option<&mut ColNr>,
    charcol: bool,
) -> Result<(), Failed> {
    if arg.v_type() != VAR_LIST {
        return Err(Failed);
    }
    let Some(l) = arg.list_ref() else {
        return Err(Failed);
    };
    // Without a buffer number the List is 2..4 items, with one 3..5.
    let (least, most) = if fnum.is_none() { (2, 4) } else { (3, 5) };
    let n_items = list_len(Some(l));
    if n_items < least || n_items > most {
        return Err(Failed);
    }

    let mut i = 0;
    if let Some(fnum) = fnum.as_deref_mut() {
        // A null `error` means "do not report".
        let mut n = list_find_nr(Some(l), i, None) as c_int;
        i += 1;
        if n < 0 {
            return Err(Failed);
        }
        if n == 0 {
            n = Buf::current().handle as c_int; // buffer 0 is "current"
        }
        *fnum = n;
    }

    let n = list_find_nr(Some(l), i, None) as c_int;
    i += 1;
    if n < 0 {
        return Err(Failed);
    }
    pos.lnum = n as LineNr;

    let mut n = list_find_nr(Some(l), i, None) as c_int;
    i += 1;
    if n < 0 {
        return Err(Failed);
    }
    if charcol {
        let handle = fnum.map_or(Buf::current().handle as c_int, |fnum| *fnum);
        let Some(buf) = find_buf(handle).filter(|b| !b.b_ml.ml_mfp.is_null()) else {
            return Err(Failed);
        };
        let lnum = if pos.lnum == 0 {
            Win::current().w_cursor.lnum
        } else {
            pos.lnum
        };
        n = buf_charidx_to_byteidx(Some(buf), lnum, n) + 1;
    }
    pos.col = n as ColNr;

    // A missing or negative offset is no offset.
    let off = list_find_nr(Some(l), i, None) as c_int;
    pos.coladd = if off < 0 { 0 } else { off as ColNr };

    if let Some(curswant) = curswant {
        *curswant = list_find_nr(Some(l), i + 1, None) as ColNr;
    }
    Ok(())
}
