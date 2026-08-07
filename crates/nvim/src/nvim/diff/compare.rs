//! When two lines count as equal.
//!
//! `'diffopt'`'s `iwhite`, `iwhiteall`, `iwhiteeol`, `iblank` and `icase` all
//! mean "ignore this difference", and this is where they are applied: `diff_cmp`
//! compares two lines under the current flags, `diff_equal_char` is the
//! character-level rule underneath it, and `diff_equal_entry` lifts the answer to
//! a whole diff block.  Only the external diff needs them -- the internal one
//! passes the flags down to `xdl_diff` -- but the block-level answers are read on
//! both paths.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn diff_equal_entry(
    mut dp: *mut diff_T,
    mut idx1: ::core::ffi::c_int,
    mut idx2: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if (*dp).df_count[idx1 as usize] != (*dp).df_count[idx2 as usize] {
            return false_0 != 0;
        }
        if diff_check_sanity(curtab.get(), dp) == FAIL {
            return false_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as linenr_T) < (*dp).df_count[idx1 as usize] {
            let mut line: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                (*curtab.get()).tp_diffbuf[idx1 as usize] as *mut buf_T,
                (*dp).df_lnum[idx1 as usize] + i as linenr_T,
            ));
            let mut cmp: ::core::ffi::c_int = diff_cmp(
                line,
                ml_get_buf(
                    (*curtab.get()).tp_diffbuf[idx2 as usize] as *mut buf_T,
                    (*dp).df_lnum[idx2 as usize] + i as linenr_T,
                ),
            );
            xfree(line as *mut ::core::ffi::c_void);
            if cmp != 0 as ::core::ffi::c_int {
                return false_0 != 0;
            }
            i += 1;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn diff_equal_char(
    p1: *const ::core::ffi::c_char,
    p2: *const ::core::ffi::c_char,
    len: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let l: ::core::ffi::c_int = utfc_ptr2len(p1);
        if l != utfc_ptr2len(p2) {
            return false_0 != 0;
        }
        if l > 1 as ::core::ffi::c_int {
            if strncmp(p1, p2, l as size_t) != 0 as ::core::ffi::c_int
                && (diff_flags.get() & DIFF_ICASE == 0
                    || utf_fold(utf_ptr2char(p1)) != utf_fold(utf_ptr2char(p2)))
            {
                return false_0 != 0;
            }
            *len = l;
        } else {
            if *p1 as ::core::ffi::c_int != *p2 as ::core::ffi::c_int
                && (diff_flags.get() & DIFF_ICASE == 0
                    || tolower(*p1 as uint8_t as ::core::ffi::c_int)
                        != tolower(*p2 as uint8_t as ::core::ffi::c_int))
            {
                return false_0 != 0;
            }
            *len = 1 as ::core::ffi::c_int;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn diff_cmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if diff_flags.get() & DIFF_IBLANK != 0
            && (*skipwhite(s1) as ::core::ffi::c_int == NUL
                || *skipwhite(s2) as ::core::ffi::c_int == NUL)
        {
            return 0 as ::core::ffi::c_int;
        }
        if diff_flags.get() & (DIFF_ICASE | ALL_WHITE_DIFF) == 0 as ::core::ffi::c_int {
            return strcmp(s1, s2);
        }
        if diff_flags.get() & DIFF_ICASE != 0 && diff_flags.get() & ALL_WHITE_DIFF == 0 {
            return mb_stricmp(s1, s2);
        }
        let mut p1: *mut ::core::ffi::c_char = s1;
        let mut p2: *mut ::core::ffi::c_char = s2;
        while *p1 as ::core::ffi::c_int != NUL && *p2 as ::core::ffi::c_int != NUL {
            if diff_flags.get() & DIFF_IWHITE != 0
                && ascii_iswhite(*p1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && ascii_iswhite(*p2 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                || diff_flags.get() & DIFF_IWHITEALL != 0
                    && (ascii_iswhite(*p1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || ascii_iswhite(*p2 as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
            {
                p1 = skipwhite(p1);
                p2 = skipwhite(p2);
            } else {
                let mut l: ::core::ffi::c_int = 0;
                if !diff_equal_char(p1, p2, &raw mut l) {
                    break;
                }
                p1 = p1.offset(l as isize);
                p2 = p2.offset(l as isize);
            }
        }
        p1 = skipwhite(p1);
        p2 = skipwhite(p2);
        if *p1 as ::core::ffi::c_int != NUL || *p2 as ::core::ffi::c_int != NUL {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}
