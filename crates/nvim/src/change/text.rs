//! Inserting and deleting text, in bytes, characters and lines.
//!
//! The primitives every operator eventually calls. [`ins_char_bytes`] is the
//! one with the substance: it is also Replace mode's insert, so it pushes the
//! bytes it overwrites onto the replace stack, and under Virtual Replace it
//! has to count *cells* rather than bytes, which can consume several
//! characters or none at all. [`del_bytes`] is its mirror and carries the
//! `fixpos`/'virtualedit' question of where the cursor lands when the last
//! character of a line goes away, plus 'delcombine'. [`truncate_line`] and
//! [`del_lines`] are the line-level pair.
//!
//! Every one of these ends in [`inserted_bytes`], which is what turns the edit
//! into an extmark splice and a buffer-update event.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::memline::MlFlags;
use crate::siemsg;
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, Failed, NUL};
use crate::winlayer::{Buf, Win};

/// `memmove` between two places inside this module's own line buffers.
///
/// Every call below moves bytes within, or between, an allocation this file
/// made and sized for them, which is the promise the one region here pays.
fn move_bytes(dst: *mut c_char, src: *const c_char, n: size_t) {
    // SAFETY: the caller sized `dst` for `n` bytes read from `src`; the two
    // may overlap, which is what `memmove` is for.
    unsafe { memmove(dst.cast::<c_void>(), src.cast::<c_void>(), n) };
}

/// The line the cursor is on, and its length.
fn cursor_line() -> (*mut c_char, colnr_T) {
    let lnum = cur_win().w_cursor.lnum;
    // SAFETY: the cursor is on a valid line of the current buffer.
    (ml_get(lnum), ml_get_len(lnum))
}

/// Insert the NUL-terminated string `p` at the cursor.
///
/// # Safety
/// `p` must be NUL-terminated. The caller must have prepared for undo.
pub unsafe fn ins_bytes(p: *mut c_char) {
    unsafe { ins_bytes_len(p, cstr::bytes_at(p).len()) };
}

/// Insert `len` bytes of `p` at the cursor, one character at a time.
///
/// # Safety
/// `p` must point to at least `len` readable bytes. The caller must have
/// prepared for undo.
pub unsafe fn ins_bytes_len(p: *mut c_char, len: size_t) {
    let mut i: size_t = 0;
    while i < len {
        // The `_len` form so that a truncated sequence at the end does not
        // read past `p[len]`.
        let n = unsafe { utfc_ptr2len_len(p.add(i), len.wrapping_sub(i) as c_int) } as size_t;
        unsafe { ins_char_bytes(p.add(i), n) };
        i = i.wrapping_add(n);
    }
}

/// Insert or replace the single character `c` at the cursor.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn ins_char(c: c_int) {
    let mut buf: [c_char; 7] = [0; 7];
    let n = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) } as size_t;
    // `c` being 0x100, 0x200, ... would encode to a leading NUL byte, which
    // must not go into the line; CTRL-V u9900 reaches this.
    if buf[0] == 0 {
        buf[0] = b'\n' as c_char;
    }
    unsafe { ins_char_bytes(buf.as_mut_ptr(), n) };
}

/// How many bytes at the cursor Virtual Replace mode has to consume to make
/// room for `buf`, and how many bytes the new text takes once padded.
///
/// Each typed character replaces one or more existing ones, or none at all for
/// a TAB, because what is being matched is screen cells. Overshooting the
/// column the new character ends at means the difference has to be filled with
/// spaces, which is what the returned `newlen` carries.
///
/// # Safety
/// `oldp` must be the current line and `col` a byte offset into it; `buf` must
/// hold the character about to be inserted.
unsafe fn vreplace_extent(
    buf: *mut c_char,
    oldp: *mut c_char,
    col: size_t,
    charlen: size_t,
) -> (size_t, size_t) {
    // Disable 'list' while measuring, unless 'cpo' has the `L` flag: it
    // changes how wide a TAB looks.
    let old_list = cur_win().w_onebuf_opt.wo_list;
    if old_list != 0 && !cpo_has(CpoFlag::LISTWM) {
        cur_win().w_onebuf_opt.wo_list = false as c_int;
    }

    let mut oldlen: size_t = 0;
    let mut newlen: size_t = charlen;
    let mut vcol: colnr_T = 0;
    let win = curwin.get();
    let cursor = cur_win().cursor().raw();
    let novcol = ::core::ptr::null_mut::<colnr_T>();
    // SAFETY: the current window and its own cursor; only the middle column
    // is asked for, and it is a local.
    unsafe { getvcol(Win::new(win), cursor, novcol, &raw mut vcol, novcol) };
    // SAFETY: the current window is live and `buf` holds the character.
    let new_vcol = vcol + unsafe { win_chartabsize(Win::new(win), buf, vcol) };
    // The byte `n` past the insertion point; `oldp` is NUL-terminated and the
    // walk stops at that NUL.
    let at = |n: size_t| oldp.wrapping_add(col).wrapping_add(n);
    // SAFETY: `at(oldlen)` is inside the current line, in all three calls.
    while c_int::from(unsafe { *at(oldlen) }) != NUL && vcol < new_vcol {
        vcol += unsafe { win_chartabsize(Win::new(win), at(oldlen), vcol) };
        // A TAB that lands exactly where the new character ends does not
        // need removing.
        if vcol > new_vcol && c_int::from(unsafe { *at(oldlen) }) == TAB {
            break;
        }
        oldlen += unsafe { utfc_ptr2len(at(oldlen)) } as size_t;
        // Took off a bit too much: pad with spaces.
        if vcol > new_vcol {
            newlen = newlen.wrapping_add((vcol - new_vcol) as size_t);
        }
    }

    cur_win().w_onebuf_opt.wo_list = old_list;
    (oldlen, newlen)
}

/// Insert `charlen` bytes of `buf` -- one whole character -- at the cursor,
/// replacing what is there in Replace and Virtual Replace modes.
///
/// The caller has already turned bytes into a character; this only ever writes
/// one.
///
/// # Safety
/// `buf` must point to at least `charlen` readable bytes. The caller must have
/// prepared for undo.
pub unsafe fn ins_char_bytes(buf: *mut c_char, charlen: size_t) {
    // Break tabs if needed.
    if virtual_active(cur_win()) && cur_win().w_cursor.coladd > 0 {
        unsafe { coladvance_force(getviscol()) };
    }

    let col = cur_win().w_cursor.col as size_t;
    let lnum = cur_win().w_cursor.lnum;
    let (oldp, line_bytes) = cursor_line();
    let linelen = (line_bytes as size_t).wrapping_add(1); // includes the NUL

    // The defaults are the values for when not replacing: nothing deleted,
    // the whole character inserted.
    let mut oldlen: size_t = 0;
    let mut newlen: size_t = charlen;

    if State.get() & REPLACE_FLAG != 0 {
        if State.get() & VREPLACE_FLAG != 0 {
            (oldlen, newlen) = unsafe { vreplace_extent(buf, oldp, col, charlen) };
        } else if c_int::from(unsafe { *oldp.add(col) }) != NUL {
            oldlen = unsafe { utfc_ptr2len(oldp.add(col)) } as size_t;
        }
        // Push the replaced bytes onto the replace stack so BS can put them
        // back. A multi-byte character goes on the other way around, so
        // that its first byte -- which carries the length -- pops first.
        unsafe { replace_push_nul() };
        unsafe { replace_push(oldp.add(col), oldlen) };
    }

    let bytes = linelen.wrapping_add(newlen).wrapping_sub(oldlen);
    // SAFETY: `xmalloc` aborts rather than answer null.
    let newp = unsafe { xmalloc(bytes) }.cast::<c_char>();

    if col > 0 {
        move_bytes(newp, oldp, col);
    }
    let p = newp.wrapping_add(col);
    if linelen > col.wrapping_add(oldlen) {
        let tail = linelen.wrapping_sub(col).wrapping_sub(oldlen);
        let from = oldp.wrapping_add(col).wrapping_add(oldlen);
        move_bytes(p.wrapping_add(newlen), from, tail);
    }
    move_bytes(p, buf, charlen);
    // Fill the rest with spaces when Virtual Replace took off too much.
    for i in charlen..newlen {
        // SAFETY: `newp` was sized for `newlen` bytes at `col`.
        unsafe { *p.add(i) = b' ' as c_char };
    }

    // SAFETY: `newp` is our own NUL-terminated line, which the buffer takes
    // over, and `lnum` is the cursor line.
    let _ = unsafe { ml_replace(lnum, newp, false) };
    unsafe { inserted_bytes(lnum, col as colnr_T, oldlen as c_int, newlen as c_int) };

    // In Insert or Replace mode with 'showmatch', briefly show the match
    // for a closing bracket.
    if p_sm.get() != 0
        && State.get() & MODE_INSERT != 0
        && msg_silent.get() == 0
        && !ins_compl_active()
    {
        unsafe { showmatch(utf_ptr2char(buf)) };
    }

    if p_ri.get() == 0 || State.get() & REPLACE_FLAG != 0 {
        // Normal insert: move the cursor right.
        cur_win().w_cursor.col += charlen as colnr_T;
    }
}

/// Insert `slen` bytes of `s` at the cursor.
///
/// Unlike [`ins_char_bytes`] this does *not* handle Replace mode.
///
/// # Safety
/// `s` must point to at least `slen` readable bytes. The caller must have
/// prepared for undo.
pub unsafe fn ins_str(s: *mut c_char, slen: size_t) {
    let lnum = cur_win().w_cursor.lnum;

    if virtual_active(cur_win()) && cur_win().w_cursor.coladd > 0 {
        unsafe { coladvance_force(getviscol()) };
    }

    let col = cur_win().w_cursor.col;
    let (oldp, oldlen) = cursor_line();

    // SAFETY: `xmalloc` aborts rather than answer null.
    let newp = unsafe { xmalloc((oldlen as size_t).wrapping_add(slen).wrapping_add(1)) };
    let newp = newp.cast::<c_char>();
    let at = newp.wrapping_offset(col as isize);
    if col > 0 {
        move_bytes(newp, oldp, col as size_t);
    }
    move_bytes(at, s, slen);
    // The tail, including the NUL. The cursor is inside the line, so this
    // is never negative.
    let bytes = oldlen - col + 1;
    debug_assert!(bytes >= 0);
    let tail = oldp.wrapping_offset(col as isize);
    move_bytes(at.wrapping_add(slen), tail, bytes as size_t);
    // SAFETY: `newp` is our own NUL-terminated line, which the buffer takes
    // over, and `lnum` is the cursor line.
    let _ = unsafe { ml_replace(lnum, newp, false) };
    unsafe { inserted_bytes(lnum, col, 0, slen as c_int) };
    cur_win().w_cursor.col += slen as colnr_T;
}

/// Delete the character under the cursor.
///
/// With `fixpos`, don't leave the cursor on the NUL past the end of the line.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn del_char(fixpos: bool) -> Result<(), Failed> {
    // Make sure the cursor is at the start of a character.
    unsafe { mb_adjust_cursor() };
    if c_int::from(unsafe { *get_cursor_pos_ptr() }) == NUL {
        return Err(Failed);
    }
    unsafe { del_chars(1, fixpos as c_int) }
}

/// [`del_bytes`] counted in characters rather than bytes.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn del_chars(count: c_int, fixpos: c_int) -> Result<(), Failed> {
    let mut bytes = 0;
    let mut p = get_cursor_pos_ptr();
    let mut i = 0;
    while i < count && c_int::from(unsafe { *p }) != NUL {
        let l = unsafe { utfc_ptr2len(p) };
        bytes += l;
        p = unsafe { p.offset(l as isize) };
        i += 1;
    }
    unsafe { del_bytes(bytes, fixpos != 0, true) }
}

/// Delete `count` bytes at the cursor.
///
/// With `fixpos_arg`, don't leave the cursor on the NUL past the end of the
/// line; with `use_delcombine`, 'delcombine' applies, so that deleting a
/// character that carries combining marks takes only the last mark.
///
/// Answers `Err` on the NUL past the end of the line or for a negative
/// `count`, `Ok` otherwise.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn del_bytes(
    mut count: colnr_T,
    fixpos_arg: bool,
    use_delcombine: bool,
) -> Result<(), Failed> {
    let lnum = cur_win().w_cursor.lnum;
    let mut col = cur_win().w_cursor.col;
    let mut fixpos = fixpos_arg;
    let (oldp, oldlen) = cursor_line();

    // Nothing to do on the NUL after the line.
    if col >= oldlen {
        return Err(Failed);
    }
    if count == 0 {
        return Ok(());
    }
    if count < 1 {
        siemsg!(
            "E292: Invalid count for del_bytes(): {}",
            int64_t::from(count)
        );
        return Err(Failed);
    }

    // With 'delcombine', deleting (less than) one character takes only the
    // last combining character off it -- and then the cursor must not move,
    // because the base character is still there.
    if p_deco.get() != 0
        && use_delcombine
        && unsafe { utfc_ptr2len(oldp.offset(col as isize)) } >= count
    {
        let p0 = unsafe { oldp.offset(col as isize) };
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        if unsafe { utf_composinglike(p0, p0.offset(utf_ptr2len(p0) as isize), &raw mut state) } {
            // Walk to the last composing character; there can be several.
            let mut n = col;
            loop {
                col = n;
                count = unsafe { utf_ptr2len(oldp.offset(n as isize)) };
                n += count;
                if !unsafe {
                    utf_composinglike(
                        oldp.offset(col as isize),
                        oldp.offset(n as isize),
                        &raw mut state,
                    )
                } {
                    break;
                }
            }
            fixpos = false;
        }
    }

    // What is left to move up, including the trailing NUL.
    let mut movelen = oldlen - col - count + 1;
    if movelen <= 1 {
        // The count reached the end of the line, so clamp it. Taking off
        // the last character of a non-blank line would leave the cursor on
        // the NUL, which `fixpos` forbids unless Insert mode is about to
        // restart or 'virtualedit' contains "onemore".
        if col > 0
            && fixpos
            && restart_edit.get() == 0
            && get_ve_flags(cur_win()) & kOptVeFlagOnemore as ::core::ffi::c_uint == 0
        {
            cur_win().w_cursor.col -= 1;
            cur_win().w_cursor.coladd = 0;
            cur_win().w_cursor.col -=
                unsafe { utf_head_off(oldp, oldp.offset(cur_win().w_cursor.col as isize)) };
        }
        count = oldlen - col;
        movelen = 1;
    }
    let newlen = oldlen - count;

    // An already-allocated line can be edited in place; one that is still
    // memory-mapped has to be copied.
    let alloc_newp = !unsafe { ml_line_alloced() };
    let newp = if alloc_newp {
        // SAFETY: `xmallocz` aborts rather than answer null.
        let newp = unsafe { xmallocz(newlen as size_t) }.cast::<c_char>();
        move_bytes(newp, oldp, col as size_t);
        newp
    } else {
        unsafe { ml_add_deleted_len(cur_buf().b_ml.cached_text(), oldlen as ssize_t) };
        oldp
    };
    let from = oldp
        .wrapping_offset(col as isize)
        .wrapping_offset(count as isize);
    move_bytes(newp.wrapping_offset(col as isize), from, movelen as size_t);
    if alloc_newp {
        let _ = unsafe { ml_replace(lnum, newp, false) };
    } else {
        cur_buf().b_ml.set_cached_len(newlen + 1);
    }

    unsafe { inserted_bytes(lnum, col, count, 0) };
    Ok(())
}

/// Delete everything on the cursor line from the cursor onwards.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn truncate_line(fixpos: c_int) {
    let lnum = cur_win().w_cursor.lnum;
    let col = cur_win().w_cursor.col;
    let (old_line, old_len) = cursor_line();
    // SAFETY: a static empty string, or the cursor line's first `col` bytes.
    let newp = unsafe {
        if col == 0 {
            xstrdup(c"".as_ptr())
        } else {
            xstrnsave(old_line, col as size_t)
        }
    };
    let deleted = old_len - col;

    // SAFETY: `newp` is our own NUL-terminated line, which the buffer takes
    // over, and `lnum` is the cursor line.
    let _ = unsafe { ml_replace(lnum, newp, false) };
    unsafe { inserted_bytes(lnum, col, deleted, 0) };

    // Don't leave the cursor past the end of the line.
    if fixpos != 0 && cur_win().w_cursor.col > 0 {
        cur_win().w_cursor.col -= 1;
    }
}

/// Delete `nlines` lines at the cursor, with the "N fewer lines" message.
///
/// The cursor column is reset and the line clamped into the buffer; the
/// cursor's line is *not* otherwise moved.
///
/// # Safety
/// The cursor must be on a valid line. With `undo` false the caller must have
/// prepared for undo itself.
pub unsafe fn del_lines(nlines: linenr_T, undo: bool) {
    let first = cur_win().w_cursor.lnum;
    if nlines <= 0 {
        return;
    }
    if undo && u_savedel(first, nlines).is_err() {
        return;
    }

    let mut n = 0;
    while n < nlines {
        if cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
            break; // nothing to delete
        }
        let _ = unsafe { ml_delete_flags(first, ML_DEL_MESSAGE) };
        n += 1;
        // Delete the *same* line over and over, until the buffer runs out.
        if first > cur_buf().b_ml.ml_line_count {
            break;
        }
    }

    cur_win().w_cursor.col = 0;
    check_cursor_lnum(unsafe { Win::current() });
    unsafe { deleted_lines_mark(first, n) };
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
