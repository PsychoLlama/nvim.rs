//! Turning diff output back into blocks.
//!
//! `diff_read` walks whatever the engine produced -- unified or `ed`-style text
//! from an external diff, or the hunks `xdiff_out` collected from the internal
//! one -- and `process_hunk` merges each into the tabpage's block list, growing,
//! splitting or joining the existing blocks as the new range overlaps them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn extract_hunk_internal(
    mut dout: *mut diffout_T,
    mut hunk: *mut diffhunk_T,
    mut line_idx: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut eof: bool = *line_idx >= (*dout).dout_ga.ga_len;
        if !eof {
            let c2rust_fresh7 = *line_idx;
            *line_idx = *line_idx + 1;
            *hunk = *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset(c2rust_fresh7 as isize);
        }
        return eof;
    }
}

unsafe extern "C" fn extract_hunk(
    mut fd: *mut FILE,
    mut hunk: *mut diffhunk_T,
    mut diffstyle: *mut diffstyle_T,
) -> bool {
    unsafe {
        loop {
            let mut line: [::core::ffi::c_char; 50] = [0; 50];
            if vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd) {
                return true_0 != 0;
            }
            if *diffstyle as ::core::ffi::c_uint
                == DIFF_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if *(*__ctype_b_loc()).offset(
                    *(&raw mut line as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int
                        as isize,
                ) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                {
                    *diffstyle = DIFF_ED;
                } else if strncmp(
                    &raw mut line as *mut ::core::ffi::c_char,
                    b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                    3 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    *diffstyle = DIFF_UNIFIED;
                } else {
                    if !(strncmp(
                        &raw mut line as *mut ::core::ffi::c_char,
                        b"--- \0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                        && vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd)
                            as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        && strncmp(
                            &raw mut line as *mut ::core::ffi::c_char,
                            b"+++ \0".as_ptr() as *const ::core::ffi::c_char,
                            4 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        && vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd)
                            as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        && strncmp(
                            &raw mut line as *mut ::core::ffi::c_char,
                            b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                            3 as size_t,
                        ) == 0 as ::core::ffi::c_int)
                    {
                        continue;
                    }
                    *diffstyle = DIFF_UNIFIED;
                }
            }
            if *diffstyle as ::core::ffi::c_uint
                == DIFF_ED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if *(*__ctype_b_loc()).offset(
                    *(&raw mut line as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int
                        as isize,
                ) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                {
                    continue;
                }
                if parse_diff_ed(&raw mut line as *mut ::core::ffi::c_char, hunk) == FAIL {
                    continue;
                }
            } else {
                '_c2rust_label: {
                    if *diffstyle as ::core::ffi::c_uint
                        == DIFF_UNIFIED as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                    } else {
                        __assert_fail(
                            b"*diffstyle == DIFF_UNIFIED\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/diff.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1726 as ::core::ffi::c_uint,
                            b"_Bool extract_hunk(FILE *, diffhunk_T *, diffstyle_T *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                if strncmp(
                    &raw mut line as *mut ::core::ffi::c_char,
                    b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                    3 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    continue;
                }
                if parse_diff_unified(&raw mut line as *mut ::core::ffi::c_char, hunk) == FAIL {
                    continue;
                }
            }
            return false_0 != 0;
        }
    }
}

unsafe extern "C" fn process_hunk(
    mut dpp: *mut *mut diff_T,
    mut dprevp: *mut *mut diff_T,
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
    mut hunk: *mut diffhunk_T,
    mut notsetp: *mut bool,
) {
    unsafe {
        let mut dp: *mut diff_T = *dpp;
        let mut dprev: *mut diff_T = *dprevp;
        while !dp.is_null()
            && (*hunk).lnum_orig
                > (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
        {
            if *notsetp {
                diff_copy_entry(dprev, dp, idx_orig, idx_new);
            }
            dprev = dp;
            dp = (*dp).df_next;
            *notsetp = true_0 != 0;
        }
        if !dp.is_null()
            && (*hunk).lnum_orig
                <= (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
            && (*hunk).lnum_orig + (*hunk).count_orig as linenr_T
                >= (*dp).df_lnum[idx_orig as usize]
        {
            let mut dpl: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
            dpl = dp;
            while !(*dpl).df_next.is_null() {
                if ((*hunk).lnum_orig + (*hunk).count_orig as linenr_T)
                    < (*(*dpl).df_next).df_lnum[idx_orig as usize]
                {
                    break;
                }
                dpl = (*dpl).df_next;
            }
            let mut off: linenr_T = (*dp).df_lnum[idx_orig as usize] - (*hunk).lnum_orig;
            if off > 0 as linenr_T {
                let mut i: ::core::ffi::c_int = idx_orig;
                while i < idx_new {
                    if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                        (*dp).df_lnum[i as usize] -= off;
                        (*dp).df_count[i as usize] += off;
                    }
                    i += 1;
                }
                (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new;
                (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T;
            } else if *notsetp {
                (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new + off;
                (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T - off;
            } else {
                let mut orig_size_in_dp: ::core::ffi::c_int = if ((*hunk).count_orig as linenr_T)
                    < (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
                        - (*hunk).lnum_orig
                {
                    (*hunk).count_orig
                } else {
                    (*dp).df_lnum[idx_orig as usize] as ::core::ffi::c_int
                        + (*dp).df_count[idx_orig as usize] as ::core::ffi::c_int
                        - (*hunk).lnum_orig as ::core::ffi::c_int
                };
                let mut size_diff: ::core::ffi::c_int = (*hunk).count_new - orig_size_in_dp;
                (*dp).df_count[idx_new as usize] = ((*dp).df_count[idx_new as usize]
                    as ::core::ffi::c_int
                    + size_diff) as linenr_T;
                off = (*hunk).lnum_new + (*hunk).count_new as linenr_T
                    - ((*dp).df_lnum[idx_new as usize] + (*dp).df_count[idx_new as usize]);
                if off > 0 as linenr_T {
                    (*dp).df_count[idx_new as usize] += off;
                }
            }
            off = (*hunk).lnum_orig + (*hunk).count_orig as linenr_T
                - ((*dpl).df_lnum[idx_orig as usize] + (*dpl).df_count[idx_orig as usize]);
            if off < 0 as linenr_T {
                if *notsetp as ::core::ffi::c_int != 0 || dp != dpl {
                    (*dp).df_count[idx_new as usize] += -off;
                }
                off = 0 as ::core::ffi::c_int as linenr_T;
            }
            let mut i_0: ::core::ffi::c_int = idx_orig;
            while i_0 < idx_new {
                if !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null() {
                    (*dp).df_count[i_0 as usize] = (*dpl).df_lnum[i_0 as usize]
                        + (*dpl).df_count[i_0 as usize]
                        - (*dp).df_lnum[i_0 as usize]
                        + off;
                }
                i_0 += 1;
            }
            let mut dn: *mut diff_T = (*dp).df_next;
            (*dp).df_next = (*dpl).df_next;
            while dn != (*dp).df_next {
                dpl = (*dn).df_next;
                clear_diffblock(dn);
                dn = dpl;
            }
        } else {
            dp = diff_alloc_new(curtab.get(), dprev, dp);
            (*dp).df_lnum[idx_orig as usize] = (*hunk).lnum_orig;
            (*dp).df_count[idx_orig as usize] = (*hunk).count_orig as linenr_T;
            (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new;
            (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T;
            let mut i_1: ::core::ffi::c_int = idx_orig + 1 as ::core::ffi::c_int;
            while i_1 < idx_new {
                if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null() {
                    diff_copy_entry(dprev, dp, idx_orig, i_1);
                }
                i_1 += 1;
            }
        }
        *notsetp = false_0 != 0;
        *dpp = dp;
        *dprevp = dprev;
    }
}

pub(crate) unsafe extern "C" fn diff_read(
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
    mut dio: *mut diffio_T,
) {
    unsafe {
        let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
        let mut line_hunk_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
        let mut dout: *mut diffout_T = &raw mut (*dio).dio_diff;
        let mut notset: bool = true_0 != 0;
        let mut diffstyle: diffstyle_T = DIFF_NONE;
        if (*dio).dio_internal == 0 {
            fd = os_fopen(
                (*dout).dout_fname,
                b"r\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if fd.is_null() {
                emsg(gettext(
                    b"E98: Cannot read diff output\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
        }
        loop {
            let mut hunk: diffhunk_T = diffhunk_T {
                lnum_orig: 0 as linenr_T,
                count_orig: 0,
                lnum_new: 0,
                count_new: 0,
            };
            let mut eof: bool = if (*dio).dio_internal != 0 {
                extract_hunk_internal(dout, &raw mut hunk, &raw mut line_hunk_idx)
                    as ::core::ffi::c_int
            } else {
                extract_hunk(fd, &raw mut hunk, &raw mut diffstyle) as ::core::ffi::c_int
            } != 0;
            if eof {
                break;
            }
            process_hunk(
                &raw mut dp,
                &raw mut dprev,
                idx_orig,
                idx_new,
                &raw mut hunk,
                &raw mut notset,
            );
        }
        while !dp.is_null() {
            if notset {
                diff_copy_entry(dprev, dp, idx_orig, idx_new);
            }
            dprev = dp;
            dp = (*dp).df_next;
            notset = true_0 != 0;
        }
        if !fd.is_null() {
            fclose(fd);
        }
    }
}

unsafe extern "C" fn parse_diff_ed(
    mut line: *mut ::core::ffi::c_char,
    mut hunk: *mut diffhunk_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut l1: ::core::ffi::c_int = 0;
        let mut l2: ::core::ffi::c_int = 0;
        let mut p: *mut ::core::ffi::c_char = line;
        let mut f1: linenr_T = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t);
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
            l1 = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
        } else {
            l1 = f1 as ::core::ffi::c_int;
        }
        if *p as ::core::ffi::c_int != 'a' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != 'c' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != 'd' as ::core::ffi::c_int
        {
            return FAIL;
        }
        let c2rust_fresh6 = p;
        p = p.offset(1);
        let mut difftype: ::core::ffi::c_int = *c2rust_fresh6 as uint8_t as ::core::ffi::c_int;
        let mut f2: ::core::ffi::c_int =
            getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
            l2 = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
        } else {
            l2 = f2;
        }
        if (l1 as linenr_T) < f1 || l2 < f2 {
            return FAIL;
        }
        if difftype == 'a' as ::core::ffi::c_int {
            (*hunk).lnum_orig = f1 + 1 as linenr_T;
            (*hunk).count_orig = 0 as ::core::ffi::c_int;
        } else {
            (*hunk).lnum_orig = f1;
            (*hunk).count_orig = (l1 as linenr_T - f1 + 1 as linenr_T) as ::core::ffi::c_int;
        }
        if difftype == 'd' as ::core::ffi::c_int {
            (*hunk).lnum_new = f2 as linenr_T + 1 as linenr_T;
            (*hunk).count_new = 0 as ::core::ffi::c_int;
        } else {
            (*hunk).lnum_new = f2 as linenr_T;
            (*hunk).count_new = l2 - f2 + 1 as ::core::ffi::c_int;
        }
        return OK;
    }
}

unsafe extern "C" fn parse_diff_unified(
    mut line: *mut ::core::ffi::c_char,
    mut hunk: *mut diffhunk_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = line;
        let c2rust_fresh0 = p;
        p = p.offset(1);
        if *c2rust_fresh0 as ::core::ffi::c_int == '@' as ::core::ffi::c_int
            && {
                let c2rust_fresh1 = p;
                p = p.offset(1);
                *c2rust_fresh1 as ::core::ffi::c_int == '@' as ::core::ffi::c_int
            }
            && {
                let c2rust_fresh2 = p;
                p = p.offset(1);
                *c2rust_fresh2 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            }
            && {
                let c2rust_fresh3 = p;
                p = p.offset(1);
                *c2rust_fresh3 as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            }
        {
            let mut oldcount: ::core::ffi::c_int = 0;
            let mut newline: linenr_T = 0;
            let mut newcount: ::core::ffi::c_int = 0;
            let mut oldline: linenr_T = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t);
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
                oldcount = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
            } else {
                oldcount = 1 as ::core::ffi::c_int;
            }
            let c2rust_fresh4 = p;
            p = p.offset(1);
            if *c2rust_fresh4 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int && {
                let c2rust_fresh5 = p;
                p = p.offset(1);
                *c2rust_fresh5 as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            } {
                newline =
                    getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) as linenr_T;
                if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                    p = p.offset(1);
                    newcount = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
                } else {
                    newcount = 1 as ::core::ffi::c_int;
                }
            } else {
                return FAIL;
            }
            if oldcount == 0 as ::core::ffi::c_int {
                oldline = (oldline as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
            }
            if newcount == 0 as ::core::ffi::c_int {
                newline = (newline as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
            }
            if newline == 0 as linenr_T {
                newline = 1 as ::core::ffi::c_int as linenr_T;
            }
            (*hunk).lnum_orig = oldline;
            (*hunk).count_orig = oldcount;
            (*hunk).lnum_new = newline;
            (*hunk).count_new = newcount;
            return OK;
        }
        return FAIL;
    }
}
