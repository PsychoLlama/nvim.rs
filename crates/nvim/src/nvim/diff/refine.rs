//! The `inline:char` and `inline:word` sub-diff.
//!
//! Under those two `'diffopt'` values a changed block is diffed *again*, one
//! character or one word per "line", and the resulting hunks become the column
//! ranges.  `diff_find_change_inline_diff` builds that sub-problem and runs the
//! internal engine over it; the two `diff_refine_inline_*_highlight` functions
//! turn its output back into ranges, the word one merging ranges separated by
//! less than `'diffopt'`'s `inline-word-gap`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn diff_refine_inline_char_highlight(
    mut dp_orig: *mut diff_T,
    mut linemap: *mut garray_T,
    mut idx1: ::core::ffi::c_int,
) {
    unsafe {
        let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        loop {
            let mut has_unmerged_gaps: bool = false_0 != 0;
            let mut has_merged_gaps: bool = false_0 != 0;
            let mut dp: *mut diff_T = dp_orig;
            while !dp.is_null() && !(*dp).df_next.is_null() {
                if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] - 1 as linenr_T
                    >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                    || (*(*dp).df_next).df_lnum[idx1 as usize] - 1 as linenr_T
                        >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                {
                    dp = (*dp).df_next;
                } else {
                    let mut entry1: *mut linemap_entry_T =
                        ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                            (*(&raw mut (*dp).df_lnum as *mut linenr_T).offset(idx1 as isize)
                                + *(&raw mut (*dp).df_count as *mut linenr_T).offset(idx1 as isize)
                                - 1 as linenr_T) as isize,
                        );
                    let mut entry2: *mut linemap_entry_T =
                        ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                            (*(&raw mut (*(*dp).df_next).df_lnum as *mut linenr_T)
                                .offset(idx1 as isize)
                                - 1 as linenr_T) as isize,
                        );
                    if (*entry1).lineoff != (*entry2).lineoff {
                        dp = (*dp).df_next;
                    } else {
                        let mut gap: linenr_T = (*(*dp).df_next).df_lnum[idx1 as usize]
                            - ((*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize]);
                        if gap <= 3 as linenr_T {
                            let mut max_df_count: linenr_T = 0 as linenr_T;
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < DB_COUNT {
                                max_df_count = if max_df_count
                                    > (*dp).df_count[i as usize]
                                        + (*(*dp).df_next).df_count[i as usize]
                                {
                                    max_df_count
                                } else {
                                    (*dp).df_count[i as usize]
                                        + (*(*dp).df_next).df_count[i as usize]
                                };
                                i += 1;
                            }
                            if max_df_count >= gap * 4 as linenr_T {
                                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while i_0 < DB_COUNT {
                                    (*dp).df_count[i_0 as usize] = (*(*dp).df_next).df_lnum
                                        [i_0 as usize]
                                        + (*(*dp).df_next).df_count[i_0 as usize]
                                        - (*dp).df_lnum[i_0 as usize];
                                    i_0 += 1;
                                }
                                let mut dp_next: *mut diff_T = (*dp).df_next;
                                (*dp).df_next = (*dp_next).df_next;
                                clear_diffblock(dp_next);
                                has_merged_gaps = true_0 != 0;
                                continue;
                            } else {
                                has_unmerged_gaps = true_0 != 0;
                            }
                        }
                        dp = (*dp).df_next;
                    }
                }
            }
            if !has_unmerged_gaps || !has_merged_gaps {
                break;
            }
            let c2rust_fresh9 = pass;
            pass = pass + 1;
            if c2rust_fresh9 >= 4 as ::core::ffi::c_int {
                break;
            }
        }
    }
}

unsafe extern "C" fn diff_refine_inline_word_highlight(
    mut dp_orig: *mut diff_T,
    mut linemap: *mut garray_T,
    mut idx1: ::core::ffi::c_int,
    mut start_lnum: linenr_T,
) {
    unsafe {
        let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        loop {
            let mut dp: *mut diff_T = dp_orig;
            while !dp.is_null() && !(*dp).df_next.is_null() {
                if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] - 1 as linenr_T
                    >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                    || (*(*dp).df_next).df_lnum[idx1 as usize] - 1 as linenr_T
                        >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                {
                    dp = (*dp).df_next;
                } else {
                    let mut entry1: *mut linemap_entry_T =
                        ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                            (*(&raw mut (*dp).df_lnum as *mut linenr_T).offset(idx1 as isize)
                                + *(&raw mut (*dp).df_count as *mut linenr_T).offset(idx1 as isize)
                                - 2 as linenr_T) as isize,
                        );
                    let mut entry2: *mut linemap_entry_T =
                        ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                            (*(&raw mut (*(*dp).df_next).df_lnum as *mut linenr_T)
                                .offset(idx1 as isize)
                                - 1 as linenr_T) as isize,
                        );
                    if (*entry1).lineoff != (*entry2).lineoff {
                        dp = (*dp).df_next;
                    } else {
                        let mut gap_start: ::core::ffi::c_int = (*entry1).byte_start
                            as ::core::ffi::c_int
                            + (*entry1).num_bytes as ::core::ffi::c_int;
                        let mut gap_end: ::core::ffi::c_int =
                            (*entry2).byte_start as ::core::ffi::c_int;
                        let mut gap_size: ::core::ffi::c_int = gap_end - gap_start;
                        if gap_size <= 0 as ::core::ffi::c_int || gap_size > diff_word_gap.get() {
                            dp = (*dp).df_next;
                        } else {
                            let mut line: *mut ::core::ffi::c_char = ml_get_buf(
                                (*curtab.get()).tp_diffbuf[idx1 as usize] as *mut buf_T,
                                start_lnum + (*entry1).lineoff as linenr_T,
                            );
                            let mut gap_text: *mut ::core::ffi::c_char =
                                line.offset(gap_start as isize);
                            let mut only_non_word: bool = true_0 != 0;
                            let mut has_content: bool = false_0 != 0;
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < gap_size
                                && *gap_text.offset(i as isize) as ::core::ffi::c_int != NUL
                            {
                                has_content = true_0 != 0;
                                let mut char_class: ::core::ffi::c_int = mb_get_class_tab(
                                    gap_text.offset(i as isize),
                                    &raw mut (**(&raw mut (*curtab.get()).tp_diffbuf
                                        as *mut *mut buf_T)
                                        .offset(idx1 as isize))
                                    .b_chartab as *mut uint64_t,
                                );
                                if char_class == 2 as ::core::ffi::c_int {
                                    only_non_word = false_0 != 0;
                                    break;
                                } else {
                                    i += 1;
                                }
                            }
                            if has_content as ::core::ffi::c_int != 0
                                && only_non_word as ::core::ffi::c_int != 0
                            {
                                let mut total_change_bytes: ::core::ffi::c_long =
                                    0 as ::core::ffi::c_long;
                                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while i_0 < DB_COUNT {
                                    if !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null() {
                                        let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while (k as linenr_T) < (*dp).df_count[i_0 as usize] {
                                            let mut idx: ::core::ffi::c_int =
                                                (*dp).df_lnum[i_0 as usize] as ::core::ffi::c_int
                                                    + k
                                                    - 1 as ::core::ffi::c_int;
                                            if idx < (*linemap.offset(i_0 as isize)).ga_len {
                                                total_change_bytes +=
                                                    (*((*linemap.offset(i_0 as isize)).ga_data
                                                        as *mut linemap_entry_T)
                                                        .offset(idx as isize))
                                                    .num_bytes
                                                        as ::core::ffi::c_long;
                                            }
                                            k += 1;
                                        }
                                        let mut k_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while (k_0 as linenr_T)
                                            < (*(*dp).df_next).df_count[i_0 as usize]
                                        {
                                            let mut idx_0: ::core::ffi::c_int =
                                                (*(*dp).df_next).df_lnum[i_0 as usize]
                                                    as ::core::ffi::c_int
                                                    + k_0
                                                    - 1 as ::core::ffi::c_int;
                                            if idx_0 < (*linemap.offset(i_0 as isize)).ga_len {
                                                total_change_bytes +=
                                                    (*((*linemap.offset(i_0 as isize)).ga_data
                                                        as *mut linemap_entry_T)
                                                        .offset(idx_0 as isize))
                                                    .num_bytes
                                                        as ::core::ffi::c_long;
                                            }
                                            k_0 += 1;
                                        }
                                    }
                                    i_0 += 1;
                                }
                                if total_change_bytes
                                    >= (gap_size * 2 as ::core::ffi::c_int) as ::core::ffi::c_long
                                {
                                    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while i_1 < DB_COUNT {
                                        if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null() {
                                            (*dp).df_count[i_1 as usize] = (*(*dp).df_next).df_lnum
                                                [i_1 as usize]
                                                + (*(*dp).df_next).df_count[i_1 as usize]
                                                - (*dp).df_lnum[i_1 as usize];
                                        }
                                        i_1 += 1;
                                    }
                                    let mut dp_next: *mut diff_T = (*dp).df_next;
                                    (*dp).df_next = (*dp_next).df_next;
                                    clear_diffblock(dp_next);
                                    continue;
                                }
                            }
                            dp = (*dp).df_next;
                        }
                    }
                }
            }
            let c2rust_fresh10 = pass;
            pass = pass + 1;
            if c2rust_fresh10 >= 4 as ::core::ffi::c_int {
                break;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn diff_find_change_inline_diff(mut dp: *mut diff_T) {
    unsafe {
        let mut new_diff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let save_diff_algorithm: ::core::ffi::c_int = diff_algorithm.get();
        let mut dio: diffio_T = diffio_T {
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
        ga_init(
            &raw mut dio.dio_diff.dout_ga,
            ::core::mem::size_of::<diffhunk_T>() as ::core::ffi::c_int,
            1000 as ::core::ffi::c_int,
        );
        dio.dio_internal = true_0;
        (*diff_algorithm.ptr()) |= XDF_INDENT_HEURISTIC;
        let mut orig_diff: *mut diff_T = (*curtab.get()).tp_first_diff;
        (*curtab.get()).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
        let mut orig_diffbuf: [*mut buf_T; 8] = [::core::ptr::null_mut::<buf_T>(); 8];
        memcpy(
            &raw mut orig_diffbuf as *mut *mut buf_T as *mut ::core::ffi::c_void,
            &raw mut (*curtab.get()).tp_diffbuf as *mut *mut buf_T as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[*mut buf_T; 8]>(),
        );
        let mut linemap: [garray_T; 8] = [garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        }; 8];
        let mut file1_str: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut file2_str: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut file1_str,
            1 as ::core::ffi::c_int,
            1024 as ::core::ffi::c_int,
        );
        ga_init(
            &raw mut file2_str,
            1 as ::core::ffi::c_int,
            1024 as ::core::ffi::c_int,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            ga_init(
                (&raw mut linemap as *mut garray_T).offset(i as isize),
                ::core::mem::size_of::<linemap_entry_T>() as ::core::ffi::c_int,
                128 as ::core::ffi::c_int,
            );
            i += 1;
        }
        let mut file1_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        '_done: {
            while i_0 < DB_COUNT {
                dio.dio_diff.dout_ga.ga_len = 0 as ::core::ffi::c_int;
                let mut buf: *mut buf_T = (*curtab.get()).tp_diffbuf[i_0 as usize] as *mut buf_T;
                if !(buf.is_null() || (*buf).b_ml.ml_mfp.is_null()) {
                    if (*dp).df_count[i_0 as usize] == 0 as linenr_T {
                        (*curtab.get()).tp_diffbuf[i_0 as usize] = ::core::ptr::null_mut::<buf_T>();
                    } else {
                        if file1_idx == -1 as ::core::ffi::c_int {
                            file1_idx = i_0;
                        }
                        let mut curstr: *mut garray_T = if file1_idx != i_0 {
                            &raw mut file2_str
                        } else {
                            &raw mut file1_str
                        };
                        let mut numlines: linenr_T = 0 as linenr_T;
                        (*curstr).ga_len = 0 as ::core::ffi::c_int;
                        let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while (off as linenr_T) < (*dp).df_count[i_0 as usize] {
                            let mut curline: *mut ::core::ffi::c_char = ml_get_buf(
                                (*curtab.get()).tp_diffbuf[i_0 as usize] as *mut buf_T,
                                (*dp).df_lnum[i_0 as usize] + off as linenr_T,
                            );
                            let mut in_keyword: bool = false_0 != 0;
                            let mut last_white: bool = false_0 != 0;
                            let mut eol_ga_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                            let mut eol_linemap_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                            let mut eol_numlines: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                            let mut s: *mut ::core::ffi::c_char = curline;
                            while *s as ::core::ffi::c_int != NUL {
                                let mut new_in_keyword: bool = false_0 != 0;
                                if diff_flags.get() & DIFF_INLINE_WORD != 0 {
                                    new_in_keyword = mb_get_class_tab(
                                        s,
                                        &raw mut (**(&raw mut (*curtab.get()).tp_diffbuf
                                            as *mut *mut buf_T)
                                            .offset(file1_idx as isize))
                                        .b_chartab
                                            as *mut uint64_t,
                                    ) == 2 as ::core::ffi::c_int;
                                }
                                if in_keyword as ::core::ffi::c_int != 0 && !new_in_keyword {
                                    ga_append(curstr, NL as uint8_t);
                                    numlines += 1;
                                }
                                if ascii_iswhite(*s as ::core::ffi::c_int) {
                                    if diff_flags.get() & DIFF_IWHITEALL != 0 {
                                        in_keyword = false_0 != 0;
                                        s = skipwhite(s);
                                        continue;
                                    } else if diff_flags.get() & DIFF_IWHITEEOL != 0
                                        || diff_flags.get() & DIFF_IWHITE != 0
                                    {
                                        if !last_white {
                                            eol_ga_len = (*curstr).ga_len;
                                            eol_linemap_len = linemap[i_0 as usize].ga_len;
                                            eol_numlines = numlines as ::core::ffi::c_int;
                                            last_white = true_0 != 0;
                                        }
                                    }
                                } else if diff_flags.get() & DIFF_IWHITEEOL != 0
                                    || diff_flags.get() & DIFF_IWHITE != 0
                                {
                                    last_white = false_0 != 0;
                                    eol_ga_len = -1 as ::core::ffi::c_int;
                                    eol_linemap_len = -1 as ::core::ffi::c_int;
                                    eol_numlines = -1 as ::core::ffi::c_int;
                                }
                                let mut char_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                                if *s as ::core::ffi::c_int == NL {
                                    ga_append(curstr, NUL as uint8_t);
                                } else {
                                    char_len = utfc_ptr2len(s);
                                    if ascii_iswhite(*s as ::core::ffi::c_int) as ::core::ffi::c_int
                                        != 0
                                        && diff_flags.get() & DIFF_IWHITE != 0
                                    {
                                        char_len =
                                            skipwhite(s).offset_from(s) as ::core::ffi::c_int;
                                    }
                                    if diff_flags.get() & DIFF_ICASE != 0 {
                                        let mut c: ::core::ffi::c_int = utf_ptr2char(s);
                                        let mut c_len: ::core::ffi::c_int = utf_char2len(c);
                                        c = utf_fold(c);
                                        let mut cbuf: [::core::ffi::c_char; 22] = [0; 22];
                                        let mut c_fold_len: ::core::ffi::c_int = utf_char2bytes(
                                            c,
                                            &raw mut cbuf as *mut ::core::ffi::c_char,
                                        );
                                        ga_concat_len(
                                            curstr,
                                            &raw mut cbuf as *mut ::core::ffi::c_char,
                                            c_fold_len as size_t,
                                        );
                                        if char_len > c_len {
                                            ga_concat_len(
                                                curstr,
                                                s.offset(c_len as isize),
                                                (char_len - c_len) as size_t,
                                            );
                                        }
                                    } else {
                                        ga_concat_len(curstr, s, char_len as size_t);
                                    }
                                }
                                if !new_in_keyword {
                                    ga_append(curstr, NL as uint8_t);
                                    numlines += 1;
                                }
                                if !new_in_keyword
                                    || new_in_keyword as ::core::ffi::c_int != 0 && !in_keyword
                                {
                                    let mut linemap_entry: linemap_entry_T = linemap_entry_T {
                                        byte_start: s.offset_from(curline) as colnr_T,
                                        num_bytes: char_len as colnr_T,
                                        lineoff: off,
                                    };
                                    ga_grow(
                                        (&raw mut linemap as *mut garray_T).offset(i_0 as isize),
                                        1 as ::core::ffi::c_int,
                                    );
                                    *(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                        .offset(linemap[i_0 as usize].ga_len as isize) =
                                        linemap_entry;
                                    linemap[i_0 as usize].ga_len += 1;
                                } else {
                                    (*(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                        .offset(
                                            (linemap[i_0 as usize].ga_len - 1 as ::core::ffi::c_int)
                                                as isize,
                                        ))
                                    .num_bytes += char_len;
                                }
                                in_keyword = new_in_keyword;
                                s = s.offset(char_len as isize);
                            }
                            if in_keyword {
                                ga_append(curstr, NL as uint8_t);
                                numlines += 1;
                            }
                            if diff_flags.get() & DIFF_IWHITEEOL != 0
                                || diff_flags.get() & DIFF_IWHITE != 0
                            {
                                if eol_ga_len != -1 as ::core::ffi::c_int {
                                    (*curstr).ga_len = eol_ga_len;
                                    linemap[i_0 as usize].ga_len = eol_linemap_len;
                                    numlines = eol_numlines as linenr_T;
                                }
                            }
                            if diff_flags.get() & DIFF_IWHITEALL == 0 {
                                ga_append(curstr, NL as uint8_t);
                                numlines += 1;
                                let mut linemap_entry_0: linemap_entry_T = linemap_entry_T {
                                    byte_start: s.offset_from(curline) as colnr_T,
                                    num_bytes: ::core::mem::size_of::<::core::ffi::c_int>()
                                        as colnr_T,
                                    lineoff: off,
                                };
                                ga_grow(
                                    (&raw mut linemap as *mut garray_T).offset(i_0 as isize),
                                    1 as ::core::ffi::c_int,
                                );
                                *(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                    .offset(linemap[i_0 as usize].ga_len as isize) =
                                    linemap_entry_0;
                                linemap[i_0 as usize].ga_len += 1;
                            }
                            off += 1;
                        }
                        if file1_idx != i_0 {
                            dio.dio_new.din_mmfile.ptr =
                                (*curstr).ga_data as *mut ::core::ffi::c_char;
                            dio.dio_new.din_mmfile.size = (*curstr).ga_len;
                        } else {
                            dio.dio_orig.din_mmfile.ptr =
                                (*curstr).ga_data as *mut ::core::ffi::c_char;
                            dio.dio_orig.din_mmfile.size = (*curstr).ga_len;
                        }
                        if file1_idx != i_0 {
                            let mut diff_status: ::core::ffi::c_int =
                                diff_file_internal(&raw mut dio);
                            if diff_status == FAIL {
                                break '_done;
                            }
                            diff_read(0 as ::core::ffi::c_int, i_0, &raw mut dio);
                            clear_diffout(&raw mut dio.dio_diff);
                        }
                    }
                }
                i_0 += 1;
            }
            new_diff = (*curtab.get()).tp_first_diff;
            if diff_flags.get() & DIFF_INLINE_WORD != 0 && file1_idx != -1 as ::core::ffi::c_int {
                diff_refine_inline_word_highlight(
                    new_diff,
                    &raw mut linemap as *mut garray_T,
                    file1_idx,
                    (*dp).df_lnum[file1_idx as usize],
                );
            } else if diff_flags.get() & DIFF_INLINE_CHAR != 0
                && file1_idx != -1 as ::core::ffi::c_int
            {
                diff_refine_inline_char_highlight(
                    new_diff,
                    &raw mut linemap as *mut garray_T,
                    file1_idx,
                );
            }
            (*dp).df_changes.ga_len = 0 as ::core::ffi::c_int;
            while !new_diff.is_null() {
                let mut change: diffline_change_T = diffline_change_S {
                    dc_start: [0 as colnr_T; 8],
                    dc_end: [0; 8],
                    dc_start_lnum_off: [0; 8],
                    dc_end_lnum_off: [0; 8],
                };
                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_1 < DB_COUNT {
                    if (*new_diff).df_lnum[i_1 as usize] > 0 as linenr_T {
                        let mut diff_lnum: linenr_T =
                            (*new_diff).df_lnum[i_1 as usize] - 1 as linenr_T;
                        let mut diff_lnum_end: linenr_T =
                            diff_lnum + (*new_diff).df_count[i_1 as usize];
                        if diff_lnum >= linemap[i_1 as usize].ga_len as linenr_T {
                            change.dc_start[i_1 as usize] = MAXCOL as ::core::ffi::c_int as colnr_T;
                            change.dc_start_lnum_off[i_1 as usize] = INT_MAX;
                        } else {
                            change.dc_start[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                                as *mut linemap_entry_T)
                                .offset(diff_lnum as isize))
                            .byte_start;
                            change.dc_start_lnum_off[i_1 as usize] =
                                (*(linemap[i_1 as usize].ga_data as *mut linemap_entry_T)
                                    .offset(diff_lnum as isize))
                                .lineoff;
                        }
                        if diff_lnum == diff_lnum_end {
                            change.dc_end[i_1 as usize] = change.dc_start[i_1 as usize];
                            change.dc_end_lnum_off[i_1 as usize] =
                                change.dc_start_lnum_off[i_1 as usize];
                        } else if diff_lnum_end - 1 as linenr_T
                            >= linemap[i_1 as usize].ga_len as linenr_T
                        {
                            change.dc_end[i_1 as usize] = MAXCOL as ::core::ffi::c_int as colnr_T;
                            change.dc_end_lnum_off[i_1 as usize] = INT_MAX;
                        } else {
                            change.dc_end[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                                as *mut linemap_entry_T)
                                .offset((diff_lnum_end - 1 as linenr_T) as isize))
                            .byte_start
                                + (*(linemap[i_1 as usize].ga_data as *mut linemap_entry_T)
                                    .offset((diff_lnum_end - 1 as linenr_T) as isize))
                                .num_bytes;
                            change.dc_end_lnum_off[i_1 as usize] =
                                (*(linemap[i_1 as usize].ga_data as *mut linemap_entry_T)
                                    .offset((diff_lnum_end - 1 as linenr_T) as isize))
                                .lineoff;
                        }
                    }
                    i_1 += 1;
                }
                ga_grow(&raw mut (*dp).df_changes, 1 as ::core::ffi::c_int);
                *((*dp).df_changes.ga_data as *mut diffline_change_T)
                    .offset((*dp).df_changes.ga_len as isize) = change;
                (*dp).df_changes.ga_len += 1;
                new_diff = (*new_diff).df_next;
            }
        }
        diff_algorithm.set(save_diff_algorithm);
        (*dp).has_changes = true_0 != 0;
        diff_clear(curtab.get());
        (*curtab.get()).tp_first_diff = orig_diff;
        memcpy(
            &raw mut (*curtab.get()).tp_diffbuf as *mut *mut buf_T as *mut ::core::ffi::c_void,
            &raw mut orig_diffbuf as *mut *mut buf_T as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[*mut buf_T; 8]>(),
        );
        ga_clear(&raw mut file1_str);
        ga_clear(&raw mut file2_str);
        clear_diffout(&raw mut dio.dio_diff);
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < DB_COUNT {
            ga_clear((&raw mut linemap as *mut garray_T).offset(i_2 as isize));
            i_2 += 1;
        }
    }
}
