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
use crate::buffer::buflist_findpat_prefix;
use crate::charset::skip;
use crate::message_fmt::msg_bytes;
use crate::optionstr::OptString;
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
        source: None,
        cond_stack: None,
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
    let idx_cur = tp.diff_index(Buf::current());
    if idx_cur == DB_COUNT {
        emsg_gettext(c"E99: Current buffer is not in diff mode".as_ptr());
        return;
    }
    let cmdidx = excmd.cmdidx;
    let mut idx_other = 0;
    // SAFETY: the command's own NUL-terminated argument.
    if c_int::from(excmd.line.byte_at(excmd.line.arg)) == NUL {
        // No argument: the other side is the one other buffer in the diff,
        // and it is an error if there are two of them to choose from.
        let mut found_not_ma = false;
        while idx_other < DB_COUNT {
            if let Some(buf) = tp.diffbuf(idx_other as usize).filter(|b| !b.is_current()) {
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
            if let Some(buf) = tp.diffbuf(i as usize).filter(|b| !b.is_current())
                && writable_target(buf, cmdidx)
            {
                let msg = c"E101: More than two buffers in diff mode, don't know which one to use";
                emsg_gettext(msg.as_ptr());
                return;
            }
        }
    } else {
        // The argument names the other buffer, by number or by pattern,
        // its trailing white space dropped.
        let arg = excmd.line.arg;
        let mut end = excmd.line.end_of(arg);
        while end > arg && ascii_iswhite(c_int::from(excmd.line.byte_at(end - 1))) {
            end -= 1;
        }
        let digits = excmd
            .line
            .slice_at(arg, end - arg)
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        let nr = if arg + digits == end {
            // `atol`'s answer, cut to an `int` as upstream's cast cuts it.
            skip::decimal(excmd.line.rest_of(arg)).0 as c_int
        } else {
            let found =
                buflist_findpat_prefix(excmd.line.cstr_from(arg), end - arg, false, true, false);
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
        idx_other = tp.diff_index(buf);
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
        let line1 = excmd.line1;
        // The short circuit is upstream's.
        let below_end = line1 == Buf::current().b_ml.ml_line_count
            && diff_check_with_linestatus(Win::current(), line1) == (0, 0)
            && (line1 == 1 as LineNr || {
                let (fill, linestatus) = diff_check_with_linestatus(Win::current(), line1 - 1);
                fill >= 0 && linestatus == 0
            });
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
        aucmd_prepbuf(&mut aco, tp.used_diffbuf(idx_other as usize));
    }
    let (idx_from, idx_to) = if put {
        (idx_cur, idx_other)
    } else {
        (idx_other, idx_cur)
    };
    '_theend: {
        if Buf::current().b_changed == 0 {
            change_warning(Buf::current(), 0);
            // The warning can run autocommands, which can move us.
            if tp.diff_index(Buf::current()) != idx_to {
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
            aucmd_restbuf(&mut aco);
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
    if tp.tp_diff.is_empty() {
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
        diff_redraw(false);
        let here = Buf::current_or_none();
        apply_autocmds(AutoEvent::DiffUpdated, None, None, false, here);
    }
}

/// Copy the lines of the blocks between `line1` and `line2` from one buffer
/// to the other.
///
/// The walk runs with `diff_busy` set, so `diff_mark_adjust_tp` only shifts
/// line numbers instead of rebuilding the block list underneath it; the
/// blocks are then patched up here as each one is copied. `start_skip` and
/// `end_skip` are how much of the first and last block the range cuts off.
///
/// The walk holds an index, never a block: `u_save` and `changed_lines` run
/// user code, and the list's generation says whether the index still means
/// what it did. Any change of shape stops the walk there.
fn diffgetput(
    addr_count: c_int,
    idx_cur: c_int,
    idx_from: c_int,
    idx_to: c_int,
    line1: LineNr,
    line2: LineNr,
) {
    let (idx_cur, idx_from, idx_to) = (idx_cur as usize, idx_from as usize, idx_to as usize);
    let mut tp = TabPage::current();
    let mut off = 0 as LineNr;
    let mut at = 0;
    while at < tp.tp_diff.len() {
        if addr_count == 0 {
            // Without a range, a run of blocks that touch is taken as one.
            while let Some(next) = tp.tp_diff.get(at + 1)
                && next.df_lnum[idx_cur] == tp.tp_diff[at].end(idx_cur)
                && next.df_lnum[idx_cur] == line1 + off + 1 as LineNr
            {
                at += 1;
            }
        }
        let (df_lnum, df_count) = (tp.tp_diff[at].df_lnum, tp.tp_diff[at].df_count);
        let end = |idx: usize| df_lnum[idx] + df_count[idx];
        if df_lnum[idx_cur] > line2 + off {
            break;
        }
        let mut freed = false;
        let mut lnum = df_lnum[idx_to];
        let mut count = df_count[idx_to];
        let generation = tp.tp_diff_gen;
        // The short circuit is upstream's.
        let undoable =
            end(idx_cur) > line1 + off && u_save(lnum - 1 as LineNr, lnum + count).is_ok();
        // `u_save` can run `FileChangedRO`.
        if tp.tp_diff_gen != generation {
            break;
        }
        if undoable {
            // With a range, the first and last block of it are only partly
            // copied; `start_skip`/`end_skip` are the parts left out.
            let mut start_skip = 0 as LineNr;
            let mut end_skip = 0 as LineNr;
            if addr_count > 0 {
                start_skip = line1 + off - df_lnum[idx_cur];
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
                end_skip = end(idx_cur) - 1 as LineNr - (line2 + off);
                if end_skip > 0 as LineNr {
                    if idx_cur == idx_from {
                        count = count.min(df_count[idx_cur] - start_skip - end_skip);
                    } else {
                        count -= end_skip;
                        end_skip = (df_count[idx_from] - start_skip - count).max(0);
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
            while i < df_count[idx_from] - start_skip - end_skip {
                let src = tp.used_diffbuf(idx_from);
                let nr = df_lnum[idx_from] + start_skip + i;
                if nr > src.b_ml.ml_line_count {
                    break;
                }
                // A copy, because `ml_append` writes into the buffer this
                // line was read from. Its length counts the terminator,
                // which is why the copy keeps one.
                let text = Lines::in_buffer(src).line_copy(nr);
                let at = lnum + i - 1 as LineNr;
                let _ = ml_append_cstr(at, text.as_cstr());
                added += 1;
                if buf_empty && Buf::current().b_ml.ml_line_count == 2 as LineNr {
                    buf_empty = false;
                    let _ = ml_delete(2 as LineNr);
                }
                i += 1;
            }
            let new_count = df_count[idx_to] + added as LineNr;
            tp.tp_diff[at].df_count[idx_to] = new_count;

            // A block that now reads the same in every buffer is not a
            // difference any more.
            let mut freed_ranges = None;
            if start_skip == 0 as LineNr && end_skip == 0 as LineNr {
                let slots = tp.tp_diffbuf;
                let dp = &tp.tp_diff[at];
                let all_equal = (0..DB_COUNT as usize).all(|i| {
                    slots[i].is_none()
                        || i == idx_from
                        || i == idx_to
                        || diff_equal_entry(dp, &slots, idx_from, i)
                });
                if all_equal {
                    // The ranges, kept for the fold update below; the block
                    // itself goes now.
                    let dp = tp.with_diff(|blocks| blocks.remove(at));
                    freed_ranges = Some((dp.df_lnum, dp.df_count));
                    freed = true;
                }
            }
            let generation = tp.tp_diff_gen;

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
            if let Some((lnum, count)) = &freed_ranges {
                diff_fold_update(lnum, count, idx_to as c_int);
            }
            // `mark_adjust` may drop blocks, and `changed_lines` runs
            // autocommands and buffer-update callbacks: if either changed
            // the list's shape, the index no longer means this block.
            // Upstream re-checks only when lines were added or removed, and
            // otherwise writes through a block that may be gone.
            if tp.tp_diff_gen != generation {
                break;
            }
            if !freed {
                // `mark_adjust` may have changed the count in a wrong way.
                tp.tp_diff[at].df_count[idx_to] = new_count;
            }
            if idx_cur == idx_to {
                off += amount;
            }
        }
        if !freed {
            at += 1;
        }
    }
}
