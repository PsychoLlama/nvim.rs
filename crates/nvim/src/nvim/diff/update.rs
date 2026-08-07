//! Recomputing a tabpage's diff.
//!
//! `ex_diffupdate` is `:diffupdate`; `diff_try_update` is the body it and every
//! implicit recompute share -- write each buffer out (`diff_write_buffer` for the
//! internal engine, `diff_write` through a temp file for the external one), run
//! the diff, read the hunks back.  It is also where the fall back from the
//! external diff to the internal one happens.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn clear_diffin(mut din: *mut diffin_T) {
    unsafe {
        if (*din).din_fname.is_null() {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*din).din_mmfile.ptr as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        } else {
            os_remove((*din).din_fname);
        };
    }
}

pub(crate) unsafe extern "C" fn clear_diffout(mut dout: *mut diffout_T) {
    unsafe {
        if (*dout).dout_fname.is_null() {
            ga_clear(&raw mut (*dout).dout_ga);
        } else {
            os_remove((*dout).dout_fname);
        };
    }
}

pub(crate) unsafe extern "C" fn diff_write_buffer(
    mut buf: *mut buf_T,
    mut m: *mut mmfile_t,
    mut start: linenr_T,
    mut end: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if end < 0 as linenr_T {
            end = (*buf).b_ml.ml_line_count;
        }
        if (*buf).b_ml.ml_flags & ML_EMPTY != 0 || end < start {
            (*m).ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*m).size = 0 as ::core::ffi::c_int;
            return OK;
        }
        let mut len: size_t = 0 as size_t;
        let mut lnum: linenr_T = start;
        while lnum <= end {
            len = len.wrapping_add((ml_get_buf_len(buf, lnum) as size_t).wrapping_add(1 as size_t));
            lnum += 1;
        }
        let mut ptr: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        (*m).ptr = ptr;
        (*m).size = len as ::core::ffi::c_int;
        len = 0 as size_t;
        let mut lnum_0: linenr_T = start;
        while lnum_0 <= end {
            let mut s: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum_0);
            if diff_flags.get() & DIFF_ICASE != 0 {
                while *s as ::core::ffi::c_int != NUL {
                    let mut c: ::core::ffi::c_int = 0;
                    let mut c_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let mut cbuf: [::core::ffi::c_char; 22] = [0; 22];
                    if *s as ::core::ffi::c_int == NL {
                        c = NUL;
                    } else {
                        c = utf_ptr2char(s);
                        c_len = utf_char2len(c);
                        c = utf_fold(c);
                    }
                    let orig_len: ::core::ffi::c_int = utfc_ptr2len(s);
                    if utf_char2bytes(c, &raw mut cbuf as *mut ::core::ffi::c_char) != c_len {
                        memmove(
                            ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                            s as *const ::core::ffi::c_void,
                            orig_len as size_t,
                        );
                    } else {
                        memmove(
                            ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                            &raw mut cbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                            c_len as size_t,
                        );
                        if orig_len > c_len {
                            memmove(
                                ptr.offset(len as isize).offset(c_len as isize)
                                    as *mut ::core::ffi::c_void,
                                s.offset(c_len as isize) as *const ::core::ffi::c_void,
                                (orig_len - c_len) as size_t,
                            );
                        }
                    }
                    s = s.offset(orig_len as isize);
                    len = len.wrapping_add(orig_len as size_t);
                }
            } else {
                let mut slen: size_t = strlen(s);
                memmove(
                    ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                    s as *const ::core::ffi::c_void,
                    slen,
                );
                memchrsub(
                    ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                    NL as ::core::ffi::c_char,
                    NUL as ::core::ffi::c_char,
                    slen,
                );
                len = len.wrapping_add(slen);
            }
            let c2rust_fresh8 = len;
            len = len.wrapping_add(1);
            *ptr.offset(c2rust_fresh8 as isize) = NL as ::core::ffi::c_char;
            lnum_0 += 1;
        }
        return OK;
    }
}

unsafe extern "C" fn diff_write(
    mut buf: *mut buf_T,
    mut din: *mut diffin_T,
    mut start: linenr_T,
    mut end: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*din).din_fname.is_null() {
            return diff_write_buffer(buf, &raw mut (*din).din_mmfile, start, end);
        }
        if frames_locked() {
            return FAIL;
        }
        if end < 0 as linenr_T {
            end = (*buf).b_ml.ml_line_count;
        }
        let mut save_ml_flags: ::core::ffi::c_int = (*buf).b_ml.ml_flags;
        let mut save_ff: *mut ::core::ffi::c_char = (*buf).b_p_ff;
        (*buf).b_p_ff = xstrdup(b"unix\0".as_ptr() as *const ::core::ffi::c_char);
        let save_cmod_flags: bool = (*cmdmod.ptr()).cmod_flags != 0;
        (*cmdmod.ptr()).cmod_flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
        if end < start {
            end = start;
            (*buf).b_ml.ml_flags |= ML_EMPTY;
        }
        let mut r: ::core::ffi::c_int = buf_write(
            buf,
            (*din).din_fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            start,
            end,
            ::core::ptr::null_mut::<exarg_T>(),
            WriteRequest::filter(),
        );
        (*cmdmod.ptr()).cmod_flags = save_cmod_flags as ::core::ffi::c_int;
        free_string_option((*buf).b_p_ff);
        (*buf).b_p_ff = save_ff;
        (*buf).b_ml.ml_flags = (*buf).b_ml.ml_flags & !ML_EMPTY | save_ml_flags & ML_EMPTY;
        return r;
    }
}

unsafe extern "C" fn lnum_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lnum1: linenr_T = *(s1 as *mut linenr_T);
        let mut lnum2: linenr_T = *(s2 as *mut linenr_T);
        if lnum1 < lnum2 {
            return -1 as ::core::ffi::c_int;
        }
        if lnum1 > lnum2 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn diff_try_update(
    mut dio: *mut diffio_T,
    mut idx_orig: ::core::ffi::c_int,
    mut eap: *mut exarg_T,
) {
    unsafe {
        let mut num_anchors: ::core::ffi::c_int = 0;
        let mut anchors: [[linenr_T; 20]; 8] = [[0; 20]; 8];
        '_theend: {
            if (*dio).dio_internal != 0 {
                ga_init(
                    &raw mut (*dio).dio_diff.dout_ga,
                    ::core::mem::size_of::<diffhunk_T>() as ::core::ffi::c_int,
                    100 as ::core::ffi::c_int,
                );
            } else {
                (*dio).dio_orig.din_fname = vim_tempname();
                (*dio).dio_new.din_fname = vim_tempname();
                (*dio).dio_diff.dout_fname = vim_tempname();
                if (*dio).dio_orig.din_fname.is_null()
                    || (*dio).dio_new.din_fname.is_null()
                    || (*dio).dio_diff.dout_fname.is_null()
                {
                    break '_theend;
                } else if check_external_diff(dio) == FAIL {
                    break '_theend;
                }
            }
            if !eap.is_null() && (*eap).forceit != 0 {
                let mut idx_new: ::core::ffi::c_int = idx_orig;
                while idx_new < DB_COUNT {
                    let mut buf: *mut buf_T =
                        (*curtab.get()).tp_diffbuf[idx_new as usize] as *mut buf_T;
                    if buf_valid(buf) {
                        buf_check_timestamp(buf);
                    }
                    idx_new += 1;
                }
            }
            num_anchors = INT_MAX;
            anchors = [[0; 20]; 8];
            memset(
                &raw mut anchors as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[[linenr_T; 20]; 8]>(),
            );
            if diff_flags.get() & DIFF_ANCHOR != 0 {
                let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while idx < DB_COUNT {
                    if !(*curtab.get()).tp_diffbuf[idx as usize].is_null() {
                        let mut buf_num_anchors: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if parse_diffanchors(
                            false_0 != 0,
                            (*curtab.get()).tp_diffbuf[idx as usize] as *mut buf_T,
                            &raw mut *(&raw mut anchors as *mut [linenr_T; 20]).offset(idx as isize)
                                as *mut linenr_T,
                            &raw mut buf_num_anchors,
                        ) != OK
                        {
                            emsg(gettext(
                                &raw const e_failed_to_find_all_diff_anchors
                                    as *const ::core::ffi::c_char,
                            ));
                            num_anchors = 0 as ::core::ffi::c_int;
                            memset(
                                &raw mut anchors as *mut ::core::ffi::c_void,
                                0 as ::core::ffi::c_int,
                                ::core::mem::size_of::<[[linenr_T; 20]; 8]>(),
                            );
                            break;
                        } else {
                            if buf_num_anchors < num_anchors {
                                num_anchors = buf_num_anchors;
                            }
                            if buf_num_anchors > 0 as ::core::ffi::c_int {
                                qsort(
                                    &raw mut *(&raw mut anchors as *mut [linenr_T; 20])
                                        .offset(idx as isize)
                                        as *mut linenr_T
                                        as *mut ::core::ffi::c_void,
                                    buf_num_anchors as size_t,
                                    ::core::mem::size_of::<linenr_T>(),
                                    Some(
                                        lnum_compare
                                            as unsafe extern "C" fn(
                                                *const ::core::ffi::c_void,
                                                *const ::core::ffi::c_void,
                                            )
                                                -> ::core::ffi::c_int,
                                    ),
                                );
                            }
                        }
                    }
                    idx += 1;
                }
            }
            if num_anchors == INT_MAX {
                num_anchors = 0 as ::core::ffi::c_int;
            }
            let mut anchor_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            loop {
                if anchor_i > num_anchors {
                    break '_theend;
                }
                let mut orig_diff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
                if anchor_i != 0 as ::core::ffi::c_int {
                    orig_diff = (*curtab.get()).tp_first_diff;
                    (*curtab.get()).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
                }
                let mut lnum_start: linenr_T = if anchor_i == 0 as ::core::ffi::c_int {
                    1 as linenr_T
                } else {
                    anchors[idx_orig as usize][(anchor_i - 1 as ::core::ffi::c_int) as usize]
                };
                let mut lnum_end: linenr_T = if anchor_i == num_anchors {
                    -1 as linenr_T
                } else {
                    anchors[idx_orig as usize][anchor_i as usize] - 1 as linenr_T
                };
                let mut buf_0: *mut buf_T =
                    (*curtab.get()).tp_diffbuf[idx_orig as usize] as *mut buf_T;
                if diff_write(buf_0, &raw mut (*dio).dio_orig, lnum_start, lnum_end) == FAIL {
                    if !orig_diff.is_null() {
                        (*curtab.get()).tp_first_diff = orig_diff;
                        diff_clear(curtab.get());
                    }
                    break '_theend;
                } else {
                    let mut idx_new_0: ::core::ffi::c_int = idx_orig + 1 as ::core::ffi::c_int;
                    while idx_new_0 < DB_COUNT {
                        buf_0 = (*curtab.get()).tp_diffbuf[idx_new_0 as usize] as *mut buf_T;
                        if !(buf_0.is_null() || (*buf_0).b_ml.ml_mfp.is_null()) {
                            lnum_start = if anchor_i == 0 as ::core::ffi::c_int {
                                1 as linenr_T
                            } else {
                                anchors[idx_new_0 as usize]
                                    [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                            };
                            lnum_end = if anchor_i == num_anchors {
                                -1 as linenr_T
                            } else {
                                anchors[idx_new_0 as usize][anchor_i as usize] - 1 as linenr_T
                            };
                            if diff_write(buf_0, &raw mut (*dio).dio_new, lnum_start, lnum_end)
                                != FAIL
                            {
                                if diff_file(dio) != FAIL {
                                    diff_read(idx_orig, idx_new_0, dio);
                                    clear_diffin(&raw mut (*dio).dio_new);
                                    clear_diffout(&raw mut (*dio).dio_diff);
                                }
                            }
                        }
                        idx_new_0 += 1;
                    }
                    clear_diffin(&raw mut (*dio).dio_orig);
                    if anchor_i != 0 as ::core::ffi::c_int {
                        let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
                        while !dp.is_null() {
                            let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while idx_0 < DB_COUNT {
                                if anchors[idx_0 as usize]
                                    [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                                    > 0 as linenr_T
                                {
                                    (*dp).df_lnum[idx_0 as usize] = ((*dp).df_lnum[idx_0 as usize]
                                        as ::core::ffi::c_int
                                        + (anchors[idx_0 as usize]
                                            [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                                            - 1 as linenr_T)
                                            as ::core::ffi::c_int)
                                        as linenr_T;
                                }
                                idx_0 += 1;
                            }
                            dp = (*dp).df_next;
                        }
                        if !orig_diff.is_null() {
                            let mut last_diff: *mut diff_T = orig_diff;
                            while !(*last_diff).df_next.is_null() {
                                last_diff = (*last_diff).df_next;
                            }
                            (*last_diff).df_next = (*curtab.get()).tp_first_diff;
                            (*curtab.get()).tp_first_diff = orig_diff;
                        }
                    }
                    anchor_i += 1;
                }
            }
        }
        xfree((*dio).dio_orig.din_fname as *mut ::core::ffi::c_void);
        xfree((*dio).dio_new.din_fname as *mut ::core::ffi::c_void);
        xfree((*dio).dio_diff.dout_fname as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn diff_internal() -> ::core::ffi::c_int {
    unsafe {
        return (diff_flags.get() & DIFF_INTERNAL != 0 as ::core::ffi::c_int
            && *p_dex.get() as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
    }
}

pub unsafe fn ex_diffupdate(mut eap: *mut exarg_T) {
    unsafe {
        let mut idx_new: ::core::ffi::c_int = 0;
        let mut diffio: diffio_T = diffio_T {
            dio_orig: diffin_T {
                din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                din_mmfile: mmfile_t {
                    ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
            },
            dio_new: diffin_T {
                din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                din_mmfile: mmfile_t {
                    ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
            },
            dio_diff: diffout_T {
                dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                dout_ga: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
            },
            dio_internal: 0,
        };
        if diff_busy.get() {
            diff_need_update.set(true_0 != 0);
            return;
        }
        let mut had_diffs: ::core::ffi::c_int =
            !(*curtab.get()).tp_first_diff.is_null() as ::core::ffi::c_int;
        diff_clear(curtab.get());
        (*curtab.get()).tp_diff_invalid = false_0;
        let mut idx_orig: ::core::ffi::c_int = 0;
        idx_orig = 0 as ::core::ffi::c_int;
        while idx_orig < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[idx_orig as usize].is_null() {
                break;
            }
            idx_orig += 1;
        }
        if idx_orig != DB_COUNT {
            idx_new = 0;
            idx_new = idx_orig + 1 as ::core::ffi::c_int;
            while idx_new < DB_COUNT {
                if !(*curtab.get()).tp_diffbuf[idx_new as usize].is_null() {
                    break;
                }
                idx_new += 1;
            }
            if idx_new != DB_COUNT {
                diffio = diffio_T {
                    dio_orig: diffin_T {
                        din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        din_mmfile: mmfile_t {
                            ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            size: 0,
                        },
                    },
                    dio_new: diffin_T {
                        din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        din_mmfile: mmfile_t {
                            ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            size: 0,
                        },
                    },
                    dio_diff: diffout_T {
                        dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        dout_ga: garray_T {
                            ga_len: 0,
                            ga_maxlen: 0,
                            ga_itemsize: 0,
                            ga_growsize: 0,
                            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        },
                    },
                    dio_internal: 0,
                };
                diffio.dio_internal = diff_internal();
                diff_try_update(&raw mut diffio, idx_orig, eap);
                (*curwin.get()).w_valid_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
            }
        }
        if had_diffs != 0 || !(*curtab.get()).tp_first_diff.is_null() {
            diff_redraw(true_0 != 0);
            apply_autocmds(
                EVENT_DIFFUPDATED,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    }
}
