//! 'foldmethod' = "marker": folds that live in the buffer text as `{{{` and
//! `}}}`.
//!
//! This is the only fold method whose folds are *stored*, so creating and
//! deleting one edits the buffer (and is undoable). 'foldmarker' is split
//! into its two halves by [`parse_marker`], which every function here
//! requires to have run.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::ascii_isdigit;
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::extmark::extmark_splice_cols;
use crate::main::e_modifiable;
use crate::mbyte::utfc_ptr2len;
use crate::memline::{ml_get_buf, ml_get_buf_len, ml_replace_buf};
use crate::memory::{xmalloc, xmemcpyz};
use crate::message::emsg;
use crate::ops::skip_comment;
use crate::os::cshim::{gettext, strncmp, strstr};
use crate::strings::vim_strchr;
use crate::undo::u_save;
use ::libc::{atoi, memcpy, strcpy, strlen};
use core::ffi::{c_char, c_int, c_void};

use super::*;

use crate::winlayer::Buf;
/// Create a fold from line "start" to line "end" (inclusive) in window `wp`
/// by adding markers.
///
/// # Safety
/// `wp` must have a live buffer, and `start`/`end` must be lines inside it.
pub(super) unsafe fn fold_create_markers(wp: Win, start: pos_T, end: pos_T) {
    let buf = wp.w_buffer;
    // SAFETY: a live buffer.
    if unsafe { (*buf).b_p_ma } == 0 {
        // SAFETY: a static message.
        unsafe { emsg(gettext(e_modifiable.as_ptr())) };
        return;
    }
    let num_changed = (1 + end.lnum - start.lnum) as int64_t;
    // SAFETY: the caller's promise; both lines are inside the buffer.
    parse_marker(wp);
    unsafe { fold_add_marker(buf, start, wp.w_onebuf_opt.wo_fmr, foldstartmarkerlen.get()) };
    unsafe { fold_add_marker(buf, end, foldendmarker.get(), foldendmarkerlen.get()) };
    changed_lines(unsafe { Buf::new(buf) }, start.lnum, 0, end.lnum, 0, false);
    unsafe { buf_updates_send_changes(buf, start.lnum, num_changed, num_changed) };
}

/// Add "marker[markerlen]" in 'commentstring' to position `pos`.
///
/// # Safety
/// `buf` must be a live buffer, `pos` a line inside it, and
/// `marker[..markerlen]` readable.
pub(super) unsafe fn fold_add_marker(
    buf: *mut buf_T,
    pos: pos_T,
    marker: *const c_char,
    markerlen: size_t,
) {
    let lnum = pos.lnum;
    // SAFETY: the caller's promise.
    let cms = unsafe { (*buf).b_p_cms };
    // Where 'commentstring' puts the text, if it has a place for it.
    let p = unsafe { strstr(cms, c"%s".as_ptr()) };
    let line = unsafe { ml_get_buf(buf, lnum) };
    let line_len = unsafe { ml_get_buf_len(buf, lnum) } as size_t;
    if u_save(lnum - 1, lnum + 1) != OK {
        return;
    }
    let mut line_is_comment = false;
    unsafe { skip_comment(line, false, false, &raw mut line_is_comment) };
    let newline = unsafe {
        xmalloc(
            line_len
                .wrapping_add(markerlen)
                .wrapping_add(strlen(cms))
                .wrapping_add(1),
        )
    } as *mut c_char;
    unsafe { strcpy(newline, line) };
    let added = if p.is_null() || line_is_comment {
        // No '%s' in 'commentstring', or the line already is a comment:
        // the marker goes on bare.
        unsafe {
            xmemcpyz(
                newline.add(line_len) as *mut c_void,
                marker as *const c_void,
                markerlen,
            )
        };
        markerlen
    } else {
        unsafe { strcpy(newline.add(line_len), cms) };
        unsafe {
            memcpy(
                newline.add(line_len).offset(p.offset_from(cms)) as *mut c_void,
                marker as *const c_void,
                markerlen,
            )
        };
        unsafe {
            strcpy(
                newline
                    .add(line_len)
                    .offset(p.offset_from(cms))
                    .add(markerlen),
                p.offset(2),
            )
        };
        markerlen
            .wrapping_add(unsafe { strlen(cms) })
            .wrapping_sub(2)
    };
    unsafe { ml_replace_buf(buf, lnum, newline, false, false) };
    if added != 0 {
        unsafe {
            extmark_splice_cols(
                buf,
                lnum as c_int - 1,
                line_len as colnr_T,
                0,
                added as colnr_T,
                kExtmarkUndo,
            )
        };
    }
}

/// Delete the markers for a fold, causing it to be deleted.
///
/// `lnum_off` — offset for fold.top()
///
/// # Safety
/// `fold` must be one of `wp`'s folds at `lnum_off`, and [`parse_marker`]
/// must have run for `wp`.
pub(super) unsafe fn delete_fold_markers(wp: Win, fold: Fold, recursive: bool, lnum_off: linenr_T) {
    if recursive {
        for child in fold.nested().folds() {
            // SAFETY: the caller's promise, one level down.
            unsafe { delete_fold_markers(wp, child, true, lnum_off + fold.top()) };
        }
    }
    // SAFETY: the caller's promise.
    unsafe {
        fold_del_marker(
            wp.w_buffer,
            fold.top() + lnum_off,
            wp.w_onebuf_opt.wo_fmr,
            foldstartmarkerlen.get(),
        )
    };
    unsafe {
        fold_del_marker(
            wp.w_buffer,
            fold.last() + lnum_off,
            foldendmarker.get(),
            foldendmarkerlen.get(),
        )
    };
}

/// Delete marker "marker[markerlen]" at the end of line "lnum".
/// Delete 'commentstring' if it matches.
/// If the marker is not found, there is no error message.  Could be a missing
/// close-marker.
///
/// # Safety
/// `buf` must be a live buffer and `marker[..markerlen]` readable.
pub(super) unsafe fn fold_del_marker(
    buf: *mut buf_T,
    lnum: linenr_T,
    marker: *mut c_char,
    markerlen: size_t,
) {
    // SAFETY: the caller's promise.
    if lnum > unsafe { (*buf).b_ml.ml_line_count } {
        return;
    }
    // SAFETY: the caller's promise; `line` is NUL-terminated, so the walk
    // below stops inside it.
    let cms = unsafe { (*buf).b_p_cms };
    let line = unsafe { ml_get_buf(buf, lnum) };
    let mut p = line;
    while unsafe { *p } as c_int != NUL {
        if unsafe { strncmp(p, marker, markerlen) } != 0 {
            p = unsafe { p.offset(1) };
            continue;
        }
        let mut len = markerlen;
        // A numbered marker, `{{{2`.
        if ascii_isdigit(unsafe { *p.add(len) } as c_int) {
            len = len.wrapping_add(1);
        }
        if unsafe { *cms } as c_int != NUL {
            // The marker may be wrapped in 'commentstring'; if it is, the
            // comment goes with it.
            let cms2 = unsafe { strstr(cms, c"%s".as_ptr()) };
            if !cms2.is_null()
                && unsafe { p.offset_from(line) } >= unsafe { cms2.offset_from(cms) }
                && unsafe {
                    strncmp(
                        p.offset(-(cms2.offset_from(cms))),
                        cms,
                        cms2.offset_from(cms) as size_t,
                    )
                } == 0
                && unsafe { strncmp(p.add(len), cms2.offset(2), strlen(cms2.offset(2))) } == 0
            {
                p = unsafe { p.offset(-(cms2.offset_from(cms))) };
                len = len.wrapping_add(unsafe { strlen(cms) }.wrapping_sub(2));
            }
        }
        if u_save(lnum - 1, lnum + 1) == OK {
            let newline = unsafe {
                xmalloc(
                    (ml_get_buf_len(buf, lnum) as size_t)
                        .wrapping_sub(len)
                        .wrapping_add(1),
                )
            } as *mut c_char;
            debug_assert!(p >= line, "p >= line");
            unsafe {
                memcpy(
                    newline as *mut c_void,
                    line as *const c_void,
                    p.offset_from(line) as size_t,
                )
            };
            unsafe { strcpy(newline.offset(p.offset_from(line)), p.add(len)) };
            unsafe { ml_replace_buf(buf, lnum, newline, false, false) };
            unsafe {
                extmark_splice_cols(
                    buf,
                    lnum as c_int - 1,
                    p.offset_from(line) as colnr_T,
                    len as colnr_T,
                    0,
                    kExtmarkUndo,
                )
            };
        }
        break;
    }
}

/// Parse 'foldmarker' and set "foldendmarker", "foldstartmarkerlen" and
/// "foldendmarkerlen".
/// Relies on the option value to have been checked for correctness already.
///
/// Note that `foldendmarker` points *into* 'foldmarker', so it dangles the
/// moment the option is set again — which is why every caller re-runs this.
///
pub(super) fn parse_marker(wp: Win) {
    let foldmarker = wp.w_onebuf_opt.wo_fmr;
    // SAFETY: 'foldmarker' has already been validated as two non-empty
    // halves separated by a comma, so the comma is there.
    let comma = unsafe { vim_strchr(foldmarker, ',' as c_int) };
    foldstartmarkerlen.set(unsafe { comma.offset_from(foldmarker) } as size_t);
    let end = unsafe { comma.offset(1) };
    foldendmarker.set(end);
    foldendmarkerlen.set(unsafe { strlen(end) });
}

/// Low level function to get the foldlevel for the "marker" method.
/// "foldendmarker", "foldstartmarkerlen" and "foldendmarkerlen" must have been
/// set before calling this.
/// Requires that flp->lvl is set to the fold level of the previous line!
/// Careful: This means you can't call this function twice on the same line.
/// Doesn't use any caching.
/// Sets flp->start when a start marker was found.
///
/// # Safety
/// `line` must name a line inside its window's buffer, and [`parse_marker`]
/// must have run for that window.
pub(super) unsafe fn foldlevel_marker(line: FLine) {
    let flp = line.raw();
    // SAFETY: the caller's promise; the line is NUL-terminated, so the scan
    // below stops inside it.
    let start_lvl = unsafe { (*flp).lvl };
    let startmarker = unsafe { (*(*flp).wp).w_onebuf_opt.wo_fmr };
    let cstart = unsafe { *startmarker };
    let cend = unsafe { *foldendmarker.get() };
    unsafe { (*flp).start = 0 };
    unsafe { (*flp).lvl_next = (*flp).lvl };
    let mut s = unsafe { ml_get_buf((*(*flp).wp).w_buffer, (*flp).lnum + (*flp).off) };
    while unsafe { *s } != 0 {
        if unsafe { *s } as c_int == cstart as c_int
            && unsafe {
                strncmp(
                    s.offset(1),
                    startmarker.offset(1),
                    foldstartmarkerlen.get().wrapping_sub(1),
                )
            } == 0
        {
            s = unsafe { s.add(foldstartmarkerlen.get()) };
            if ascii_isdigit(unsafe { *s } as c_int) {
                // `{{{N` sets the level outright.
                let n = unsafe { atoi(s) };
                if n > 0 {
                    unsafe { (*flp).lvl = n };
                    unsafe { (*flp).lvl_next = n };
                    unsafe { (*flp).start = if n - start_lvl > 1 { n - start_lvl } else { 1 } };
                }
            } else {
                unsafe { (*flp).lvl += 1 };
                unsafe { (*flp).lvl_next += 1 };
                unsafe { (*flp).start += 1 };
            }
        } else if unsafe { *s } as c_int == cend as c_int
            && unsafe {
                strncmp(
                    s.offset(1),
                    foldendmarker.get().offset(1),
                    foldendmarkerlen.get().wrapping_sub(1),
                )
            } == 0
        {
            s = unsafe { s.add(foldendmarkerlen.get()) };
            if ascii_isdigit(unsafe { *s } as c_int) {
                let n = unsafe { atoi(s) };
                if n > 0 {
                    unsafe { (*flp).lvl = n };
                    unsafe { (*flp).lvl_next = (n - 1).min(start_lvl) };
                }
            } else {
                unsafe { (*flp).lvl_next -= 1 };
            }
        } else {
            s = unsafe { s.offset(utfc_ptr2len(s) as isize) };
        }
    }
    unsafe { (*flp).lvl_next = (*flp).lvl_next.max(0) };
}
