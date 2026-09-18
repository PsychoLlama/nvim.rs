//! `:diffget`, `:diffput`, `do` and `dp`.
//!
//! All three spellings end in `diffgetput`, which copies one diff block's lines
//! between two buffers.  The command forms accept a range, which is in *this*
//! buffer's line numbers and has to be mapped onto the block list before the
//! copy; `nv_diffgetput` is the Normal-mode form, where the range is the block
//! under the cursor.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::message_fmt::msg_bytes;
use crate::optionstr::LocalOptStr;
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::types::CmdIdx;
use crate::types::EcmdCmd;
use crate::types::{CmdLine, ExArgt, NUL};
use crate::winlayer::{Buf, TabPage, Win, windows};
use core::ffi::{c_char, c_int, c_uint};

/// `emsg(gettext(msg))`, the pair every error here is reported through.
fn emsg_gettext(msg: *const c_char) {
    // SAFETY: a static message string, and the editor exists.
    unsafe { emsg(gettext_ptr(msg)) };
}

/// Whether `:diffput` may write into `buffer` -- or the command is not
/// `:diffput` at all, in which case nothing has to be modifiable.
fn writable_target(buffer: Buf, cmdidx: CmdIdx) -> bool {
    cmdidx != CmdIdx::diffput || buffer.b_p_ma != 0
}

/// `do` and `dp`: get or put the diff block under the cursor.
///
/// With a count the block is named by *buffer number* rather than by
/// position, which is the only way to choose a side in a three-way diff.
pub fn nv_diffgetput(put: bool, count: size_t) {
    if buf_is_prompt(current_buf()) {
        vim_beep(kOptBoFlagOperator as c_int as c_uint);
        return;
    }
    let mut ea: ExArg = ExArg {
        line: CmdLine::EMPTY,
        cmdidx: if put {
            CmdIdx::diffput
        } else {
            CmdIdx::diffget
        },
        argt: ExArgt::NONE,
        skip: false,
        forceit: false,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: CmdAddr::Lines,
        flags: 0,
        do_ecmd_cmd: EcmdCmd::None,
        do_ecmd_lnum: 0,
        append: false,
        usefilter: false,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: false,
        mkdir_p: false,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: None,
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut(),
    };
    if count != 0 as size_t {
        ea.set_arg_text(format!("{count}").as_bytes());
    }
    ea.line1 = Win::current().w_cursor.lnum;
    ea.line2 = Win::current().w_cursor.lnum;
    // SAFETY: `ea` is a local of this frame.
    ex_diffgetput(&mut ea);
}

/// `:diffget` and `:diffput`, with their optional range and buffer argument.
///
/// Three things have to be resolved before the copy: which buffer is the
/// other side (the argument, or the only other buffer in the diff), which
/// line range in *this* buffer is meant, and -- for `:diffput` -- that the
/// destination is modifiable.
pub fn ex_diffgetput(excmd: &mut ExArg) {
    let tp = TabPage::current();
    let idx_cur = diff_slot(Buf::current(), tp);
    if idx_cur == DB_COUNT {
        emsg_gettext(c"E99: Current buffer is not in diff mode".as_ptr());
        return;
    }
    let cmdidx = excmd.cmdidx;
    let mut idx_other = 0;
    // SAFETY: the command's own NUL-terminated argument.
    if unsafe { *excmd.arg_ptr() } as c_int == NUL {
        // No argument: the other side is the one other buffer in the diff,
        // and it is an error if there are two of them to choose from.
        let mut found_not_ma = false;
        while idx_other < DB_COUNT {
            let buf = tp.diffbuf(idx_other as usize);
            if buf.raw() != Buf::current_raw() && !buf.raw().is_null() {
                if writable_target(buf, cmdidx) {
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
            let buf = tp.diffbuf(i as usize);
            if buf.raw() != Buf::current_raw()
                && !buf.raw().is_null()
                && writable_target(buf, cmdidx)
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
        let mut at = excmd.line.end_of(excmd.line.arg);
        while at > excmd.line.arg && ascii_iswhite(c_int::from(excmd.line.byte_at(at - 1))) {
            at -= 1;
        }
        let p = excmd.line.ptr_at(at);
        let mut digits = 0;
        // SAFETY: the walk stops at `p`, which is inside the argument.
        while unsafe {
            ascii_isdigit(*excmd.arg_ptr().add(digits) as c_int) && excmd.arg_ptr().add(digits) < p
        } {
            digits += 1;
        }
        let nr = if excmd.arg_ptr().wrapping_add(digits) == p {
            // SAFETY: a NUL-terminated decimal number.
            unsafe { atol(excmd.arg_ptr()) as c_int }
        } else {
            // SAFETY: the argument and the end of it.
            let found = unsafe { buflist_findpat(excmd.arg_ptr(), p, false, true, false) };
            if found < 0 {
                return;
            }
            found
        };
        let Some(buf) = find_buf(nr) else {
            // SAFETY: the command's own argument, for the one `%s`.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E102: Can't find buffer \"{arg}\"");
            return;
        };
        if buf.raw() == Buf::current_raw() {
            return;
        }
        idx_other = diff_slot(buf, tp);
        if idx_other == DB_COUNT {
            // SAFETY: as above.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E103: Buffer \"{arg}\" is not in diff mode");
            return;
        }
    }

    diff_busy.set(true);
    if excmd.addr_count == 0 {
        // Without a range the block *above* the cursor is meant, except at
        // the very end of the buffer where the filler below it is.
        let mut linestatus = 0;
        let status = &raw mut linestatus;
        let line1 = excmd.line1;
        // SAFETY: the current window is live and `linestatus` is a local, in
        // both calls; the short circuit is upstream's.
        let below_end = line1 == Buf::current().b_ml.ml_line_count
            && unsafe { diff_check_with_linestatus(Win::current(), line1, status) } == 0
            && linestatus == 0
            && (line1 == 1 as LineNr
                || unsafe { diff_check_with_linestatus(Win::current(), line1 - 1, status) } >= 0
                    && linestatus == 0);
        if below_end {
            excmd.line2 += 1;
        } else if line1 > 0 as LineNr {
            excmd.line1 -= 1;
        }
    }

    // `:diffput` writes into the *other* buffer, so the autocommand context
    // moves there for the copy.
    let mut aco = AcoSave::default();
    let put = cmdidx != CmdIdx::diffget;
    if put {
        // SAFETY: `aco` is a local, and `other` a live buffer of the diff.
        unsafe { aucmd_prepbuf(&raw mut aco, tp.diffbuf(idx_other as usize)) };
    }
    let (idx_from, idx_to) = if put {
        (idx_cur, idx_other)
    } else {
        (idx_other, idx_cur)
    };
    '_theend: {
        if Buf::current().b_changed == 0 {
            // SAFETY: the current buffer is live.
            unsafe { change_warning(Buf::current(), 0) };
            // The warning can run autocommands, which can move us.
            if diff_slot(Buf::current(), tp) != idx_to {
                emsg_gettext(c"E787: Buffer changed unexpectedly".as_ptr());
                break '_theend;
            }
        }
        let (line1, line2) = (excmd.line1, excmd.line2);
        diffgetput(excmd.addr_count, idx_cur, idx_from, idx_to, line1, line2);
        if put {
            if KeyTyped.get() {
                u_sync(false);
            }
            // SAFETY: `aco` was filled in by `aucmd_prepbuf` above.
            unsafe { aucmd_restbuf(&raw mut aco) };
        }
    }
    diff_busy.set(false);
    if diff_need_update.get() {
        // SAFETY: no `ExArg` is being passed on.
        diff_update(None);
    }
    // SAFETY: the current window is live, in both calls.
    check_cursor(Win::current());
    changed_line_abv_curs();
    if tp.tp_first_diff.is_null() {
        // The last block went away: the diff folds have nothing left to
        // describe, so every window folding by `diff` is rebuilt.
        for wp in windows() {
            let by_diff = wp.w_onebuf_opt.wo_fdm.first_byte() as c_int == 'd' as c_int;
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
        diff_redraw(false);
        let here = Buf::current_or_none();
        unsafe { apply_autocmds(AutoEvent::DiffUpdated, nul, nul, false, here) };
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
    line1: LineNr,
    line2: LineNr,
) {
    let (idx_cur, idx_from, idx_to) = (idx_cur as usize, idx_from as usize, idx_to as usize);
    let tp = TabPage::current();
    let mut off = 0 as LineNr;
    let mut dprev = ::core::ptr::null_mut::<DiffBlock>();
    let mut cursor = Df::first(tp);
    while let Some(mut dp) = cursor {
        if addr_count == 0 {
            // Without a range, a run of blocks that touch is taken as one.
            while let Some(next) = dp.next()
                && next.df_lnum[idx_cur] == dp.end(idx_cur)
                && next.df_lnum[idx_cur] == line1 + off + 1 as LineNr
            {
                dprev = dp.raw();
                dp = next;
            }
        }
        if dp.df_lnum[idx_cur] > line2 + off {
            break;
        }
        cursor = Some(dp);
        // The freed block's ranges, taken before it goes: the fold update at
        // the tail still needs its line numbers. Only the two arrays, never
        // the block -- a `DiffBlock` also carries a `GArray` and its list
        // links, and `diff_free` releases all three.
        let mut freed: Option<([LineNr; DB_COUNT as usize], [LineNr; DB_COUNT as usize])> = None;
        let mut lnum = dp.df_lnum[idx_to];
        let mut count = dp.df_count[idx_to];
        // SAFETY: the editor exists; the short circuit is upstream's.
        let undoable =
            dp.end(idx_cur) > line1 + off && u_save(lnum - 1 as LineNr, lnum + count).is_ok();
        if undoable {
            // With a range, the first and last block of it are only partly
            // copied; `start_skip`/`end_skip` are the parts left out.
            let mut start_skip = 0 as LineNr;
            let mut end_skip = 0 as LineNr;
            if addr_count > 0 {
                start_skip = line1 + off - dp.df_lnum[idx_cur];
                if start_skip > 0 as LineNr {
                    if start_skip > count {
                        lnum += count;
                        count = 0 as LineNr;
                    } else {
                        count -= start_skip;
                        lnum += start_skip;
                    }
                } else {
                    start_skip = 0 as LineNr;
                }
                end_skip = dp.end(idx_cur) - 1 as LineNr - (line2 + off);
                if end_skip > 0 as LineNr {
                    if idx_cur == idx_from {
                        count = count.min(dp.df_count[idx_cur] - start_skip - end_skip);
                    } else {
                        count -= end_skip;
                        end_skip = (dp.df_count[idx_from] - start_skip - count).max(0);
                    }
                } else {
                    end_skip = 0 as LineNr;
                }
            }

            let mut buf_empty = buf_is_empty(Buf::current());
            let mut added: c_int = 0;
            for _ in 0..count {
                buf_empty = Buf::current().b_ml.ml_line_count == 1 as LineNr;
                if ml_delete(lnum).is_ok() {
                    added -= 1;
                }
            }
            let mut i = 0 as LineNr;
            while i < dp.df_count[idx_from] - start_skip - end_skip {
                let src = tp.diffbuf(idx_from);
                let nr = dp.df_lnum[idx_from] + start_skip + i;
                if nr > src.b_ml.ml_line_count {
                    break;
                }
                // A copy, because `ml_append` writes into the buffer this
                // line was read from. Its length counts the terminator,
                // which is why the copy keeps one.
                let text = Lines::in_buffer(src).line_copy(nr);
                let (at, ptr) = (lnum + i - 1 as LineNr, text.as_cstr().as_ptr().cast_mut());
                let len = text.len() as ColNr + 1;
                // SAFETY: the editor exists, and `text` is this frame's own
                // NUL-terminated copy of the line.
                let _ = unsafe { ml_append(at, ptr, len, false) };
                added += 1;
                if buf_empty && Buf::current().b_ml.ml_line_count == 2 as LineNr {
                    buf_empty = false;
                    let _ = ml_delete(2 as LineNr);
                }
                i += 1;
            }
            let new_count = dp.df_count[idx_to] + added as LineNr;
            dp.df_count[idx_to] = new_count;

            // A block that now reads the same in every buffer is not a
            // difference any more.
            if start_skip == 0 as LineNr && end_skip == 0 as LineNr {
                let all_equal = (0..DB_COUNT as usize).all(|i| {
                    tp.tp_diffbuf[i].is_null()
                        || i == idx_from
                        || i == idx_to
                        || dp.equal_entry(idx_from, i)
                });
                if all_equal {
                    freed = Some((dp.df_lnum, dp.df_count));
                    // SAFETY: a live block and its predecessor in the list;
                    // the answer is the following block, or null.
                    let next = unsafe { diff_free(tp, dprev, dp.raw()) };
                    // SAFETY: as above.
                    cursor = unsafe { Df::from_raw(next) };
                }
            }

            let last = lnum + count - 1 as LineNr;
            let max = MAXLNUM as LineNr;
            let amount = added as LineNr;
            if added != 0 {
                mark_adjust(lnum, last, max, amount, kExtmarkNOOP);
                if Win::current().w_cursor.lnum >= lnum {
                    if Win::current().w_cursor.lnum >= lnum + count {
                        let moved = Win::current().w_cursor.lnum + amount;
                        Win::current().w_cursor.lnum = moved.min(Buf::current().b_ml.ml_line_count);
                    } else if added < 0 {
                        Win::current().w_cursor.lnum = lnum;
                    }
                }
            }
            let cb = Buf::current();
            extmark_adjust(cb, lnum, last, max, amount, kExtmarkUndo);
            changed_lines(cb, lnum, 0 as ColNr, lnum + count, amount, true);
            if let Some((lnum, count)) = &freed {
                diff_fold_update(lnum, count, idx_to as c_int);
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
