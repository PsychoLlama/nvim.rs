//! `:move` and `:copy` -- relocating a range of lines within the buffer.
//!
//! `do_move` is the harder one: it has to move the lines, then fix up every
//! mark, extmark and fold that pointed into either the source or the
//! destination, and it does that by adjusting the ranges rather than replaying
//! the move.  `ex_copy` is `:copy`/`:t`, which only ever appends.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_move(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut dest: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if dest >= line1 && dest < line2 {
            emsg(gettext(
                c"E134: Cannot move a range of lines into itself".as_ptr(),
            ));
            return FAIL;
        }
        if dest == line1 - 1 as linenr_T || dest == line2 {
            (*curwin.get()).w_cursor.lnum = if dest >= line1 {
                dest
            } else {
                dest + (line2 - line1) + 1 as linenr_T
            };
            return OK;
        }
        let mut start_byte: bcount_t = ml_find_line_or_offset(
            curbuf.get(),
            line1,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        ) as bcount_t;
        let mut end_byte: bcount_t = ml_find_line_or_offset(
            curbuf.get(),
            line2 + 1 as linenr_T,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        ) as bcount_t;
        let mut extent_byte: bcount_t = end_byte - start_byte;
        let mut dest_byte: bcount_t = ml_find_line_or_offset(
            curbuf.get(),
            dest + 1 as linenr_T,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        ) as bcount_t;
        let mut num_lines: linenr_T = line2 - line1 + 1 as linenr_T;
        if u_save(dest, dest + 1 as linenr_T) == FAIL {
            return FAIL;
        }
        let mut l: linenr_T = 0;
        let mut extra: linenr_T = 0;
        extra = 0 as ::core::ffi::c_int as linenr_T;
        l = line1;
        while l <= line2 {
            let mut str: *mut ::core::ffi::c_char =
                xstrnsave(ml_get(l + extra), ml_get_len(l + extra) as size_t);
            ml_append(dest + l - line1, str, 0 as colnr_T, false_0 != 0);
            xfree(str as *mut ::core::ffi::c_void);
            if dest < line1 {
                extra += 1;
            }
            l += 1;
        }
        let mut last_line: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
        mark_adjust_nofold(line1, line2, last_line - line2, 0 as linenr_T, kExtmarkNOOP);
        (*disable_fold_update.ptr()) += 1;
        changed_lines(
            curbuf.get(),
            last_line - num_lines + 1 as linenr_T,
            0 as colnr_T,
            last_line + 1 as linenr_T,
            num_lines,
            false_0 != 0,
        );
        (*disable_fold_update.ptr()) -= 1;
        let mut line_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut byte_off: bcount_t = 0 as bcount_t;
        if dest >= line2 {
            mark_adjust_nofold(
                line2 + 1 as linenr_T,
                dest,
                -num_lines,
                0 as linenr_T,
                kExtmarkNOOP,
            );
            let mut tab: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tab.is_null() {
                let mut win: *mut win_T = if tab == curtab.get() {
                    firstwin.get()
                } else {
                    (*tab).tp_firstwin
                };
                while !win.is_null() {
                    if (*win).w_buffer == curbuf.get() {
                        foldMoveRange(win, &raw mut (*win).w_folds, line1, line2, dest);
                    }
                    win = (*win).w_next;
                }
                tab = (*tab).tp_next as *mut tabpage_T;
            }
            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*curbuf.get()).b_op_start.lnum = dest - num_lines + 1 as linenr_T;
                (*curbuf.get()).b_op_end.lnum = dest;
            }
            line_off = -num_lines as ::core::ffi::c_int;
            byte_off = -extent_byte;
        } else {
            mark_adjust_nofold(
                dest + 1 as linenr_T,
                line1 - 1 as linenr_T,
                num_lines,
                0 as linenr_T,
                kExtmarkNOOP,
            );
            let mut tab_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tab_0.is_null() {
                let mut win_0: *mut win_T = if tab_0 == curtab.get() {
                    firstwin.get()
                } else {
                    (*tab_0).tp_firstwin
                };
                while !win_0.is_null() {
                    if (*win_0).w_buffer == curbuf.get() {
                        foldMoveRange(
                            win_0,
                            &raw mut (*win_0).w_folds,
                            dest + 1 as linenr_T,
                            line1 - 1 as linenr_T,
                            line2,
                        );
                    }
                    win_0 = (*win_0).w_next;
                }
                tab_0 = (*tab_0).tp_next as *mut tabpage_T;
            }
            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*curbuf.get()).b_op_start.lnum = dest + 1 as linenr_T;
                (*curbuf.get()).b_op_end.lnum = dest + num_lines;
            }
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
        }
        mark_adjust_nofold(
            last_line - num_lines + 1 as linenr_T,
            last_line,
            -(last_line - dest - extra),
            0 as linenr_T,
            kExtmarkNOOP,
        );
        (*disable_fold_update.ptr()) += 1;
        changed_lines(
            curbuf.get(),
            last_line - num_lines + 1 as linenr_T,
            0 as colnr_T,
            last_line + 1 as linenr_T,
            -extra,
            false_0 != 0,
        );
        (*disable_fold_update.ptr()) -= 1;
        buf_updates_send_changes(
            curbuf.get(),
            dest + 1 as linenr_T,
            num_lines as int64_t,
            0 as int64_t,
        );
        if u_save(line1 + extra - 1 as linenr_T, line2 + extra + 1 as linenr_T) == FAIL {
            return FAIL;
        }
        l = line1;
        while l <= line2 {
            ml_delete_flags(line1 + extra, ML_DEL_MESSAGE as ::core::ffi::c_int);
            l += 1;
        }
        if global_busy.get() == 0 && num_lines as OptInt > p_report.get() {
            smsg_c!(
                0 as ::core::ffi::c_int,
                ngettext(
                    c"%ld line moved".as_ptr(),
                    c"%ld lines moved".as_ptr(),
                    num_lines as ::core::ffi::c_ulong,
                ),
                num_lines as int64_t,
            );
        }
        extmark_move_region(
            curbuf.get(),
            line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as colnr_T,
            start_byte,
            line2 as ::core::ffi::c_int - line1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            0 as colnr_T,
            extent_byte,
            dest as ::core::ffi::c_int + line_off,
            0 as colnr_T,
            dest_byte + byte_off,
            kExtmarkUndo,
        );
        if dest >= line1 {
            (*curwin.get()).w_cursor.lnum = dest;
        } else {
            (*curwin.get()).w_cursor.lnum = dest + (line2 - line1) + 1 as linenr_T;
        }
        if line1 < dest {
            dest = (dest as ::core::ffi::c_int + (num_lines + 1 as linenr_T) as ::core::ffi::c_int)
                as linenr_T;
            last_line = (*curbuf.get()).b_ml.ml_line_count;
            dest = if dest < last_line + 1 as linenr_T {
                dest
            } else {
                last_line + 1 as linenr_T
            };
            changed_lines(
                curbuf.get(),
                line1,
                0 as colnr_T,
                dest,
                0 as linenr_T,
                false_0 != 0,
            );
        } else {
            changed_lines(
                curbuf.get(),
                dest + 1 as linenr_T,
                0 as colnr_T,
                line1 + num_lines,
                0 as linenr_T,
                false_0 != 0,
            );
        }
        buf_updates_send_changes(
            curbuf.get(),
            line1 + extra,
            0 as int64_t,
            num_lines as int64_t,
        );
        return OK;
    }
}

pub unsafe extern "C" fn ex_copy(mut line1: linenr_T, mut line2: linenr_T, mut n: linenr_T) {
    unsafe {
        let mut count: linenr_T = line2 - line1 + 1 as linenr_T;
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start.lnum = n + 1 as linenr_T;
            (*curbuf.get()).b_op_end.lnum = n + count;
            (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
        }
        if u_save(n, n + 1 as linenr_T) == FAIL {
            return;
        }
        (*curwin.get()).w_cursor.lnum = n;
        while line1 <= line2 {
            let mut p: *mut ::core::ffi::c_char =
                xstrnsave(ml_get(line1), ml_get_len(line1) as size_t);
            ml_append((*curwin.get()).w_cursor.lnum, p, 0 as colnr_T, false_0 != 0);
            xfree(p as *mut ::core::ffi::c_void);
            if line1 == n {
                line1 = (*curwin.get()).w_cursor.lnum;
            }
            line1 += 1;
            if (*curwin.get()).w_cursor.lnum < line1 {
                line1 += 1;
            }
            if (*curwin.get()).w_cursor.lnum < line2 {
                line2 += 1;
            }
            (*curwin.get()).w_cursor.lnum += 1;
        }
        appended_lines_mark(n, count as ::core::ffi::c_int);
        if VIsual_active.get() {
            check_pos(curbuf.get(), VIsual.ptr());
        }
        msgmore(count as ::core::ffi::c_int);
    }
}
