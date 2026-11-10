//! Which columns of a changed line changed.
//!
//! [`diff_find_change`] answers, for one line of one window, the list of
//! changed column ranges the drawer paints `DiffText` over.
//! [`diff_find_change_simple`] is the `inline:simple` rule -- one range, from
//! the first differing byte to the last -- and [`DiffLine::change`] is how
//! the drawer reads a range back out.  [`f_diff_hl_id`] is the Vimscript front
//! door to the same answer, and the only way to observe any of it without a
//! screen.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::*;
use crate::charset::skip;
use crate::cstr::byte_at;
use crate::narrow::as_index;
use crate::narrow::len_as_int;
use crate::narrow::number_as_int;
use crate::types::DiffLineChanges;
use crate::winlayer::{Buf, BufId, TabPage, Win};
use core::ffi::c_int;

/// Throw away the cached inline changes of the block holding `lnum`.
///
/// Only the `inline:char`/`inline:word` modes cache anything; the simple rule
/// is recomputed per line.
pub fn diff_update_line(lnum: LineNr) {
    if DIFF_FLAGS.get() & ALL_INLINE_DIFF == 0 {
        return;
    }
    let tp = TabPage::current();
    let idx = tp.diff_index(Buf::current());
    if idx == DB_COUNT {
        return;
    }
    let idx = as_index(idx);
    tp.with_diff(|blocks| {
        if let Some(dp) = blocks.iter_mut().find(|dp| lnum <= dp.end(idx)) {
            dp.has_changes = false;
            dp.df_changes.clear();
        }
    });
}

impl DiffLine {
    /// How many changed ranges the line has.
    pub(crate) fn num_changes(&self) -> c_int {
        match self.changes {
            DiffLineChanges::None => 0,
            DiffLineChanges::Simple { .. } => 1,
            DiffLineChanges::Inline { num, .. } => num.cast_signed(),
        }
    }

    /// Change `i` of an `inline:char`/`inline:word` line, read back out of
    /// its block -- `None` once the list has changed shape, or the block's
    /// cache has been dropped, since the line was looked at.
    fn inline_change(&self, i: c_int) -> Option<DiffLineChange> {
        let DiffLineChanges::Inline {
            block,
            first,
            num,
            generation,
        } = self.changes
        else {
            return None;
        };
        let i = u32::try_from(i).ok().filter(|&i| i < num)?;
        let tp = TabPage::current();
        if tp.tp_diff_gen != generation {
            return None;
        }
        let block = tp.tp_diff.get(block as usize)?;
        block.df_changes.get((first + i) as usize).copied()
    }

    /// Where change `i` starts in this line's buffer, as the block stored
    /// it: the column the drawer compares against to step to the next one.
    /// A change that can no longer be read starts past every column.
    pub(crate) fn raw_start(&self, i: c_int) -> c_int {
        match self.changes {
            DiffLineChanges::Simple { start, .. } => start.get(),
            _ => self
                .inline_change(i)
                .map_or(c_int::MAX, |ch| ch.dc_start[as_index(self.bufidx)].get()),
        }
    }

    /// Read change `i`'s column range on this line: `(start, end, added)`.
    ///
    /// A change can span several lines, so a range that starts above this
    /// line begins at column 0 and one that ends below it runs to the end of
    /// the line. `added` says the range is an *addition* rather than a
    /// change, which the caller paints `DiffTextAdd` instead of `DiffText`;
    /// `inline:simple`'s one range never is. A change that can no longer be
    /// read -- Lua run by the drawer rebuilt the list -- is an empty range
    /// past every column.
    pub(crate) fn change(&self, i: c_int) -> (c_int, c_int, bool) {
        if let DiffLineChanges::Simple { start, end } = self.changes {
            return (start.get(), end.get(), false);
        }
        let Some(ch) = self.inline_change(i) else {
            return (c_int::MAX, c_int::MAX, false);
        };
        let buf = as_index(self.bufidx);
        let lineoff = self.lineoff;
        let start = if ch.dc_start_lnum_off[buf] < lineoff {
            0
        } else {
            ch.dc_start[buf].get()
        };
        let end = if ch.dc_end_lnum_off[buf] > lineoff {
            c_int::MAX
        } else {
            ch.dc_end[buf].get()
        };
        // An addition is a change that is empty in every *other* buffer.
        let added = (0..DB_COUNT as usize).all(|i| {
            i == buf
                || ch.dc_start[i] == ch.dc_end[i]
                    && ch.dc_end_lnum_off[i] == ch.dc_start_lnum_off[i]
        });
        (start, end, added)
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
    let flags = DIFF_FLAGS.get();
    let (mut si_org, mut si_new) = (0usize, 0usize);
    while si_org < org.len() {
        let w_org = ascii_iswhite(c_int::from(byte_at(org, si_org)));
        let w_new = ascii_iswhite(c_int::from(byte_at(new, si_new)));
        if flags & DIFF_IWHITE != 0 && w_org && w_new
            || flags & DIFF_IWHITEALL != 0 && (w_org || w_new)
        {
            si_org = org.len() - skip::past_white(&org[si_org..]).len();
            si_new = new.len() - skip::past_white(&new[si_new.min(new.len())..]).len();
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
    let flags = DIFF_FLAGS.get();
    let (mut ei_org, mut ei_new) = (len_as_int(org.len()), len_as_int(new.len()));
    while ei_org >= start && ei_new >= si_new && ei_org >= 0 && ei_new >= 0 {
        let w_org = ascii_iswhite(c_int::from(byte_at(org, as_index(ei_org))));
        let w_new = ascii_iswhite(c_int::from(byte_at(new, as_index(ei_new))));
        if flags & DIFF_IWHITE != 0 && w_org && w_new
            || flags & DIFF_IWHITEALL != 0 && (w_org || w_new)
        {
            while ei_org >= start && ascii_iswhite(c_int::from(byte_at(org, as_index(ei_org)))) {
                ei_org -= 1;
            }
            while ei_new >= si_new && ascii_iswhite(c_int::from(byte_at(new, as_index(ei_new)))) {
                ei_new -= 1;
            }
        } else {
            // Both ends move to the head of the character covering them
            // before being compared, exactly as the prefix walk does.
            let p1 = as_index(ei_org) - head_off(org, as_index(ei_org));
            let p2 = as_index(ei_new) - head_off(new, as_index(ei_new));
            match char_len(&org[p1.min(org.len())..], &new[p2.min(new.len())..]) {
                Some(l) => {
                    ei_org -= len_as_int(l);
                    ei_new -= len_as_int(l);
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
    window: Win,
    lnum: LineNr,
    dp: &DiffBlock,
    slots: &DiffSlots,
    idx: c_int,
    startp: &mut c_int,
    endp: &mut c_int,
) -> bool {
    // A copy: the loop below reads every other buffer of the diff, and it
    // is this window's own on the turn that skips it.
    let line_org =
        (DIFF_FLAGS.get() & DIFF_INLINE_NONE == 0).then(|| window.buffer().lines().line_copy(lnum));
    let off = lnum - dp.df_lnum[as_index(idx)];
    let mut added = true;
    for (i, slot) in slots.iter().enumerate() {
        // A line past the other buffer's count is a filler line there,
        // which says nothing about this one.
        let Some(buf) = slot.and_then(BufId::get) else {
            continue;
        };
        if len_as_int(i) == idx || off >= dp.df_count[i] {
            continue;
        }
        added = false;
        let Some(line_org) = line_org.as_deref() else {
            break; // `inline:none` wants only the answer above.
        };
        let org = line_org;
        let other = dp.df_lnum[i] + off;
        let mut lines = buf.lines();
        let new = lines.line(other);

        let (si_org, si_new) = common_prefix(org, new);
        *startp = (*startp).min(len_as_int(si_org));
        if byte_at(org, si_org) != 0 || byte_at(new, si_new) != 0 {
            *endp = (*endp).max(common_suffix(org, new, *startp, len_as_int(si_new)));
        }
    }
    added
}

/// The changed column ranges covering `lnum` in `window`, written to
/// `diffline`.
///
/// Answers whether the line is an addition rather than a change.  Under
/// `inline:none`/`inline:simple` that is one range; under
/// `inline:char`/`inline:word` it is a copy of the run of the block's
/// cached `df_changes` that covers the line, computed once per block by
/// [`diff_find_change_inline_diff`].
pub(crate) fn diff_find_change(window: Win, lnum: LineNr, diffline: &mut DiffLine) -> bool {
    let tp = TabPage::current();
    let idx = tp.diff_index(window.buffer());
    if idx == DB_COUNT {
        return false;
    }
    let slots = tp.tp_diffbuf;
    let generation = tp.tp_diff_gen;
    tp.with_diff(|blocks| {
        // The first block this line is not already past.
        let Some(block) = blocks.iter().position(|dp| lnum < dp.end(as_index(idx))) else {
            return false;
        };
        let dp = &mut blocks[block];
        if diff_check_sanity(&slots, dp).is_err() {
            return false;
        }
        let off = (lnum - dp.df_lnum[as_index(idx)]) as c_int;
        diffline.bufidx = idx;
        diffline.lineoff = off;

        if DIFF_FLAGS.get() & ALL_INLINE_DIFF == 0 {
            let mut change_start = MAXCOL;
            let mut change_end = -1;
            let start = &mut change_start;
            let end = &mut change_end;
            let added = diff_find_change_simple(window, lnum, dp, &slots, idx, start, end);
            diffline.changes = DiffLineChanges::Simple {
                start: ColNr::new(change_start),
                end: ColNr::new(change_end + 1),
            };
            return added;
        }

        if !dp.has_changes {
            diff_find_change_inline_diff(dp, &slots);
        }
        // The changes are stored for the whole block; this line's share of
        // them is the run whose line-offset span covers `off`.
        let changes = &dp.df_changes;
        let at = as_index(idx);
        let mut first = None;
        let mut num_changes = 0;
        let mut change_idx = 0;
        while change_idx < changes.len() {
            let change = &changes[change_idx];
            if change.dc_end_lnum_off[at] >= off {
                if change.dc_start_lnum_off[at] > off {
                    break;
                }
                first.get_or_insert(change_idx);
                num_changes += 1;
            }
            change_idx += 1;
        }
        let first = first.unwrap_or(0);
        diffline.changes = DiffLineChanges::Inline {
            block: u32::try_from(block).expect("fewer blocks than lines"),
            first: u32::try_from(first).expect("fewer changes than lines"),
            num: u32::try_from(num_changes).expect("fewer changes than lines"),
            generation,
        };

        // The line is an addition when the block's *last* change is the only
        // one covering it and it is empty in every other buffer, which the
        // inline diff marks with a `dc_start_lnum_off` of `INT_MAX`.
        if num_changes != 1 || change_idx != changes.len() {
            return false;
        }
        let last = &changes[changes.len() - 1];
        (0..DB_COUNT as usize).all(|i| {
            len_as_int(i) == idx || slots[i].is_none() || last.dc_start_lnum_off[i] == c_int::MAX
        })
    })
}

/// `diff_hlID(lnum, col)`: the highlight group one column of one line takes.
///
/// The answer is cached across calls, because the drawer asks it once per
/// column of a line -- but only under `inline:none`/`inline:simple`, where
/// one line has one range.  With `inline:char`/`inline:word` a line can carry
/// several, so the cache is bypassed and `diffline` is walked per column.
pub fn f_diff_hl_id(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    static PREV_LNUM: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
    static CHANGEDTICK: GlobalCell<VarNumber> = GlobalCell::new(0);
    static FNUM: GlobalCell<c_int> = GlobalCell::new(0);
    static PREV_DIFF_FLAGS: GlobalCell<c_int> = GlobalCell::new(0);
    static CHANGE_START: GlobalCell<c_int> = GlobalCell::new(0);
    static CHANGE_END: GlobalCell<c_int> = GlobalCell::new(0);
    static HL_ID: GlobalCell<Hlf> = GlobalCell::new(HLF_NONE);

    let mut diffline = DiffLine::default();
    let cache_results = DIFF_FLAGS.get() & ALL_INLINE_DIFF == 0;
    let lnum = tv_get_lnum(&args[0]).max(LineNr::ZERO);

    let stale = !cache_results
        || lnum != PREV_LNUM.get()
        || CHANGEDTICK.get() != buf_get_changedtick(Buf::current())
        || FNUM.get() != Buf::current().handle.get()
        || DIFF_FLAGS.get() != PREV_DIFF_FLAGS.get();
    if stale {
        let (_, linestatus) = diff_check_with_linestatus(Win::current(), lnum);
        HL_ID.set(match linestatus {
            LINE_CHANGED => {
                CHANGE_START.set(MAXCOL);
                CHANGE_END.set(-1);
                let added = diff_find_change(Win::current(), lnum, &mut diffline);
                if added {
                    HLF_ADD
                } else {
                    // Only `inline:simple` caches, and its one range is
                    // read as stored.
                    if let DiffLineChanges::Simple { start, end } = diffline.changes
                        && cache_results
                    {
                        CHANGE_START.set(start.get());
                        CHANGE_END.set(end.get());
                    }
                    HLF_CHD
                }
            }
            // `LINE_INSERTED`: the line has no counterpart at all.
            n if n < 0 => HLF_ADD,
            _ => HLF_NONE,
        });
        if cache_results {
            PREV_LNUM.set(lnum);
            CHANGEDTICK.set(buf_get_changedtick(Buf::current()));
            FNUM.set(Buf::current().handle.get());
            PREV_DIFF_FLAGS.set(DIFF_FLAGS.get());
        }
    }

    if HL_ID.get() == HLF_CHD || HL_ID.get() == HLF_TXD {
        let col = number_as_int(tv_get_number(&args[1])) - 1;
        if cache_results {
            HL_ID.set(if col >= CHANGE_START.get() && col < CHANGE_END.get() {
                HLF_TXD
            } else {
                HLF_CHD
            });
        } else {
            HL_ID.set(HLF_CHD);
            for i in 0..diffline.num_changes() {
                let (start, end, added) = diffline.change(i);
                if col >= start && col < end {
                    HL_ID.set(if added { HLF_TXA } else { HLF_TXD });
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
    let id = VarNumber::from(HL_ID.get());
    result.write_number(id);
}
