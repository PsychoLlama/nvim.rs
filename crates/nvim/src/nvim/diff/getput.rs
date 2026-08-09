//! `:diffget`, `:diffput`, `do` and `dp`.
//!
//! All three spellings end in `diffgetput`, which copies one diff block's lines
//! between two buffers.  The command forms accept a range, which is in *this*
//! buffer's line numbers and has to be mapped onto the block list before the
//! copy; `nv_diffgetput` is the Normal-mode form, where the range is the block
//! under the cursor.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::semsg_c;
use ::core::ffi::{c_char, c_int};

/// `do` and `dp`: get or put the diff block under the cursor.
///
/// With a count the block is named by *buffer number* rather than by
/// position, which is the only way to choose a side in a three-way diff.
pub unsafe fn nv_diffgetput(put: bool, count: size_t) {
    unsafe {
        if bt_prompt(curbuf.get()) {
            vim_beep(kOptBoFlagOperator as c_int as ::core::ffi::c_uint);
            return;
        }
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut c_char>(),
            arglens: ::core::ptr::null_mut(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
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
            cstack: ::core::ptr::null_mut(),
        };
        let mut buf: [c_char; 30] = [0; 30];
        if count == 0 as size_t {
            ea.arg = c"".as_ptr() as *mut c_char;
        } else {
            vim_snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 30]>(),
                c"%zu".as_ptr(),
                count,
            );
            ea.arg = &raw mut buf as *mut c_char;
        }
        if put {
            ea.cmdidx = CMD_diffput;
        } else {
            ea.cmdidx = CMD_diffget;
        }
        ea.addr_count = 0;
        ea.line1 = (*curwin.get()).w_cursor.lnum;
        ea.line2 = (*curwin.get()).w_cursor.lnum;
        ex_diffgetput(&raw mut ea);
    }
}

/// `:diffget` and `:diffput`, with their optional range and buffer argument.
///
/// Three things have to be resolved before the copy: which buffer is the
/// other side (the argument, or the only other buffer in the diff), which
/// line range in *this* buffer is meant, and -- for `:diffput` -- that the
/// destination is modifiable.
pub unsafe fn ex_diffgetput(eap: *mut exarg_T) {
    unsafe {
        let mut idx_other: c_int = 0;
        let mut idx_cur: c_int = diff_buf_idx(curbuf.get(), curtab.get());
        if idx_cur == DB_COUNT {
            emsg(gettext(c"E99: Current buffer is not in diff mode".as_ptr()));
            return;
        }
        if *(*eap).arg as c_int == NUL {
            let mut found_not_ma: bool = false;
            idx_other = 0;
            while idx_other < DB_COUNT {
                if (*curtab.get()).tp_diffbuf[idx_other as usize] != curbuf.get()
                    && !(*curtab.get()).tp_diffbuf[idx_other as usize].is_null()
                {
                    if (*eap).cmdidx as c_int != CMD_diffput as c_int
                        || (*(*curtab.get()).tp_diffbuf[idx_other as usize]).b_p_ma != 0
                    {
                        break;
                    }
                    found_not_ma = true;
                }
                idx_other += 1;
            }
            if idx_other == DB_COUNT {
                if found_not_ma {
                    emsg(gettext(
                        c"E793: No other buffer in diff mode is modifiable".as_ptr(),
                    ));
                } else {
                    emsg(gettext(c"E100: No other buffer in diff mode".as_ptr()));
                }
                return;
            }
            let mut i: c_int = idx_other + 1;
            while i < DB_COUNT {
                if (*curtab.get()).tp_diffbuf[i as usize] != curbuf.get()
                    && !(*curtab.get()).tp_diffbuf[i as usize].is_null()
                    && ((*eap).cmdidx as c_int != CMD_diffput as c_int
                        || (*(*curtab.get()).tp_diffbuf[i as usize]).b_p_ma != 0)
                {
                    emsg(gettext(
                        c"E101: More than two buffers in diff mode, don't know which one to use"
                            .as_ptr(),
                    ));
                    return;
                }
                i += 1;
            }
        } else {
            let mut p: *mut c_char = (*eap).arg.add(strlen((*eap).arg));
            while p > (*eap).arg && ascii_iswhite(*p.offset(-1) as c_int) as c_int != 0 {
                p = p.offset(-1);
            }
            let mut i_0: c_int = 0;
            i_0 = 0;
            while ascii_isdigit(*(*eap).arg.offset(i_0 as isize) as c_int) as c_int != 0
                && (*eap).arg.offset(i_0 as isize) < p
            {
                i_0 += 1;
            }
            if (*eap).arg.offset(i_0 as isize) == p {
                i_0 = atol((*eap).arg) as c_int;
            } else {
                i_0 = buflist_findpat((*eap).arg, p, false, true, false);
                if i_0 < 0 {
                    return;
                }
            }
            let mut buf: *mut buf_T = buflist_findnr(i_0);
            if buf.is_null() {
                semsg_c!(
                    gettext(c"E102: Can't find buffer \"%s\"".as_ptr()),
                    (*eap).arg,
                );
                return;
            }
            if buf == curbuf.get() {
                return;
            }
            idx_other = diff_buf_idx(buf, curtab.get());
            if idx_other == DB_COUNT {
                semsg_c!(
                    gettext(c"E103: Buffer \"%s\" is not in diff mode".as_ptr()),
                    (*eap).arg,
                );
                return;
            }
        }
        diff_busy.set(true);
        if (*eap).addr_count == 0 {
            let mut linestatus: c_int = 0;
            if (*eap).line1 == (*curbuf.get()).b_ml.ml_line_count
                && (diff_check_with_linestatus(curwin.get(), (*eap).line1, &raw mut linestatus)
                    == 0
                    && linestatus == 0)
                && ((*eap).line1 == 1 as linenr_T
                    || diff_check_with_linestatus(
                        curwin.get(),
                        (*eap).line1 - 1 as linenr_T,
                        &raw mut linestatus,
                    ) >= 0
                        && linestatus == 0)
            {
                (*eap).line2 += 1;
            } else if (*eap).line1 > 0 as linenr_T {
                (*eap).line1 -= 1;
            }
        }
        let mut aco: aco_save_T = aco_save_T::default();
        if (*eap).cmdidx as c_int != CMD_diffget as c_int {
            aucmd_prepbuf(
                &raw mut aco,
                (*curtab.get()).tp_diffbuf[idx_other as usize] as *mut buf_T,
            );
        }
        let idx_from: c_int = if (*eap).cmdidx as c_int == CMD_diffget as c_int {
            idx_other
        } else {
            idx_cur
        };
        let idx_to: c_int = if (*eap).cmdidx as c_int == CMD_diffget as c_int {
            idx_cur
        } else {
            idx_other
        };
        '_theend: {
            if (*curbuf.get()).b_changed == 0 {
                change_warning(curbuf.get(), 0);
                if diff_buf_idx(curbuf.get(), curtab.get()) != idx_to {
                    emsg(gettext(c"E787: Buffer changed unexpectedly".as_ptr()));
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
            if (*eap).cmdidx as c_int != CMD_diffget as c_int {
                if KeyTyped.get() {
                    u_sync(false);
                }
                aucmd_restbuf(&raw mut aco);
            }
        }
        diff_busy.set(false);
        if diff_need_update.get() {
            ex_diffupdate(::core::ptr::null_mut());
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
                    && *(*wp).w_onebuf_opt.wo_fdm as c_int == 'd' as c_int
                    && (*wp).w_onebuf_opt.wo_fen != 0
                {
                    foldUpdateAll(wp);
                }
                wp = (*wp).w_next;
            }
        }
        if diff_need_update.get() {
            diff_need_update.set(false);
        } else {
            diff_redraw(false);
            apply_autocmds(
                EVENT_DIFFUPDATED,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false,
                curbuf.get(),
            );
        };
    }
}

/// Copy the lines of the blocks between `line1` and `line2` from one buffer
/// to the other.
///
/// The walk runs with `diff_busy` set, so `diff_mark_adjust_tp` only shifts
/// line numbers instead of rebuilding the block list underneath it; the
/// blocks are then patched up here as each one is copied. `start_skip` and
/// `end_skip` are how much of the first and last block the range cuts off.
unsafe fn diffgetput(
    addr_count: c_int,
    idx_cur: c_int,
    idx_from: c_int,
    idx_to: c_int,
    line1: linenr_T,
    line2: linenr_T,
) {
    unsafe {
        let mut off: linenr_T = 0 as linenr_T;
        let mut dprev: *mut diff_T = ::core::ptr::null_mut();
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
                df_next: ::core::ptr::null_mut(),
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
            let mut did_free: bool = false;
            let mut lnum: linenr_T = (*dp).df_lnum[idx_to as usize];
            let mut count: linenr_T = (*dp).df_count[idx_to as usize];
            if (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize] > line1 + off
                && u_save(lnum - 1 as linenr_T, lnum + count) != FAIL
            {
                let mut start_skip: linenr_T = 0 as linenr_T;
                let mut end_skip: linenr_T = 0 as linenr_T;
                if addr_count > 0 {
                    start_skip = line1 + off - (*dp).df_lnum[idx_cur as usize];
                    if start_skip > 0 as linenr_T {
                        if start_skip > count {
                            lnum += count;
                            count = 0 as linenr_T;
                        } else {
                            count -= start_skip;
                            lnum += start_skip;
                        }
                    } else {
                        start_skip = 0 as linenr_T;
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
                        end_skip = 0 as linenr_T;
                    }
                }
                let mut buf_empty: bool = buf_is_empty(curbuf.get());
                let mut added: c_int = 0;
                let mut i: c_int = 0;
                while (i as linenr_T) < count {
                    buf_empty = (*curbuf.get()).b_ml.ml_line_count == 1 as linenr_T;
                    if ml_delete(lnum) == OK {
                        added -= 1;
                    }
                    i += 1;
                }
                let mut i_0: c_int = 0;
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
                    let mut p: *mut c_char = xstrdup(ml_get_buf(
                        (*curtab.get()).tp_diffbuf[idx_from as usize] as *mut buf_T,
                        nr,
                    ));
                    ml_append(
                        lnum + i_0 as linenr_T - 1 as linenr_T,
                        p,
                        0 as colnr_T,
                        false,
                    );
                    xfree(p as *mut ::core::ffi::c_void);
                    added += 1;
                    if buf_empty && (*curbuf.get()).b_ml.ml_line_count == 2 as linenr_T {
                        buf_empty = false;
                        ml_delete(2 as linenr_T);
                    }
                    i_0 += 1;
                }
                let mut new_count: linenr_T = (*dp).df_count[idx_to as usize] + added as linenr_T;
                (*dp).df_count[idx_to as usize] = new_count;
                if start_skip == 0 as linenr_T && end_skip == 0 as linenr_T {
                    let mut i_1: c_int = 0;
                    i_1 = 0;
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
                        did_free = true;
                        dp = diff_free(curtab.get(), dprev, dp);
                    }
                }
                if added != 0 {
                    mark_adjust(
                        lnum,
                        lnum + count - 1 as linenr_T,
                        MAXLNUM as c_int as linenr_T,
                        added as linenr_T,
                        kExtmarkNOOP,
                    );
                    if (*curwin.get()).w_cursor.lnum >= lnum {
                        if (*curwin.get()).w_cursor.lnum >= lnum + count {
                            (*curwin.get()).w_cursor.lnum =
                                ((*curwin.get()).w_cursor.lnum as c_int + added) as linenr_T;
                            (*curwin.get()).w_cursor.lnum = if (*curwin.get()).w_cursor.lnum
                                < (*curbuf.get()).b_ml.ml_line_count
                            {
                                (*curwin.get()).w_cursor.lnum
                            } else {
                                (*curbuf.get()).b_ml.ml_line_count
                            };
                        } else if added < 0 {
                            (*curwin.get()).w_cursor.lnum = lnum;
                        }
                    }
                }
                extmark_adjust(
                    curbuf.get(),
                    lnum,
                    lnum + count - 1 as linenr_T,
                    MAXLNUM as c_int as linenr_T,
                    added as linenr_T,
                    kExtmarkUndo,
                );
                changed_lines(
                    curbuf.get(),
                    lnum,
                    0 as colnr_T,
                    lnum + count,
                    added as linenr_T,
                    true,
                );
                if did_free {
                    diff_fold_update(&raw mut dfree, idx_to);
                }
                if added != 0 && !valid_diff(dp) {
                    break;
                }
                if !did_free {
                    (*dp).df_count[idx_to as usize] = new_count;
                }
                if idx_cur == idx_to {
                    off = (off as c_int + added) as linenr_T;
                }
            }
            if !did_free {
                dprev = dp;
                dp = (*dp).df_next;
            }
        }
    }
}
