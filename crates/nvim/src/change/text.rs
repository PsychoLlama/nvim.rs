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
#![allow(unsafe_code)]

use crate::cstr;
use crate::mbyte::{cluster_len, head_off};
use crate::memline::MlFlags;
use crate::memory::XString;
use crate::siemsg;
use core::ffi::{c_char, c_int};
use core::slice;

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
    unsafe { dst.cast::<u8>().copy_from(src.cast(), n) };
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
        // Bounded by the bytes left, so that a truncated sequence at the end
        // does not read past `p[len]`.
        let n = cluster_len(unsafe { cstr::slice_at(p.add(i), len - i) });
        unsafe { ins_char_bytes(p.add(i), n) };
        i = i.wrapping_add(n);
    }
}

/// Insert or replace the single character `c` at the cursor.
pub fn ins_char(c: c_int) {
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
fn vreplace_extent(buf: *mut c_char, col: size_t, charlen: size_t) -> (size_t, size_t) {
    // Disable 'list' while measuring, unless 'cpo' has the `L` flag: it
    // changes how wide a TAB looks.
    let old_list = Win::current().w_onebuf_opt.wo_list;
    if old_list != 0 && !cpo_has(CpoFlag::LISTWM) {
        Win::current().w_onebuf_opt.wo_list = false as c_int;
    }

    let mut oldlen: size_t = 0;
    let mut newlen: size_t = charlen;
    let mut vcol: ColNr = 0;
    let win = Win::current();
    let cursor = Win::current().cursor().raw();
    let novcol = ::core::ptr::null_mut::<ColNr>();
    // SAFETY: the current window and its own cursor; only the middle column
    // is asked for, and it is a local.
    unsafe { getvcol(win, cursor, novcol, &raw mut vcol, novcol) };
    // SAFETY: the current window is live and `buf` holds the character.
    let new_vcol = vcol + unsafe { win_chartabsize(win, buf, vcol) };
    // The line is read here, not handed in: `getvcol` above reads the
    // memline too, so a slice taken in front of it is not one to hold.
    let mut lines = Buf::current().lines();
    let line = lines.line(Win::current().w_cursor.lnum);
    while col + oldlen < line.len() && vcol < new_vcol {
        let at = &line[col + oldlen..];
        // SAFETY: the current window is live and `at` is the rest of its
        // cursor line, terminated by the memline's own NUL.
        vcol += unsafe { win_chartabsize(win, at.as_ptr().cast::<c_char>().cast_mut(), vcol) };
        // A TAB that lands exactly where the new character ends does not
        // need removing.
        if vcol > new_vcol && c_int::from(at[0]) == TAB {
            break;
        }
        oldlen += cluster_len(at);
        // Took off a bit too much: pad with spaces.
        if vcol > new_vcol {
            newlen = newlen.wrapping_add((vcol - new_vcol) as size_t);
        }
    }

    Win::current().w_onebuf_opt.wo_list = old_list;
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
    if virtual_active(Win::current()) && Win::current().w_cursor.coladd > 0 {
        coladvance_force(getviscol());
    }

    let col = Win::current().w_cursor.col as size_t;
    let lnum = Win::current().w_cursor.lnum;

    // The defaults are the values for when not replacing: nothing deleted,
    // the whole character inserted.
    let mut oldlen: size_t = 0;
    let mut newlen: size_t = charlen;

    if State.get() & REPLACE_FLAG != 0 {
        if State.get() & VREPLACE_FLAG != 0 {
            // Measures the line for itself: the screen-column walk it does
            // reads the memline, so a line taken before it is not one to
            // hold across it.
            (oldlen, newlen) = vreplace_extent(buf, col, charlen);
        } else {
            let mut lines = Buf::current().lines();
            let tail = &lines.line(lnum)[col..];
            if !tail.is_empty() {
                oldlen = cluster_len(tail);
            }
        }
        // Push the replaced bytes onto the replace stack so BS can put them
        // back. A multi-byte character goes on the other way around, so
        // that its first byte -- which carries the length -- pops first.
        replace_push_nul();
        let mut lines = Buf::current().lines();
        replace_push(&lines.line(lnum)[col..col + oldlen]);
    }

    // The new line is the head, the character, whatever padding Virtual
    // Replace asked for, and the tail the replaced bytes left behind.
    let newline = {
        let mut lines = Buf::current().lines();
        let old = lines.line(lnum);
        let mut newline = XString::with_capacity(old.len() + newlen - oldlen);
        newline.push_bytes(&old[..col]);
        // SAFETY: the caller promises `charlen` readable bytes at `buf`.
        newline.push_bytes(unsafe { slice::from_raw_parts(buf.cast::<u8>(), charlen) });
        // Fill the rest with spaces when Virtual Replace took off too much.
        for _ in charlen..newlen {
            newline.push_byte(b' ');
        }
        newline.push_bytes(&old[col + oldlen..]);
        newline
    };

    let _ = ml_replace_owned(lnum, newline);
    inserted_bytes(lnum, col as ColNr, oldlen as c_int, newlen as c_int);

    // In Insert or Replace mode with 'showmatch', briefly show the match
    // for a closing bracket.
    if p_sm() && State.get() & MODE_INSERT != 0 && msg_silent.get() == 0 && !ins_compl_active() {
        unsafe { showmatch(utf_ptr2char(buf)) };
    }

    if !p_ri() || State.get() & REPLACE_FLAG != 0 {
        // Normal insert: move the cursor right.
        Win::current().w_cursor.col += charlen as ColNr;
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
    let lnum = Win::current().w_cursor.lnum;

    if virtual_active(Win::current()) && Win::current().w_cursor.coladd > 0 {
        coladvance_force(getviscol());
    }

    let col = Win::current().w_cursor.col as size_t;

    let newline = {
        let mut lines = Buf::current().lines();
        let old = lines.line(lnum);
        let mut newline = XString::with_capacity(old.len() + slen);
        newline.push_bytes(&old[..col]);
        // SAFETY: the caller promises `slen` readable bytes at `s`.
        newline.push_bytes(unsafe { slice::from_raw_parts(s.cast::<u8>(), slen) });
        newline.push_bytes(&old[col..]);
        newline
    };
    let _ = ml_replace_owned(lnum, newline);
    let col = col as ColNr;
    inserted_bytes(lnum, col, 0, slen as c_int);
    Win::current().w_cursor.col += slen as ColNr;
}

/// Delete the character under the cursor.
///
/// With `fixpos`, don't leave the cursor on the NUL past the end of the line.
pub fn del_char(fixpos: bool) -> Result<(), Failed> {
    // Make sure the cursor is at the start of a character.
    mb_adjust_cursor();
    if c_int::from(unsafe { *get_cursor_pos_ptr() }) == NUL {
        return Err(Failed);
    }
    del_chars(1, fixpos as c_int)
}

/// [`del_bytes`] counted in characters rather than bytes.
pub fn del_chars(count: c_int, fixpos: c_int) -> Result<(), Failed> {
    let mut bytes = 0;
    let mut p = get_cursor_pos_ptr();
    let mut i = 0;
    while i < count && c_int::from(unsafe { *p }) != NUL {
        let l = unsafe { utfc_ptr2len(p) };
        bytes += l;
        p = unsafe { p.offset(l as isize) };
        i += 1;
    }
    del_bytes(bytes, fixpos != 0, true)
}

/// Delete `count` bytes at the cursor.
///
/// With `fixpos_arg`, don't leave the cursor on the NUL past the end of the
/// line; with `use_delcombine`, 'delcombine' applies, so that deleting a
/// character that carries combining marks takes only the last mark.
///
/// Answers `Err` on the NUL past the end of the line or for a negative
/// `count`, `Ok` otherwise.
pub fn del_bytes(mut count: ColNr, fixpos_arg: bool, use_delcombine: bool) -> Result<(), Failed> {
    let lnum = Win::current().w_cursor.lnum;
    let mut col = Win::current().w_cursor.col;
    let mut fixpos = fixpos_arg;
    let oldlen = Buf::current().lines().line_len(lnum);

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
    if p_deco() && use_delcombine {
        let mut lines = Buf::current().lines();
        let line = lines.line(lnum);
        // The line's own bytes: every offset taken off it below is a column
        // reached by stepping whole characters from `col`.
        let base = line.as_ptr().cast::<c_char>();
        if cluster_len(&line[col as usize..]) >= count as usize {
            // SAFETY: `col` is a column of the line.
            let p0 = unsafe { base.offset(col as isize) };
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            // SAFETY: `p0` and the byte after its character, both inside the
            // line or its terminator.
            if unsafe { utf_composinglike(p0, p0.offset(utf_ptr2len(p0) as isize), &raw mut state) }
            {
                // Walk to the last composing character; there can be several.
                let mut n = col;
                loop {
                    col = n;
                    // SAFETY: `n` is a column of the line.
                    count = unsafe { utf_ptr2len(base.offset(n as isize)) };
                    n += count;
                    // SAFETY: two columns of the line, the second of which
                    // may be its terminator.
                    if !unsafe {
                        utf_composinglike(
                            base.offset(col as isize),
                            base.offset(n as isize),
                            &raw mut state,
                        )
                    } {
                        break;
                    }
                }
                fixpos = false;
            }
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
            && get_ve_flags(Win::current()) & kOptVeFlagOnemore as ::core::ffi::c_uint == 0
        {
            Win::current().w_cursor.col -= 1;
            Win::current().w_cursor.coladd = 0;
            let at = Win::current().w_cursor.col as usize;
            let back = head_off(Buf::current().lines().line(lnum), at);
            Win::current().w_cursor.col -= ColNr::try_from(back).unwrap_or(0);
        }
        count = oldlen - col;
        movelen = 1;
    }
    let newlen = oldlen - count;

    // An already-allocated line can be edited in place; one that is still
    // memory-mapped has to be copied.
    if ml_line_alloced() {
        // The cached line is this buffer's own block, `oldlen + 1` bytes of
        // it, so the tail moves up inside it. `movelen` counts the NUL.
        ml_add_deleted_len(Buf::current().b_ml.cached_bytes(), oldlen as usize);
        let oldp = Buf::current().b_ml.cached_text();
        let from = oldp
            .wrapping_offset(col as isize)
            .wrapping_offset(count as isize);
        move_bytes(oldp.wrapping_offset(col as isize), from, movelen as size_t);
        Buf::current().b_ml.set_cached_len(newlen + 1);
    } else {
        let newline = {
            let mut lines = Buf::current().lines();
            let old = lines.line(lnum);
            let mut newline = XString::with_capacity(newlen as usize);
            newline.push_bytes(&old[..col as usize]);
            newline.push_bytes(&old[(col + count) as usize..]);
            newline
        };
        let _ = ml_replace_owned(lnum, newline);
    }

    inserted_bytes(lnum, col, count, 0);
    Ok(())
}

/// Delete everything on the cursor line from the cursor onwards.
pub fn truncate_line(fixpos: c_int) {
    let lnum = Win::current().w_cursor.lnum;
    let col = Win::current().w_cursor.col;
    let mut lines = Buf::current().lines();
    let old_len = lines.line_len(lnum);
    let newp = XString::from_bytes(&lines.line(lnum)[..col as usize]);
    let deleted = old_len - col;

    let _ = ml_replace_owned(lnum, newp);
    inserted_bytes(lnum, col, deleted, 0);

    // Don't leave the cursor past the end of the line.
    if fixpos != 0 && Win::current().w_cursor.col > 0 {
        Win::current().w_cursor.col -= 1;
    }
}

/// Delete `nlines` lines at the cursor, with the "N fewer lines" message.
///
/// The cursor column is reset and the line clamped into the buffer; the
/// cursor's line is *not* otherwise moved.
pub fn del_lines(nlines: LineNr, undo: bool) {
    let first = Win::current().w_cursor.lnum;
    if nlines <= 0 {
        return;
    }
    if undo && u_savedel(first, nlines).is_err() {
        return;
    }

    let mut n = 0;
    while n < nlines {
        if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
            break; // nothing to delete
        }
        let _ = ml_delete_flags(first, ML_DEL_MESSAGE);
        n += 1;
        // Delete the *same* line over and over, until the buffer runs out.
        if first > Buf::current().b_ml.ml_line_count {
            break;
        }
    }

    Win::current().w_cursor.col = 0;
    check_cursor_lnum(Win::current());
    deleted_lines_mark(first, n);
}

/// Line `lnum` of the current buffer becomes `line`: the memline copies it
/// (`copy`) or takes it over. The pointer form of the memline's replace
/// ([`ml_replace_buf_text`], [`ml_replace_owned`]), kept here beside most of
/// the callers that still hold their new line as a pointer, until they hold
/// it as an owned string.
///
/// # Safety
/// `line` is null or NUL-terminated; unless `copy`, it is an `xmalloc` block
/// that nobody else frees.
pub(crate) unsafe fn ml_replace(lnum: LineNr, line: *mut c_char, copy: bool) -> Result<(), Failed> {
    if line.is_null() {
        return Err(Failed); // just checking...
    }
    // SAFETY: the caller's NUL-terminated line, read for the copy, or the
    // caller's block, handed over.
    unsafe {
        if copy {
            ml_replace_buf_text(Buf::current(), lnum, cstr::bytes_at(line))
        } else {
            ml_replace_owned(lnum, XString::from_raw(line))
        }
    }
}

/// [`ml_replace`] of the first `len` bytes at `line`.
///
/// # Safety
/// `line` holds `len` readable bytes; unless `copy`, it is an `xmalloc`
/// block that nobody else frees, and this frees it.
pub(crate) unsafe fn ml_replace_len(
    lnum: LineNr,
    line: *mut c_char,
    len: usize,
    copy: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's `len` bytes, copied; then its block, freed.
    unsafe {
        let replaced = ml_replace_buf_text(Buf::current(), lnum, cstr::slice_at(line, len));
        if !copy {
            xfree(line.cast());
        }
        replaced
    }
}
