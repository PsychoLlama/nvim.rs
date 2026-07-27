use crate::src::nvim::buffer_updates::buf_updates_send_changes;
use crate::src::nvim::change::changed_lines;
use crate::src::nvim::extmark::extmark_splice_cols;
use crate::src::nvim::main::e_modifiable;
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len, ml_replace_buf};
use crate::src::nvim::memory::{xmalloc, xmemcpyz};
use crate::src::nvim::message::emsg;
use crate::src::nvim::ops::skip_comment;
use crate::src::nvim::os::libc::{atoi, gettext, memcpy, strcpy, strlen, strncmp, strstr};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::undo::u_save;
use core::ffi::{c_char, c_int, c_void};

use super::*;

/// Create a fold from line "start" to line "end" (inclusive) in window `wp`
/// by adding markers.
pub(super) unsafe extern "C" fn foldCreateMarkers(
    mut wp: *mut win_T,
    mut start: pos_T,
    mut end: pos_T,
) {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if (*buf).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const c_char));
        return;
    }
    parseMarker(wp);
    foldAddMarker(
        buf,
        start,
        (*wp).w_onebuf_opt.wo_fmr,
        foldstartmarkerlen.get(),
    );
    foldAddMarker(buf, end, foldendmarker.get(), foldendmarkerlen.get());
    changed_lines(buf, start.lnum, 0, end.lnum, 0, false);
    let mut num_changed: int64_t = (1 + end.lnum - start.lnum) as int64_t;
    buf_updates_send_changes(buf, start.lnum, num_changed, num_changed);
}

/// Add "marker[markerlen]" in 'commentstring' to position `pos`.
pub(super) unsafe extern "C" fn foldAddMarker(
    mut buf: *mut buf_T,
    mut pos: pos_T,
    mut marker: *const c_char,
    mut markerlen: size_t,
) {
    let mut cms: *mut c_char = (*buf).b_p_cms;
    let mut p: *mut c_char = strstr((*buf).b_p_cms, c"%s".as_ptr());
    let mut line_is_comment: bool = false;
    let mut lnum: linenr_T = pos.lnum;
    let mut line: *mut c_char = ml_get_buf(buf, lnum);
    let mut line_len: size_t = ml_get_buf_len(buf, lnum) as size_t;
    let mut added: size_t = 0;
    if u_save(lnum - 1, lnum + 1) != OK {
        return;
    }
    skip_comment(line, false, false, &raw mut line_is_comment);
    let mut newline: *mut c_char = xmalloc(
        line_len
            .wrapping_add(markerlen)
            .wrapping_add(strlen(cms))
            .wrapping_add(1),
    ) as *mut c_char;
    strcpy(newline, line);
    if p.is_null() || line_is_comment {
        xmemcpyz(
            newline.add(line_len) as *mut c_void,
            marker as *const c_void,
            markerlen,
        );
        added = markerlen;
    } else {
        strcpy(newline.add(line_len), cms);
        memcpy(
            newline.add(line_len).offset(p.offset_from(cms)) as *mut c_void,
            marker as *const c_void,
            markerlen,
        );
        strcpy(
            newline
                .add(line_len)
                .offset(p.offset_from(cms))
                .add(markerlen),
            p.offset(2),
        );
        added = markerlen.wrapping_add(strlen(cms)).wrapping_sub(2);
    }
    ml_replace_buf(buf, lnum, newline, false, false);
    if added != 0 {
        extmark_splice_cols(
            buf,
            lnum as c_int - 1,
            line_len as colnr_T,
            0,
            added as colnr_T,
            kExtmarkUndo,
        );
    }
}

/// Delete the markers for a fold, causing it to be deleted.
///
/// `lnum_off` — offset for fp->fd_top
pub(super) unsafe extern "C" fn deleteFoldMarkers(
    mut wp: *mut win_T,
    mut fp: *mut fold_T,
    mut recursive: bool,
    mut lnum_off: linenr_T,
) {
    if recursive {
        let mut i: c_int = 0;
        while i < (*fp).fd_nested.ga_len {
            deleteFoldMarkers(
                wp,
                fold_at(&(*fp).fd_nested, i),
                true,
                lnum_off + (*fp).fd_top,
            );
            i += 1;
        }
    }
    foldDelMarker(
        (*wp).w_buffer,
        (*fp).fd_top + lnum_off,
        (*wp).w_onebuf_opt.wo_fmr,
        foldstartmarkerlen.get(),
    );
    foldDelMarker(
        (*wp).w_buffer,
        (*fp).fd_top + lnum_off + (*fp).fd_len - 1,
        foldendmarker.get(),
        foldendmarkerlen.get(),
    );
}

/// Delete marker "marker[markerlen]" at the end of line "lnum".
/// Delete 'commentstring' if it matches.
/// If the marker is not found, there is no error message.  Could be a missing
/// close-marker.
pub(super) unsafe extern "C" fn foldDelMarker(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut marker: *mut c_char,
    mut markerlen: size_t,
) {
    if lnum > (*buf).b_ml.ml_line_count {
        return;
    }
    let mut cms: *mut c_char = (*buf).b_p_cms;
    let mut line: *mut c_char = ml_get_buf(buf, lnum);
    let mut p: *mut c_char = line;
    while *p as c_int != NUL {
        if strncmp(p, marker, markerlen) != 0 {
            p = p.offset(1);
        } else {
            let mut len: size_t = markerlen;
            if ascii_isdigit(*p.add(len) as c_int) {
                len = len.wrapping_add(1);
            }
            if *cms as c_int != NUL {
                let mut cms2: *mut c_char = strstr(cms, c"%s".as_ptr());
                if !cms2.is_null()
                    && p.offset_from(line) >= cms2.offset_from(cms)
                    && strncmp(
                        p.offset(-(cms2.offset_from(cms))),
                        cms,
                        cms2.offset_from(cms) as size_t,
                    ) == 0
                    && strncmp(p.add(len), cms2.offset(2), strlen(cms2.offset(2))) == 0
                {
                    p = p.offset(-(cms2.offset_from(cms)));
                    len = len.wrapping_add(strlen(cms).wrapping_sub(2));
                }
            }
            if u_save(lnum - 1, lnum + 1) == OK {
                let mut newline: *mut c_char = xmalloc(
                    (ml_get_buf_len(buf, lnum) as size_t)
                        .wrapping_sub(len)
                        .wrapping_add(1),
                ) as *mut c_char;
                assert!(p >= line, "p >= line");
                memcpy(
                    newline as *mut c_void,
                    line as *const c_void,
                    p.offset_from(line) as size_t,
                );
                strcpy(newline.offset(p.offset_from(line)), p.add(len));
                ml_replace_buf(buf, lnum, newline, false, false);
                extmark_splice_cols(
                    buf,
                    lnum as c_int - 1,
                    p.offset_from(line) as colnr_T,
                    len as colnr_T,
                    0,
                    kExtmarkUndo,
                );
            }
            break;
        }
    }
}

/// Parse 'foldmarker' and set "foldendmarker", "foldstartmarkerlen" and
/// "foldendmarkerlen".
/// Relies on the option value to have been checked for correctness already.
pub(super) unsafe extern "C" fn parseMarker(mut wp: *mut win_T) {
    foldendmarker.set(vim_strchr((*wp).w_onebuf_opt.wo_fmr, ',' as c_int));
    let c2rust_fresh0 = foldendmarker.get();
    foldendmarker.set((*foldendmarker.ptr()).offset(1));
    foldstartmarkerlen.set(c2rust_fresh0.offset_from((*wp).w_onebuf_opt.wo_fmr) as size_t);
    foldendmarkerlen.set(strlen(foldendmarker.get()));
}

/// Low level function to get the foldlevel for the "marker" method.
/// "foldendmarker", "foldstartmarkerlen" and "foldendmarkerlen" must have been
/// set before calling this.
/// Requires that flp->lvl is set to the fold level of the previous line!
/// Careful: This means you can't call this function twice on the same line.
/// Doesn't use any caching.
/// Sets flp->start when a start marker was found.
pub(super) unsafe extern "C" fn foldlevelMarker(mut flp: *mut fline_T) {
    let mut start_lvl: c_int = (*flp).lvl;
    let mut startmarker: *mut c_char = (*(*flp).wp).w_onebuf_opt.wo_fmr;
    let mut cstart: c_char = *startmarker;
    startmarker = startmarker.offset(1);
    let mut cend: c_char = *foldendmarker.get();
    (*flp).start = 0;
    (*flp).lvl_next = (*flp).lvl;
    let mut s: *mut c_char = ml_get_buf((*(*flp).wp).w_buffer, (*flp).lnum + (*flp).off);
    while *s != 0 {
        if *s as c_int == cstart as c_int
            && strncmp(
                s.offset(1),
                startmarker,
                (*foldstartmarkerlen.ptr()).wrapping_sub(1),
            ) == 0
        {
            s = s.add(foldstartmarkerlen.get());
            if ascii_isdigit(*s as c_int) {
                let mut n: c_int = atoi(s);
                if n > 0 {
                    (*flp).lvl = n;
                    (*flp).lvl_next = n;
                    (*flp).start = if n - start_lvl > 1 { n - start_lvl } else { 1 };
                }
            } else {
                (*flp).lvl += 1;
                (*flp).lvl_next += 1;
                (*flp).start += 1;
            }
        } else if *s as c_int == cend as c_int
            && strncmp(
                s.offset(1),
                (*foldendmarker.ptr()).offset(1),
                (*foldendmarkerlen.ptr()).wrapping_sub(1),
            ) == 0
        {
            s = s.add(foldendmarkerlen.get());
            if ascii_isdigit(*s as c_int) {
                let mut n_0: c_int = atoi(s);
                if n_0 > 0 {
                    (*flp).lvl = n_0;
                    (*flp).lvl_next = n_0 - 1;
                    (*flp).lvl_next = if (*flp).lvl_next < start_lvl {
                        (*flp).lvl_next
                    } else {
                        start_lvl
                    };
                }
            } else {
                (*flp).lvl_next -= 1;
            }
        } else {
            s = s.offset(utfc_ptr2len(s) as isize);
        }
    }
    (*flp).lvl_next = if (*flp).lvl_next > 0 {
        (*flp).lvl_next
    } else {
        0
    };
}
