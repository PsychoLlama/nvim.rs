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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::memline::MlFlags;
use crate::types::{FAIL, NUL, OK};
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

/// Release one side of a diff: the temp file if there is one, else the
/// memory image.
unsafe fn clear_diffin(din: *mut diffin_T) {
    unsafe {
        if (*din).din_fname.is_null() {
            xfree((*din).din_mmfile.ptr.cast());
            (*din).din_mmfile.ptr = ::core::ptr::null_mut();
        } else {
            os_remove((*din).din_fname);
        }
    }
}

/// Release the diff's output: the temp file if there is one, else the hunk
/// array.
pub(crate) unsafe fn clear_diffout(dout: *mut diffout_T) {
    unsafe {
        if (*dout).dout_fname.is_null() {
            ga_clear(&raw mut (*dout).dout_ga);
        } else {
            os_remove((*dout).dout_fname);
        }
    }
}

/// Write lines `start`..`end` of `buf` into a memory image for `xdl_diff`.
///
/// The image is one NL-terminated line after another.  A NL *inside* a line
/// stands for a NUL byte in the file -- `ml_get_buf` answers the two swapped
/// -- and is written back as one, so the terminators are unambiguous.
/// `icase` is applied here, by folding each character, because xdiff has no
/// flag for it.
pub(crate) unsafe fn diff_write_buffer(
    buf: *mut buf_T,
    m: *mut mmfile_t,
    start: linenr_T,
    mut end: linenr_T,
) -> c_int {
    unsafe {
        if end < 0 {
            end = (*buf).b_ml.ml_line_count;
        }
        if (*buf).b_ml.ml_flags.has(MlFlags::EMPTY) || end < start {
            *m = MMFILE_INIT;
            return OK;
        }

        let len = (start..=end)
            .map(|lnum| ml_get_buf_len(buf, lnum) as usize + 1)
            .sum::<usize>();
        let ptr = xmalloc(len) as *mut c_char;
        *m = mmfile_t {
            ptr,
            size: len as c_int,
        };
        let out = ::core::slice::from_raw_parts_mut(ptr.cast::<u8>(), len);

        let mut at = 0;
        for lnum in start..=end {
            let line = CStr::from_ptr(ml_get_buf(buf, lnum)).to_bytes();
            if diff_flags.get() & DIFF_ICASE == 0 {
                out[at..at + line.len()].copy_from_slice(line);
                memchrsub(
                    out.as_mut_ptr().add(at).cast(),
                    NL as c_char,
                    NUL as c_char,
                    line.len(),
                );
                at += line.len();
            } else {
                at += fold_line(line, &mut out[at..]);
            }
            out[at] = NL as u8;
            at += 1;
        }
        OK
    }
}

/// Copy `line` into `out` with every character case-folded, answering how
/// many bytes were written.
///
/// Always exactly `line.len()`: where the folded form is a different length
/// from the original the *original* is written instead, which is what keeps
/// the caller's precomputed allocation exact.  A NUL byte stands for a NL,
/// and folds to itself.
unsafe fn fold_line(line: &[u8], out: &mut [u8]) -> usize {
    unsafe {
        let mut at = 0;
        while at < line.len() {
            let s = line.as_ptr().add(at).cast::<c_char>();
            let (folded, c_len) = if line[at] == NL as u8 {
                (NUL, 1)
            } else {
                let c = utf_ptr2char(s);
                (utf_fold(c), utf_char2len(c))
            };
            let orig_len = utfc_ptr2len(s) as usize;
            // MB_MAXBYTES + 1.
            let mut cbuf = [0u8; 22];
            if utf_char2bytes(folded, cbuf.as_mut_ptr().cast()) == c_len {
                let c_len = c_len as usize;
                out[at..at + c_len].copy_from_slice(&cbuf[..c_len]);
                if orig_len > c_len {
                    // Composing characters follow; they are not folded.
                    out[at + c_len..at + orig_len]
                        .copy_from_slice(&line[at + c_len..at + orig_len]);
                }
            } else {
                out[at..at + orig_len].copy_from_slice(&line[at..at + orig_len]);
            }
            at += orig_len;
        }
        at
    }
}

/// Write lines `start`..`end` of `buf` out for the external diff.
///
/// The internal engine wants a memory image, which is what `din_fname` being
/// NULL selects; otherwise the lines go through a temp file.
unsafe fn diff_write(
    buf: *mut buf_T,
    din: *mut diffin_T,
    start: linenr_T,
    mut end: linenr_T,
) -> c_int {
    unsafe {
        if (*din).din_fname.is_null() {
            return diff_write_buffer(buf, &raw mut (*din).din_mmfile, start, end);
        }
        // Writing a buffer runs `aucmd_prepbuf`/`aucmd_restbuf`, which can
        // change the window layout -- and re-entering `winframe_remove` is a
        // use after free.
        if frames_locked() {
            return FAIL;
        }
        if end < 0 {
            end = (*buf).b_ml.ml_line_count;
        }

        let was_empty = (*buf).b_ml.ml_flags.masked(MlFlags::EMPTY);
        let save_ff = (*buf).b_p_ff;
        // The diff must see the file the way the buffer holds it.
        (*buf).b_p_ff = xstrdup(c"unix".as_ptr());
        // Writing the buffer is an implementation detail of the diff, so it
        // must not move the '[ and '] marks.
        //
        // Upstream saves the whole `cmod_flags` bit set into a `bool` and
        // restores it from there, so every flag that was set comes back as
        // the single bit 1. Reproduced; see O-B15-17.
        let save_cmod_flags = !(*cmdmod.ptr()).cmod_flags.is_empty();
        (*cmdmod.ptr()).cmod_flags |= CmdModFlags::LOCKMARKS;
        if end < start {
            // The range names a completely empty file.
            end = start;
            (*buf).b_ml.ml_flags |= MlFlags::EMPTY;
        }
        let r = buf_write(
            buf,
            (*din).din_fname,
            ::core::ptr::null_mut(),
            start,
            end,
            ::core::ptr::null_mut(),
            WriteRequest::filter(),
        );
        (*cmdmod.ptr()).cmod_flags = CmdModFlags::SANDBOX.when(save_cmod_flags);
        free_string_option((*buf).b_p_ff);
        (*buf).b_p_ff = save_ff;
        (*buf).b_ml.ml_flags = (*buf).b_ml.ml_flags.without(MlFlags::EMPTY) | was_empty;
        r
    }
}

/// Recompute the current tabpage's blocks with `idx_orig` as the reference
/// buffer.
///
/// Every other buffer is diffed against that one in turn, and `diff_read`
/// merges each answer into the shared block list.  With `'diffopt'`'s
/// `anchor` the whole thing runs once per segment between anchors, and the
/// segments' block lists are shifted back into place and chained together.
unsafe fn diff_try_update(dio: *mut diffio_T, idx_orig: c_int, eap: *mut exarg_T) {
    unsafe {
        let tp = curtab.get();
        let idx_orig = idx_orig as usize;
        let mut anchors = [[0 as linenr_T; MAX_DIFF_ANCHORS as usize]; DB_COUNT as usize];
        'theend: {
            if (*dio).dio_internal != 0 {
                ga_init(
                    &raw mut (*dio).dio_diff.dout_ga,
                    ::core::mem::size_of::<diffhunk_T>() as c_int,
                    100,
                );
            } else {
                (*dio).dio_orig.din_fname = vim_tempname();
                (*dio).dio_new.din_fname = vim_tempname();
                (*dio).dio_diff.dout_fname = vim_tempname();
                if (*dio).dio_orig.din_fname.is_null()
                    || (*dio).dio_new.din_fname.is_null()
                    || (*dio).dio_diff.dout_fname.is_null()
                    || check_external_diff(dio) == FAIL
                {
                    break 'theend;
                }
            }

            // `:diffupdate!` re-reads any buffer that changed on disk first.
            if !eap.is_null() && (*eap).forceit != 0 {
                for idx in idx_orig..DB_COUNT as usize {
                    let buf = (*tp).tp_diffbuf[idx];
                    if buf_valid(buf) {
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
                    if (*tp).tp_diffbuf[idx].is_null() {
                        continue;
                    }
                    let mut buf_num_anchors = 0;
                    if parse_diffanchors(
                        false,
                        (*tp).tp_diffbuf[idx],
                        anchors[idx].as_mut_ptr(),
                        &raw mut buf_num_anchors,
                    ) != OK
                    {
                        emsg(gettext(
                            &raw const e_failed_to_find_all_diff_anchors as *const c_char,
                        ));
                        num_anchors = 0;
                        anchors = [[0; MAX_DIFF_ANCHORS as usize]; DB_COUNT as usize];
                        break;
                    }
                    num_anchors = num_anchors.min(buf_num_anchors);
                    if buf_num_anchors > 0 {
                        anchors[idx][..buf_num_anchors as usize].sort_unstable();
                    }
                }
            }
            if num_anchors == c_int::MAX {
                num_anchors = 0;
            }

            // One diff per segment: `[1, a1)`, `[a1, a2)`, … `[aN, end]`.
            for anchor_i in 0..=num_anchors as usize {
                let segment = |idx: usize| {
                    (
                        if anchor_i == 0 {
                            1
                        } else {
                            anchors[idx][anchor_i - 1]
                        },
                        if anchor_i == num_anchors as usize {
                            -1
                        } else {
                            anchors[idx][anchor_i] - 1
                        },
                    )
                };
                // Each segment builds its own list, which is appended to the
                // ones before it once its line numbers have been corrected.
                let orig_diff = if anchor_i == 0 {
                    ::core::ptr::null_mut()
                } else {
                    let head = (*tp).tp_first_diff;
                    (*tp).tp_first_diff = ::core::ptr::null_mut();
                    head
                };

                let (start, end) = segment(idx_orig);
                if diff_write(
                    (*tp).tp_diffbuf[idx_orig],
                    &raw mut (*dio).dio_orig,
                    start,
                    end,
                ) == FAIL
                {
                    if !orig_diff.is_null() {
                        (*tp).tp_first_diff = orig_diff;
                        diff_clear(tp);
                    }
                    break 'theend;
                }
                for idx_new in idx_orig + 1..DB_COUNT as usize {
                    let buf = (*tp).tp_diffbuf[idx_new];
                    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
                        continue;
                    }
                    let (start, end) = segment(idx_new);
                    if diff_write(buf, &raw mut (*dio).dio_new, start, end) != FAIL
                        && diff_file(dio) != FAIL
                    {
                        diff_read(idx_orig as c_int, idx_new as c_int, dio);
                        clear_diffin(&raw mut (*dio).dio_new);
                        clear_diffout(&raw mut (*dio).dio_diff);
                    }
                }
                clear_diffin(&raw mut (*dio).dio_orig);

                if anchor_i == 0 {
                    continue;
                }
                // This segment's diff was computed over lines starting at 1;
                // shift it down to where the segment really begins.
                let mut dp = (*tp).tp_first_diff;
                while !dp.is_null() {
                    for (idx, row) in anchors.iter().enumerate() {
                        let anchor = row[anchor_i - 1];
                        if anchor > 0 {
                            (*dp).df_lnum[idx] += anchor - 1;
                        }
                    }
                    dp = (*dp).df_next;
                }
                if !orig_diff.is_null() {
                    let mut last = orig_diff;
                    while !(*last).df_next.is_null() {
                        last = (*last).df_next;
                    }
                    (*last).df_next = (*tp).tp_first_diff;
                    (*tp).tp_first_diff = orig_diff;
                }
            }
        }
        xfree((*dio).dio_orig.din_fname.cast());
        xfree((*dio).dio_new.din_fname.cast());
        xfree((*dio).dio_diff.dout_fname.cast());
    }
}

/// Whether the built-in diff engine is what a recompute would use.
///
/// `'diffexpr'` overrides `'diffopt'`'s `internal`.
pub unsafe fn diff_internal() -> c_int {
    unsafe { c_int::from(diff_flags.get() & DIFF_INTERNAL != 0 && *p_dex.get() == 0) }
}

/// `:diffupdate`, and every implicit recompute.
pub unsafe fn ex_diffupdate(eap: *mut exarg_T) {
    unsafe {
        // A recompute asked for from inside `:diffget`/`:diffput` is deferred
        // to that command's tail, where `diff_need_update` is read.
        if diff_busy.get() {
            diff_need_update.set(true);
            return;
        }
        let tp = curtab.get();
        let had_diffs = !(*tp).tp_first_diff.is_null();
        diff_clear(tp);
        (*tp).tp_diff_invalid = 0;

        // The first two buffers in the tabpage: everything is diffed against
        // the first, so there is nothing to do without a second.
        let first_two = (0..DB_COUNT)
            .find(|&i| !(*tp).tp_diffbuf[i as usize].is_null())
            .filter(|&idx_orig| {
                (idx_orig + 1..DB_COUNT).any(|i| !(*tp).tp_diffbuf[i as usize].is_null())
            });
        if let Some(idx_orig) = first_two {
            let mut diffio = diffio_T {
                dio_orig: DIFFIN_INIT,
                dio_new: DIFFIN_INIT,
                dio_diff: diffout_T {
                    dout_fname: ::core::ptr::null_mut(),
                    dout_ga: GA_EMPTY_INIT_VALUE,
                },
                dio_internal: diff_internal(),
            };
            diff_try_update(&raw mut diffio, idx_orig, eap);
            (*curwin.get()).w_valid_cursor.lnum = 0;
        }

        if had_diffs || !(*tp).tp_first_diff.is_null() {
            diff_redraw(true);
            apply_autocmds(
                EVENT_DIFFUPDATED,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                curbuf.get(),
            );
        }
    }
}
