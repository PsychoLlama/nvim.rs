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

use crate::siemsg_c;
use ::core::ffi::{c_char, c_int, c_void};

use super::*;

/// Insert the NUL-terminated string `p` at the cursor.
///
/// # Safety
/// `p` must be NUL-terminated. The caller must have prepared for undo.
pub unsafe fn ins_bytes(p: *mut c_char) {
    unsafe {
        ins_bytes_len(p, strlen(p));
    }
}

/// Insert `len` bytes of `p` at the cursor, one character at a time.
///
/// # Safety
/// `p` must point to at least `len` readable bytes. The caller must have
/// prepared for undo.
pub unsafe fn ins_bytes_len(p: *mut c_char, len: size_t) {
    unsafe {
        let mut i: size_t = 0;
        while i < len {
            // The `_len` form so that a truncated sequence at the end does not
            // read past `p[len]`.
            let n = utfc_ptr2len_len(p.add(i), len.wrapping_sub(i) as c_int) as size_t;
            ins_char_bytes(p.add(i), n);
            i = i.wrapping_add(n);
        }
    }
}

/// Insert or replace the single character `c` at the cursor.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn ins_char(c: c_int) {
    unsafe {
        let mut buf: [c_char; 7] = [0; 7];
        let n = utf_char2bytes(c, buf.as_mut_ptr()) as size_t;
        // `c` being 0x100, 0x200, ... would encode to a leading NUL byte, which
        // must not go into the line; CTRL-V u9900 reaches this.
        if buf[0] == 0 {
            buf[0] = b'\n' as c_char;
        }
        ins_char_bytes(buf.as_mut_ptr(), n);
    }
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
    unsafe {
        // Disable 'list' while measuring, unless 'cpo' has the `L` flag: it
        // changes how wide a TAB looks.
        let old_list = (*curwin.get()).w_onebuf_opt.wo_list;
        if old_list != 0 && vim_strchr(p_cpo.get(), CPO_LISTWM).is_null() {
            (*curwin.get()).w_onebuf_opt.wo_list = false as c_int;
        }

        let mut oldlen: size_t = 0;
        let mut newlen: size_t = charlen;
        let mut vcol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            ::core::ptr::null_mut(),
            &raw mut vcol,
            ::core::ptr::null_mut(),
        );
        let new_vcol = vcol + win_chartabsize(curwin.get(), buf, vcol);
        while c_int::from(*oldp.add(col.wrapping_add(oldlen))) != NUL && vcol < new_vcol {
            vcol += win_chartabsize(curwin.get(), oldp.add(col).add(oldlen), vcol);
            // A TAB that lands exactly where the new character ends does not
            // need removing.
            if vcol > new_vcol && c_int::from(*oldp.add(col.wrapping_add(oldlen))) == TAB {
                break;
            }
            oldlen = oldlen.wrapping_add(utfc_ptr2len(oldp.add(col).add(oldlen)) as size_t);
            // Took off a bit too much: pad with spaces.
            if vcol > new_vcol {
                newlen = newlen.wrapping_add((vcol - new_vcol) as size_t);
            }
        }

        (*curwin.get()).w_onebuf_opt.wo_list = old_list;
        (oldlen, newlen)
    }
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
    unsafe {
        // Break tabs if needed.
        if virtual_active(curwin.get()) && (*curwin.get()).w_cursor.coladd > 0 {
            coladvance_force(getviscol());
        }

        let col = (*curwin.get()).w_cursor.col as size_t;
        let lnum = (*curwin.get()).w_cursor.lnum;
        let oldp = ml_get(lnum);
        let linelen = (ml_get_len(lnum) as size_t).wrapping_add(1); // includes the NUL

        // The defaults are the values for when not replacing: nothing deleted,
        // the whole character inserted.
        let mut oldlen: size_t = 0;
        let mut newlen: size_t = charlen;

        if State.get() & REPLACE_FLAG != 0 {
            if State.get() & VREPLACE_FLAG != 0 {
                (oldlen, newlen) = vreplace_extent(buf, oldp, col, charlen);
            } else if c_int::from(*oldp.add(col)) != NUL {
                oldlen = utfc_ptr2len(oldp.add(col)) as size_t;
            }
            // Push the replaced bytes onto the replace stack so BS can put them
            // back. A multi-byte character goes on the other way around, so
            // that its first byte -- which carries the length -- pops first.
            replace_push_nul();
            replace_push(oldp.add(col), oldlen);
        }

        let newp = xmalloc(linelen.wrapping_add(newlen).wrapping_sub(oldlen)) as *mut c_char;

        if col > 0 {
            memmove(newp as *mut c_void, oldp as *const c_void, col);
        }
        let p = newp.add(col);
        if linelen > col.wrapping_add(oldlen) {
            memmove(
                p.add(newlen) as *mut c_void,
                oldp.add(col).add(oldlen) as *const c_void,
                linelen.wrapping_sub(col).wrapping_sub(oldlen),
            );
        }
        memmove(p as *mut c_void, buf as *const c_void, charlen);
        // Fill the rest with spaces when Virtual Replace took off too much.
        for i in charlen..newlen {
            *p.add(i) = b' ' as c_char;
        }

        ml_replace(lnum, newp, false);
        inserted_bytes(lnum, col as colnr_T, oldlen as c_int, newlen as c_int);

        // In Insert or Replace mode with 'showmatch', briefly show the match
        // for a closing bracket.
        if p_sm.get() != 0
            && State.get() & MODE_INSERT != 0
            && msg_silent.get() == 0
            && !ins_compl_active()
        {
            showmatch(utf_ptr2char(buf));
        }

        if p_ri.get() == 0 || State.get() & REPLACE_FLAG != 0 {
            // Normal insert: move the cursor right.
            (*curwin.get()).w_cursor.col += charlen as colnr_T;
        }
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
    unsafe {
        let lnum = (*curwin.get()).w_cursor.lnum;

        if virtual_active(curwin.get()) && (*curwin.get()).w_cursor.coladd > 0 {
            coladvance_force(getviscol());
        }

        let col = (*curwin.get()).w_cursor.col;
        let oldp = ml_get(lnum);
        let oldlen = ml_get_len(lnum);

        let newp = xmalloc((oldlen as size_t).wrapping_add(slen).wrapping_add(1)) as *mut c_char;
        if col > 0 {
            memmove(newp as *mut c_void, oldp as *const c_void, col as size_t);
        }
        memmove(
            newp.offset(col as isize) as *mut c_void,
            s as *const c_void,
            slen,
        );
        // The tail, including the NUL. The cursor is inside the line, so this
        // is never negative.
        let bytes = oldlen - col + 1;
        assert!(bytes >= 0);
        memmove(
            newp.offset(col as isize).add(slen) as *mut c_void,
            oldp.offset(col as isize) as *const c_void,
            bytes as size_t,
        );
        ml_replace(lnum, newp, false);
        inserted_bytes(lnum, col, 0, slen as c_int);
        (*curwin.get()).w_cursor.col += slen as colnr_T;
    }
}

/// Delete the character under the cursor.
///
/// With `fixpos`, don't leave the cursor on the NUL past the end of the line.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn del_char(fixpos: bool) -> c_int {
    unsafe {
        // Make sure the cursor is at the start of a character.
        mb_adjust_cursor();
        if c_int::from(*get_cursor_pos_ptr()) == NUL {
            return FAIL;
        }
        del_chars(1, fixpos as c_int)
    }
}

/// [`del_bytes`] counted in characters rather than bytes.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn del_chars(count: c_int, fixpos: c_int) -> c_int {
    unsafe {
        let mut bytes = 0;
        let mut p = get_cursor_pos_ptr();
        let mut i = 0;
        while i < count && c_int::from(*p) != NUL {
            let l = utfc_ptr2len(p);
            bytes += l;
            p = p.offset(l as isize);
            i += 1;
        }
        del_bytes(bytes, fixpos != 0, true)
    }
}

/// Delete `count` bytes at the cursor.
///
/// With `fixpos_arg`, don't leave the cursor on the NUL past the end of the
/// line; with `use_delcombine`, 'delcombine' applies, so that deleting a
/// character that carries combining marks takes only the last mark.
///
/// Answers `FAIL` on the NUL past the end of the line or for a negative
/// `count`, `OK` otherwise.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn del_bytes(mut count: colnr_T, fixpos_arg: bool, use_delcombine: bool) -> c_int {
    unsafe {
        let lnum = (*curwin.get()).w_cursor.lnum;
        let mut col = (*curwin.get()).w_cursor.col;
        let mut fixpos = fixpos_arg;
        let oldp = ml_get(lnum);
        let oldlen = ml_get_len(lnum);

        // Nothing to do on the NUL after the line.
        if col >= oldlen {
            return FAIL;
        }
        if count == 0 {
            return OK;
        }
        if count < 1 {
            siemsg_c!(
                c"E292: Invalid count for del_bytes(): %ld".as_ptr(),
                int64_t::from(count),
            );
            return FAIL;
        }

        // With 'delcombine', deleting (less than) one character takes only the
        // last combining character off it -- and then the cursor must not move,
        // because the base character is still there.
        if p_deco.get() != 0 && use_delcombine && utfc_ptr2len(oldp.offset(col as isize)) >= count {
            let p0 = oldp.offset(col as isize);
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            if utf_composinglike(p0, p0.offset(utf_ptr2len(p0) as isize), &raw mut state) {
                // Walk to the last composing character; there can be several.
                let mut n = col;
                loop {
                    col = n;
                    count = utf_ptr2len(oldp.offset(n as isize));
                    n += count;
                    if !utf_composinglike(
                        oldp.offset(col as isize),
                        oldp.offset(n as isize),
                        &raw mut state,
                    ) {
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
                && get_ve_flags(curwin.get()) & kOptVeFlagOnemore as ::core::ffi::c_uint == 0
            {
                (*curwin.get()).w_cursor.col -= 1;
                (*curwin.get()).w_cursor.coladd = 0;
                (*curwin.get()).w_cursor.col -=
                    utf_head_off(oldp, oldp.offset((*curwin.get()).w_cursor.col as isize));
            }
            count = oldlen - col;
            movelen = 1;
        }
        let newlen = oldlen - count;

        // An already-allocated line can be edited in place; one that is still
        // memory-mapped has to be copied.
        let alloc_newp = ml_line_alloced() == 0;
        let newp = if alloc_newp {
            let newp = xmallocz(newlen as size_t) as *mut c_char;
            memmove(newp as *mut c_void, oldp as *const c_void, col as size_t);
            newp
        } else {
            ml_add_deleted_len((*curbuf.get()).b_ml.ml_line_ptr, oldlen as ssize_t);
            oldp
        };
        memmove(
            newp.offset(col as isize) as *mut c_void,
            oldp.offset(col as isize).offset(count as isize) as *const c_void,
            movelen as size_t,
        );
        if alloc_newp {
            ml_replace(lnum, newp, false);
        } else {
            (*curbuf.get()).b_ml.ml_line_textlen = newlen + 1;
        }

        inserted_bytes(lnum, col, count, 0);
        OK
    }
}

/// Delete everything on the cursor line from the cursor onwards.
///
/// # Safety
/// The caller must have prepared for undo.
pub unsafe fn truncate_line(fixpos: c_int) {
    unsafe {
        let lnum = (*curwin.get()).w_cursor.lnum;
        let col = (*curwin.get()).w_cursor.col;
        let old_line = ml_get(lnum);
        let newp = if col == 0 {
            xstrdup(c"".as_ptr())
        } else {
            xstrnsave(old_line, col as size_t)
        };
        let deleted = ml_get_len(lnum) - col;

        ml_replace(lnum, newp, false);
        inserted_bytes(lnum, (*curwin.get()).w_cursor.col, deleted, 0);

        // Don't leave the cursor past the end of the line.
        if fixpos != 0 && (*curwin.get()).w_cursor.col > 0 {
            (*curwin.get()).w_cursor.col -= 1;
        }
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
    unsafe {
        let first = (*curwin.get()).w_cursor.lnum;
        if nlines <= 0 {
            return;
        }
        if undo && u_savedel(first, nlines) == FAIL {
            return;
        }

        let mut n = 0;
        while n < nlines {
            if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                break; // nothing to delete
            }
            ml_delete_flags(first, ML_DEL_MESSAGE);
            n += 1;
            // Delete the *same* line over and over, until the buffer runs out.
            if first > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
        }

        (*curwin.get()).w_cursor.col = 0;
        check_cursor_lnum(curwin.get());
        deleted_lines_mark(first, n);
    }
}
