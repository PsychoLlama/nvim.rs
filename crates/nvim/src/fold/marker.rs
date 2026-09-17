//! 'foldmethod' = "marker": folds that live in the buffer text as `{{{` and
//! `}}}`.
//!
//! This is the only fold method whose folds are *stored*, so creating and
//! deleting one edits the buffer (and is undoable). 'foldmarker' is split
//! into its two halves by [`parse_marker`], which every function here
//! requires to have run.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::charset::skip;
use crate::cstr;
use crate::cstr::byte_at;
use crate::extmark::extmark_splice_cols;
use crate::mbyte::cluster_len;
use crate::memline::{Lines, ml_replace_buf_len};
use crate::message::e_modifiable;
use crate::message::emsg;
use crate::ops::skip_comment;
use crate::os::cshim::gettext;
use crate::strings::vim_strchr;
use crate::undo::u_save;
use core::ffi::{c_char, c_int};

use super::*;

use crate::optionstr::LocalOptStr;
use crate::winlayer::Buf;
/// Create a fold from line "start" to line "end" (inclusive) in window `window`
/// by adding markers.
pub(super) fn fold_create_markers(window: Win, start: Pos, end: Pos) {
    let buf = window.buffer();
    if buf.b_p_ma == 0 {
        emsg(gettext(e_modifiable));
        return;
    }
    let num_changed = (1 + end.lnum - start.lnum) as int64_t;
    // SAFETY: the caller's promise; both lines are inside the buffer, and
    // `parse_marker` has just written the two markers and their lengths.
    parse_marker(window);
    // SAFETY: as above -- both spans are inside 'foldmarker'.
    let (open, close) = unsafe {
        (
            cstr::slice_at(
                window.w_onebuf_opt.wo_fmr.value_ptr(),
                foldstartmarkerlen.get(),
            ),
            cstr::slice_at(foldendmarker.get(), foldendmarkerlen.get()),
        )
    };
    fold_add_marker(buf, start, open);
    fold_add_marker(buf, end, close);
    changed_lines(buf, start.lnum, 0, end.lnum, 0, false);
    buf_updates_send_changes(buf, start.lnum, num_changed, num_changed);
}

/// Add `marker` in 'commentstring' to position `pos`.
pub(super) fn fold_add_marker(buffer: Buf, pos: Pos, marker: &[u8]) {
    let lnum = pos.lnum;
    // 'commentstring', and where in it the marker's text goes.
    // SAFETY: the buffer's own option value, NUL-terminated.
    let cms = unsafe { cstr::bytes_at(buffer.b_p_cms.value_ptr()) };
    let text_at = cms.windows(2).position(|w| w == b"%s");
    if u_save(lnum - 1, lnum + 1).is_err() {
        return;
    }
    // A copy, because the line is rewritten from it below.
    let text = Lines::in_buffer(buffer).line_copy(lnum);
    // Does the line already end inside a comment?
    let mut line_is_comment = false;
    // SAFETY: the copy is NUL-terminated and the flag is this frame's.
    unsafe {
        skip_comment(
            text.as_cstr().as_ptr().cast_mut(),
            false,
            false,
            &raw mut line_is_comment,
        )
    };

    let mut new = Vec::with_capacity(text.len() + marker.len() + cms.len() + 1);
    new.extend_from_slice(&text);
    let added = match text_at {
        // No '%s' in 'commentstring', or the line already is a comment:
        // the marker goes on bare.
        Some(at) if !line_is_comment => {
            new.extend_from_slice(&cms[..at]);
            new.extend_from_slice(marker);
            new.extend_from_slice(&cms[at + 2..]);
            marker.len() + cms.len() - 2
        }
        _ => {
            new.extend_from_slice(marker);
            marker.len()
        }
    };
    new.push(NUL as u8);
    // SAFETY: a live buffer, a line inside it, and `new` is this frame's
    // NUL-terminated text, which `copy` says the memline duplicates.
    let _ = unsafe {
        ml_replace_buf_len(
            buffer,
            lnum,
            new.as_mut_ptr().cast::<c_char>(),
            new.len() - 1,
            true,
            false,
        )
    };
    if added != 0 {
        extmark_splice_cols(
            buffer,
            lnum as c_int - 1,
            text.len() as ColNr,
            0,
            added as ColNr,
            kExtmarkUndo,
        );
    }
}

/// Delete the markers for a fold, causing it to be deleted.
///
/// `lnum_off` — offset for fold.top()
pub(super) fn delete_fold_markers(window: Win, fold: FoldRef, recursive: bool, lnum_off: LineNr) {
    if recursive {
        for child in fold.nested().folds() {
            delete_fold_markers(window, child, true, lnum_off + fold.top());
        }
    }
    // SAFETY: the caller's promise, which includes `parse_marker` having run.
    // SAFETY: as above -- both spans are inside 'foldmarker'.
    let (open, close) = unsafe {
        (
            cstr::slice_at(
                window.w_onebuf_opt.wo_fmr.value_ptr(),
                foldstartmarkerlen.get(),
            ),
            cstr::slice_at(foldendmarker.get(), foldendmarkerlen.get()),
        )
    };
    fold_del_marker(window.buffer(), fold.top() + lnum_off, open);
    fold_del_marker(window.buffer(), fold.last() + lnum_off, close);
}

/// Delete `marker` at the end of line `lnum`, and the 'commentstring' around
/// it if that matches too.
///
/// If the marker is not found, there is no error message.  Could be a missing
/// close-marker.
pub(super) fn fold_del_marker(buffer: Buf, lnum: LineNr, marker: &[u8]) {
    if lnum > buffer.b_ml.ml_line_count {
        return;
    }
    // SAFETY: the buffer's own option value, NUL-terminated.
    let cms = unsafe { cstr::bytes_at(buffer.b_p_cms.value_ptr()) };
    let text_at = cms.windows(2).position(|w| w == b"%s");
    // A copy, because the line is rewritten from it below and `u_save` runs
    // in between.
    let text = Lines::in_buffer(buffer).line_copy(lnum);

    let Some(mut start) = (0..text.len()).find(|&at| text[at..].starts_with(marker)) else {
        return;
    };
    let mut len = marker.len();
    // A numbered marker, `{{{2`.
    if ascii_isdigit(c_int::from(byte_at(&text, start + len))) {
        len += 1;
    }
    // The marker may be wrapped in 'commentstring'; if it is, the comment
    // goes with it.
    if let Some(before) = text_at
        && start >= before
        && text[start - before..].starts_with(&cms[..before])
        && text[start + len..].starts_with(&cms[before + 2..])
    {
        start -= before;
        len += cms.len() - 2;
    }
    if u_save(lnum - 1, lnum + 1).is_err() {
        return;
    }
    let mut new = Vec::with_capacity(text.len() - len + 1);
    new.extend_from_slice(&text[..start]);
    new.extend_from_slice(&text[start + len..]);
    new.push(NUL as u8);
    // SAFETY: a live buffer, a line inside it, and `new` is this frame's
    // NUL-terminated text, which `copy` says the memline duplicates.
    let _ = unsafe {
        ml_replace_buf_len(
            buffer,
            lnum,
            new.as_mut_ptr().cast::<c_char>(),
            new.len() - 1,
            true,
            false,
        )
    };
    extmark_splice_cols(
        buffer,
        lnum as c_int - 1,
        start as ColNr,
        len as ColNr,
        0,
        kExtmarkUndo,
    );
}

/// Parse 'foldmarker' and set "foldendmarker", "foldstartmarkerlen" and
/// "foldendmarkerlen".
/// Relies on the option value to have been checked for correctness already.
///
/// Note that `foldendmarker` points *into* 'foldmarker', so it dangles the
/// moment the option is set again — which is why every caller re-runs this.
///
pub(super) fn parse_marker(window: Win) {
    let foldmarker = window.w_onebuf_opt.wo_fmr.value_ptr();
    // SAFETY: 'foldmarker' has already been validated as two non-empty
    // halves separated by a comma, so the comma is there.
    let comma = unsafe { vim_strchr(foldmarker, ',' as c_int) };
    foldstartmarkerlen.set(unsafe { comma.offset_from(foldmarker) } as size_t);
    let end = unsafe { comma.offset(1) };
    foldendmarker.set(end);
    foldendmarkerlen.set(unsafe { cstr::bytes_at(end) }.len());
}

/// Low level function to get the foldlevel for the "marker" method.
/// "foldendmarker", "foldstartmarkerlen" and "foldendmarkerlen" must have been
/// set before calling this.
/// Requires that flp->lvl is set to the fold level of the previous line!
/// Careful: This means you can't call this function twice on the same line.
/// Doesn't use any caching.
/// Sets flp->start when a start marker was found.
pub(super) fn foldlevel_marker(line: FLine) {
    let flp = line.raw();
    // SAFETY: the caller's promise -- a live window, and `parse_marker` has
    // written the two markers and their lengths.
    let (window, start_lvl) = unsafe { ((*flp).wp, (*flp).lvl) };
    let startmarker = unsafe {
        cstr::slice_at(
            (*window).w_onebuf_opt.wo_fmr.value_ptr(),
            foldstartmarkerlen.get(),
        )
    };
    let endmarker = unsafe { cstr::slice_at(foldendmarker.get(), foldendmarkerlen.get()) };
    unsafe { (*flp).start = 0 };
    unsafe { (*flp).lvl_next = (*flp).lvl };

    // SAFETY: the window's own buffer, and the line the caller named.
    let buffer = unsafe { Buf::new((*window).w_buffer) };
    let lnum = unsafe { (*flp).lnum + (*flp).off };
    let mut lines = buffer.lines();
    let text = lines.line(lnum);

    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        // The first byte is compared before the rest of the marker, which is
        // what makes this a scan and not a search.
        if rest.first() == startmarker.first() && rest.starts_with(startmarker) {
            at += startmarker.len();
            match marker_number(&text[at..]) {
                // `{{{N` sets the level outright.
                Some(n) => unsafe {
                    (*flp).lvl = n;
                    (*flp).lvl_next = n;
                    (*flp).start = if n - start_lvl > 1 { n - start_lvl } else { 1 };
                },
                None => unsafe {
                    (*flp).lvl += 1;
                    (*flp).lvl_next += 1;
                    (*flp).start += 1;
                },
            }
        } else if rest.first() == endmarker.first() && rest.starts_with(endmarker) {
            at += endmarker.len();
            match marker_number(&text[at..]) {
                Some(n) => unsafe {
                    (*flp).lvl = n;
                    (*flp).lvl_next = (n - 1).min(start_lvl);
                },
                None => unsafe { (*flp).lvl_next -= 1 },
            }
        } else {
            at += cluster_len(rest);
        }
    }
    unsafe { (*flp).lvl_next = (*flp).lvl_next.max(0) };
}

/// The positive number a `{{{N` marker carries, if it carries one.
///
/// Upstream reads it with `atoi`, whose answer for a run of digits that
/// overflows an `int` is undefined; this saturates instead. A fold level that
/// large is nonsense either way, and the clamp is the only difference.
fn marker_number(rest: &[u8]) -> Option<c_int> {
    let digits = &rest[..skip::digits(rest)];
    let n = digits.iter().fold(0 as c_int, |n, &b| {
        n.saturating_mul(10).saturating_add(c_int::from(b - b'0'))
    });
    (n > 0).then_some(n)
}
