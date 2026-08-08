//! Moving the cursor one line or one character, from anywhere.
//!
//! These live in edit.c for historical reasons and are called from all over
//! the tree; they are not Insert-mode specific.  What they have in common is
//! that each is the *legal* version of an obvious operation: `oneright` and
//! `oneleft` refuse to step onto the NUL past the end of a line unless
//! 'virtualedit' or 'whichwrap' allow it and know about composing
//! characters; `cursor_up`/`cursor_down` treat a closed fold as one line and
//! skip concealed lines; `beginline` is the "go to the start of the line"
//! whose meaning 'startofline' changes.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn beginline(mut flags: ::core::ffi::c_int) {
    unsafe {
        if flags & BL_SOL != 0 && p_sol.get() == 0 {
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        } else {
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            if flags & (BL_WHITE | BL_SOL) != 0 {
                let mut ptr: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                while ascii_iswhite(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    && !(flags & BL_FIX != 0
                        && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL)
                {
                    (*curwin.get()).w_cursor.col += 1;
                    ptr = ptr.offset(1);
                }
            }
            (*curwin.get()).w_set_curswant = true_0;
        }
        adjust_skipcol();
    }
}

pub unsafe extern "C" fn oneright() -> ::core::ffi::c_int {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if virtual_active(curwin.get()) {
            let mut prevpos: pos_T = (*curwin.get()).w_cursor;
            ptr = get_cursor_pos_ptr();
            coladvance(
                curwin.get(),
                getviscol()
                    + (if *ptr as ::core::ffi::c_int != TAB
                        && vim_isprintc(utf_ptr2char(ptr)) as ::core::ffi::c_int != 0
                    {
                        ptr2cells(ptr)
                    } else {
                        1 as colnr_T
                    }),
            );
            (*curwin.get()).w_set_curswant = true_0;
            return if prevpos.col != (*curwin.get()).w_cursor.col
                || prevpos.coladd != (*curwin.get()).w_cursor.coladd
            {
                OK
            } else {
                FAIL
            };
        }
        ptr = get_cursor_pos_ptr();
        if *ptr as ::core::ffi::c_int == NUL {
            return FAIL;
        }
        let mut l: ::core::ffi::c_int = utfc_ptr2len(ptr);
        if *ptr.offset(l as isize) as ::core::ffi::c_int == NUL
            && get_ve_flags(curwin.get())
                & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
        {
            return FAIL;
        }
        (*curwin.get()).w_cursor.col += l;
        (*curwin.get()).w_set_curswant = true_0;
        adjust_skipcol();
        return OK;
    }
}

pub unsafe extern "C" fn oneleft() -> ::core::ffi::c_int {
    unsafe {
        if virtual_active(curwin.get()) {
            let mut v: ::core::ffi::c_int = getviscol();
            if v == 0 as ::core::ffi::c_int {
                return FAIL;
            }
            let mut width: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            loop {
                coladvance(curwin.get(), v as colnr_T - width as colnr_T);
                if getviscol() < v {
                    break;
                }
                width += 1;
            }
            if (*curwin.get()).w_cursor.coladd == 1 as ::core::ffi::c_int {
                let mut ptr: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                if *ptr as ::core::ffi::c_int != TAB
                    && vim_isprintc(utf_ptr2char(ptr)) as ::core::ffi::c_int != 0
                    && ptr2cells(ptr) > 1 as ::core::ffi::c_int
                {
                    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
            (*curwin.get()).w_set_curswant = true_0;
            adjust_skipcol();
            return OK;
        }
        if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
            return FAIL;
        }
        (*curwin.get()).w_set_curswant = true_0;
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        adjust_skipcol();
        return OK;
    }
}

pub unsafe extern "C" fn cursor_up_inner(
    mut wp: *mut win_T,
    mut n: linenr_T,
    mut skip_conceal: bool,
) {
    unsafe {
        let mut lnum: linenr_T = (*wp).w_cursor.lnum;
        if n >= lnum {
            lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else if win_lines_concealed(wp) {
            hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
            loop {
                let c2rust_fresh3 = n;
                n = n - 1;
                if c2rust_fresh3 == 0 {
                    break;
                }
                lnum -= 1;
                if lnum <= 1 as linenr_T {
                    break;
                }
                n = (n as ::core::ffi::c_int
                    + (skip_conceal as ::core::ffi::c_int != 0
                        && decor_conceal_line(
                            wp,
                            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            true_0 != 0,
                        ) as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int) as linenr_T;
                if n > 0 as linenr_T
                    || !(State.get() & MODE_INSERT != 0
                        || fdo_flags.get()
                            & kOptFdoFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0)
                {
                    hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
                }
            }
            lnum = if lnum > 1 as linenr_T {
                lnum
            } else {
                1 as linenr_T
            };
        } else {
            lnum -= n;
        }
        (*wp).w_cursor.lnum = lnum;
    }
}

pub unsafe extern "C" fn cursor_up(mut n: linenr_T, mut upd_topline: bool) -> ::core::ffi::c_int {
    unsafe {
        if n > 0 as linenr_T && (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
            return FAIL;
        }
        cursor_up_inner(curwin.get(), n, false_0 != 0);
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
        if upd_topline {
            update_topline(curwin.get());
        }
        return OK;
    }
}

pub unsafe extern "C" fn cursor_down_inner(
    mut wp: *mut win_T,
    mut n: ::core::ffi::c_int,
    mut skip_conceal: bool,
) {
    unsafe {
        let mut lnum: linenr_T = (*wp).w_cursor.lnum;
        let mut line_count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
        if lnum + n as linenr_T >= line_count {
            lnum = line_count;
        } else if win_lines_concealed(wp) {
            let mut last: linenr_T = 0;
            loop {
                let c2rust_fresh2 = n;
                n = n - 1;
                if c2rust_fresh2 == 0 {
                    break;
                }
                if hasFoldingWin(
                    wp,
                    lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut last,
                    true_0 != 0,
                    ::core::ptr::null_mut::<foldinfo_T>(),
                ) {
                    lnum = last + 1 as linenr_T;
                } else {
                    lnum += 1;
                }
                if lnum >= line_count {
                    break;
                }
                n += (skip_conceal as ::core::ffi::c_int != 0
                    && decor_conceal_line(
                        wp,
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int;
            }
            lnum = if lnum < line_count { lnum } else { line_count };
        } else {
            lnum += n as linenr_T;
        }
        (*wp).w_cursor.lnum = lnum;
    }
}

pub unsafe extern "C" fn cursor_down(
    mut n: ::core::ffi::c_int,
    mut upd_topline: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        hasFoldingWin(
            curwin.get(),
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut lnum,
            true_0 != 0,
            ::core::ptr::null_mut::<foldinfo_T>(),
        );
        if n > 0 as ::core::ffi::c_int && lnum >= (*(*curwin.get()).w_buffer).b_ml.ml_line_count {
            return FAIL;
        }
        cursor_down_inner(curwin.get(), n, false_0 != 0);
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
        if upd_topline {
            update_topline(curwin.get());
        }
        return OK;
    }
}
