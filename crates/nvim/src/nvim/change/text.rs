//! Inserting and deleting text, in bytes, characters and lines.
//!
//! The primitives every operator eventually calls.  `ins_char_bytes` is the one
//! with the substance: it is also Replace mode's insert, so it pushes the bytes
//! it overwrites onto the replace stack, and it has to respect a composing
//! character joining the character *before* the cursor rather than replacing
//! it.  `del_bytes` is its mirror and carries the `fixpos`/'virtualedit'
//! question of where the cursor lands when the last character of a line goes
//! away.  `del_lines` and `truncate_line` are the line-level pair.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ins_bytes(mut p: *mut ::core::ffi::c_char) {
    unsafe {
        ins_bytes_len(p, strlen(p));
    }
}

pub unsafe extern "C" fn ins_bytes_len(mut p: *mut ::core::ffi::c_char, mut len: size_t) {
    unsafe {
        let mut n: size_t = 0;
        let mut i: size_t = 0 as size_t;
        while i < len {
            n = utfc_ptr2len_len(
                p.offset(i as isize),
                len.wrapping_sub(i) as ::core::ffi::c_int,
            ) as size_t;
            ins_char_bytes(p.offset(i as isize), n);
            i = i.wrapping_add(n);
        }
    }
}

pub unsafe extern "C" fn ins_char(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 7] = [0; 7];
        let mut n: size_t = utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as size_t;
        if buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            buf[0 as ::core::ffi::c_int as usize] = '\n' as ::core::ffi::c_char;
        }
        ins_char_bytes(&raw mut buf as *mut ::core::ffi::c_char, n);
    }
}

pub unsafe extern "C" fn ins_char_bytes(mut buf: *mut ::core::ffi::c_char, mut charlen: size_t) {
    unsafe {
        if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
        {
            coladvance_force(getviscol());
        }
        let mut col: size_t = (*curwin.get()).w_cursor.col as size_t;
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut linelen: size_t = (ml_get_len(lnum) as size_t).wrapping_add(1 as size_t);
        let mut oldlen: size_t = 0 as size_t;
        let mut newlen: size_t = charlen;
        if State.get() & REPLACE_FLAG != 0 {
            if State.get() & VREPLACE_FLAG != 0 {
                let mut old_list: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
                if old_list != 0 && vim_strchr(p_cpo.get(), CPO_LISTWM).is_null() {
                    (*curwin.get()).w_onebuf_opt.wo_list = false_0;
                }
                let mut vcol: colnr_T = 0;
                getvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut vcol,
                    ::core::ptr::null_mut::<colnr_T>(),
                );
                let mut new_vcol: colnr_T = vcol + win_chartabsize(curwin.get(), buf, vcol);
                while *oldp.offset(col.wrapping_add(oldlen) as isize) as ::core::ffi::c_int != NUL
                    && vcol < new_vcol
                {
                    vcol += win_chartabsize(
                        curwin.get(),
                        oldp.offset(col as isize).offset(oldlen as isize),
                        vcol,
                    );
                    if vcol > new_vcol
                        && *oldp.offset(col.wrapping_add(oldlen) as isize) as ::core::ffi::c_int
                            == TAB
                    {
                        break;
                    }
                    oldlen = oldlen.wrapping_add(utfc_ptr2len(
                        oldp.offset(col as isize).offset(oldlen as isize),
                    ) as size_t);
                    if vcol > new_vcol {
                        newlen = newlen.wrapping_add((vcol - new_vcol) as size_t);
                    }
                }
                (*curwin.get()).w_onebuf_opt.wo_list = old_list;
            } else if *oldp.offset(col as isize) as ::core::ffi::c_int != NUL {
                oldlen = utfc_ptr2len(oldp.offset(col as isize)) as size_t;
            }
            replace_push_nul();
            replace_push(oldp.offset(col as isize), oldlen);
        }
        let mut newp: *mut ::core::ffi::c_char =
            xmalloc(linelen.wrapping_add(newlen).wrapping_sub(oldlen)) as *mut ::core::ffi::c_char;
        if col > 0 as size_t {
            memmove(
                newp as *mut ::core::ffi::c_void,
                oldp as *const ::core::ffi::c_void,
                col,
            );
        }
        let mut p: *mut ::core::ffi::c_char = newp.offset(col as isize);
        if linelen > col.wrapping_add(oldlen) {
            memmove(
                p.offset(newlen as isize) as *mut ::core::ffi::c_void,
                oldp.offset(col as isize).offset(oldlen as isize) as *const ::core::ffi::c_void,
                linelen.wrapping_sub(col).wrapping_sub(oldlen),
            );
        }
        memmove(
            p as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            charlen,
        );
        let mut i: size_t = charlen;
        while i < newlen {
            *p.offset(i as isize) = ' ' as ::core::ffi::c_char;
            i = i.wrapping_add(1);
        }
        ml_replace(lnum, newp, false_0 != 0);
        inserted_bytes(
            lnum,
            col as colnr_T,
            oldlen as ::core::ffi::c_int,
            newlen as ::core::ffi::c_int,
        );
        if p_sm.get() != 0
            && State.get() & MODE_INSERT != 0
            && msg_silent.get() == 0 as ::core::ffi::c_int
            && !ins_compl_active()
        {
            showmatch(utf_ptr2char(buf));
        }
        if p_ri.get() == 0 || State.get() & REPLACE_FLAG != 0 {
            (*curwin.get()).w_cursor.col += charlen as colnr_T;
        }
    }
}

pub unsafe extern "C" fn ins_str(mut s: *mut ::core::ffi::c_char, mut slen: size_t) {
    unsafe {
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
        {
            coladvance_force(getviscol());
        }
        let mut col: colnr_T = (*curwin.get()).w_cursor.col;
        let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut oldlen: ::core::ffi::c_int = ml_get_len(lnum);
        let mut newp: *mut ::core::ffi::c_char = xmalloc(
            (oldlen as size_t)
                .wrapping_add(slen)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        if col > 0 as ::core::ffi::c_int {
            memmove(
                newp as *mut ::core::ffi::c_void,
                oldp as *const ::core::ffi::c_void,
                col as size_t,
            );
        }
        memmove(
            newp.offset(col as isize) as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            slen,
        );
        let mut bytes: ::core::ffi::c_int =
            oldlen - col as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if bytes >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"bytes >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/change.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    836 as ::core::ffi::c_uint,
                    b"void ins_str(char *, size_t)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            newp.offset(col as isize).offset(slen as isize) as *mut ::core::ffi::c_void,
            oldp.offset(col as isize) as *const ::core::ffi::c_void,
            bytes as size_t,
        );
        ml_replace(lnum, newp, false_0 != 0);
        inserted_bytes(
            lnum,
            col,
            0 as ::core::ffi::c_int,
            slen as ::core::ffi::c_int,
        );
        (*curwin.get()).w_cursor.col += slen as colnr_T;
    }
}

pub unsafe extern "C" fn del_char(mut fixpos: bool) -> ::core::ffi::c_int {
    unsafe {
        mb_adjust_cursor();
        if *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL {
            return FAIL;
        }
        return del_chars(1 as ::core::ffi::c_int, fixpos as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn del_chars(
    mut count: ::core::ffi::c_int,
    mut fixpos: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut bytes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count && *p as ::core::ffi::c_int != NUL {
            let mut l: ::core::ffi::c_int = utfc_ptr2len(p);
            bytes += l;
            p = p.offset(l as isize);
            i += 1;
        }
        return del_bytes(bytes as colnr_T, fixpos != 0, true_0 != 0);
    }
}

pub unsafe extern "C" fn del_bytes(
    mut count: colnr_T,
    mut fixpos_arg: bool,
    mut use_delcombine: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut col: colnr_T = (*curwin.get()).w_cursor.col;
        let mut fixpos: bool = fixpos_arg;
        let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut oldlen: colnr_T = ml_get_len(lnum);
        if col >= oldlen {
            return FAIL;
        }
        if count == 0 as ::core::ffi::c_int {
            return OK;
        }
        if count < 1 as ::core::ffi::c_int {
            siemsg(
                b"E292: Invalid count for del_bytes(): %ld\0".as_ptr()
                    as *const ::core::ffi::c_char,
                count as int64_t,
            );
            return FAIL;
        }
        if p_deco.get() != 0
            && use_delcombine as ::core::ffi::c_int != 0
            && utfc_ptr2len(oldp.offset(col as isize)) >= count
        {
            let mut p0: *mut ::core::ffi::c_char = oldp.offset(col as isize);
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            if utf_composinglike(p0, p0.offset(utf_ptr2len(p0) as isize), &raw mut state) {
                let mut n: ::core::ffi::c_int = col as ::core::ffi::c_int;
                loop {
                    col = n as colnr_T;
                    count = utf_ptr2len(oldp.offset(n as isize)) as colnr_T;
                    n += count as ::core::ffi::c_int;
                    if !utf_composinglike(
                        oldp.offset(col as isize),
                        oldp.offset(n as isize),
                        &raw mut state,
                    ) {
                        break;
                    }
                }
                fixpos = false_0 != 0;
            }
        }
        let mut movelen: ::core::ffi::c_int =
            oldlen as ::core::ffi::c_int - col as ::core::ffi::c_int - count as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int;
        if movelen <= 1 as ::core::ffi::c_int {
            if col > 0 as ::core::ffi::c_int
                && fixpos as ::core::ffi::c_int != 0
                && restart_edit.get() == 0 as ::core::ffi::c_int
                && get_ve_flags(curwin.get())
                    & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0 as ::core::ffi::c_uint
            {
                (*curwin.get()).w_cursor.col -= 1;
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                (*curwin.get()).w_cursor.col -=
                    utf_head_off(oldp, oldp.offset((*curwin.get()).w_cursor.col as isize));
            }
            count = oldlen - col;
            movelen = 1 as ::core::ffi::c_int;
        }
        let mut newlen: colnr_T = oldlen - count;
        let mut alloc_newp: bool = ml_line_alloced() == 0;
        let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !alloc_newp {
            ml_add_deleted_len((*curbuf.get()).b_ml.ml_line_ptr, oldlen as ssize_t);
            newp = oldp;
        } else {
            newp = xmallocz(newlen as size_t) as *mut ::core::ffi::c_char;
            memmove(
                newp as *mut ::core::ffi::c_void,
                oldp as *const ::core::ffi::c_void,
                col as size_t,
            );
        }
        memmove(
            newp.offset(col as isize) as *mut ::core::ffi::c_void,
            oldp.offset(col as isize).offset(count as isize) as *const ::core::ffi::c_void,
            movelen as size_t,
        );
        if alloc_newp {
            ml_replace(lnum, newp, false_0 != 0);
        } else {
            (*curbuf.get()).b_ml.ml_line_textlen =
                (newlen as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
        }
        inserted_bytes(
            lnum,
            col,
            count as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        return OK;
    }
}

pub unsafe extern "C" fn truncate_line(mut fixpos: ::core::ffi::c_int) {
    unsafe {
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut col: colnr_T = (*curwin.get()).w_cursor.col;
        let mut old_line: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut newp: *mut ::core::ffi::c_char = if col == 0 as ::core::ffi::c_int {
            xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            xstrnsave(old_line, col as size_t)
        };
        let mut deleted: ::core::ffi::c_int = ml_get_len(lnum) - col as ::core::ffi::c_int;
        ml_replace(lnum, newp, false_0 != 0);
        inserted_bytes(
            lnum,
            (*curwin.get()).w_cursor.col,
            deleted,
            0 as ::core::ffi::c_int,
        );
        if fixpos != 0 && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col -= 1;
        }
    }
}

pub unsafe extern "C" fn del_lines(mut nlines: linenr_T, mut undo: bool) {
    unsafe {
        let mut n: ::core::ffi::c_int = 0;
        let mut first: linenr_T = (*curwin.get()).w_cursor.lnum;
        if nlines <= 0 as linenr_T {
            return;
        }
        if undo as ::core::ffi::c_int != 0 && u_savedel(first, nlines) == FAIL {
            return;
        }
        n = 0 as ::core::ffi::c_int;
        while (n as linenr_T) < nlines {
            if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                break;
            }
            ml_delete_flags(first, ML_DEL_MESSAGE as ::core::ffi::c_int);
            n += 1;
            if first > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
        }
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        check_cursor_lnum(curwin.get());
        deleted_lines_mark(first, n);
    }
}
