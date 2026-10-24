//! Recomputing a tabpage's diff.
//!
//! [`ex_diffupdate`] is `:diffupdate`; [`diff_try_update`] is the body it and
//! every implicit recompute share -- write each buffer out
//! ([`diff_write_buffer`] for the internal engine, [`diff_write`] through a
//! temp file for the external one), run the diff, read the hunks back.
//!
//! `'diffanchors'` is implemented here rather than in the engine: the
//! buffers are split at the anchor lines and each segment is diffed on its
//! own, then the resulting block lists are shifted back into place and
//! concatenated.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_flags, cmdmod_set_flags};
use crate::mbyte::{c_cluster_len, char_at, encode_char};
use crate::memline::MlFlags;
use crate::types::{Failed, NUL};
use crate::winlayer::{Buf, TabPage, Win};
use core::ffi::CStr;
use core::ffi::c_int;

/// Release one side of a diff: the temp file if there is one, else the
/// memory image.
fn clear_diffin(din: &mut DiffIn) {
    match &din.fname {
        None => din.text = Vec::new(),
        Some(name) => {
            os_remove(name.as_cstr());
        }
    }
}

/// Release the diff's output: the temp file if there is one, else the hunk
/// array.
fn clear_diffout(dout: &mut DiffOut) {
    match &dout.fname {
        None => dout.hunks = Vec::new(),
        Some(name) => {
            os_remove(name.as_cstr());
        }
    }
}

/// Write lines `start`..`end` of `buffer` into `out` as one memory image for
/// the internal engine.
///
/// The image is one NL-terminated line after another.  A NL *inside* a line
/// stands for a NUL byte in the file -- `ml_get_buf` answers the two swapped
/// -- and is written back as one, so the terminators are unambiguous.
/// `icase` is applied here, by folding each character, because xdiff has no
/// flag for it.
pub(crate) fn diff_write_buffer(buffer: Buf, out: &mut Vec<u8>, start: LineNr, mut end: LineNr) {
    out.clear();
    if end < 0 {
        end = buffer.b_ml.ml_line_count;
    }
    if buffer.b_ml.ml_flags.has(MlFlags::EMPTY) || end < start {
        return;
    }

    let mut lines = buffer.lines();
    let len = (start..=end)
        .map(|lnum| lines.line(lnum).len() + 1)
        .sum::<usize>();
    out.resize(len, 0);

    let mut at = 0;
    for lnum in start..=end {
        let line = lines.line(lnum);
        if diff_flags.get() & DIFF_ICASE == 0 {
            let copy = &mut out[at..at + line.len()];
            copy.copy_from_slice(line);
            for byte in copy.iter_mut().filter(|byte| **byte == b'\n') {
                *byte = 0;
            }
            at += line.len();
        } else {
            at += fold_line(line, &mut out[at..]);
        }
        out[at] = b'\n';
        at += 1;
    }
}

/// Copy `line` into `out` with every character case-folded, answering how
/// many bytes were written.
///
/// Always exactly `line.len()`: where the folded form is a different length
/// from the original the *original* is written instead, which is what keeps
/// the caller's precomputed allocation exact.  A NUL byte stands for a NL,
/// and folds to itself.
fn fold_line(line: &[u8], out: &mut [u8]) -> usize {
    let mut at = 0;
    while at < line.len() {
        // The line's tail, read as the C string it is the end of.
        let rest = &line[at..];
        let (folded, c_len) = if line[at] == b'\n' {
            (NUL, 1)
        } else {
            let c = char_at(rest);
            (utf_fold(c), usize::try_from(utf_char2len(c)).unwrap_or(1))
        };
        let orig_len = c_cluster_len(rest);
        // MB_MAXBYTES + 1.
        let mut cbuf = [0u8; 22];
        let same_len = encode_char(folded, &mut cbuf) == c_len;
        if same_len {
            out[at..at + c_len].copy_from_slice(&cbuf[..c_len]);
            if orig_len > c_len {
                // Composing characters follow; they are not folded.
                out[at + c_len..at + orig_len].copy_from_slice(&line[at + c_len..at + orig_len]);
            }
        } else {
            out[at..at + orig_len].copy_from_slice(&line[at..at + orig_len]);
        }
        at += orig_len;
    }
    at
}

/// Write lines `start`..`end` of `buffer` out for the diff.
///
/// The internal engine wants a memory image, which is what `din.fname`
/// being `None` selects; otherwise the lines go through a temp file.
fn diff_write(
    mut buffer: Buf,
    din: &mut DiffIn,
    start: LineNr,
    mut end: LineNr,
) -> Result<(), Failed> {
    let Some(name) = &din.fname else {
        diff_write_buffer(buffer, &mut din.text, start, end);
        return Ok(());
    };
    // Writing a buffer runs `aucmd_prepbuf`/`aucmd_restbuf`, which can
    // change the window layout -- and re-entering `winframe_remove` is a
    // use after free.
    if frames_locked() {
        return Err(Failed);
    }
    if end < 0 {
        end = buffer.b_ml.ml_line_count;
    }

    let was_empty = buffer.b_ml.ml_flags.masked(MlFlags::EMPTY);
    // The diff must see the file the way the buffer holds it.
    let save_ff = buffer.b_p_ff.replace(XString::from_cstr(c"unix"));
    // Writing the buffer is an implementation detail of the diff, so it
    // must not move the '[ and '] marks.
    //
    // Upstream saves the whole `cmod_flags` bit set into a `bool` and
    // restores it from there, so every flag that was set comes back as
    // the single bit 1. Reproduced; see O-B15-17.
    let save_cmod_flags = !cmdmod_flags().is_empty();
    cmdmod_add_flags(CmdModFlags::LOCKMARKS);
    if end < start {
        // The range names a completely empty file.
        end = start;
        buffer.b_ml.ml_flags |= MlFlags::EMPTY;
    }
    let req = WriteRequest::filter();
    // No short name and no `ExArg` are wanted.
    let r = buf_write(
        buffer,
        WriteName::Other(name.as_cstr()),
        None,
        start,
        end,
        None,
        req,
    );
    cmdmod_set_flags(CmdModFlags::SANDBOX.when(save_cmod_flags));
    // The forced "unix" goes when the saved value moves back in.
    buffer.b_p_ff = save_ff;
    buffer.b_ml.ml_flags = buffer.b_ml.ml_flags.without(MlFlags::EMPTY) | was_empty;
    r
}

/// Recompute the current tabpage's blocks with `idx_orig` as the reference
/// buffer.
///
/// Every other buffer is diffed against that one in turn, and `diff_read`
/// merges each answer into the shared block list.  With `'diffopt'`'s
/// `anchor` the whole thing runs once per segment between anchors, and the
/// segments' block lists are shifted back into place and chained together.
///
fn diff_try_update(dio: &mut DiffIo, idx_orig: c_int, excmd: Option<&mut ExArg>) {
    let tp = TabPage::current();
    let idx_orig = usize::try_from(idx_orig).unwrap_or(usize::MAX);
    let mut anchors = [[0 as LineNr; MAX_DIFF_ANCHORS as usize]; DB_COUNT as usize];
    'theend: {
        if dio.internal {
            dio.diff.hunks.clear();
        } else {
            dio.orig.fname = temp_name();
            dio.new.fname = temp_name();
            dio.diff.fname = temp_name();
            if dio
                .temp_names()
                .is_none_or(|names| check_external_diff(names).is_err())
            {
                break 'theend;
            }
        }

        // `:diffupdate!` re-reads any buffer that changed on disk first.
        let forceit = excmd.as_deref().is_some_and(|command| command.forceit);
        if forceit {
            for idx in idx_orig..DB_COUNT as usize {
                // A diff buffer may already have been wiped by an earlier
                // turn's autocommands; the slot's id answers `None` for it,
                // and a valid buffer is what `buf_check_timestamp` wants.
                if let Some(buf) = tp.diffbuf(idx) {
                    buf_check_timestamp(buf);
                }
            }
        }

        // Every buffer has to supply the same number of anchors, or the
        // segments would not line up; the smallest count wins, and a
        // buffer whose `'diffanchors'` does not resolve cancels them all.
        let mut num_anchors = c_int::MAX;
        if diff_flags.get() & DIFF_ANCHOR != 0 {
            for idx in 0..DB_COUNT as usize {
                if tp.tp_diffbuf[idx].is_none() {
                    continue;
                }
                let found = parse_diffanchors(false, tp.used_diffbuf(idx), Some(&mut anchors[idx]));
                let Ok(buf_num_anchors) = found else {
                    emsg(gettext(e_failed_to_find_all_diff_anchors));
                    num_anchors = 0;
                    anchors = [[0; MAX_DIFF_ANCHORS as usize]; DB_COUNT as usize];
                    break;
                };
                num_anchors = num_anchors.min(buf_num_anchors);
                if buf_num_anchors > 0 {
                    anchors[idx][..usize::try_from(buf_num_anchors).unwrap_or(0)].sort_unstable();
                }
            }
        }
        if num_anchors == c_int::MAX {
            num_anchors = 0;
        }

        // One diff per segment: `[1, a1)`, `[a1, a2)`, … `[aN, end]`.
        let num_anchors = usize::try_from(num_anchors).unwrap_or(0);
        for anchor_i in 0..=num_anchors {
            let segment = |idx: usize| {
                (
                    if anchor_i == 0 {
                        1
                    } else {
                        anchors[idx][anchor_i - 1]
                    },
                    if anchor_i == num_anchors {
                        -1
                    } else {
                        anchors[idx][anchor_i] - 1
                    },
                )
            };
            // Each segment builds its own list, which is appended to the
            // ones before it once its line numbers have been corrected.
            let earlier = if anchor_i == 0 {
                Vec::new()
            } else {
                tp.with_diff(::core::mem::take)
            };

            let (start, end) = segment(idx_orig);
            let wrote = diff_write(tp.used_diffbuf(idx_orig), &mut dio.orig, start, end);
            if wrote.is_err() {
                if anchor_i != 0 {
                    // Upstream puts the earlier segments back only to
                    // clear the lot.
                    drop(earlier);
                    diff_clear(tp);
                }
                break 'theend;
            }
            for idx_new in idx_orig + 1..DB_COUNT as usize {
                let Some(buf) = tp.diffbuf(idx_new).filter(|b| b.b_ml.ml_mfp.is_some()) else {
                    continue;
                };
                let (start, end) = segment(idx_new);
                if diff_write(buf, &mut dio.new, start, end).is_ok() && diff_file(dio).is_ok() {
                    if let Some(hunks) = HunkSource::open(dio) {
                        let current = TabPage::current();
                        let slots = current.tp_diffbuf;
                        current.with_diff(|blocks| {
                            diff_read(blocks, &slots, idx_orig, idx_new, hunks);
                        });
                    }
                    clear_diffin(&mut dio.new);
                    clear_diffout(&mut dio.diff);
                }
            }
            clear_diffin(&mut dio.orig);

            if anchor_i == 0 {
                continue;
            }
            // This segment's diff was computed over lines starting at 1;
            // shift it down to where the segment really begins, and put the
            // earlier segments in front of it.
            tp.with_diff(|blocks| {
                for dp in blocks.iter_mut() {
                    for (idx, row) in anchors.iter().enumerate() {
                        let anchor = row[anchor_i - 1];
                        if anchor > 0 {
                            dp.df_lnum[idx] += anchor - 1;
                        }
                    }
                }
                let this_segment = ::core::mem::replace(blocks, earlier);
                blocks.extend(this_segment);
            });
        }
    }
    dio.orig.fname = None;
    dio.new.fname = None;
    dio.diff.fname = None;
}

/// Whether the built-in diff engine is what a recompute would use.
///
/// `'diffexpr'` overrides `'diffopt'`'s `internal`.
pub fn diff_internal() -> c_int {
    // SAFETY: `p_dex` is the `'diffexpr'` option string.
    let no_diffexpr = p_dex(CStr::is_empty);
    c_int::from(diff_flags.get() & DIFF_INTERNAL != 0 && no_diffexpr)
}

/// `:diffupdate`.
pub fn ex_diffupdate(excmd: &mut ExArg) {
    diff_update(Some(excmd));
}

/// Recompute the tab page's diffs.
///
/// Split from the handler because most recomputes are implicit — a fold
/// opening, a scroll, a `:diffget` finishing — and have no Ex command to
/// hand down to 'diffexpr'.
pub(crate) fn diff_update(excmd: Option<&mut ExArg>) {
    // A recompute asked for from inside `:diffget`/`:diffput` is deferred
    // to that command's tail, where `diff_need_update` is read.
    if diff_busy.get() {
        diff_need_update.set(true);
        return;
    }
    let mut tp = TabPage::current();
    let had_diffs = !tp.tp_diff.is_empty();
    diff_clear(tp);
    tp.tp_diff_invalid = 0;

    // The first two buffers in the tabpage: everything is diffed against
    // the first, so there is nothing to do without a second.
    let first_two = (0..DB_COUNT)
        .find(|&i| tp.tp_diffbuf[i.cast_unsigned() as usize].is_some())
        .filter(|&idx_orig| {
            (idx_orig + 1..DB_COUNT).any(|i| tp.tp_diffbuf[i.cast_unsigned() as usize].is_some())
        });
    if let Some(idx_orig) = first_two {
        let mut diffio = DiffIo {
            internal: diff_internal() != 0,
            ..DiffIo::default()
        };
        diff_try_update(&mut diffio, idx_orig, excmd);
        Win::current().w_valid_cursor.lnum = 0;
    }

    if had_diffs || !tp.tp_diff.is_empty() {
        diff_redraw(true);
        let buffer = Buf::current_or_none();
        apply_autocmds(AutoEvent::DiffUpdated, None, None, false, buffer);
    }
}
