//! Parsing `'diffopt'` and `'diffanchors'`.
//!
//! `diffopt_changed` is the option's validator *and* its effect: it parses the
//! whole comma-separated value into the `diff_flags`, `diff_algorithm`,
//! `diff_word_gap` and `linematch_lines` cells, rejecting the value whole if any
//! item is unknown.  `parse_diffanchors` is the separate `'diffanchors'` grammar,
//! which names line ranges the diff must be split at.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn parse_diffanchors(
    mut check_only: bool,
    mut buf: *mut buf_T,
    mut anchors: *mut linenr_T,
    mut num_anchors: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut dia: *mut ::core::ffi::c_char = if *(*buf).b_p_dia as ::core::ffi::c_int == NUL {
            p_dia.get()
        } else {
            (*buf).b_p_dia
        };
        let mut orig_curbuf: *mut buf_T = curbuf.get();
        let mut orig_curwin: *mut win_T = curwin.get();
        let mut bufwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if check_only {
            bufwin = curwin.get();
        } else {
            bufwin = firstwin.get();
            while !bufwin.is_null() {
                if (*bufwin).w_buffer == buf && (*bufwin).w_onebuf_opt.wo_diff != 0 {
                    break;
                }
                bufwin = (*bufwin).w_next;
            }
            if bufwin.is_null() && *dia as ::core::ffi::c_int != NUL {
                emsg(gettext(
                    &raw const e_diff_anchors_with_hidden_windows as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
        }
        i = 0 as ::core::ffi::c_int;
        while i < MAX_DIFF_ANCHORS && *dia as ::core::ffi::c_int != NUL {
            if *dia as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                return FAIL;
            }
            curbuf.set(buf);
            curwin.set(bufwin);
            let mut errormsg: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            let mut lnum: linenr_T = get_address(
                ::core::ptr::null_mut::<exarg_T>(),
                &raw mut dia,
                ADDR_LINES,
                check_only,
                true_0 != 0,
                false_0,
                1 as ::core::ffi::c_int,
                &raw mut errormsg,
            );
            curbuf.set(orig_curbuf);
            curwin.set(orig_curwin);
            if !errormsg.is_null() {
                emsg(errormsg);
            }
            if dia.is_null() {
                return FAIL;
            }
            if *dia as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                && *dia as ::core::ffi::c_int != NUL
            {
                return FAIL;
            }
            if !check_only
                && (lnum == MAXLNUM as ::core::ffi::c_int as linenr_T
                    || lnum <= 0 as linenr_T
                    || lnum > (*buf).b_ml.ml_line_count + 1 as linenr_T)
            {
                emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
                return FAIL;
            }
            if !anchors.is_null() {
                *anchors.offset(i as isize) = lnum;
            }
            if *dia as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                dia = dia.offset(1);
            }
            i += 1;
        }
        if i == MAX_DIFF_ANCHORS && *dia as ::core::ffi::c_int != NUL {
            semsg(
                gettext(
                    &raw const e_cannot_have_more_than_nr_diff_anchors
                        as *const ::core::ffi::c_char,
                ),
                MAX_DIFF_ANCHORS,
            );
            return FAIL;
        }
        if !num_anchors.is_null() {
            *num_anchors = i;
        }
        return OK;
    }
}

pub unsafe extern "C" fn diffanchors_changed(mut buflocal: bool) -> ::core::ffi::c_int {
    unsafe {
        let mut result: ::core::ffi::c_int = parse_diffanchors(
            true_0 != 0,
            curbuf.get(),
            ::core::ptr::null_mut::<linenr_T>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if result == OK && diff_flags.get() & DIFF_ANCHOR != 0 {
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                if !buflocal {
                    (*tp).tp_diff_invalid = true_0;
                } else {
                    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while idx < DB_COUNT {
                        if (*tp).tp_diffbuf[idx as usize] == curbuf.get() {
                            (*tp).tp_diff_invalid = true_0;
                            break;
                        } else {
                            idx += 1;
                        }
                    }
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        return result;
    }
}

pub unsafe extern "C" fn diffopt_changed() -> ::core::ffi::c_int {
    unsafe {
        let mut diff_context_new: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
        let mut linematch_lines_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut diff_flags_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut diff_foldcolumn_new: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
        let mut diff_algorithm_new: u64 = 0;
        let mut diff_indent_heuristic: u64 = 0;
        let mut p: *mut ::core::ffi::c_char = p_dip.get();
        while *p as ::core::ffi::c_int != NUL {
            if strncmp(
                p,
                b"filler\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(6 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_FILLER;
            } else if strncmp(
                p,
                b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(6 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_ANCHOR;
            } else if strncmp(
                p,
                b"context:\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_isdigit(*p.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(8 as ::core::ffi::c_int as isize);
                diff_context_new = getdigits_int(&raw mut p, false_0 != 0, diff_context_new);
            } else if strncmp(
                p,
                b"iblank\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(6 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_IBLANK;
            } else if strncmp(
                p,
                b"icase\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(5 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_ICASE;
            } else if strncmp(
                p,
                b"iwhiteall\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(9 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_IWHITEALL;
            } else if strncmp(
                p,
                b"iwhiteeol\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(9 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_IWHITEEOL;
            } else if strncmp(
                p,
                b"iwhite\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(6 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_IWHITE;
            } else if strncmp(
                p,
                b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(10 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_HORIZONTAL;
            } else if strncmp(
                p,
                b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(8 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_VERTICAL;
            } else if strncmp(
                p,
                b"foldcolumn:\0".as_ptr() as *const ::core::ffi::c_char,
                11 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_isdigit(*p.offset(11 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(11 as ::core::ffi::c_int as isize);
                diff_foldcolumn_new = getdigits_int(&raw mut p, false_0 != 0, diff_foldcolumn_new);
            } else if strncmp(
                p,
                b"hiddenoff\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(9 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_HIDDEN_OFF;
            } else if strncmp(
                p,
                b"closeoff\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(8 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_CLOSE_OFF;
            } else if strncmp(
                p,
                b"followwrap\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(10 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_FOLLOWWRAP;
            } else if strncmp(
                p,
                b"indent-heuristic\0".as_ptr() as *const ::core::ffi::c_char,
                16 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(16 as ::core::ffi::c_int as isize);
                diff_indent_heuristic = XDF_INDENT_HEURISTIC;
            } else if strncmp(
                p,
                b"internal\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(8 as ::core::ffi::c_int as isize);
                diff_flags_new |= DIFF_INTERNAL;
            } else if strncmp(
                p,
                b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(10 as ::core::ffi::c_int as isize);
                if strncmp(
                    p,
                    b"myers\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(5 as ::core::ffi::c_int as isize);
                    diff_algorithm_new = 0;
                } else if strncmp(
                    p,
                    b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
                    7 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(7 as ::core::ffi::c_int as isize);
                    diff_algorithm_new = XDF_NEED_MINIMAL;
                } else if strncmp(
                    p,
                    b"patience\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(8 as ::core::ffi::c_int as isize);
                    diff_algorithm_new = XDF_PATIENCE_DIFF;
                } else if strncmp(
                    p,
                    b"histogram\0".as_ptr() as *const ::core::ffi::c_char,
                    9 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(9 as ::core::ffi::c_int as isize);
                    diff_algorithm_new = XDF_HISTOGRAM_DIFF;
                } else {
                    return FAIL;
                }
            } else if strncmp(
                p,
                b"inline:\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(7 as ::core::ffi::c_int as isize);
                if strncmp(
                    p,
                    b"none\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(4 as ::core::ffi::c_int as isize);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_NONE;
                } else if strncmp(
                    p,
                    b"simple\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(6 as ::core::ffi::c_int as isize);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_SIMPLE;
                } else if strncmp(
                    p,
                    b"char\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(4 as ::core::ffi::c_int as isize);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_CHAR;
                } else if strncmp(
                    p,
                    b"word\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    p = p.offset(4 as ::core::ffi::c_int as isize);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_WORD;
                } else {
                    return FAIL;
                }
            } else if strncmp(
                p,
                b"linematch:\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_isdigit(*p.offset(10 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(10 as ::core::ffi::c_int as isize);
                linematch_lines_new = getdigits_int(&raw mut p, false_0 != 0, linematch_lines_new);
                diff_flags_new |= DIFF_LINEMATCH;
                diff_flags_new |= DIFF_FILLER;
            }
            if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != NUL
            {
                return FAIL;
            }
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        }
        diff_algorithm_new |= diff_indent_heuristic;
        if diff_flags_new & DIFF_HORIZONTAL != 0 && diff_flags_new & DIFF_VERTICAL != 0 {
            return FAIL;
        }
        if diff_flags.get() != diff_flags_new || diff_algorithm.get() != diff_algorithm_new {
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                (*tp).tp_diff_invalid = true_0;
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        diff_flags.set(diff_flags_new);
        diff_context.set(if diff_context_new == 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            diff_context_new
        });
        linematch_lines.set(linematch_lines_new);
        diff_foldcolumn.set(diff_foldcolumn_new);
        diff_algorithm.set(diff_algorithm_new);
        diff_redraw(true_0 != 0);
        check_scrollbind(0 as linenr_T, 0 as ::core::ffi::c_int);
        return OK;
    }
}

pub unsafe extern "C" fn diffopt_horizontal() -> bool {
    return diff_flags.get() & DIFF_HORIZONTAL != 0 as ::core::ffi::c_int;
}

pub unsafe extern "C" fn diffopt_hiddenoff() -> bool {
    return diff_flags.get() & DIFF_HIDDEN_OFF != 0 as ::core::ffi::c_int;
}

pub unsafe extern "C" fn diffopt_closeoff() -> bool {
    return diff_flags.get() & DIFF_CLOSE_OFF != 0 as ::core::ffi::c_int;
}

pub unsafe extern "C" fn diffopt_filler() -> bool {
    return diff_flags.get() & DIFF_FILLER != 0 as ::core::ffi::c_int;
}
