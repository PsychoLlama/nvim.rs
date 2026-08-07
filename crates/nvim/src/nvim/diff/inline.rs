//! Which columns of a changed line changed.
//!
//! [`diff_find_change`] answers, for one line of one window, the list of
//! changed column ranges the drawer paints `DiffText` over.
//! [`diff_find_change_simple`] is the `inline:simple` rule -- one range, from
//! the first differing byte to the last -- and [`diff_change_parse`] is how
//! the drawer reads a range back out.  [`f_diff_hlID`] is the Vimscript front
//! door to the same answer, and the only way to observe any of it without a
//! screen.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use ::core::ffi::c_int;
use ::std::ffi::CStr;

/// Throw away the cached inline changes of the block holding `lnum`.
///
/// Only the `inline:char`/`inline:word` modes cache anything; the simple rule
/// is recomputed per line.
pub unsafe fn diff_update_line(lnum: linenr_T) {
    unsafe {
        if diff_flags.get() & ALL_INLINE_DIFF == 0 {
            return;
        }
        let idx = diff_buf_idx(curbuf.get(), curtab.get());
        if idx == DB_COUNT {
            return;
        }
        let mut dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() && lnum > (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            dp = (*dp).df_next;
        }
        if !dp.is_null() {
            (*dp).has_changes = false;
            (*dp).df_changes.ga_len = 0;
        }
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
    unsafe {
        let buf = (*diffline).bufidx as usize;
        let lineoff = (*diffline).lineoff;
        *change_start = if (*change).dc_start_lnum_off[buf] < lineoff {
            0
        } else {
            (*change).dc_start[buf] as c_int
        };
        *change_end = if (*change).dc_end_lnum_off[buf] > lineoff {
            c_int::MAX
        } else {
            (*change).dc_end[buf] as c_int
        };
        if change == simple_diffline_change.ptr() {
            return false;
        }
        // An addition is a change that is empty in every *other* buffer.
        (0..DB_COUNT as usize).all(|i| {
            i == buf
                || (*change).dc_start[i] == (*change).dc_end[i]
                    && (*change).dc_end_lnum_off[i] == (*change).dc_start_lnum_off[i]
        })
    }
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
unsafe fn diff_find_change_simple(
    wp: *mut win_T,
    lnum: linenr_T,
    dp: *const diff_T,
    idx: c_int,
    startp: *mut c_int,
    endp: *mut c_int,
) -> bool {
    unsafe {
        // A copy: every `ml_get_buf` below invalidates the last one's buffer.
        let line_org = (diff_flags.get() & DIFF_INLINE_NONE == 0)
            .then(|| CStr::from_ptr(ml_get_buf((*wp).w_buffer, lnum)).to_owned());
        let off = lnum - (*dp).df_lnum[idx as usize];
        let mut added = true;
        for i in 0..DB_COUNT as usize {
            let buf = (*curtab.get()).tp_diffbuf[i];
            // A line past the other buffer's count is a filler line there,
            // which says nothing about this one.
            if buf.is_null() || i as c_int == idx || off >= (*dp).df_count[i] {
                continue;
            }
            added = false;
            let Some(line_org) = line_org.as_deref() else {
                break; // `inline:none` wants only the answer above.
            };
            let org = line_org.to_bytes();
            let new = CStr::from_ptr(ml_get_buf(buf, (*dp).df_lnum[i] + off)).to_bytes();

            let (si_org, si_new) = common_prefix(org, new);
            *startp = (*startp).min(si_org as c_int);
            if byte_at(org, si_org) != 0 || byte_at(new, si_new) != 0 {
                *endp = (*endp).max(common_suffix(org, new, *startp, si_new as c_int));
            }
        }
        added
    }
}

/// The changed column ranges covering `lnum` in `wp`, as `diffline`.
///
/// Answers whether the line is an addition rather than a change.  Under
/// `inline:none`/`inline:simple` that is one range in
/// [`simple_diffline_change`]; under `inline:char`/`inline:word` it is a
/// window onto the block's cached `df_changes`, computed once by
/// [`diff_find_change_inline_diff`].
pub unsafe fn diff_find_change(wp: *mut win_T, lnum: linenr_T, diffline: *mut diffline_T) -> bool {
    unsafe {
        let idx = diff_buf_idx((*wp).w_buffer, curtab.get());
        if idx == DB_COUNT {
            return false;
        }
        let mut dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() && lnum >= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            dp = (*dp).df_next;
        }
        if dp.is_null() || diff_check_sanity(curtab.get(), dp) == FAIL {
            return false;
        }
        let off = (lnum - (*dp).df_lnum[idx as usize]) as c_int;

        if diff_flags.get() & ALL_INLINE_DIFF == 0 {
            let mut change_start = MAXCOL as c_int;
            let mut change_end = -1;
            let added = diff_find_change_simple(
                wp,
                lnum,
                dp,
                idx,
                &raw mut change_start,
                &raw mut change_end,
            );
            change_end += 1;
            let change = simple_diffline_change.ptr();
            *change = diffline_change_T {
                dc_start: [0; 8],
                dc_end: [0; 8],
                dc_start_lnum_off: [0; 8],
                dc_end_lnum_off: [0; 8],
            };
            (*change).dc_start[idx as usize] = change_start as colnr_T;
            (*change).dc_end[idx as usize] = change_end as colnr_T;
            (*change).dc_start_lnum_off[idx as usize] = off;
            (*change).dc_end_lnum_off[idx as usize] = off;
            *diffline = diffline_S {
                changes: change,
                num_changes: 1,
                bufidx: idx,
                lineoff: off,
            };
            return added;
        }

        if !(*dp).has_changes {
            diff_find_change_inline_diff(dp);
        }
        // The changes are stored for the whole block; this line's window into
        // them is the run whose line-offset span covers `off`.
        let changes = (*dp).df_changes.ga_data as *mut diffline_change_T;
        let mut first = ::core::ptr::null_mut::<diffline_change_T>();
        let mut num_changes = 0;
        let mut change_idx = 0;
        while change_idx < (*dp).df_changes.ga_len {
            let change = changes.offset(change_idx as isize);
            if (*change).dc_end_lnum_off[idx as usize] >= off {
                if (*change).dc_start_lnum_off[idx as usize] > off {
                    break;
                }
                if first.is_null() {
                    first = change;
                }
                num_changes += 1;
            }
            change_idx += 1;
        }
        *diffline = diffline_S {
            changes: first,
            num_changes,
            bufidx: idx,
            lineoff: off,
        };

        // The line is an addition when the block's *last* change is the only
        // one covering it and it is empty in every other buffer, which the
        // inline diff marks with a `dc_start_lnum_off` of `INT_MAX`.
        if num_changes != 1 || change_idx != (*dp).df_changes.ga_len {
            return false;
        }
        let last = changes.offset(((*dp).df_changes.ga_len - 1) as isize);
        (0..DB_COUNT as usize).all(|i| {
            i as c_int == idx
                || (*curtab.get()).tp_diffbuf[i].is_null()
                || (*last).dc_start_lnum_off[i] == c_int::MAX
        })
    }
}

/// `diff_hlID(lnum, col)`: the highlight group one column of one line takes.
///
/// The answer is cached across calls, because the drawer asks it once per
/// column of a line -- but only under `inline:none`/`inline:simple`, where
/// one line has one range.  With `inline:char`/`inline:word` a line can carry
/// several, so the cache is bypassed and `diffline` is walked per column.
pub unsafe extern "C" fn f_diff_hlID(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
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
        let lnum = tv_get_lnum(argvars).max(0);

        if !cache_results
            || lnum != prev_lnum.get()
            || changedtick.get() != buf_get_changedtick(curbuf.get())
            || fnum.get() != (*curbuf.get()).handle
            || diff_flags.get() != prev_diff_flags.get()
        {
            let mut linestatus = 0;
            diff_check_with_linestatus(curwin.get(), lnum, &raw mut linestatus);
            hlID.set(match linestatus {
                LINE_CHANGED => {
                    change_start.set(MAXCOL as c_int);
                    change_end.set(-1);
                    if diff_find_change(curwin.get(), lnum, &raw mut diffline) {
                        HLF_ADD
                    } else {
                        if diffline.num_changes > 0 && cache_results {
                            change_start.set(
                                (*diffline.changes).dc_start[diffline.bufidx as usize] as c_int,
                            );
                            change_end
                                .set((*diffline.changes).dc_end[diffline.bufidx as usize] as c_int);
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
                changedtick.set(buf_get_changedtick(curbuf.get()));
                fnum.set((*curbuf.get()).handle);
                prev_diff_flags.set(diff_flags.get());
            }
        }

        if hlID.get() == HLF_CHD || hlID.get() == HLF_TXD {
            let col = tv_get_number(argvars.offset(1)) as c_int - 1;
            if cache_results {
                hlID.set(if col >= change_start.get() && col < change_end.get() {
                    HLF_TXD
                } else {
                    HLF_CHD
                });
            } else {
                hlID.set(HLF_CHD);
                for i in 0..diffline.num_changes {
                    let added = diff_change_parse(
                        &raw mut diffline,
                        diffline.changes.offset(i as isize),
                        change_start.ptr(),
                        change_end.ptr(),
                    );
                    if col >= change_start.get() && col < change_end.get() {
                        hlID.set(if added { HLF_TXA } else { HLF_TXD });
                        break;
                    }
                    // The ranges are in column order, so a column left of
                    // this one's start is left of all the rest too.
                    if col < change_start.get() {
                        break;
                    }
                }
            }
        }
        (*rettv).vval.v_number = hlID.get() as varnumber_T;
    }
}
