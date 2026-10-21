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

#![forbid(unsafe_code)]

use crate::cstr;
use crate::mbyte::{char_at, char_len, cluster_len, encode_char, head_off, iscomposing};
use crate::memline::MlFlags;
use crate::memory::XString;
use crate::siemsg;
use core::ffi::c_int;

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, Failed, NUL};
use crate::winlayer::{Buf, Win};

/// Insert `bytes` at the cursor, one character at a time, so that Replace
/// mode overwrites -- and the replace stack records -- each in turn.
///
/// The caller must have prepared for undo.
pub fn ins_bytes(bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        // Bounded by the bytes left, so that a truncated sequence at the end
        // is a character of its own.
        let n = cluster_len(&bytes[i..]);
        ins_char_bytes(&bytes[i..i + n]);
        i += n;
    }
}

/// Insert or replace the single character `c` at the cursor.
pub fn ins_char(c: c_int) {
    let mut buf = [0u8; 7];
    let n = encode_char(c, &mut buf);
    // `c` being 0x100, 0x200, ... would encode to a leading NUL byte, which
    // must not go into the line; CTRL-V u9900 reaches this.
    if buf[0] == 0 {
        buf[0] = b'\n';
    }
    ins_char_bytes(&buf[..n]);
}

/// How many bytes at the cursor Virtual Replace mode has to consume to make
/// room for `new`, and how many bytes the new text takes once padded.
///
/// Each typed character replaces one or more existing ones, or none at all for
/// a TAB, because what is being matched is screen cells. Overshooting the
/// column the new character ends at means the difference has to be filled with
/// spaces, which is what the returned `newlen` carries.
fn vreplace_extent(new: &[u8], col: usize) -> (usize, usize) {
    // Disable 'list' while measuring, unless 'cpo' has the `L` flag: it
    // changes how wide a TAB looks.
    let old_list = Win::current().w_onebuf_opt.wo_list;
    if old_list != 0 && !cpo_has(CpoFlag::LISTWM) {
        Win::current().w_onebuf_opt.wo_list = false as c_int;
    }

    let mut oldlen = 0;
    let mut newlen = new.len();
    let win = Win::current();
    let mut vcol: ColNr = win.vcol_triple(win.cursor()).1;
    let new_vcol = vcol + win_chartabsize(win, new, vcol);
    // The line is read here, not handed in: `getvcol` above reads the
    // memline too, so a slice taken in front of it is not one to hold.
    let mut lines = Buf::current().lines();
    let line = lines.line(Win::current().w_cursor.lnum);
    while col + oldlen < line.len() && vcol < new_vcol {
        let at = &line[col + oldlen..];
        vcol += win_chartabsize(win, at, vcol);
        // A TAB that lands exactly where the new character ends does not
        // need removing.
        if vcol > new_vcol && c_int::from(at[0]) == TAB {
            break;
        }
        oldlen += cluster_len(at);
        // Took off a bit too much: pad with spaces.
        if vcol > new_vcol {
            newlen += (vcol - new_vcol) as usize;
        }
    }

    Win::current().w_onebuf_opt.wo_list = old_list;
    (oldlen, newlen)
}

/// Insert `new` -- one whole character -- at the cursor, replacing what is
/// there in Replace and Virtual Replace modes.
///
/// The caller has already turned bytes into a character; this only ever writes
/// one. The caller must have prepared for undo.
pub fn ins_char_bytes(new: &[u8]) {
    // Break tabs if needed.
    if virtual_active(Win::current()) && Win::current().w_cursor.coladd > 0 {
        coladvance_force(getviscol());
    }

    let col = Win::current().w_cursor.col as usize;
    let lnum = Win::current().w_cursor.lnum;
    let charlen = new.len();

    // The defaults are the values for when not replacing: nothing deleted,
    // the whole character inserted.
    let mut oldlen = 0;
    let mut newlen = charlen;

    if State.get() & REPLACE_FLAG != 0 {
        if State.get() & VREPLACE_FLAG != 0 {
            // Measures the line for itself: the screen-column walk it does
            // reads the memline, so a line taken before it is not one to
            // hold across it.
            (oldlen, newlen) = vreplace_extent(new, col);
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
        newline.push_bytes(new);
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
        showmatch(char_at(new));
    }

    if !p_ri() || State.get() & REPLACE_FLAG != 0 {
        // Normal insert: move the cursor right.
        Win::current().w_cursor.col += charlen as ColNr;
    }
}

/// Insert `text` at the cursor, all at once.
///
/// Unlike [`ins_char_bytes`] this does *not* handle Replace mode. The caller
/// must have prepared for undo.
pub fn ins_str(text: &[u8]) {
    let lnum = Win::current().w_cursor.lnum;

    if virtual_active(Win::current()) && Win::current().w_cursor.coladd > 0 {
        coladvance_force(getviscol());
    }

    let col = Win::current().w_cursor.col as usize;

    let newline = {
        let mut lines = Buf::current().lines();
        let old = lines.line(lnum);
        let mut newline = XString::with_capacity(old.len() + text.len());
        newline.push_bytes(&old[..col]);
        newline.push_bytes(text);
        newline.push_bytes(&old[col..]);
        newline
    };
    let _ = ml_replace_owned(lnum, newline);
    inserted_bytes(lnum, col as ColNr, 0, text.len() as c_int);
    Win::current().w_cursor.col += text.len() as ColNr;
}

/// Delete the character under the cursor.
///
/// With `fixpos`, don't leave the cursor on the NUL past the end of the line.
pub fn del_char(fixpos: bool) -> Result<(), Failed> {
    // Make sure the cursor is at the start of a character.
    mb_adjust_cursor();
    let cursor = Win::current().w_cursor;
    let mut lines = Buf::current().lines();
    if cstr::byte_at(lines.line(cursor.lnum), cursor.col as usize) == NUL as u8 {
        return Err(Failed);
    }
    del_chars(1, fixpos as c_int)
}

/// [`del_bytes`] counted in characters rather than bytes.
pub fn del_chars(count: c_int, fixpos: c_int) -> Result<(), Failed> {
    let cursor = Win::current().w_cursor;
    let col = cursor.col as usize;
    let bytes = {
        let mut lines = Buf::current().lines();
        let line = lines.line(cursor.lnum);
        let mut at = col;
        let mut i = 0;
        while i < count && at < line.len() {
            at += cluster_len(&line[at..]);
            i += 1;
        }
        at - col
    };
    del_bytes(bytes as ColNr, fixpos != 0, true)
}

/// Whether the character at byte `next` of `line` belongs to the grapheme
/// cluster of the one at byte `at` -- upstream's `utf_composinglike`, with
/// the walk state it carries.
fn composes_with(line: &[u8], at: usize, next: usize, state: &mut GraphemeState) -> bool {
    // ASCII never combines, and this is the hot answer; the end of the line
    // reads as its NUL.
    cstr::byte_at(line, next) >= 0x80
        && iscomposing(char_at(&line[at..]), char_at(&line[next..]), Some(state))
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
        let at = col as usize;
        if cluster_len(&line[at..]) >= count as usize {
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            if composes_with(line, at, at + char_len(&line[at..]), &mut state) {
                // Walk to the last composing character; there can be several.
                let mut n = at;
                loop {
                    col = n as ColNr;
                    count = char_len(&line[n..]) as ColNr;
                    n += count as usize;
                    if !composes_with(line, col as usize, n, &mut state) {
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
    // in its block has to be copied.
    if ml_line_alloced() {
        // The cached line is this buffer's own, `oldlen + 1` bytes of it, so
        // the tail moves up inside it. `movelen` counts the NUL.
        ml_add_deleted_len(Buf::current().b_ml.cached_bytes(), oldlen as usize);
        let mut buffer = Buf::current();
        let line = buffer
            .b_ml
            .owned_bytes_with_nul_mut()
            .expect("an allocated line is owned");
        let (col, count, movelen) = (col as usize, count as usize, movelen as usize);
        line.copy_within(col + count..col + count + movelen, col);
        buffer.b_ml.set_cached_len(newlen + 1);
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
