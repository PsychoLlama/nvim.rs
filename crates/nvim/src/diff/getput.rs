//! `:diffget`, `:diffput`, `do` and `dp`.
//!
//! All three spellings end in `diffgetput`, which copies one diff block's lines
//! between two buffers.  The command forms accept a range, which is in *this*
//! buffer's line numbers and has to be mapped onto the block list before the
//! copy; `nv_diffgetput` is the Normal-mode form, where the range is the block
//! under the cursor.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::message_fmt::c_str;
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::types::{ExArgt, NUL};
use crate::winlayer::{Buf, Live, TabPage, Win, windows};
use core::ffi::{c_char, c_int, c_uint};

/// `emsg(gettext(msg))`, the pair every error here is reported through.
fn emsg_gettext(msg: *const c_char) {
    // SAFETY: a static message string, and the editor exists.
    unsafe { emsg(gettext_ptr(msg)) };
}

/// Whether `:diffput` may write into `buf` -- or the command is not
/// `:diffput` at all, in which case nothing has to be modifiable.
fn writable_target(buf: Buf, cmdidx: c_int) -> bool {
    cmdidx != CMD_diffput as c_int || buf.b_p_ma != 0
}

/// `do` and `dp`: get or put the diff block under the cursor.
///
/// With a count the block is named by *buffer number* rather than by
/// position, which is the only way to choose a side in a three-way diff.
pub unsafe fn nv_diffgetput(put: bool, count: size_t) {
    if buf_is_prompt(current_buf()) {
        // SAFETY: the editor exists.
        unsafe { vim_beep(kOptBoFlagOperator as c_int as c_uint) };
        return;
    }
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: if put { CMD_diffput } else { CMD_diffget },
        argt: ExArgt::NONE,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: CmdAddr::Lines,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
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
        errmsg: None,
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut(),
    };
    let mut nrbuf: [c_char; 30] = [0; 30];
    if count == 0 as size_t {
        ea.arg = c"".as_ptr() as *mut c_char;
    } else {
        let at = nrbuf.as_mut_ptr();
        // SAFETY: `nrbuf` holds the 30 bytes `vim_snprintf` is told about.
        unsafe { vim_snprintf(at, 30, c"%zu".as_ptr(), count) };
        ea.arg = at;
    }
    ea.line1 = cur_win().w_cursor.lnum;
    ea.line2 = cur_win().w_cursor.lnum;
    // SAFETY: `ea` is a local of this frame.
    unsafe { ex_diffgetput(&raw mut ea) };
}

/// `:diffget` and `:diffput`, with their optional range and buffer argument.
///
/// Three things have to be resolved before the copy: which buffer is the
/// other side (the argument, or the only other buffer in the diff), which
/// line range in *this* buffer is meant, and -- for `:diffput` -- that the
/// destination is modifiable.
///
/// # Safety
/// `eap` must be a live command.
pub unsafe fn ex_diffgetput(eap: *mut exarg_T) {
    // SAFETY: the caller's command.
    let mut eap = unsafe { Live::<exarg_T>::new(eap) };
    let tp = cur_tab();
    let idx_cur = diff_slot(cur_buf(), tp);
    if idx_cur == DB_COUNT {
        emsg_gettext(c"E99: Current buffer is not in diff mode".as_ptr());
        return;
    }
    let cmdidx = eap.cmdidx as c_int;
    let mut idx_other = 0;
    // SAFETY: the command's own NUL-terminated argument.
    if unsafe { *eap.arg } as c_int == NUL {
        // No argument: the other side is the one other buffer in the diff,
        // and it is an error if there are two of them to choose from.
        let mut found_not_ma = false;
        while idx_other < DB_COUNT {
            let buf = tp.tp_diffbuf[idx_other as usize];
            if buf != curbuf.get() && !buf.is_null() {
                if writable_target(unsafe { Buf::new(buf) }, cmdidx) {
                    break;
                }
                found_not_ma = true;
            }
            idx_other += 1;
        }
        if idx_other == DB_COUNT {
            if found_not_ma {
                emsg_gettext(c"E793: No other buffer in diff mode is modifiable".as_ptr());
            } else {
                emsg_gettext(c"E100: No other buffer in diff mode".as_ptr());
            }
            return;
        }
        for i in idx_other + 1..DB_COUNT {
            let buf = tp.tp_diffbuf[i as usize];
            if buf != curbuf.get()
                && !buf.is_null()
                && writable_target(unsafe { Buf::new(buf) }, cmdidx)
            {
                let msg = c"E101: More than two buffers in diff mode, don't know which one to use";
                emsg_gettext(msg.as_ptr());
                return;
            }
        }
    } else {
        // The argument names the other buffer, by number or by pattern.
        // SAFETY: the command's own NUL-terminated argument; `p` walks back
        // over its own bytes and stops at its start.
        let mut p = unsafe { eap.arg.add(cstr::bytes_at(eap.arg).len()) };
        // SAFETY: as above.
        while p > eap.arg && ascii_iswhite(unsafe { *p.sub(1) } as c_int) {
            p = p.wrapping_sub(1);
        }
        let mut digits = 0;
        // SAFETY: the walk stops at `p`, which is inside the argument.
        while unsafe { ascii_isdigit(*eap.arg.add(digits) as c_int) && eap.arg.add(digits) < p } {
            digits += 1;
        }
        let nr = if eap.arg.wrapping_add(digits) == p {
            // SAFETY: a NUL-terminated decimal number.
            unsafe { atol(eap.arg) as c_int }
        } else {
            // SAFETY: the argument and the end of it.
            let found = unsafe { buflist_findpat(eap.arg, p, false, true, false) };
            if found < 0 {
                return;
            }
            found
        };
        let Some(buf) = find_buf(nr) else {
            // SAFETY: the command's own argument, for the one `%s`.
            let arg = unsafe { c_str(eap.arg) };
            semsg!("E102: Can't find buffer \"{arg}\"");
            return;
        };
        if buf.raw() == curbuf.get() {
            return;
        }
        idx_other = diff_slot(buf, tp);
        if idx_other == DB_COUNT {
            // SAFETY: as above.
            let arg = unsafe { c_str(eap.arg) };
            semsg!("E103: Buffer \"{arg}\" is not in diff mode");
            return;
        }
    }

    diff_busy.set(true);
    if eap.addr_count == 0 {
        // Without a range the block *above* the cursor is meant, except at
        // the very end of the buffer where the filler below it is.
        let mut linestatus = 0;
        let status = &raw mut linestatus;
        let line1 = eap.line1;
        // SAFETY: the current window is live and `linestatus` is a local, in
        // both calls; the short circuit is upstream's.
        let below_end = line1 == cur_buf().b_ml.ml_line_count
            && unsafe { diff_check_with_linestatus(cur_win(), line1, status) } == 0
            && linestatus == 0
            && (line1 == 1 as linenr_T
                || unsafe { diff_check_with_linestatus(cur_win(), line1 - 1, status) } >= 0
                    && linestatus == 0);
        if below_end {
            eap.line2 += 1;
        } else if line1 > 0 as linenr_T {
            eap.line1 -= 1;
        }
    }

    // `:diffput` writes into the *other* buffer, so the autocommand context
    // moves there for the copy.
    let mut aco = aco_save_T::default();
    let put = cmdidx != CMD_diffget as c_int;
    if put {
        let other = tp.tp_diffbuf[idx_other as usize];
        // SAFETY: `aco` is a local, and `other` a live buffer of the diff.
        unsafe { aucmd_prepbuf(&raw mut aco, other) };
    }
    let (idx_from, idx_to) = if put {
        (idx_cur, idx_other)
    } else {
        (idx_other, idx_cur)
    };
    '_theend: {
        if cur_buf().b_changed == 0 {
            // SAFETY: the current buffer is live.
            unsafe { change_warning(cur_buf(), 0) };
            // The warning can run autocommands, which can move us.
            if diff_slot(cur_buf(), tp) != idx_to {
                emsg_gettext(c"E787: Buffer changed unexpectedly".as_ptr());
                break '_theend;
            }
        }
        let (line1, line2) = (eap.line1, eap.line2);
        diffgetput(eap.addr_count, idx_cur, idx_from, idx_to, line1, line2);
        if put {
            if KeyTyped.get() {
                // SAFETY: the editor exists.
                u_sync(false);
            }
            // SAFETY: `aco` was filled in by `aucmd_prepbuf` above.
            unsafe { aucmd_restbuf(&raw mut aco) };
        }
    }
    diff_busy.set(false);
    if diff_need_update.get() {
        // SAFETY: no `exarg_T` is being passed on.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    // SAFETY: the current window is live, in both calls.
    check_cursor(cur_win());
    unsafe { changed_line_abv_curs() };
    if tp.tp_first_diff.is_null() {
        // The last block went away: the diff folds have nothing left to
        // describe, so every window folding by `diff` is rebuilt.
        for wp in windows() {
            // SAFETY: a live window's `'foldmethod'` is a NUL-terminated
            // option string.
            let by_diff = unsafe { *wp.w_onebuf_opt.wo_fdm } as c_int == 'd' as c_int;
            if wp.w_onebuf_opt.wo_diff != 0 && by_diff && wp.w_onebuf_opt.wo_fen != 0 {
                // SAFETY: a live window.
                fold_update_all(wp);
            }
        }
    }
    if diff_need_update.get() {
        diff_need_update.set(false);
    } else {
        let nul = ::core::ptr::null_mut::<c_char>();
        // SAFETY: the editor exists; `DiffUpdated` takes no file name.
        unsafe { diff_redraw(false) };
        unsafe { apply_autocmds(EVENT_DIFFUPDATED, nul, nul, false, curbuf.get()) };
    }
}

/// Copy the lines of the blocks between `line1` and `line2` from one buffer
/// to the other.
///
/// The walk runs with `diff_busy` set, so `diff_mark_adjust_tp` only shifts
/// line numbers instead of rebuilding the block list underneath it; the
/// blocks are then patched up here as each one is copied. `start_skip` and
/// `end_skip` are how much of the first and last block the range cuts off.
fn diffgetput(
    addr_count: c_int,
    idx_cur: c_int,
    idx_from: c_int,
    idx_to: c_int,
    line1: linenr_T,
    line2: linenr_T,
) {
    let (idx_cur, idx_from, idx_to) = (idx_cur as usize, idx_from as usize, idx_to as usize);
    let tp = cur_tab();
    let mut off = 0 as linenr_T;
    let mut dprev = ::core::ptr::null_mut::<diff_T>();
    let mut cursor = Df::first(tp);
    while let Some(mut dp) = cursor {
        if addr_count == 0 {
            // Without a range, a run of blocks that touch is taken as one.
            while let Some(next) = dp.next()
                && next.df_lnum[idx_cur] == dp.end(idx_cur)
                && next.df_lnum[idx_cur] == line1 + off + 1 as linenr_T
            {
                dprev = dp.raw();
                dp = next;
            }
        }
        if dp.df_lnum[idx_cur] > line2 + off {
            break;
        }
        cursor = Some(dp);
        // A copy of the block, taken when it is freed: the fold update at
        // the tail still needs its line numbers.
        let mut freed: Option<diff_T> = None;
        let mut lnum = dp.df_lnum[idx_to];
        let mut count = dp.df_count[idx_to];
        // SAFETY: the editor exists; the short circuit is upstream's.
        let undoable =
            dp.end(idx_cur) > line1 + off && u_save(lnum - 1 as linenr_T, lnum + count).is_ok();
        if undoable {
            // With a range, the first and last block of it are only partly
            // copied; `start_skip`/`end_skip` are the parts left out.
            let mut start_skip = 0 as linenr_T;
            let mut end_skip = 0 as linenr_T;
            if addr_count > 0 {
                start_skip = line1 + off - dp.df_lnum[idx_cur];
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
                end_skip = dp.end(idx_cur) - 1 as linenr_T - (line2 + off);
                if end_skip > 0 as linenr_T {
                    if idx_cur == idx_from {
                        count = count.min(dp.df_count[idx_cur] - start_skip - end_skip);
                    } else {
                        count -= end_skip;
                        end_skip = (dp.df_count[idx_from] - start_skip - count).max(0);
                    }
                } else {
                    end_skip = 0 as linenr_T;
                }
            }

            // SAFETY: the current buffer is live.
            let mut buf_empty = unsafe { buf_is_empty(curbuf.get()) };
            let mut added: c_int = 0;
            for _ in 0..count {
                buf_empty = cur_buf().b_ml.ml_line_count == 1 as linenr_T;
                // SAFETY: the editor exists and `lnum` is a line of it.
                if unsafe { ml_delete(lnum) }.is_ok() {
                    added -= 1;
                }
            }
            let mut i = 0 as linenr_T;
            while i < dp.df_count[idx_from] - start_skip - end_skip {
                let src = tp.tp_diffbuf[idx_from];
                let nr = dp.df_lnum[idx_from] + start_skip + i;
                // SAFETY: a live buffer of the diff.
                if nr > unsafe { (*src).b_ml.ml_line_count } {
                    break;
                }
                // SAFETY: a live buffer and a line number inside it.
                let p = unsafe { xstrdup(ml_get_buf(src, nr)) };
                // SAFETY: the editor exists; `p` is our own copy of the line.
                let _ = unsafe { ml_append(lnum + i - 1 as linenr_T, p, 0 as colnr_T, false) };
                unsafe { xfree(p.cast()) };
                added += 1;
                if buf_empty && cur_buf().b_ml.ml_line_count == 2 as linenr_T {
                    buf_empty = false;
                    // SAFETY: the buffer holds the two lines just counted.
                    let _ = unsafe { ml_delete(2 as linenr_T) };
                }
                i += 1;
            }
            let new_count = dp.df_count[idx_to] + added as linenr_T;
            dp.df_count[idx_to] = new_count;

            // A block that now reads the same in every buffer is not a
            // difference any more.
            if start_skip == 0 as linenr_T && end_skip == 0 as linenr_T {
                let all_equal = (0..DB_COUNT as usize).all(|i| {
                    tp.tp_diffbuf[i].is_null()
                        || i == idx_from
                        || i == idx_to
                        || dp.equal_entry(idx_from, i)
                });
                if all_equal {
                    freed = Some(*dp);
                    // SAFETY: a live block and its predecessor in the list;
                    // the answer is the following block, or null.
                    let next = unsafe { diff_free(tp, dprev, dp.raw()) };
                    // SAFETY: as above.
                    cursor = unsafe { Df::from_raw(next) };
                }
            }

            let last = lnum + count - 1 as linenr_T;
            let max = MAXLNUM as c_int as linenr_T;
            let amount = added as linenr_T;
            if added != 0 {
                // SAFETY: the editor exists.
                unsafe { mark_adjust(lnum, last, max, amount, kExtmarkNOOP) };
                if cur_win().w_cursor.lnum >= lnum {
                    if cur_win().w_cursor.lnum >= lnum + count {
                        let moved = cur_win().w_cursor.lnum + amount;
                        cur_win().w_cursor.lnum = moved.min(cur_buf().b_ml.ml_line_count);
                    } else if added < 0 {
                        cur_win().w_cursor.lnum = lnum;
                    }
                }
            }
            let cb = curbuf.get();
            // SAFETY: the current buffer is live, in both calls.
            unsafe { extmark_adjust(cb, lnum, last, max, amount, kExtmarkUndo) };
            changed_lines(
                unsafe { Buf::new(cb) },
                lnum,
                0 as colnr_T,
                lnum + count,
                amount,
                true,
            );
            if let Some(mut copy) = freed {
                // SAFETY: `copy` is this frame's copy of the freed block.
                unsafe { diff_fold_update(&raw mut copy, idx_to as c_int) };
            }
            // `changed_lines` runs autocommands, which can rebuild the list.
            let still = cursor.map_or(::core::ptr::null_mut(), Df::raw);
            if added != 0 && !diff_still_listed(still) {
                break;
            }
            if freed.is_none() {
                dp.df_count[idx_to] = new_count;
            }
            if idx_cur == idx_to {
                off += amount;
            }
        }
        if freed.is_none() {
            dprev = dp.raw();
            cursor = dp.next();
        }
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}
