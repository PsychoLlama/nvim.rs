use crate::ex_session::{put_eol, put_line};
use crate::os::libc::fprintf;
use core::ffi::{c_char, c_int};

use super::*;

/// Write commands to "fd" to restore the manual folds in window "wp".
///
/// Returns fAIL if writing fails.
pub unsafe extern "C" fn put_folds(mut fd: *mut FILE, mut wp: *mut win_T) -> c_int {
    if foldmethodIsManual(wp) {
        if put_line(fd, c"silent! normal! zE".as_ptr() as *mut c_char) == FAIL
            || put_folds_recurse(fd, &raw mut (*wp).w_folds, 0) == FAIL
            || put_line(fd, c"let &fdl = &fdl".as_ptr() as *mut c_char) == FAIL
        {
            return FAIL;
        }
    }
    if (*wp).w_fold_manual {
        return put_foldopen_recurse(fd, wp, &raw mut (*wp).w_folds, 0);
    }
    return OK;
}

/// Write commands to "fd" to recreate manually created folds.
///
/// Returns fAIL when writing failed.
pub(super) unsafe extern "C" fn put_folds_recurse(
    mut fd: *mut FILE,
    mut gap: *mut garray_T,
    mut off: linenr_T,
) -> c_int {
    let mut fp: *mut fold_T = folds(&*gap);
    let mut i: c_int = 0;
    while i < (*gap).ga_len {
        if put_folds_recurse(fd, &raw mut (*fp).fd_nested, off + (*fp).fd_top) == FAIL {
            return FAIL;
        }
        if fprintf(
            fd,
            c"sil! %ld,%ldfold".as_ptr(),
            (*fp).fd_top as int64_t + off as int64_t,
            ((*fp).fd_top + off + (*fp).fd_len - 1) as int64_t,
        ) < 0
            || put_eol(fd) == FAIL
        {
            return FAIL;
        }
        fp = fp.offset(1);
        i += 1;
    }
    return OK;
}

/// Write commands to "fd" to open and close manually opened/closed folds.
///
/// Returns fAIL when writing failed.
pub(super) unsafe extern "C" fn put_foldopen_recurse(
    mut fd: *mut FILE,
    mut wp: *mut win_T,
    mut gap: *mut garray_T,
    mut off: linenr_T,
) -> c_int {
    let mut fp: *mut fold_T = folds(&*gap);
    let mut i: c_int = 0;
    while i < (*gap).ga_len {
        if (*fp).fd_flags as c_int != FD_LEVEL as c_int {
            if !((*fp).fd_nested.ga_len <= 0) {
                if fprintf(
                    fd,
                    c"%ld".as_ptr(),
                    (*fp).fd_top as int64_t + off as int64_t,
                ) < 0
                    || put_eol(fd) == FAIL
                    || put_line(fd, c"sil! normal! zo".as_ptr() as *mut c_char) == FAIL
                {
                    return FAIL;
                }
                if put_foldopen_recurse(fd, wp, &raw mut (*fp).fd_nested, off + (*fp).fd_top)
                    == FAIL
                {
                    return FAIL;
                }
                if (*fp).fd_flags as c_int == FD_CLOSED as c_int {
                    if put_fold_open_close(fd, fp, off) == FAIL {
                        return FAIL;
                    }
                }
            } else {
                let mut level: c_int = foldLevelWin(wp, off + (*fp).fd_top);
                if (*fp).fd_flags as c_int == FD_CLOSED as c_int
                    && (*wp).w_onebuf_opt.wo_fdl >= level as OptInt
                    || (*fp).fd_flags as c_int != FD_CLOSED as c_int
                        && (*wp).w_onebuf_opt.wo_fdl < level as OptInt
                {
                    if put_fold_open_close(fd, fp, off) == FAIL {
                        return FAIL;
                    }
                }
            }
        }
        fp = fp.offset(1);
        i += 1;
    }
    return OK;
}

/// Write the open or close command to "fd".
///
/// Returns fAIL when writing failed.
pub(super) unsafe extern "C" fn put_fold_open_close(
    mut fd: *mut FILE,
    mut fp: *mut fold_T,
    mut off: linenr_T,
) -> c_int {
    if fprintf(fd, c"%d".as_ptr(), (*fp).fd_top + off) < 0
        || put_eol(fd) == FAIL
        || fprintf(
            fd,
            c"sil! normal! z%c".as_ptr(),
            if (*fp).fd_flags as c_int == FD_CLOSED as c_int {
                'c' as c_int
            } else {
                'o' as c_int
            },
        ) < 0
        || put_eol(fd) == FAIL
    {
        return FAIL;
    }
    return OK;
}
