//! `:diffget`, `:diffput`, `do` and `dp`.
//!
//! All three spellings end in `diffgetput`, which copies one diff block's lines
//! between two buffers.  The command forms accept a range, which is in *this*
//! buffer's line numbers and has to be mapped onto the block list before the
//! copy; `nv_diffgetput` is the Normal-mode form, where the range is the block
//! under the cursor.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn nv_diffgetput(mut put: bool, mut count: size_t) {
    unsafe {
        if bt_prompt(curbuf.get()) {
            vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
            return;
        }
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        let mut buf: [::core::ffi::c_char; 30] = [0; 30];
        if count == 0 as size_t {
            ea.arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            vim_snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                b"%zu\0".as_ptr() as *const ::core::ffi::c_char,
                count,
            );
            ea.arg = &raw mut buf as *mut ::core::ffi::c_char;
        }
        if put {
            ea.cmdidx = CMD_diffput;
        } else {
            ea.cmdidx = CMD_diffget;
        }
        ea.addr_count = 0 as ::core::ffi::c_int;
        ea.line1 = (*curwin.get()).w_cursor.lnum;
        ea.line2 = (*curwin.get()).w_cursor.lnum;
        ex_diffgetput(&raw mut ea);
    }
}

pub unsafe fn ex_diffgetput(mut eap: *mut exarg_T) {
    unsafe {
        let mut idx_other: ::core::ffi::c_int = 0;
        let mut idx_cur: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
        if idx_cur == DB_COUNT {
            emsg(gettext(
                b"E99: Current buffer is not in diff mode\0".as_ptr() as *const ::core::ffi::c_char,
            ));
            return;
        }
        if *(*eap).arg as ::core::ffi::c_int == NUL {
            let mut found_not_ma: bool = false_0 != 0;
            idx_other = 0 as ::core::ffi::c_int;
            while idx_other < DB_COUNT {
                if (*curtab.get()).tp_diffbuf[idx_other as usize] != curbuf.get()
                    && !(*curtab.get()).tp_diffbuf[idx_other as usize].is_null()
                {
                    if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffput as ::core::ffi::c_int
                        || (*(*curtab.get()).tp_diffbuf[idx_other as usize]).b_p_ma != 0
                    {
                        break;
                    }
                    found_not_ma = true_0 != 0;
                }
                idx_other += 1;
            }
            if idx_other == DB_COUNT {
                if found_not_ma {
                    emsg(gettext(
                        b"E793: No other buffer in diff mode is modifiable\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                } else {
                    emsg(gettext(b"E100: No other buffer in diff mode\0".as_ptr()
                        as *const ::core::ffi::c_char));
                }
                return;
            }
            let mut i: ::core::ffi::c_int = idx_other + 1 as ::core::ffi::c_int;
            while i < DB_COUNT {
                if (*curtab.get()).tp_diffbuf[i as usize] != curbuf.get()
                    && !(*curtab.get()).tp_diffbuf[i as usize].is_null()
                    && ((*eap).cmdidx as ::core::ffi::c_int != CMD_diffput as ::core::ffi::c_int
                        || (*(*curtab.get()).tp_diffbuf[i as usize]).b_p_ma != 0)
                {
                    emsg(gettext(
                        b"E101: More than two buffers in diff mode, don't know which one to use\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ));
                    return;
                }
                i += 1;
            }
        } else {
            let mut p: *mut ::core::ffi::c_char = (*eap).arg.offset(strlen((*eap).arg) as isize);
            while p > (*eap).arg
                && ascii_iswhite(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(-1);
            }
            let mut i_0: ::core::ffi::c_int = 0;
            i_0 = 0 as ::core::ffi::c_int;
            while ascii_isdigit(*(*eap).arg.offset(i_0 as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                && (*eap).arg.offset(i_0 as isize) < p
            {
                i_0 += 1;
            }
            if (*eap).arg.offset(i_0 as isize) == p {
                i_0 = atol((*eap).arg) as ::core::ffi::c_int;
            } else {
                i_0 = buflist_findpat((*eap).arg, p, false_0 != 0, true_0 != 0, false_0 != 0);
                if i_0 < 0 as ::core::ffi::c_int {
                    return;
                }
            }
            let mut buf: *mut buf_T = buflist_findnr(i_0);
            if buf.is_null() {
                semsg(
                    gettext(
                        b"E102: Can't find buffer \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    (*eap).arg,
                );
                return;
            }
            if buf == curbuf.get() {
                return;
            }
            idx_other = diff_buf_idx(buf, curtab.get());
            if idx_other == DB_COUNT {
                semsg(
                    gettext(b"E103: Buffer \"%s\" is not in diff mode\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    (*eap).arg,
                );
                return;
            }
        }
        diff_busy.set(true_0 != 0);
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if (*eap).line1 == (*curbuf.get()).b_ml.ml_line_count
                && (diff_check_with_linestatus(curwin.get(), (*eap).line1, &raw mut linestatus)
                    == 0 as ::core::ffi::c_int
                    && linestatus == 0 as ::core::ffi::c_int)
                && ((*eap).line1 == 1 as linenr_T
                    || diff_check_with_linestatus(
                        curwin.get(),
                        (*eap).line1 - 1 as linenr_T,
                        &raw mut linestatus,
                    ) >= 0 as ::core::ffi::c_int
                        && linestatus == 0 as ::core::ffi::c_int)
            {
                (*eap).line2 += 1;
            } else if (*eap).line1 > 0 as linenr_T {
                (*eap).line1 -= 1;
            }
        }
        let mut aco: aco_save_T = aco_save_T::default();
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffget as ::core::ffi::c_int {
            aucmd_prepbuf(
                &raw mut aco,
                (*curtab.get()).tp_diffbuf[idx_other as usize] as *mut buf_T,
            );
        }
        let idx_from: ::core::ffi::c_int =
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int {
                idx_other
            } else {
                idx_cur
            };
        let idx_to: ::core::ffi::c_int =
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int {
                idx_cur
            } else {
                idx_other
            };
        '_theend: {
            if (*curbuf.get()).b_changed == 0 {
                change_warning(curbuf.get(), 0 as ::core::ffi::c_int);
                if diff_buf_idx(curbuf.get(), curtab.get()) != idx_to {
                    emsg(gettext(b"E787: Buffer changed unexpectedly\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    break '_theend;
                }
            }
            diffgetput(
                (*eap).addr_count,
                idx_cur,
                idx_from,
                idx_to,
                (*eap).line1,
                (*eap).line2,
            );
            if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffget as ::core::ffi::c_int {
                if KeyTyped.get() {
                    u_sync(false_0 != 0);
                }
                aucmd_restbuf(&raw mut aco);
            }
        }
        diff_busy.set(false_0 != 0);
        if diff_need_update.get() {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        check_cursor(curwin.get());
        changed_line_abv_curs();
        if (*curtab.get()).tp_first_diff.is_null() {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_onebuf_opt.wo_diff != 0
                    && *(*wp)
                        .w_onebuf_opt
                        .wo_fdm
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == 'd' as ::core::ffi::c_int
                    && (*wp).w_onebuf_opt.wo_fen != 0
                {
                    foldUpdateAll(wp);
                }
                wp = (*wp).w_next;
            }
        }
        if diff_need_update.get() {
            diff_need_update.set(false_0 != 0);
        } else {
            diff_redraw(false_0 != 0);
            apply_autocmds(
                EVENT_DIFFUPDATED,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        };
    }
}

unsafe extern "C" fn diffgetput(
    addr_count: ::core::ffi::c_int,
    idx_cur: ::core::ffi::c_int,
    idx_from: ::core::ffi::c_int,
    idx_to: ::core::ffi::c_int,
    line1: linenr_T,
    line2: linenr_T,
) {
    unsafe {
        let mut off: linenr_T = 0 as linenr_T;
        let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if addr_count == 0 {
                while !(*dp).df_next.is_null()
                    && (*(*dp).df_next).df_lnum[idx_cur as usize]
                        == (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize]
                    && (*(*dp).df_next).df_lnum[idx_cur as usize] == line1 + off + 1 as linenr_T
                {
                    dprev = dp;
                    dp = (*dp).df_next;
                }
            }
            if (*dp).df_lnum[idx_cur as usize] > line2 + off {
                break;
            }
            let mut dfree: diff_T = diffblock_S {
                df_next: ::core::ptr::null_mut::<diff_T>(),
                df_lnum: [0; 8],
                df_count: [0; 8],
                is_linematched: false,
                has_changes: false,
                df_changes: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
            };
            let mut did_free: bool = false_0 != 0;
            let mut lnum: linenr_T = (*dp).df_lnum[idx_to as usize];
            let mut count: linenr_T = (*dp).df_count[idx_to as usize];
            if (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize] > line1 + off
                && u_save(lnum - 1 as linenr_T, lnum + count) != FAIL
            {
                let mut start_skip: linenr_T = 0 as linenr_T;
                let mut end_skip: linenr_T = 0 as linenr_T;
                if addr_count > 0 as ::core::ffi::c_int {
                    start_skip = line1 + off - (*dp).df_lnum[idx_cur as usize];
                    if start_skip > 0 as linenr_T {
                        if start_skip > count {
                            lnum += count;
                            count = 0 as ::core::ffi::c_int as linenr_T;
                        } else {
                            count -= start_skip;
                            lnum += start_skip;
                        }
                    } else {
                        start_skip = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    end_skip = (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize]
                        - 1 as linenr_T
                        - (line2 + off);
                    if end_skip > 0 as linenr_T {
                        if idx_cur == idx_from {
                            count = if count
                                < (*dp).df_count[idx_cur as usize] - start_skip - end_skip
                            {
                                count
                            } else {
                                (*dp).df_count[idx_cur as usize] - start_skip - end_skip
                            };
                        } else {
                            count -= end_skip;
                            end_skip = if (*dp).df_count[idx_from as usize] - start_skip - count
                                > 0 as linenr_T
                            {
                                (*dp).df_count[idx_from as usize] - start_skip - count
                            } else {
                                0 as linenr_T
                            };
                        }
                    } else {
                        end_skip = 0 as ::core::ffi::c_int as linenr_T;
                    }
                }
                let mut buf_empty: bool = buf_is_empty(curbuf.get());
                let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (i as linenr_T) < count {
                    buf_empty = (*curbuf.get()).b_ml.ml_line_count == 1 as linenr_T;
                    if ml_delete(lnum) == OK {
                        added -= 1;
                    }
                    i += 1;
                }
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (i_0 as linenr_T) < (*dp).df_count[idx_from as usize] - start_skip - end_skip
                {
                    let mut nr: linenr_T =
                        (*dp).df_lnum[idx_from as usize] + start_skip + i_0 as linenr_T;
                    if nr
                        > (*(*curtab.get()).tp_diffbuf[idx_from as usize])
                            .b_ml
                            .ml_line_count
                    {
                        break;
                    }
                    let mut p: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                        (*curtab.get()).tp_diffbuf[idx_from as usize] as *mut buf_T,
                        nr,
                    ));
                    ml_append(
                        lnum + i_0 as linenr_T - 1 as linenr_T,
                        p,
                        0 as colnr_T,
                        false_0 != 0,
                    );
                    xfree(p as *mut ::core::ffi::c_void);
                    added += 1;
                    if buf_empty as ::core::ffi::c_int != 0
                        && (*curbuf.get()).b_ml.ml_line_count == 2 as linenr_T
                    {
                        buf_empty = false_0 != 0;
                        ml_delete(2 as linenr_T);
                    }
                    i_0 += 1;
                }
                let mut new_count: linenr_T = (*dp).df_count[idx_to as usize] + added as linenr_T;
                (*dp).df_count[idx_to as usize] = new_count;
                if start_skip == 0 as linenr_T && end_skip == 0 as linenr_T {
                    let mut i_1: ::core::ffi::c_int = 0;
                    i_1 = 0 as ::core::ffi::c_int;
                    while i_1 < DB_COUNT {
                        if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null()
                            && i_1 != idx_from
                            && i_1 != idx_to
                            && !diff_equal_entry(dp, idx_from as usize, i_1 as usize)
                        {
                            break;
                        }
                        i_1 += 1;
                    }
                    if i_1 == DB_COUNT {
                        dfree = *dp;
                        did_free = true_0 != 0;
                        dp = diff_free(curtab.get(), dprev, dp);
                    }
                }
                if added != 0 as ::core::ffi::c_int {
                    mark_adjust(
                        lnum,
                        lnum + count - 1 as linenr_T,
                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                        added as linenr_T,
                        kExtmarkNOOP,
                    );
                    if (*curwin.get()).w_cursor.lnum >= lnum {
                        if (*curwin.get()).w_cursor.lnum >= lnum + count {
                            (*curwin.get()).w_cursor.lnum =
                                ((*curwin.get()).w_cursor.lnum as ::core::ffi::c_int + added)
                                    as linenr_T;
                            (*curwin.get()).w_cursor.lnum = if (*curwin.get()).w_cursor.lnum
                                < (*curbuf.get()).b_ml.ml_line_count
                            {
                                (*curwin.get()).w_cursor.lnum
                            } else {
                                (*curbuf.get()).b_ml.ml_line_count
                            };
                        } else if added < 0 as ::core::ffi::c_int {
                            (*curwin.get()).w_cursor.lnum = lnum;
                        }
                    }
                }
                extmark_adjust(
                    curbuf.get(),
                    lnum,
                    lnum + count - 1 as linenr_T,
                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                    added as linenr_T,
                    kExtmarkUndo,
                );
                changed_lines(
                    curbuf.get(),
                    lnum,
                    0 as colnr_T,
                    lnum + count,
                    added as linenr_T,
                    true_0 != 0,
                );
                if did_free {
                    diff_fold_update(&raw mut dfree, idx_to);
                }
                if added != 0 as ::core::ffi::c_int && !valid_diff(dp) {
                    break;
                }
                if !did_free {
                    (*dp).df_count[idx_to as usize] = new_count;
                }
                if idx_cur == idx_to {
                    off = (off as ::core::ffi::c_int + added) as linenr_T;
                }
            }
            if !did_free {
                dprev = dp;
                dp = (*dp).df_next;
            }
        }
    }
}
