//! 'foldmethod' = "marker": folds that live in the buffer text as `{{{` and
//! `}}}`.
//!
//! This is the only fold method whose folds are *stored*, so creating and
//! deleting one edits the buffer (and is undoable). 'foldmarker' is split
//! into its two halves by [`parse_marker`], which every function here
//! requires to have run.

#![forbid(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::charset::skip;
use crate::cstr::byte_at;
use crate::extmark::extmark_splice_cols;
use crate::mbyte::cluster_len;
use crate::memline::{Lines, ml_replace_buf_text};
use crate::message::e_modifiable;
use crate::message::emsg;
use crate::ops::skip_comment;
use crate::os::cshim::gettext;
use crate::undo::u_save;
use core::ffi::c_int;

use super::*;

use crate::optionstr::OptString;
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
    parse_marker(window);
    // Copies: `u_save` may run autocommands, which may set 'foldmarker'.
    let (open, close) = {
        let (open, close) = markers(window.w_onebuf_opt.wo_fmr.bytes());
        (open.to_vec(), close.to_vec())
    };
    fold_add_marker(buf, start, &open);
    fold_add_marker(buf, end, &close);
    changed_lines(buf, start.lnum, 0, end.lnum, 0, false);
    buf_updates_send_changes(buf, start.lnum, num_changed, num_changed);
}

/// 'foldmarker' (`foldmarker`, the option's bytes) cut into its two halves
/// where [`parse_marker`] last measured them.
///
/// The lengths are the stored ones rather than a fresh split, as upstream
/// reads `foldstartmarkerlen`; a cut past the end of the value is clamped.
pub(super) fn markers(foldmarker: &[u8]) -> (&[u8], &[u8]) {
    let start_len = foldstartmarkerlen.get().min(foldmarker.len());
    let rest = foldmarker.get(start_len + 1..).unwrap_or_default();
    let end_len = foldendmarkerlen.get().min(rest.len());
    (&foldmarker[..start_len], &rest[..end_len])
}

/// Add `marker` in 'commentstring' to position `pos`.
pub(super) fn fold_add_marker(buffer: Buf, pos: Pos, marker: &[u8]) {
    let lnum = pos.lnum;
    // 'commentstring', and where in it the marker's text goes.
    // A copy: `u_save` may run autocommands, which may set 'commentstring'.
    let cms = buffer.b_p_cms.get();
    let text_at = cms.windows(2).position(|w| w == b"%s");
    if u_save(lnum - 1, lnum + 1).is_err() {
        return;
    }
    // A copy, because the line is rewritten from it below.
    let text = Lines::in_buffer(buffer).line_copy(lnum);
    // Does the line already end inside a comment?
    let (_, line_is_comment) = skip_comment(&text, false, false);

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
    let _ = ml_replace_buf_text(buffer, lnum, &new[..new.len() - 1]);
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

/// The lines whose markers make `fold` (and, with `recursive`, the folds
/// nested in it), as `(first, last)` buffer lines in the order they are
/// deleted: the nested folds first.
///
/// Collected before any marker goes: deleting one edits the buffer, which
/// runs `u_save` and so maybe autocommands, and the tree is not held across
/// that. Deleting a marker never moves a line, so the numbers stay good.
///
/// `lnum_off` — offset for `fold.top`
pub(super) fn marker_spans(
    fold: &Fold,
    recursive: bool,
    lnum_off: LineNr,
) -> Vec<(LineNr, LineNr)> {
    fn collect(fold: &Fold, recursive: bool, lnum_off: LineNr, out: &mut Vec<(LineNr, LineNr)>) {
        if recursive {
            for child in &fold.nested {
                collect(child, true, lnum_off + fold.top, out);
            }
        }
        out.push((fold.top + lnum_off, fold.last() + lnum_off));
    }
    let mut spans = Vec::new();
    collect(fold, recursive, lnum_off, &mut spans);
    spans
}

/// Delete the markers at each `(first, last)` of `spans`, causing those
/// folds to be deleted.
pub(super) fn delete_fold_markers(window: Win, spans: &[(LineNr, LineNr)]) {
    for &(first, last) in spans {
        // Re-read per fold, as upstream reads 'foldmarker' per call; copied
        // because the deletion may run autocommands.
        let (open, close) = {
            let (open, close) = markers(window.w_onebuf_opt.wo_fmr.bytes());
            (open.to_vec(), close.to_vec())
        };
        fold_del_marker(window.buffer(), first, &open);
        fold_del_marker(window.buffer(), last, &close);
    }
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
    // A copy: `u_save` may run autocommands, which may set 'commentstring'.
    let cms = buffer.b_p_cms.get();
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
    let _ = ml_replace_buf_text(buffer, lnum, &new[..new.len() - 1]);
    extmark_splice_cols(
        buffer,
        lnum as c_int - 1,
        start as ColNr,
        len as ColNr,
        0,
        kExtmarkUndo,
    );
}

/// Parse 'foldmarker' and set "foldstartmarkerlen" and "foldendmarkerlen".
/// The end marker starts just past the comma, at `foldstartmarkerlen + 1`.
/// Relies on the option value to have been checked for correctness already.
///
/// The lengths describe the value as it is now, so they go stale the
/// moment the option is set again — which is why every caller re-runs this.
///
pub(super) fn parse_marker(window: Win) {
    let foldmarker = window.w_onebuf_opt.wo_fmr.bytes();
    // 'foldmarker' has already been validated as two non-empty halves
    // separated by a comma.
    let comma = foldmarker
        .iter()
        .position(|&b| b == b',')
        .unwrap_or(foldmarker.len());
    foldstartmarkerlen.set(comma);
    let end = foldmarker.get(comma + 1..).unwrap_or_default();
    foldendmarkerlen.set(end.iter().position(|&b| b == 0).unwrap_or(end.len()));
}

/// Low level function to get the foldlevel for the "marker" method.
/// "foldstartmarkerlen" and "foldendmarkerlen" must have been
/// set before calling this.
/// Requires that flp->lvl is set to the fold level of the previous line!
/// Careful: This means you can't call this function twice on the same line.
/// Doesn't use any caching.
/// Sets flp->start when a start marker was found.
pub(super) fn foldlevel_marker(line: &mut FoldLine) {
    let window = line.win;
    let start_lvl = line.lvl;
    let (startmarker, endmarker) = markers(window.w_onebuf_opt.wo_fmr.bytes());
    line.start = 0;
    line.lvl_next = line.lvl;

    let buffer = window.buffer();
    let lnum = line.lnum + line.off;
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
                Some(n) => {
                    line.lvl = n;
                    line.lvl_next = n;
                    line.start = if n - start_lvl > 1 { n - start_lvl } else { 1 };
                }
                None => {
                    line.lvl += 1;
                    line.lvl_next += 1;
                    line.start += 1;
                }
            }
        } else if rest.first() == endmarker.first() && rest.starts_with(endmarker) {
            at += endmarker.len();
            match marker_number(&text[at..]) {
                Some(n) => {
                    line.lvl = n;
                    line.lvl_next = (n - 1).min(start_lvl);
                }
                None => line.lvl_next -= 1,
            }
        } else {
            at += cluster_len(rest);
        }
    }
    line.lvl_next = line.lvl_next.max(0);
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
