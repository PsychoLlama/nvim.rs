//! Walking the block tree, and reading a line out of it.
//!
//! A memline is a B-tree: pointer blocks branch by line count, and data blocks
//! hold the text. `ml_find_line` is the walk every read and write starts from,
//! and the one place the block stack in `ml_locked`/`ml_stack` is built.
//! `ml_get_buf_impl` is the read on top of it, and `ml_flush_line` the write
//! back of a line that `ml_replace` handed out.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ml_get_buf_impl(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut will_change: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static questions: GlobalCell<[::core::ffi::c_char; 4]> = GlobalCell::new([0; 4]);
        if (*buf).b_ml.ml_mfp.is_null() {
            (*buf).b_ml.ml_line_textlen = 1 as ::core::ffi::c_int as colnr_T;
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        '_errorret: {
            if lnum > (*buf).b_ml.ml_line_count {
                if recursive.get() == 0 as ::core::ffi::c_int {
                    (*recursive.ptr()) += 1;
                    siemsg(
                        gettext(
                            (e_ml_get_invalid_lnum_nr.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        lnum as int64_t,
                    );
                    (*recursive.ptr()) -= 1;
                }
                ml_flush_line(buf, false_0 != 0);
            } else {
                lnum = if lnum > 1 as linenr_T {
                    lnum
                } else {
                    1 as linenr_T
                };
                if (*buf).b_ml.ml_line_lnum != lnum {
                    ml_flush_line(buf, false_0 != 0);
                    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                    hp = ml_find_line(buf, lnum, ML_FIND as ::core::ffi::c_int);
                    if hp.is_null() {
                        if recursive.get() == 0 as ::core::ffi::c_int {
                            (*recursive.ptr()) += 1;
                            get_trans_bufname(buf);
                            shorten_dir(NameBuff.ptr() as *mut ::core::ffi::c_char);
                            siemsg(
                                gettext(
                                    (e_ml_get_cannot_find_line_nr_in_buffer_nr_str.ptr()
                                        as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                lnum as int64_t,
                                (*buf).handle,
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                            );
                            (*recursive.ptr()) -= 1;
                        }
                        break '_errorret;
                    } else {
                        let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
                        let mut idx: ::core::ffi::c_int = lnum as ::core::ffi::c_int
                            - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
                        let mut start: ::core::ffi::c_uint = *(&raw mut (*dp).db_index
                            as *mut ::core::ffi::c_uint)
                            .offset(idx as isize)
                            & DB_INDEX_MASK;
                        let mut end: ::core::ffi::c_uint = if idx == 0 as ::core::ffi::c_int {
                            (*dp).db_txt_end
                        } else {
                            *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset((idx - 1 as ::core::ffi::c_int) as isize)
                                & DB_INDEX_MASK
                        };
                        (*buf).b_ml.ml_line_ptr =
                            (dp as *mut ::core::ffi::c_char).offset(start as isize);
                        (*buf).b_ml.ml_line_textlen = end.wrapping_sub(start) as colnr_T;
                        (*buf).b_ml.ml_line_lnum = lnum;
                        (*buf).b_ml.ml_flags &= !(ML_LINE_DIRTY | ML_ALLOCATED);
                    }
                }
                if will_change {
                    (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
                    ml_add_deleted_len_buf(buf, (*buf).b_ml.ml_line_ptr, -1 as ssize_t);
                }
                return (*buf).b_ml.ml_line_ptr;
            }
        }
        strcpy(
            questions.ptr() as *mut ::core::ffi::c_char,
            b"???\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*buf).b_ml.ml_line_textlen = 4 as ::core::ffi::c_int as colnr_T;
        (*buf).b_ml.ml_line_lnum = lnum;
        return questions.ptr() as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn ml_flush_line(mut buf: *mut buf_T, mut noalloc: bool) {
    unsafe {
        static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if (*buf).b_ml.ml_line_lnum == 0 as linenr_T || (*buf).b_ml.ml_mfp.is_null() {
            return;
        }
        if (*buf).b_ml.ml_flags & ML_LINE_DIRTY != 0 {
            if entered.get() {
                return;
            }
            entered.set(true_0 != 0);
            (*buf).flush_count += 1;
            let mut lnum: linenr_T = (*buf).b_ml.ml_line_lnum;
            let mut new_line: *mut ::core::ffi::c_char = (*buf).b_ml.ml_line_ptr;
            let mut hp: *mut bhdr_T = ml_find_line(buf, lnum, ML_FIND as ::core::ffi::c_int);
            if hp.is_null() {
                siemsg(
                    gettext(b"E320: Cannot find line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    lnum as int64_t,
                );
            } else {
                let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
                let mut idx: ::core::ffi::c_int =
                    lnum as ::core::ffi::c_int - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
                let mut start: ::core::ffi::c_int =
                    (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                        & DB_INDEX_MASK) as ::core::ffi::c_int;
                let mut old_line: *mut ::core::ffi::c_char =
                    (dp as *mut ::core::ffi::c_char).offset(start as isize);
                let mut old_len: ::core::ffi::c_int = 0;
                if idx == 0 as ::core::ffi::c_int {
                    old_len = (*dp).db_txt_end as ::core::ffi::c_int - start;
                } else {
                    old_len = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset((idx - 1 as ::core::ffi::c_int) as isize)
                        & DB_INDEX_MASK) as ::core::ffi::c_int
                        - start;
                }
                let mut new_len: colnr_T = (*buf).b_ml.ml_line_textlen;
                let mut extra: ::core::ffi::c_int = new_len as ::core::ffi::c_int - old_len;
                if (*dp).db_free as ::core::ffi::c_int >= extra {
                    let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high
                        as ::core::ffi::c_int
                        - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    if extra != 0 as ::core::ffi::c_int && idx < count - 1 as ::core::ffi::c_int {
                        memmove(
                            (dp as *mut ::core::ffi::c_char)
                                .offset((*dp).db_txt_start as isize)
                                .offset(-(extra as isize))
                                as *mut ::core::ffi::c_void,
                            (dp as *mut ::core::ffi::c_char).offset((*dp).db_txt_start as isize)
                                as *const ::core::ffi::c_void,
                            (start - (*dp).db_txt_start as ::core::ffi::c_int) as size_t,
                        );
                        let mut i: ::core::ffi::c_int = idx + 1 as ::core::ffi::c_int;
                        while i < count {
                            *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset(i as isize) = (*(&raw mut (*dp).db_index
                                as *mut ::core::ffi::c_uint)
                                .offset(i as isize))
                            .wrapping_sub(extra as ::core::ffi::c_uint);
                            i += 1;
                        }
                    }
                    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize) =
                        (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(idx as isize))
                        .wrapping_sub(extra as ::core::ffi::c_uint);
                    (*dp).db_free = (*dp).db_free.wrapping_sub(extra as ::core::ffi::c_uint);
                    (*dp).db_txt_start = (*dp)
                        .db_txt_start
                        .wrapping_sub(extra as ::core::ffi::c_uint);
                    memmove(
                        old_line.offset(-(extra as isize)) as *mut ::core::ffi::c_void,
                        new_line as *const ::core::ffi::c_void,
                        new_len as size_t,
                    );
                    (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
                    if extra != 0 as ::core::ffi::c_int {
                        ml_updatechunk(buf, lnum, extra, ML_CHNK_UPDLINE);
                    }
                } else {
                    ml_append_int(
                        buf,
                        lnum,
                        new_line,
                        new_len,
                        if *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(idx as isize)
                            & DB_MARKED
                            != 0
                        {
                            ML_APPEND_MARK as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        },
                    );
                    ml_delete_int(buf, lnum, 0 as ::core::ffi::c_int);
                }
            }
            if !noalloc {
                xfree(new_line as *mut ::core::ffi::c_void);
            }
            entered.set(false_0 != 0);
        } else if (*buf).b_ml.ml_flags & ML_ALLOCATED != 0 {
            '_c2rust_label: {
                if !noalloc {
                } else {
                    __assert_fail(
                        b"!noalloc\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2969 as ::core::ffi::c_uint,
                        b"void ml_flush_line(buf_T *, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            xfree((*buf).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
        }
        (*buf).b_ml.ml_flags &= !(ML_LINE_DIRTY | ML_ALLOCATED);
        (*buf).b_ml.ml_line_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*buf).b_ml.ml_line_offset = 0 as size_t;
    }
}

pub(crate) unsafe extern "C" fn ml_new_data(
    mut mfp: *mut memfile_T,
    mut negative: bool,
    mut page_count: int64_t,
) -> *mut bhdr_T {
    unsafe {
        '_c2rust_label: {
            if page_count >= 0 as int64_t {
            } else {
                __assert_fail(
                    b"page_count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2981 as ::core::ffi::c_uint,
                    b"bhdr_T *ml_new_data(memfile_T *, _Bool, int64_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut hp: *mut bhdr_T = mf_new(mfp, negative, page_count as ::core::ffi::c_uint);
        let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
        (*dp).db_id = DATA_ID as ::core::ffi::c_int as uint16_t;
        (*dp).db_txt_end = (page_count as ::core::ffi::c_uint).wrapping_mul((*mfp).mf_page_size);
        (*dp).db_txt_start = (*dp).db_txt_end;
        (*dp).db_free = (*dp)
            .db_txt_start
            .wrapping_sub(HEADER_SIZE as ::core::ffi::c_uint);
        (*dp).db_line_count = 0 as ::core::ffi::c_long;
        return hp;
    }
}

pub(crate) unsafe extern "C" fn ml_new_ptr(mut mfp: *mut memfile_T) -> *mut bhdr_T {
    unsafe {
        let mut hp: *mut bhdr_T = mf_new(mfp, false_0 != 0, 1 as ::core::ffi::c_uint);
        let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
        (*pp).pb_id = PTR_ID as ::core::ffi::c_int as uint16_t;
        (*pp).pb_count = 0 as uint16_t;
        (*pp).pb_count_max = ((*mfp).mf_page_size as usize)
            .wrapping_sub(8 as usize)
            .wrapping_div(::core::mem::size_of::<PointerEntry>())
            as uint16_t;
        return hp;
    }
}

pub(crate) unsafe extern "C" fn ml_find_line(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut action: ::core::ffi::c_int,
) -> *mut bhdr_T {
    unsafe {
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        let mut top: ::core::ffi::c_int = 0;
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        if !(*buf).b_ml.ml_locked.is_null() {
            if action & 0x10 as ::core::ffi::c_int != 0
                && (*buf).b_ml.ml_locked_low <= lnum
                && (*buf).b_ml.ml_locked_high >= lnum
            {
                if action == ML_INSERT as ::core::ffi::c_int {
                    (*buf).b_ml.ml_locked_lineadd += 1;
                    (*buf).b_ml.ml_locked_high += 1;
                } else if action == ML_DELETE as ::core::ffi::c_int {
                    (*buf).b_ml.ml_locked_lineadd -= 1;
                    (*buf).b_ml.ml_locked_high -= 1;
                }
                return (*buf).b_ml.ml_locked;
            }
            mf_put(
                mfp,
                (*buf).b_ml.ml_locked,
                (*buf).b_ml.ml_flags & ML_LOCKED_DIRTY != 0,
                (*buf).b_ml.ml_flags & ML_LOCKED_POS != 0,
            );
            (*buf).b_ml.ml_locked = ::core::ptr::null_mut::<bhdr_T>();
            if (*buf).b_ml.ml_locked_lineadd != 0 as ::core::ffi::c_int {
                ml_lineadd(buf, (*buf).b_ml.ml_locked_lineadd);
            }
        }
        if action == ML_FLUSH as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<bhdr_T>();
        }
        let mut bnum: blocknr_T = 1 as blocknr_T;
        let mut bnum2: blocknr_T = 0;
        let mut page_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut low: linenr_T = 1 as linenr_T;
        let mut high: linenr_T = (*buf).b_ml.ml_line_count;
        if action == ML_FIND as ::core::ffi::c_int {
            top = (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
            while top >= 0 as ::core::ffi::c_int {
                let mut ip: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(top as isize);
                if (*ip).ip_low <= lnum && (*ip).ip_high >= lnum {
                    bnum = (*ip).ip_bnum;
                    low = (*ip).ip_low;
                    high = (*ip).ip_high;
                    (*buf).b_ml.ml_stack_top = top;
                    break;
                } else {
                    top -= 1;
                }
            }
            if top < 0 as ::core::ffi::c_int {
                (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
            }
        } else {
            (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
        }
        '_error_noblock: {
            loop {
                hp = mf_get(mfp, bnum, page_count as ::core::ffi::c_uint);
                if hp.is_null() {
                    break '_error_noblock;
                }
                if action == ML_INSERT as ::core::ffi::c_int {
                    high += 1;
                } else if action == ML_DELETE as ::core::ffi::c_int {
                    high -= 1;
                }
                let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
                if (*dp).db_id as ::core::ffi::c_int == DATA_ID as ::core::ffi::c_int {
                    (*buf).b_ml.ml_locked = hp;
                    (*buf).b_ml.ml_locked_low = low;
                    (*buf).b_ml.ml_locked_high = high;
                    (*buf).b_ml.ml_locked_lineadd = 0 as ::core::ffi::c_int;
                    (*buf).b_ml.ml_flags &= !(ML_LOCKED_DIRTY | ML_LOCKED_POS);
                    return hp;
                }
                let mut pp: *mut PointerBlock = dp as *mut PointerBlock;
                if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                    iemsg(gettext(
                        (e_pointer_block_id_wrong.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    break;
                } else {
                    top = ml_add_stack(buf);
                    let mut ip_0: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(top as isize);
                    (*ip_0).ip_bnum = bnum;
                    (*ip_0).ip_low = low;
                    (*ip_0).ip_high = high;
                    (*ip_0).ip_index = -1 as ::core::ffi::c_int;
                    let mut dirty: bool = false_0 != 0;
                    let mut idx: ::core::ffi::c_int = 0;
                    idx = 0 as ::core::ffi::c_int;
                    while idx < (*pp).pb_count as ::core::ffi::c_int {
                        let mut t: linenr_T = (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(idx as isize))
                        .pe_line_count;
                        low += t;
                        if low > lnum {
                            (*ip_0).ip_index = idx;
                            bnum = (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(idx as isize))
                            .pe_bnum;
                            page_count = (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(idx as isize))
                            .pe_page_count;
                            high = low - 1 as linenr_T;
                            low -= t;
                            if bnum < 0 as blocknr_T {
                                bnum2 = mf_trans_del(mfp, bnum);
                                if bnum != bnum2 {
                                    bnum = bnum2;
                                    (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset(idx as isize))
                                    .pe_bnum = bnum;
                                    dirty = true_0 != 0;
                                }
                            }
                            break;
                        } else {
                            idx += 1;
                        }
                    }
                    if idx >= (*pp).pb_count as ::core::ffi::c_int {
                        if lnum > (*buf).b_ml.ml_line_count {
                            siemsg(
                                gettext(
                                    (e_line_number_out_of_range_nr_past_the_end.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                lnum as int64_t - (*buf).b_ml.ml_line_count as int64_t,
                            );
                        } else {
                            siemsg(
                                gettext(
                                    (e_line_count_wrong_in_block_nr.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                bnum,
                            );
                        }
                        break;
                    } else {
                        if action == ML_DELETE as ::core::ffi::c_int {
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(idx as isize))
                            .pe_line_count -= 1;
                            dirty = true_0 != 0;
                        } else if action == ML_INSERT as ::core::ffi::c_int {
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(idx as isize))
                            .pe_line_count += 1;
                            dirty = true_0 != 0;
                        }
                        mf_put(mfp, hp, dirty, false_0 != 0);
                    }
                }
            }
            mf_put(mfp, hp, false_0 != 0, false_0 != 0);
        }
        if action == ML_DELETE as ::core::ffi::c_int {
            ml_lineadd(buf, 1 as ::core::ffi::c_int);
        } else if action == ML_INSERT as ::core::ffi::c_int {
            ml_lineadd(buf, -1 as ::core::ffi::c_int);
        }
        (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<bhdr_T>();
    }
}

pub(crate) unsafe extern "C" fn ml_add_stack(mut buf: *mut buf_T) -> ::core::ffi::c_int {
    unsafe {
        let mut top: ::core::ffi::c_int = (*buf).b_ml.ml_stack_top;
        if top == (*buf).b_ml.ml_stack_size {
            (*buf).b_ml.ml_stack_size += STACK_INCR;
            let mut new_size: size_t = ::core::mem::size_of::<infoptr_T>()
                .wrapping_mul((*buf).b_ml.ml_stack_size as size_t);
            (*buf).b_ml.ml_stack =
                xrealloc((*buf).b_ml.ml_stack as *mut ::core::ffi::c_void, new_size)
                    as *mut infoptr_T;
        }
        (*buf).b_ml.ml_stack_top += 1;
        return top;
    }
}

pub(crate) unsafe extern "C" fn ml_lineadd(mut buf: *mut buf_T, mut count: ::core::ffi::c_int) {
    unsafe {
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        let mut idx: ::core::ffi::c_int = (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
        while idx >= 0 as ::core::ffi::c_int {
            let mut ip: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(idx as isize);
            let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
            hp = mf_get(mfp, (*ip).ip_bnum, 1 as ::core::ffi::c_uint);
            if hp.is_null() {
                break;
            }
            let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
            if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                iemsg(gettext(
                    (e_pointer_block_id_wrong_two.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                break;
            } else {
                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                    .offset((*ip).ip_index as isize))
                .pe_line_count = ((*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                    .offset((*ip).ip_index as isize))
                .pe_line_count as ::core::ffi::c_int
                    + count) as linenr_T;
                (*ip).ip_high = ((*ip).ip_high as ::core::ffi::c_int + count) as linenr_T;
                mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                idx -= 1;
            }
        }
    }
}
