//! Byte offsets: `line2byte`, `byte2line` and `'go'`'s
//! character count.
//!
//! The tree does not store byte offsets, so `ml_find_line_or_offset` walks it
//! adding up block sizes. `ml_updatechunk` maintains the `b_ml.ml_chunksize`
//! accelerator that keeps that walk from being O(lines) on every call.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ml_updatechunk(
    mut buf: *mut buf_T,
    mut line: linenr_T,
    mut len: ::core::ffi::c_int,
    mut updtype: ::core::ffi::c_int,
) {
    unsafe {
        static ml_upd_lastbuf: GlobalCell<*mut buf_T> =
            GlobalCell::new(::core::ptr::null_mut::<buf_T>());
        static ml_upd_lastline: GlobalCell<linenr_T> = GlobalCell::new(0);
        static ml_upd_lastcurline: GlobalCell<linenr_T> = GlobalCell::new(0);
        static ml_upd_lastcurix: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        let mut curline: linenr_T = ml_upd_lastcurline.get();
        let mut curix: ::core::ffi::c_int = ml_upd_lastcurix.get();
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        if (*buf).b_ml.ml_usedchunks == -1 as ::core::ffi::c_int || len == 0 as ::core::ffi::c_int {
            return;
        }
        if (*buf).b_ml.ml_chunksize.is_null() {
            (*buf).b_ml.ml_chunksize =
                xmalloc(::core::mem::size_of::<chunksize_T>().wrapping_mul(100 as size_t))
                    as *mut chunksize_T;
            (*buf).b_ml.ml_numchunks = 100 as ::core::ffi::c_int;
            (*buf).b_ml.ml_usedchunks = 1 as ::core::ffi::c_int;
            (*(*buf)
                .b_ml
                .ml_chunksize
                .offset(0 as ::core::ffi::c_int as isize))
            .mlcs_numlines = 1 as ::core::ffi::c_int;
            (*(*buf)
                .b_ml
                .ml_chunksize
                .offset(0 as ::core::ffi::c_int as isize))
            .mlcs_totalsize = 1 as ::core::ffi::c_int;
        }
        if updtype == ML_CHNK_UPDLINE && (*buf).b_ml.ml_line_count == 1 as linenr_T {
            (*buf).b_ml.ml_usedchunks = 1 as ::core::ffi::c_int;
            (*(*buf)
                .b_ml
                .ml_chunksize
                .offset(0 as ::core::ffi::c_int as isize))
            .mlcs_numlines = 1 as ::core::ffi::c_int;
            (*(*buf)
                .b_ml
                .ml_chunksize
                .offset(0 as ::core::ffi::c_int as isize))
            .mlcs_totalsize = (*buf).b_ml.ml_line_textlen as ::core::ffi::c_int;
            return;
        }
        if buf != ml_upd_lastbuf.get()
            || line != ml_upd_lastline.get() + 1 as linenr_T
            || updtype != ML_CHNK_ADDLINE
        {
            curline = 1 as ::core::ffi::c_int as linenr_T;
            curix = 0 as ::core::ffi::c_int;
            while curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
                && line
                    >= curline
                        + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines
                            as linenr_T
            {
                curline = (curline as ::core::ffi::c_int
                    + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
                    as linenr_T;
                curix += 1;
            }
        } else if curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
            && line
                >= curline
                    + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines as linenr_T
        {
            curline = (curline as ::core::ffi::c_int
                + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
                as linenr_T;
            curix += 1;
        }
        let mut curchnk: *mut chunksize_T = (*buf).b_ml.ml_chunksize.offset(curix as isize);
        if updtype == ML_CHNK_DELLINE {
            len = -len;
        }
        (*curchnk).mlcs_totalsize += len;
        if updtype == ML_CHNK_ADDLINE {
            let mut rest: ::core::ffi::c_int = 0;
            let mut dp: *mut DataBlock = ::core::ptr::null_mut::<DataBlock>();
            (*curchnk).mlcs_numlines += 1;
            if (*buf).b_ml.ml_usedchunks + 1 as ::core::ffi::c_int >= (*buf).b_ml.ml_numchunks {
                (*buf).b_ml.ml_numchunks =
                    (*buf).b_ml.ml_numchunks * 3 as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
                (*buf).b_ml.ml_chunksize = xrealloc(
                    (*buf).b_ml.ml_chunksize as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<chunksize_T>()
                        .wrapping_mul((*buf).b_ml.ml_numchunks as size_t),
                ) as *mut chunksize_T;
            }
            if (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines
                >= MLCS_MAXL as ::core::ffi::c_int
            {
                let mut end_idx: ::core::ffi::c_int = 0;
                let mut text_end: ::core::ffi::c_int = 0;
                memmove(
                    (*buf)
                        .b_ml
                        .ml_chunksize
                        .offset(curix as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    (*buf).b_ml.ml_chunksize.offset(curix as isize) as *const ::core::ffi::c_void,
                    (((*buf).b_ml.ml_usedchunks - curix) as size_t)
                        .wrapping_mul(::core::mem::size_of::<chunksize_T>()),
                );
                let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut linecnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while curline < (*buf).b_ml.ml_line_count
                    && linecnt < MLCS_MINL as ::core::ffi::c_int
                {
                    hp = ml_find_line(buf, curline, ML_FIND as ::core::ffi::c_int);
                    if hp.is_null() {
                        (*buf).b_ml.ml_usedchunks = -1 as ::core::ffi::c_int;
                        return;
                    }
                    dp = (*hp).bh_data as *mut DataBlock;
                    let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high
                        as ::core::ffi::c_int
                        - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    let mut idx: ::core::ffi::c_int = curline as ::core::ffi::c_int
                        - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
                    curline = (*buf).b_ml.ml_locked_high + 1 as linenr_T;
                    rest = count - idx;
                    if linecnt + rest > MLCS_MINL as ::core::ffi::c_int {
                        end_idx = idx + MLCS_MINL as ::core::ffi::c_int
                            - linecnt
                            - 1 as ::core::ffi::c_int;
                        linecnt = MLCS_MINL as ::core::ffi::c_int;
                    } else {
                        end_idx = count - 1 as ::core::ffi::c_int;
                        linecnt += rest;
                    }
                    if idx == 0 as ::core::ffi::c_int {
                        text_end = (*dp).db_txt_end as ::core::ffi::c_int;
                    } else {
                        text_end = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((idx - 1 as ::core::ffi::c_int) as isize)
                            & DB_INDEX_MASK)
                            as ::core::ffi::c_int;
                    }
                    size += text_end
                        - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(end_idx as isize)
                            & DB_INDEX_MASK) as ::core::ffi::c_int;
                }
                (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines = linecnt;
                (*(*buf)
                    .b_ml
                    .ml_chunksize
                    .offset((curix + 1 as ::core::ffi::c_int) as isize))
                .mlcs_numlines -= linecnt;
                (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_totalsize = size;
                (*(*buf)
                    .b_ml
                    .ml_chunksize
                    .offset((curix + 1 as ::core::ffi::c_int) as isize))
                .mlcs_totalsize -= size;
                (*buf).b_ml.ml_usedchunks += 1;
                ml_upd_lastbuf.set(::core::ptr::null_mut::<buf_T>());
                return;
            } else if (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines
                >= MLCS_MINL as ::core::ffi::c_int
                && curix == (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
                && (*buf).b_ml.ml_line_count - line <= 1 as linenr_T
            {
                curchnk = (*buf)
                    .b_ml
                    .ml_chunksize
                    .offset(curix as isize)
                    .offset(1 as ::core::ffi::c_int as isize);
                (*buf).b_ml.ml_usedchunks += 1;
                if line == (*buf).b_ml.ml_line_count {
                    (*curchnk).mlcs_numlines = 0 as ::core::ffi::c_int;
                    (*curchnk).mlcs_totalsize = 0 as ::core::ffi::c_int;
                } else {
                    hp = ml_find_line(
                        buf,
                        (*buf).b_ml.ml_line_count,
                        ML_FIND as ::core::ffi::c_int,
                    );
                    if hp.is_null() {
                        (*buf).b_ml.ml_usedchunks = -1 as ::core::ffi::c_int;
                        return;
                    }
                    dp = (*hp).bh_data as *mut DataBlock;
                    if (*dp).db_line_count == 1 as ::core::ffi::c_long {
                        rest =
                            (*dp).db_txt_end.wrapping_sub((*dp).db_txt_start) as ::core::ffi::c_int;
                    } else {
                        rest = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(((*dp).db_line_count - 2 as ::core::ffi::c_long) as isize)
                            & DB_INDEX_MASK) as ::core::ffi::c_int
                            - (*dp).db_txt_start as ::core::ffi::c_int;
                    }
                    (*curchnk).mlcs_totalsize = rest;
                    (*curchnk).mlcs_numlines = 1 as ::core::ffi::c_int;
                    (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_totalsize -= rest;
                    (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_numlines -=
                        1 as ::core::ffi::c_int;
                }
            }
        } else if updtype == ML_CHNK_DELLINE {
            (*curchnk).mlcs_numlines -= 1;
            ml_upd_lastbuf.set(::core::ptr::null_mut::<buf_T>());
            if curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
                && (*curchnk).mlcs_numlines
                    + (*curchnk.offset(1 as ::core::ffi::c_int as isize)).mlcs_numlines
                    <= MLCS_MINL as ::core::ffi::c_int
            {
                curix += 1;
                curchnk = (*buf).b_ml.ml_chunksize.offset(curix as isize);
            } else if curix == 0 as ::core::ffi::c_int
                && (*curchnk).mlcs_numlines <= 0 as ::core::ffi::c_int
            {
                (*buf).b_ml.ml_usedchunks -= 1;
                memmove(
                    (*buf).b_ml.ml_chunksize as *mut ::core::ffi::c_void,
                    (*buf)
                        .b_ml
                        .ml_chunksize
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    ((*buf).b_ml.ml_usedchunks as size_t)
                        .wrapping_mul(::core::mem::size_of::<chunksize_T>()),
                );
                return;
            } else if curix == 0 as ::core::ffi::c_int
                || (*curchnk).mlcs_numlines > 10 as ::core::ffi::c_int
                    && (*curchnk).mlcs_numlines
                        + (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_numlines
                        > MLCS_MINL as ::core::ffi::c_int
            {
                return;
            }
            (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_numlines +=
                (*curchnk).mlcs_numlines;
            (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_totalsize +=
                (*curchnk).mlcs_totalsize;
            (*buf).b_ml.ml_usedchunks -= 1;
            if curix < (*buf).b_ml.ml_usedchunks {
                memmove(
                    (*buf).b_ml.ml_chunksize.offset(curix as isize) as *mut ::core::ffi::c_void,
                    (*buf)
                        .b_ml
                        .ml_chunksize
                        .offset(curix as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (((*buf).b_ml.ml_usedchunks - curix) as size_t)
                        .wrapping_mul(::core::mem::size_of::<chunksize_T>()),
                );
            }
            return;
        }
        ml_upd_lastbuf.set(buf);
        ml_upd_lastline.set(line);
        ml_upd_lastcurline.set(curline);
        ml_upd_lastcurix.set(curix);
    }
}

pub unsafe extern "C" fn ml_find_line_or_offset(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut offp: *mut ::core::ffi::c_int,
    mut no_ff: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        let mut text_end: ::core::ffi::c_int = 0;
        let mut offset: ::core::ffi::c_int = 0;
        let mut ffdos: ::core::ffi::c_int =
            (!no_ff && get_fileformat(buf) == EOL_DOS) as ::core::ffi::c_int;
        let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut can_cache: bool =
            lnum != 0 as linenr_T && ffdos == 0 && (*buf).b_ml.ml_line_lnum == lnum;
        if lnum == 0 as linenr_T || (*buf).b_ml.ml_line_lnum < lnum || !no_ff {
            ml_flush_line(curbuf.get(), false_0 != 0);
        } else if can_cache as ::core::ffi::c_int != 0 && (*buf).b_ml.ml_line_offset > 0 as size_t {
            return (*buf).b_ml.ml_line_offset as ::core::ffi::c_int;
        }
        if (*buf).b_ml.ml_usedchunks == -1 as ::core::ffi::c_int
            || (*buf).b_ml.ml_chunksize.is_null()
            || lnum < 0 as linenr_T
        {
            if no_ff as ::core::ffi::c_int != 0
                && !(*buf).b_ml.ml_mfp.is_null()
                && (lnum == 1 as linenr_T || lnum == 2 as linenr_T)
            {
                return lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            }
            return -1 as ::core::ffi::c_int;
        }
        if offp.is_null() {
            offset = 0 as ::core::ffi::c_int;
        } else {
            offset = *offp;
        }
        if lnum == 0 as linenr_T && offset <= 0 as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        let mut curline: linenr_T = 1 as linenr_T;
        let mut curix: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
            && (lnum != 0 as linenr_T
                && lnum
                    >= curline
                        + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines
                            as linenr_T
                || offset != 0 as ::core::ffi::c_int
                    && offset
                        > size
                            + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_totalsize
                            + ffdos
                                * (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
        {
            curline = (curline as ::core::ffi::c_int
                + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
                as linenr_T;
            size += (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_totalsize;
            if offset != 0 && ffdos != 0 {
                size += (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines;
            }
            curix += 1;
        }
        while lnum != 0 as linenr_T && curline < lnum
            || offset != 0 as ::core::ffi::c_int && size < offset
        {
            if curline > (*buf).b_ml.ml_line_count || {
                hp = ml_find_line(buf, curline, ML_FIND as ::core::ffi::c_int);
                hp.is_null()
            } {
                return -1 as ::core::ffi::c_int;
            }
            let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
            let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high as ::core::ffi::c_int
                - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int;
            let mut idx: ::core::ffi::c_int = 0;
            idx = (curline - (*buf).b_ml.ml_locked_low) as ::core::ffi::c_int;
            let mut start_idx: ::core::ffi::c_int = idx;
            if idx == 0 as ::core::ffi::c_int {
                text_end = (*dp).db_txt_end as ::core::ffi::c_int;
            } else {
                text_end = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                    .offset((idx - 1 as ::core::ffi::c_int) as isize)
                    & DB_INDEX_MASK) as ::core::ffi::c_int;
            }
            if lnum != 0 as linenr_T {
                if curline + (count as linenr_T - idx as linenr_T) >= lnum {
                    idx += (lnum - curline - 1 as linenr_T) as ::core::ffi::c_int;
                } else {
                    idx = count - 1 as ::core::ffi::c_int;
                }
            } else {
                extra = 0 as ::core::ffi::c_int;
                while offset
                    >= size + text_end
                        - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(idx as isize)
                            & DB_INDEX_MASK) as ::core::ffi::c_int
                        + ffdos
                {
                    if ffdos != 0 {
                        size += 1;
                    }
                    if idx == count - 1 as ::core::ffi::c_int {
                        extra = 1 as ::core::ffi::c_int;
                        break;
                    } else {
                        idx += 1;
                    }
                }
            }
            let mut len: ::core::ffi::c_int = text_end
                - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                    & DB_INDEX_MASK) as ::core::ffi::c_int;
            size += len;
            if offset != 0 as ::core::ffi::c_int && size >= offset {
                if size + ffdos == offset {
                    *offp = 0 as ::core::ffi::c_int;
                } else if idx == start_idx {
                    *offp = offset - size + len;
                } else {
                    *offp = offset - size + len
                        - (text_end
                            - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset((idx - 1 as ::core::ffi::c_int) as isize)
                                & DB_INDEX_MASK)
                                as ::core::ffi::c_int);
                }
                curline = (curline as ::core::ffi::c_int + (idx - start_idx + extra)) as linenr_T;
                if curline > (*buf).b_ml.ml_line_count {
                    return -1 as ::core::ffi::c_int;
                }
                return curline as ::core::ffi::c_int;
            }
            curline = (*buf).b_ml.ml_locked_high + 1 as linenr_T;
        }
        if lnum != 0 as linenr_T {
            if ffdos != 0 {
                size += (lnum - 1 as linenr_T) as ::core::ffi::c_int;
            }
            if ((*buf).b_p_fixeol == 0 || (*buf).b_p_bin != 0)
                && (*buf).b_p_eol == 0
                && lnum > (*buf).b_ml.ml_line_count
            {
                size -= ffdos + 1 as ::core::ffi::c_int;
            }
        }
        if can_cache as ::core::ffi::c_int != 0 && size > 0 as ::core::ffi::c_int {
            (*buf).b_ml.ml_line_offset = size as size_t;
        }
        return size;
    }
}

pub unsafe extern "C" fn goto_byte(mut cnt: ::core::ffi::c_int) {
    unsafe {
        let mut boff: ::core::ffi::c_int = cnt;
        ml_flush_line(curbuf.get(), false_0 != 0);
        setpcmark();
        if boff != 0 {
            boff -= 1;
        }
        let mut lnum: linenr_T =
            ml_find_line_or_offset(curbuf.get(), 0 as linenr_T, &raw mut boff, false_0 != 0)
                as linenr_T;
        if lnum < 1 as linenr_T {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        } else {
            (*curwin.get()).w_cursor.lnum = lnum;
            (*curwin.get()).w_cursor.col = boff;
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_set_curswant = true_0;
        }
        check_cursor(curwin.get());
        mb_adjust_cursor();
    }
}
