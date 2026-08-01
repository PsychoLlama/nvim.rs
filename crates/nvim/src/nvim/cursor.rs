//! Moving and validating a window's cursor position.
//!
//! Everything here works through raw `*mut win_T` / `*mut buf_T` pointers
//! rather than references. Callers interleave these calls with reads of the
//! `curwin`/`curbuf` globals — which alias the same windows — and several of
//! them re-enter through `ml_replace` and the extmark bookkeeping, so a
//! `&mut` here would invalidate a pointer the caller still holds.

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::fold::{hasAnyFolding, hasFolding};
use crate::src::nvim::main::{State, VIsual, VIsual_active, curbuf, curwin, p_sel, restart_edit};
use crate::src::nvim::mark::mark_mb_adjustpos;
use crate::src::nvim::mbyte::{utf_head_off, utf_ptr2StrCharInfo, utf_ptr2char, utfc_next};
use crate::src::nvim::memline::{
    dec, inc, ml_get_buf, ml_get_buf_len, ml_get_buf_mut, ml_get_len, ml_replace,
};
use crate::src::nvim::memory::xmallocz;
use crate::src::nvim::r#move::{
    changed_cline_bef_curs, set_valid_virtcol, validate_virtcol, win_col_off,
};
use crate::src::nvim::option::{get_sidescrolloff_value, get_ve_flags};
use crate::src::nvim::options::{kOptVeFlagAll, kOptVeFlagOnemore};
use crate::src::nvim::os::libc::{memcpy, memset};
use crate::src::nvim::plines::{
    getvcol, getvvcol, init_charsize_arg, linetabsize, linetabsize_eol, win_charsize,
};
use crate::src::nvim::state::{MODE_INSERT, MODE_TERMINAL, virtual_active};
use crate::src::nvim::types::{
    CharsizeArg, buf_T, colnr_T, int64_t, linenr_T, pos_T, size_t, win_T,
};

pub const MAXCOL: c_int = 2147483647;
const NUL: c_int = 0;
const TAB: c_int = 9;
const VALID_VIRTCOL: c_int = 0x4;

/// Virtual column of the cursor, as `getvvcol` reports it (list mode off).
///
/// # Safety
/// The current window must be valid.
pub unsafe fn getviscol() -> colnr_T {
    let win = curwin.get();
    let mut x: colnr_T = 0;
    getvvcol(
        win,
        &raw mut (*win).w_cursor,
        &raw mut x,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    x
}

/// Like [`getviscol`], but for an arbitrary position in the cursor's line.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn getviscol2(col: colnr_T, coladd: colnr_T) -> colnr_T {
    let mut pos = pos_T {
        lnum: (*curwin.get()).w_cursor.lnum,
        col,
        coladd,
    };
    let mut x: colnr_T = 0;
    getvvcol(
        curwin.get(),
        &raw mut pos,
        &raw mut x,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    x
}

/// Move the cursor to virtual column `wcol`, inserting the spaces needed to
/// land there exactly. Answers whether the column was reached.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn coladvance_force(wcol: colnr_T) -> bool {
    let win = curwin.get();
    let reached = coladvance2(win, &raw mut (*win).w_cursor, true, false, wcol);
    if wcol == MAXCOL {
        (*win).w_valid &= !VALID_VIRTCOL;
    } else {
        set_valid_virtcol(win, wcol);
    }
    reached
}

/// Move `wp`'s cursor to virtual column `wcol`, or as close as the line
/// allows. Answers whether the column was reached.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn coladvance(wp: *mut win_T, wcol: colnr_T) -> bool {
    let reached = getvpos(wp, &raw mut (*wp).w_cursor, wcol);
    // The cached virtual column is only good if the cursor did not land on a
    // tab, whose width depends on where it starts rather than on `wcol`.
    if wcol == MAXCOL || !reached {
        (*wp).w_valid &= !VALID_VIRTCOL;
    } else if *ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize)
        as c_int
        != TAB
    {
        set_valid_virtcol(curwin.get(), wcol);
    }
    reached
}

/// The shared body of [`coladvance`] and [`coladvance_force`].
///
/// `addspaces` fills the gap with real spaces when 'virtualedit' put the
/// cursor past the end of the line or inside a tab; `finetune` lets the
/// cursor stop part-way into a wide character. Answers whether `wcol_arg`
/// was reached.
unsafe fn coladvance2(
    wp: *mut win_T,
    pos: *mut pos_T,
    addspaces: bool,
    finetune: bool,
    wcol_arg: colnr_T,
) -> bool {
    // Inserting the spaces edits the buffer, which only the current window
    // may do.
    assert!(
        wp == curwin.get() || !addspaces,
        "wp == curwin || !addspaces"
    );
    let mut wcol = wcol_arg;
    let one_more = State.get() & MODE_INSERT != 0
        || State.get() & MODE_TERMINAL != 0
        || restart_edit.get() != NUL
        || (VIsual_active.get() && *p_sel.get() != b'o' as c_char)
        || (get_ve_flags(wp) & kOptVeFlagOnemore != 0 && wcol < MAXCOL);
    let line = ml_get_buf((*wp).w_buffer, (*pos).lnum);
    let linelen = ml_get_buf_len((*wp).w_buffer, (*pos).lnum);

    let mut idx;
    let mut col: colnr_T = 0;
    let mut csize: c_int = 0;
    let mut head: c_int = 0;

    // MAXCOL is i32::MAX, so '>=' in the C was an equality test.
    if wcol == MAXCOL {
        idx = linelen - 1 + one_more as c_int;
        col = wcol;
        if (addspaces || finetune) && !VIsual_active.get() {
            (*wp).w_curswant = linetabsize(wp, (*pos).lnum) + one_more as c_int;
            if (*wp).w_curswant > 0 {
                (*wp).w_curswant -= 1;
            }
        }
    } else {
        let width = (*wp).w_view_width - win_col_off(wp);
        if finetune
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && (*wp).w_view_width != 0
            && wcol >= width
            && width > 0
        {
            // With 'wrap', a column past this line's last screen line means
            // "the end of that screen line" rather than "past the line".
            csize = linetabsize_eol(wp, (*pos).lnum);
            if csize > 0 {
                csize -= 1;
            }
            if wcol / width > csize / width && (State.get() & MODE_INSERT == 0 || wcol > csize + 1)
            {
                wcol = (csize / width + 1) * width - 1;
            }
        }

        let mut csarg: CharsizeArg = ::core::mem::zeroed();
        let cstype = init_charsize_arg(&mut csarg, wp, (*pos).lnum, line);
        let mut ci = utf_ptr2StrCharInfo(line);
        while col <= wcol && *ci.ptr != 0 {
            let cs = win_charsize(cstype, col, ci.ptr, ci.chr.value, &mut csarg);
            csize = cs.width;
            head = cs.head;
            col += cs.width;
            ci = utfc_next(ci);
        }
        idx = ci.ptr.addr().wrapping_sub(line.addr()) as c_int;
        // The loop stepped one character too far, unless it stopped on the
        // NUL and the cursor is allowed to rest there.
        if col > wcol || (!virtual_active(wp) && !one_more) {
            idx -= 1;
            csize -= head;
            col -= csize;
        }

        if virtual_active(wp)
            && addspaces
            && wcol >= 0
            && ((col != wcol && col != wcol + 1) || csize > 1)
        {
            if *line.offset(idx as isize) == 0 {
                // Past the end of the line: pad it out with spaces.
                let correct = wcol - col;
                assert!(idx + correct >= 0, "STRICT_ADD overflow");
                let newline = xmallocz((idx + correct) as size_t) as *mut c_char;
                memcpy(newline as *mut c_void, line as *const c_void, idx as size_t);
                memset(
                    newline.offset(idx as isize) as *mut c_void,
                    ' ' as c_int,
                    correct as size_t,
                );
                ml_replace((*pos).lnum, newline, false);
                inserted_bytes((*pos).lnum, idx, 0, correct);
                idx += correct;
                col = wcol;
            } else {
                // Inside a wide character (a tab, normally): replace it with
                // the spaces it occupied and land among them.
                let correct = wcol - col - csize + 1;
                if -correct > csize {
                    return false;
                }
                assert!(linelen - 1 + csize >= 0, "STRICT_ADD overflow");
                let newline = xmallocz((linelen - 1 + csize) as size_t) as *mut c_char;
                memcpy(newline as *mut c_void, line as *const c_void, idx as size_t);
                memset(
                    newline.offset(idx as isize) as *mut c_void,
                    ' ' as c_int,
                    csize as size_t,
                );
                assert!(linelen - idx >= 1, "STRICT_SUB overflow");
                memcpy(
                    newline.offset(idx as isize).offset(csize as isize) as *mut c_void,
                    line.offset(idx as isize).offset(1) as *const c_void,
                    (linelen - idx - 1) as size_t,
                );
                ml_replace((*pos).lnum, newline, false);
                inserted_bytes((*pos).lnum, idx, 1, csize);
                idx += csize - 1 + correct;
                col += correct;
            }
        }
    }

    (*pos).col = idx.max(0);
    (*pos).coladd = 0;

    if finetune {
        if wcol == MAXCOL {
            // The cursor is at the end of the line and may not sit on the NUL,
            // so `coladd` spans the last character instead.
            if !one_more {
                let mut scol: colnr_T = 0;
                let mut ecol: colnr_T = 0;
                getvcol(wp, pos, &raw mut scol, ptr::null_mut(), &raw mut ecol);
                (*pos).coladd = ecol - scol;
            }
        } else {
            let b = wcol - col;
            // The upper bound rejects the absurd columns 'virtualedit' allows.
            if b > 0 && b < MAXCOL - 2 * (*wp).w_view_width {
                (*pos).coladd = b;
            }
            col += b;
        }
    }

    mark_mb_adjustpos((*wp).w_buffer, pos);
    wcol >= 0 && col >= wcol
}

/// Set `pos` to the position at virtual column `wcol` in its own line,
/// without editing the buffer. Answers whether the column was reached.
///
/// # Safety
/// `wp` must be a valid window and `pos` a position in its buffer.
pub unsafe fn getvpos(wp: *mut win_T, pos: *mut pos_T, wcol: colnr_T) -> bool {
    coladvance2(wp, pos, false, virtual_active(wp), wcol)
}

/// Move the cursor one character forward; see `inc`.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn inc_cursor() -> c_int {
    inc(&raw mut (*curwin.get()).w_cursor)
}

/// Move the cursor one character back; see `dec`.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn dec_cursor() -> c_int {
    dec(&raw mut (*curwin.get()).w_cursor)
}

/// How far `lnum` is from the cursor, counting each closed fold in between
/// as a single line.
///
/// # Safety
/// `wp` must be a valid window and `lnum` a line in its buffer.
pub unsafe fn get_cursor_rel_lnum(wp: *mut win_T, lnum: linenr_T) -> linenr_T {
    let cursor = (*wp).w_cursor.lnum;
    if lnum == cursor || hasAnyFolding(wp) == 0 {
        return lnum - cursor;
    }
    let mut from_line = lnum.min(cursor);
    let to_line = lnum.max(cursor);
    let mut retval: linenr_T = 0;
    while from_line < to_line {
        // Step to the last line of the fold `from_line` is in, if any.
        hasFolding(wp, from_line, ptr::null_mut(), &raw mut from_line);
        from_line += 1;
        retval += 1;
    }
    // The last fold reached past `to_line`, so that step was not a whole one.
    if from_line > to_line {
        retval -= 1;
    }
    if lnum < cursor { -retval } else { retval }
}

/// Clamp `pos` to a line and column that exist in `buf`.
///
/// # Safety
/// `buf` must be a valid buffer.
pub unsafe fn check_pos(buf: *mut buf_T, pos: *mut pos_T) {
    (*pos).lnum = (*pos).lnum.min((*buf).b_ml.ml_line_count);
    if (*pos).col > 0 {
        (*pos).col = (*pos).col.min(ml_get_buf_len(buf, (*pos).lnum));
    }
}

/// Clamp the cursor's line number to the buffer, preferring the start of a
/// closed fold over a line inside it.
///
/// # Safety
/// `win` must be a valid window.
pub unsafe fn check_cursor_lnum(win: *mut win_T) {
    let buf = (*win).w_buffer;
    if (*win).w_cursor.lnum > (*buf).b_ml.ml_line_count
        && !hasFolding(
            win,
            (*buf).b_ml.ml_line_count,
            &raw mut (*win).w_cursor.lnum,
            ptr::null_mut(),
        )
    {
        (*win).w_cursor.lnum = (*buf).b_ml.ml_line_count;
    }
    if (*win).w_cursor.lnum <= 0 {
        (*win).w_cursor.lnum = 1;
    }
}

/// Clamp the cursor's column to the current line, honouring the modes and
/// 'virtualedit' settings that allow it one position past the last character.
///
/// # Safety
/// `win` must be a valid window.
pub unsafe fn check_cursor_col(win: *mut win_T) {
    let oldcol = (*win).w_cursor.col;
    let oldcoladd = (*win).w_cursor.col + (*win).w_cursor.coladd;
    let cur_ve_flags = get_ve_flags(win);
    let len = ml_get_buf_len((*win).w_buffer, (*win).w_cursor.lnum);
    if len == 0 {
        (*win).w_cursor.col = 0;
    } else if (*win).w_cursor.col >= len {
        let may_rest_on_nul = State.get() & MODE_INSERT != 0
            || restart_edit.get() != 0
            || State.get() & MODE_TERMINAL != 0
            || (VIsual_active.get() && *p_sel.get() != b'o' as c_char)
            || cur_ve_flags & kOptVeFlagOnemore != 0
            || virtual_active(win);
        if may_rest_on_nul {
            (*win).w_cursor.col = len;
        } else {
            (*win).w_cursor.col = len - 1;
            mark_mb_adjustpos((*win).w_buffer, &raw mut (*win).w_cursor);
        }
    } else if (*win).w_cursor.col < 0 {
        (*win).w_cursor.col = 0;
    }

    if oldcol == MAXCOL {
        (*win).w_cursor.coladd = 0;
    } else if cur_ve_flags == kOptVeFlagAll {
        if oldcoladd > (*win).w_cursor.col {
            (*win).w_cursor.coladd = oldcoladd - (*win).w_cursor.col;
            // Don't let the cursor point past the character it is inside.
            if (*win).w_cursor.col + 1 < len {
                assert!((*win).w_cursor.coladd > 0, "win->w_cursor.coladd > 0");
                let mut cs: colnr_T = 0;
                let mut ce: colnr_T = 0;
                getvcol(
                    win,
                    &raw mut (*win).w_cursor,
                    &raw mut cs,
                    ptr::null_mut(),
                    &raw mut ce,
                );
                (*win).w_cursor.coladd = (*win).w_cursor.coladd.min(ce - cs);
            }
        } else {
            (*win).w_cursor.coladd = 0;
        }
    }
}

/// Clamp the cursor to a position that exists in `wp`'s buffer.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn check_cursor(wp: *mut win_T) {
    check_cursor_lnum(wp);
    check_cursor_col(wp);
}

/// Clamp the start of the Visual area to the current buffer.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn check_visual_pos() {
    let visual = VIsual.ptr();
    if (*visual).lnum > (*curbuf.get()).b_ml.ml_line_count {
        (*visual).lnum = (*curbuf.get()).b_ml.ml_line_count;
        (*visual).col = 0;
        (*visual).coladd = 0;
    } else {
        let len = ml_get_len((*visual).lnum);
        if (*visual).col > len {
            (*visual).col = len;
            (*visual).coladd = 0;
        }
    }
}

/// Step the cursor back off the NUL at the end of the line, where Normal
/// mode does not let it rest.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn adjust_cursor_col() {
    if (*curwin.get()).w_cursor.col > 0
        && (!VIsual_active.get() || *p_sel.get() == b'o' as c_char)
        && gchar_cursor() == NUL
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
}

/// Scroll the current window horizontally to `leftcol`, pulling the cursor
/// along if 'sidescrolloff' demands it. Answers whether the cursor moved.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn set_leftcol(leftcol: colnr_T) -> bool {
    let win = curwin.get();
    if (*win).w_leftcol == leftcol {
        return false;
    }
    (*win).w_leftcol = leftcol;
    changed_cline_bef_curs(win);
    let lastcol = ((*win).w_leftcol + (*win).w_view_width - win_col_off(win) - 1) as int64_t;
    validate_virtcol(win);

    let mut moved = false;
    let siso = get_sidescrolloff_value(win);
    if (*win).w_virtcol > (lastcol - siso) as colnr_T {
        moved = true;
        coladvance(win, (lastcol - siso) as colnr_T);
    } else if ((*win).w_virtcol as int64_t) < (*win).w_leftcol as int64_t + siso {
        moved = true;
        coladvance(win, ((*win).w_leftcol as int64_t + siso) as colnr_T);
    }

    // A wide character straddling either edge is not fully visible; step the
    // cursor off it.
    let mut s: colnr_T = 0;
    let mut e: colnr_T = 0;
    getvvcol(
        win,
        &raw mut (*win).w_cursor,
        &raw mut s,
        ptr::null_mut(),
        &raw mut e,
    );
    if e > lastcol as colnr_T {
        moved = true;
        coladvance(win, s - 1);
    } else if s < (*win).w_leftcol {
        moved = true;
        if !coladvance(win, e + 1) {
            // There is nothing to move onto; keep the character visible by
            // scrolling to it instead.
            (*win).w_leftcol = s;
            changed_cline_bef_curs(win);
        }
    }

    if moved {
        (*win).w_set_curswant = 1;
    }
    redraw_later(win, UPD_NOT_VALID);
    moved
}

/// The character under the cursor.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn gchar_cursor() -> c_int {
    utf_ptr2char(get_cursor_pos_ptr())
}

/// The character before the cursor, or -1 at the start of the line.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn char_before_cursor() -> c_int {
    if (*curwin.get()).w_cursor.col == 0 {
        return -1;
    }
    let line = get_cursor_line_ptr();
    let p = line.offset((*curwin.get()).w_cursor.col as isize);
    let prev_len = utf_head_off(line, p.offset(-1)) + 1;
    utf_ptr2char(p.offset(-(prev_len as isize)))
}

/// Overwrite the byte under the cursor.
///
/// # Safety
/// The current window and buffer must be valid, and the cursor's column must
/// lie within the line.
pub unsafe fn pchar_cursor(c: c_char) {
    *ml_get_buf_mut(curbuf.get(), (*curwin.get()).w_cursor.lnum)
        .offset((*curwin.get()).w_cursor.col as isize) = c;
}

/// The cursor's line.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn get_cursor_line_ptr() -> *mut c_char {
    ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum)
}

/// The cursor's line, from the cursor onwards.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn get_cursor_pos_ptr() -> *mut c_char {
    ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum)
        .offset((*curwin.get()).w_cursor.col as isize)
}

/// The length of the cursor's line.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn get_cursor_line_len() -> colnr_T {
    ml_get_buf_len(curbuf.get(), (*curwin.get()).w_cursor.lnum)
}

/// The number of bytes from the cursor to the end of its line.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn get_cursor_pos_len() -> colnr_T {
    ml_get_buf_len(curbuf.get(), (*curwin.get()).w_cursor.lnum) - (*curwin.get()).w_cursor.col
}
