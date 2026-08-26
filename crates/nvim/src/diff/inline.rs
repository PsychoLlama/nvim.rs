//! Which columns of a changed line changed.
//!
//! [`diff_find_change`] answers, for one line of one window, the list of
//! changed column ranges the drawer paints `DiffText` over.
//! [`diff_find_change_simple`] is the `inline:simple` rule -- one range, from
//! the first differing byte to the last -- and [`diff_change_parse`] is how
//! the drawer reads a range back out.  [`f_diff_hl_id`] is the Vimscript front
//! door to the same answer, and the only way to observe any of it without a
//! screen.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Live, TabPage, Win};
use core::ffi::c_int;
use std::ffi::CStr;

/// Throw away the cached inline changes of the block holding `lnum`.
///
/// Only the `inline:char`/`inline:word` modes cache anything; the simple rule
/// is recomputed per line.
pub unsafe fn diff_update_line(lnum: linenr_T) {
    if diff_flags.get() & ALL_INLINE_DIFF == 0 {
        return;
    }
    let tp = cur_tab();
    let idx = diff_slot(curbuf.get(), tp);
    if idx == DB_COUNT {
        return;
    }
    let idx = idx as usize;
    if let Some(mut dp) = diff_blocks(tp).find(|dp| lnum <= dp.end(idx)) {
        dp.has_changes = false;
        dp.df_changes.ga_len = 0;
    }
}

/// The single change `inline:simple` (and `inline:none`) reports.
///
/// A cell rather than a local because [`diff_find_change`] hands a pointer to
/// it back to the drawer; it is alive until the next call.
static simple_diffline_change: GlobalCell<diffline_change_T> = GlobalCell::new(diffline_change_T {
    dc_start: [0; 8],
    dc_end: [0; 8],
    dc_start_lnum_off: [0; 8],
    dc_end_lnum_off: [0; 8],
});

/// Read one change's column range for the line `diffline` describes.
///
/// A change can span several lines, so a range that starts above this line
/// begins at column 0 and one that ends below it runs to the end of the line.
/// The answer is whether the range is an *addition* rather than a change,
/// which the caller paints `DiffTextAdd` instead of `DiffText`.
pub unsafe fn diff_change_parse(
    diffline: *mut diffline_T,
    change: *mut diffline_change_T,
    change_start: *mut c_int,
    change_end: *mut c_int,
) -> bool {
    // SAFETY: the caller's line description, and one of the changes it names.
    let dl = unsafe { Live::<diffline_T>::new(diffline) };
    // SAFETY: as above.
    let ch = unsafe { Live::<diffline_change_T>::new(change) };
    let buf = dl.bufidx as usize;
    let lineoff = dl.lineoff;
    // A range that starts above this line begins at column 0, and one that
    // ends below it runs past the end.
    let start = if ch.dc_start_lnum_off[buf] < lineoff {
        0
    } else {
        ch.dc_start[buf] as c_int
    };
    let end = if ch.dc_end_lnum_off[buf] > lineoff {
        c_int::MAX
    } else {
        ch.dc_end[buf] as c_int
    };
    // SAFETY: the caller's out-parameters.
    unsafe {
        *change_start = start;
        *change_end = end;
    }
    if change == simple_diffline_change.ptr() {
        return false;
    }
    // An addition is a change that is empty in every *other* buffer.
    (0..DB_COUNT as usize).all(|i| {
        i == buf
            || ch.dc_start[i] == ch.dc_end[i] && ch.dc_end_lnum_off[i] == ch.dc_start_lnum_off[i]
    })
}

/// How much of `org` and `new` is a common prefix, under the whitespace flags.
///
/// The two walks advance independently: with `iwhite`/`iwhiteall` a run of
/// white space on one side matches a run of a different length on the other.
/// Both answers are then moved back to the head of the character covering
/// them, which need not be the same distance on each side ("nn^" against
/// "n^").
fn common_prefix(org: &[u8], new: &[u8]) -> (usize, usize) {
    let flags = diff_flags.get();
    let (mut si_org, mut si_new) = (0usize, 0usize);
    while si_org < org.len() {
        let w_org = ascii_iswhite(byte_at(org, si_org) as c_int);
        let w_new = ascii_iswhite(byte_at(new, si_new) as c_int);
        if flags & DIFF_IWHITE != 0 && w_org && w_new
            || flags & DIFF_IWHITEALL != 0 && (w_org || w_new)
        {
            si_org = org.len() - skip_white(&org[si_org..]).len();
            si_new = new.len() - skip_white(&new[si_new.min(new.len())..]).len();
        } else if let Some(l) = char_len(&org[si_org..], &new[si_new.min(new.len())..]) {
            si_org += l;
            si_new += l;
        } else {
            break;
        }
    }
    (
        si_org - head_off(org, si_org.min(org.len())),
        si_new - head_off(new, si_new.min(new.len())),
    )
}

/// The last byte of `org` that still differs, walking back from the end.
///
/// `start` is the column the prefix walk settled on -- the search may not
/// cross it -- and `si_new` the same for the other side.  The answer is
/// upstream's `ei_org`, an *inclusive* end which [`diff_find_change`] then
/// turns into an exclusive one.
fn common_suffix(org: &[u8], new: &[u8], start: c_int, si_new: c_int) -> c_int {
    let flags = diff_flags.get();
    let (mut ei_org, mut ei_new) = (org.len() as c_int, new.len() as c_int);
    while ei_org >= start && ei_new >= si_new && ei_org >= 0 && ei_new >= 0 {
        let w_org = ascii_iswhite(byte_at(org, ei_org as usize) as c_int);
        let w_new = ascii_iswhite(byte_at(new, ei_new as usize) as c_int);
        if flags & DIFF_IWHITE != 0 && w_org && w_new
            || flags & DIFF_IWHITEALL != 0 && (w_org || w_new)
        {
            while ei_org >= start && ascii_iswhite(byte_at(org, ei_org as usize) as c_int) {
                ei_org -= 1;
            }
            while ei_new >= si_new && ascii_iswhite(byte_at(new, ei_new as usize) as c_int) {
                ei_new -= 1;
            }
        } else {
            // Both ends move to the head of the character covering them
            // before being compared, exactly as the prefix walk does.
            let p1 = ei_org as usize - head_off(org, ei_org as usize);
            let p2 = ei_new as usize - head_off(new, ei_new as usize);
            match char_len(&org[p1.min(org.len())..], &new[p2.min(new.len())..]) {
                Some(l) => {
                    ei_org -= l as c_int;
                    ei_new -= l as c_int;
                }
                None => break,
            }
        }
    }
    ei_org
}

/// `inline:simple`: one changed range per line, first difference to last.
///
/// Answers whether the line is an *addition* -- present in no other buffer of
/// the block -- which is the only thing `inline:none` computes, and the
/// reason it skips the column search entirely.
///
/// `startp`/`endp` accumulate across the other buffers: `startp` takes the
/// leftmost start and `endp` the rightmost end, so a line differing from two
/// partners reports the union.
fn diff_find_change_simple(
    wp: Win,
    lnum: linenr_T,
    dp: Df,
    idx: c_int,
    startp: &mut c_int,
    endp: &mut c_int,
) -> bool {
    // A copy: every `ml_get_buf` below invalidates the last one's buffer.
    let line_org = (diff_flags.get() & DIFF_INLINE_NONE == 0).then(|| {
        // SAFETY: a live window's buffer, and a line number inside it.
        unsafe { CStr::from_ptr(ml_get_buf(wp.w_buffer, lnum)) }.to_owned()
    });
    let off = lnum - dp.df_lnum[idx as usize];
    let tp = cur_tab();
    let mut added = true;
    for i in 0..DB_COUNT as usize {
        let buf = tp.tp_diffbuf[i];
        // A line past the other buffer's count is a filler line there,
        // which says nothing about this one.
        if buf.is_null() || i as c_int == idx || off >= dp.df_count[i] {
            continue;
        }
        added = false;
        let Some(line_org) = line_org.as_deref() else {
            break; // `inline:none` wants only the answer above.
        };
        let org = line_org.to_bytes();
        let other = dp.df_lnum[i] + off;
        // SAFETY: a live buffer of the diff, and a line number inside the
        // block, so inside the buffer.
        let new = unsafe { CStr::from_ptr(ml_get_buf(buf, other)) }.to_bytes();

        let (si_org, si_new) = common_prefix(org, new);
        *startp = (*startp).min(si_org as c_int);
        if byte_at(org, si_org) != 0 || byte_at(new, si_new) != 0 {
            *endp = (*endp).max(common_suffix(org, new, *startp, si_new as c_int));
        }
    }
    added
}

/// The changed column ranges covering `lnum` in `wp`, as `diffline`.
///
/// Answers whether the line is an addition rather than a change.  Under
/// `inline:none`/`inline:simple` that is one range in
/// [`simple_diffline_change`]; under `inline:char`/`inline:word` it is a
/// window onto the block's cached `df_changes`, computed once by
/// [`diff_find_change_inline_diff`].
///
/// # Safety
/// `wp` must be a live window and `diffline` a writable `diffline_T`.
pub unsafe fn diff_find_change(wp: *mut win_T, lnum: linenr_T, diffline: *mut diffline_T) -> bool {
    // SAFETY: the caller's window.
    let wp = unsafe { Win::new(wp) };
    let tp = cur_tab();
    let idx = diff_slot(wp.w_buffer, tp);
    if idx == DB_COUNT {
        return false;
    }
    // The first block this line is not already past.
    let Some(dp) = diff_blocks(tp).find(|dp| lnum < dp.end(idx as usize)) else {
        return false;
    };
    if !dp.is_sane(tp) {
        return false;
    }
    let off = (lnum - dp.df_lnum[idx as usize]) as c_int;

    if diff_flags.get() & ALL_INLINE_DIFF == 0 {
        let mut change_start = MAXCOL as c_int;
        let mut change_end = -1;
        let start = &mut change_start;
        let end = &mut change_end;
        let added = diff_find_change_simple(wp, lnum, dp, idx, start, end);
        let mut only = diffline_change_T {
            dc_start: [0; 8],
            dc_end: [0; 8],
            dc_start_lnum_off: [0; 8],
            dc_end_lnum_off: [0; 8],
        };
        only.dc_start[idx as usize] = change_start as colnr_T;
        only.dc_end[idx as usize] = (change_end + 1) as colnr_T;
        only.dc_start_lnum_off[idx as usize] = off;
        only.dc_end_lnum_off[idx as usize] = off;
        let change = simple_diffline_change.ptr();
        let line = diffline_S {
            changes: change,
            num_changes: 1,
            bufidx: idx,
            lineoff: off,
        };
        // SAFETY: this module's own one-element answer buffer, and the
        // caller's out-parameter.
        unsafe {
            *change = only;
            *diffline = line;
        }
        return added;
    }

    if !dp.has_changes {
        // SAFETY: a live block.
        unsafe { diff_find_change_inline_diff(dp.raw()) };
    }
    // The changes are stored for the whole block; this line's window into
    // them is the run whose line-offset span covers `off`.
    let changes = dp.df_changes.ga_data as *mut diffline_change_T;
    let len = dp.df_changes.ga_len;
    let mut first = ::core::ptr::null_mut::<diffline_change_T>();
    let mut num_changes = 0;
    let mut change_idx = 0;
    while change_idx < len {
        // SAFETY: `ga_data` holds `ga_len` entries and `change_idx` is below
        // it.
        let change = unsafe { Live::new(changes.add(change_idx as usize)) };
        if change.dc_end_lnum_off[idx as usize] >= off {
            if change.dc_start_lnum_off[idx as usize] > off {
                break;
            }
            if first.is_null() {
                first = change.raw();
            }
            num_changes += 1;
        }
        change_idx += 1;
    }
    let line = diffline_S {
        changes: first,
        num_changes,
        bufidx: idx,
        lineoff: off,
    };
    // SAFETY: the caller's out-parameter.
    unsafe { *diffline = line };

    // The line is an addition when the block's *last* change is the only
    // one covering it and it is empty in every other buffer, which the
    // inline diff marks with a `dc_start_lnum_off` of `INT_MAX`.
    if num_changes != 1 || change_idx != len {
        return false;
    }
    // SAFETY: `num_changes` is 1, so `ga_len` is at least 1 and the last
    // entry is in bounds.
    let last = unsafe { Live::new(changes.add((len - 1) as usize)) };
    (0..DB_COUNT as usize).all(|i| {
        i as c_int == idx || tp.tp_diffbuf[i].is_null() || last.dc_start_lnum_off[i] == c_int::MAX
    })
}

/// `diff_hlID(lnum, col)`: the highlight group one column of one line takes.
///
/// The answer is cached across calls, because the drawer asks it once per
/// column of a line -- but only under `inline:none`/`inline:simple`, where
/// one line has one range.  With `inline:char`/`inline:word` a line can carry
/// several, so the cache is bypassed and `diffline` is walked per column.
pub unsafe fn f_diff_hl_id(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    static prev_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
    static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0);
    static fnum: GlobalCell<c_int> = GlobalCell::new(0);
    static prev_diff_flags: GlobalCell<c_int> = GlobalCell::new(0);
    static change_start: GlobalCell<c_int> = GlobalCell::new(0);
    static change_end: GlobalCell<c_int> = GlobalCell::new(0);
    static hlID: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);

    let mut diffline = diffline_S {
        changes: ::core::ptr::null_mut::<diffline_change_T>(),
        num_changes: 0,
        bufidx: 0,
        lineoff: 0,
    };
    let cache_results = diff_flags.get() & ALL_INLINE_DIFF == 0;
    // SAFETY: the caller's argument list.
    let lnum = unsafe { tv_get_lnum(argvars) }.max(0);

    let stale = !cache_results
        || lnum != prev_lnum.get()
        // SAFETY: the current buffer is live.
        || changedtick.get() != unsafe { buf_get_changedtick(curbuf.get()) }
        || fnum.get() != cur_buf().handle
        || diff_flags.get() != prev_diff_flags.get();
    if stale {
        let mut linestatus = 0;
        let status = &raw mut linestatus;
        // SAFETY: the current window is live; `linestatus` is a local.
        unsafe { diff_check_with_linestatus(cur_win().raw(), lnum, status) };
        hlID.set(match linestatus {
            LINE_CHANGED => {
                change_start.set(MAXCOL as c_int);
                change_end.set(-1);
                let out = &raw mut diffline;
                // SAFETY: the current window is live; `diffline` is a local.
                let added = unsafe { diff_find_change(cur_win().raw(), lnum, out) };
                if added {
                    HLF_ADD
                } else {
                    if diffline.num_changes > 0 && cache_results {
                        // SAFETY: a positive `num_changes` means `changes`
                        // names at least one live entry.
                        let ch = unsafe { Live::new(diffline.changes) };
                        let buf = diffline.bufidx as usize;
                        change_start.set(ch.dc_start[buf] as c_int);
                        change_end.set(ch.dc_end[buf] as c_int);
                    }
                    HLF_CHD
                }
            }
            // `LINE_INSERTED`: the line has no counterpart at all.
            n if n < 0 => HLF_ADD,
            _ => HLF_NONE,
        });
        if cache_results {
            prev_lnum.set(lnum);
            // SAFETY: the current buffer is live.
            changedtick.set(unsafe { buf_get_changedtick(curbuf.get()) });
            fnum.set(cur_buf().handle);
            prev_diff_flags.set(diff_flags.get());
        }
    }

    if hlID.get() == HLF_CHD || hlID.get() == HLF_TXD {
        // SAFETY: `diff_hlID()` is declared with two arguments.
        let col = unsafe { tv_get_number(argvars.offset(1)) } as c_int - 1;
        if cache_results {
            hlID.set(if col >= change_start.get() && col < change_end.get() {
                HLF_TXD
            } else {
                HLF_CHD
            });
        } else {
            hlID.set(HLF_CHD);
            for i in 0..diffline.num_changes {
                // Out-parameters of this frame: the cached pair above is
                // only ever read on the other side of this `if`.
                let mut start = 0;
                let mut end = 0;
                let dl = &raw mut diffline;
                let ch = diffline.changes.wrapping_add(i as usize);
                // SAFETY: `i` is below `num_changes`, so `ch` names one of
                // the line's own changes; the two ends are locals.
                let added = unsafe { diff_change_parse(dl, ch, &raw mut start, &raw mut end) };
                if col >= start && col < end {
                    hlID.set(if added { HLF_TXA } else { HLF_TXD });
                    break;
                }
                // The ranges are in column order, so a column left of
                // this one's start is left of all the rest too.
                if col < start {
                    break;
                }
            }
        }
    }
    let id = hlID.get() as varnumber_T;
    // SAFETY: the caller's result cell.
    unsafe { (*rettv).vval.v_number = id };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
