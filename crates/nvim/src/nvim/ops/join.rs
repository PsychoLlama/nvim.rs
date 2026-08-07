//! `J` and `gJ` -- joining lines.
//!
//! `do_join` builds the joined line in one allocation: it measures every
//! source line first (after `skip_comment` has trimmed a comment leader,
//! which is what 'formatoptions' `j` asks for), decides how many spaces go
//! in each seam ('joinspaces', the sentence rule, 'cpoptions' `j`, and the
//! `FO_MBYTE_JOIN` rules that suppress a space between two multi-byte
//! characters), and then copies.  The marks and the cursor are adjusted from
//! the same measurements.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn skip_comment(
    mut line: *mut ::core::ffi::c_char,
    mut process: bool,
    mut include_space: bool,
    mut is_comment: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut comment_flags: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut leader_offset: ::core::ffi::c_int =
            get_last_leader_offset(line, &raw mut comment_flags);
        *is_comment = false_0 != 0;
        if leader_offset != -1 as ::core::ffi::c_int {
            while *comment_flags != 0 {
                if *comment_flags as ::core::ffi::c_int == COM_END
                    || *comment_flags as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                {
                    break;
                }
                comment_flags = comment_flags.offset(1);
            }
            if *comment_flags as ::core::ffi::c_int != COM_END {
                *is_comment = true_0 != 0;
            }
        }
        if process as ::core::ffi::c_int == false_0 {
            return line;
        }
        let mut lead_len: ::core::ffi::c_int =
            get_leader_len(line, &raw mut comment_flags, false_0 != 0, include_space);
        if lead_len == 0 as ::core::ffi::c_int {
            return line;
        }
        while *comment_flags != 0 {
            if *comment_flags as ::core::ffi::c_int == COM_END
                || *comment_flags as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            {
                break;
            }
            comment_flags = comment_flags.offset(1);
        }
        if *comment_flags as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            || *comment_flags as ::core::ffi::c_int == NUL
        {
            line = line.offset(lead_len as isize);
        }
        return line;
    }
}

pub unsafe extern "C" fn do_join(
    mut count: size_t,
    mut insert_space: bool,
    mut save_undo: bool,
    mut use_formatoptions: bool,
    mut setmark: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut col: colnr_T = 0;
        let mut newp_len: size_t = 0;
        let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut t_1: linenr_T = 0;
        let mut curr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut curr_start: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut cend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut endcurr1: ::core::ffi::c_int = NUL;
        let mut endcurr2: ::core::ffi::c_int = NUL;
        let mut currsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sumsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ret: ::core::ffi::c_int = OK;
        let mut comments: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
        let mut remove_comments: bool = use_formatoptions as ::core::ffi::c_int != 0
            && has_format_option(FO_REMOVE_COMS) as ::core::ffi::c_int != 0;
        let mut prev_was_comment: bool = false_0 != 0;
        '_c2rust_label: {
            if count >= 1 as size_t {
            } else {
                __assert_fail(
                    b"count >= 1\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1899 as ::core::ffi::c_uint,
                    b"int do_join(size_t, _Bool, _Bool, _Bool, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if save_undo as ::core::ffi::c_int != 0
            && u_save(
                (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                (*curwin.get()).w_cursor.lnum + count as linenr_T,
            ) == FAIL
        {
            return FAIL;
        }
        let mut spaces: *mut ::core::ffi::c_char =
            xcalloc(count, 1 as size_t) as *mut ::core::ffi::c_char;
        if remove_comments {
            comments = xcalloc(count, ::core::mem::size_of::<::core::ffi::c_int>())
                as *mut ::core::ffi::c_int;
        }
        let mut t: linenr_T = 0 as linenr_T;
        '_theend: {
            while t < count as linenr_T {
                curr_start = ml_get((*curwin.get()).w_cursor.lnum + t);
                curr = curr_start;
                if t == 0 as linenr_T
                    && setmark as ::core::ffi::c_int != 0
                    && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                {
                    (*(*curwin.get()).w_buffer).b_op_start.lnum = (*curwin.get()).w_cursor.lnum;
                    (*(*curwin.get()).w_buffer).b_op_start.col = strlen(curr) as colnr_T;
                }
                if remove_comments {
                    if t > 0 as linenr_T && prev_was_comment as ::core::ffi::c_int != 0 {
                        let mut new_curr: *mut ::core::ffi::c_char = skip_comment(
                            curr,
                            true_0 != 0,
                            insert_space,
                            &raw mut prev_was_comment,
                        );
                        *comments.offset(t as isize) =
                            new_curr.offset_from(curr) as ::core::ffi::c_int;
                        curr = new_curr;
                    } else {
                        curr = skip_comment(
                            curr,
                            false_0 != 0,
                            insert_space,
                            &raw mut prev_was_comment,
                        );
                    }
                }
                if insert_space as ::core::ffi::c_int != 0 && t > 0 as linenr_T {
                    curr = skipwhite(curr);
                    if *curr as ::core::ffi::c_int != NUL
                        && *curr as ::core::ffi::c_int != ')' as ::core::ffi::c_int
                        && sumsize != 0 as ::core::ffi::c_int
                        && endcurr1 != TAB
                        && (!has_format_option(FO_MBYTE_JOIN)
                            || utf_ptr2char(curr) < 0x100 as ::core::ffi::c_int
                                && endcurr1 < 0x100 as ::core::ffi::c_int)
                        && (!has_format_option(FO_MBYTE_JOIN2)
                            || utf_ptr2char(curr) < 0x100 as ::core::ffi::c_int
                                && !utf_eat_space(endcurr1)
                            || endcurr1 < 0x100 as ::core::ffi::c_int
                                && !utf_eat_space(utf_ptr2char(curr)))
                    {
                        if endcurr1 == ' ' as ::core::ffi::c_int {
                            endcurr1 = endcurr2;
                        } else {
                            *spaces.offset(t as isize) += 1;
                        }
                        if p_js.get() != 0
                            && (endcurr1 == '.' as ::core::ffi::c_int
                                || endcurr1 == '?' as ::core::ffi::c_int
                                || endcurr1 == '!' as ::core::ffi::c_int)
                        {
                            *spaces.offset(t as isize) += 1;
                        }
                    }
                }
                if t > 0 as linenr_T && curbuf_splice_pending.get() == 0 as ::core::ffi::c_int {
                    let mut removed: colnr_T = curr.offset_from(curr_start) as colnr_T;
                    extmark_splice(
                        curbuf.get(),
                        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int,
                        sumsize as colnr_T,
                        1 as ::core::ffi::c_int,
                        removed,
                        (removed as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as bcount_t,
                        0 as ::core::ffi::c_int,
                        *spaces.offset(t as isize) as colnr_T,
                        *spaces.offset(t as isize) as bcount_t,
                        kExtmarkUndo,
                    );
                }
                currsize = strlen(curr) as ::core::ffi::c_int;
                sumsize += currsize + *spaces.offset(t as isize) as ::core::ffi::c_int;
                endcurr2 = NUL;
                endcurr1 = endcurr2;
                if insert_space as ::core::ffi::c_int != 0 && currsize > 0 as ::core::ffi::c_int {
                    cend = curr.offset(currsize as isize);
                    cend = cend.offset(
                        -((utf_head_off(curr, cend.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                    endcurr1 = utf_ptr2char(cend);
                    if cend > curr {
                        cend = cend.offset(
                            -((utf_head_off(curr, cend.offset(-(1 as ::core::ffi::c_int as isize)))
                                + 1 as ::core::ffi::c_int) as isize),
                        );
                        endcurr2 = utf_ptr2char(cend);
                    }
                }
                line_breakcheck();
                if got_int.get() {
                    ret = FAIL;
                    break '_theend;
                } else {
                    t += 1;
                }
            }
            col = sumsize as colnr_T
                - currsize as colnr_T
                - *spaces.offset(count.wrapping_sub(1 as size_t) as isize) as colnr_T;
            newp_len = sumsize as size_t;
            newp = xmallocz(newp_len) as *mut ::core::ffi::c_char;
            cend = newp.offset(sumsize as isize);
            (*curbuf_splice_pending.ptr()) += 1;
            let mut t_0: linenr_T = count as linenr_T - 1 as linenr_T;
            loop {
                cend = cend.offset(-(currsize as isize));
                memmove(
                    cend as *mut ::core::ffi::c_void,
                    curr as *const ::core::ffi::c_void,
                    currsize as size_t,
                );
                if *spaces.offset(t_0 as isize) as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                    cend =
                        cend.offset(-(*spaces.offset(t_0 as isize) as ::core::ffi::c_int as isize));
                    memset(
                        cend as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        *spaces.offset(t_0 as isize) as size_t,
                    );
                }
                let spaces_removed: ::core::ffi::c_int = (curr.offset_from(curr_start)
                    - *spaces.offset(t_0 as isize) as isize)
                    as ::core::ffi::c_int;
                let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum + t_0;
                let mut mincol: colnr_T = 0 as colnr_T;
                let mut lnum_amount: linenr_T = -t_0;
                let mut col_amount: colnr_T =
                    (cend.offset_from(newp) - spaces_removed as isize) as colnr_T;
                mark_col_adjust(lnum, mincol, lnum_amount, col_amount, spaces_removed);
                if t_0 == 0 as linenr_T {
                    break;
                }
                curr_start = ml_get((*curwin.get()).w_cursor.lnum + t_0 - 1 as linenr_T);
                curr = curr_start;
                if remove_comments {
                    curr = curr.offset(*comments.offset((t_0 - 1 as linenr_T) as isize) as isize);
                }
                if insert_space as ::core::ffi::c_int != 0 && t_0 > 1 as linenr_T {
                    curr = skipwhite(curr);
                }
                currsize = strlen(curr) as ::core::ffi::c_int;
                t_0 -= 1;
            }
            ml_replace_len((*curwin.get()).w_cursor.lnum, newp, newp_len, false_0 != 0);
            if setmark as ::core::ffi::c_int != 0
                && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                (*(*curwin.get()).w_buffer).b_op_end.lnum = (*curwin.get()).w_cursor.lnum;
                (*(*curwin.get()).w_buffer).b_op_end.col = sumsize as colnr_T;
            }
            changed_lines(
                curbuf.get(),
                (*curwin.get()).w_cursor.lnum,
                currsize as colnr_T,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
            t_1 = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum += 1;
            del_lines(count as linenr_T - 1 as linenr_T, false_0 != 0);
            (*curwin.get()).w_cursor.lnum = t_1;
            (*curbuf_splice_pending.ptr()) -= 1;
            (*curbuf.get()).deleted_bytes2 = 0 as size_t;
            (*curwin.get()).w_cursor.col = (if !vim_strchr(p_cpo.get(), CPO_JOINCOL).is_null() {
                currsize
            } else {
                col as ::core::ffi::c_int
            }) as colnr_T;
            check_cursor_col(curwin.get());
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_set_curswant = true_0;
        }
        xfree(spaces as *mut ::core::ffi::c_void);
        if remove_comments {
            xfree(comments as *mut ::core::ffi::c_void);
        }
        return ret;
    }
}
