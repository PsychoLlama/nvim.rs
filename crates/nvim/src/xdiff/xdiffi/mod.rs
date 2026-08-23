//! `xdiffi.c`: the classic algorithm, hunk compaction, and the entry point.
//!
//! [`split`] and [`recs_cmp`] are Myers' O(ND) walk — "An O(ND) Difference
//! Algorithm and its Variations", Eugene Myers — run as divide and conquer:
//! a box is split at the point where the forward and backward frontiers
//! meet, and the two halves are split again. The heuristics in [`split`] are
//! what keep it out of the pathological cases, at the cost of a suboptimal
//! script.
//!
//! Ported from LibXDiff by Davide Libenzi (File Differential Library),
//! Copyright (C) 2003 Davide Libenzi. LibXDiff is LGPL-2.1-or-later, and
//! this port stays under that license (text: licenses/LGPL-2.1.txt).
//!
//! The second half — sliding each run of changed lines to wherever it reads
//! best, which is where `'diffopt'`'s `indent-heuristic` lives — is in
//! [`compact`].

#![forbid(unsafe_code)]

use core::ops::{Index, IndexMut};

use crate::xdiff::ffi::Emit;
use crate::xdiff::xemit::{emit_diff, get_hunk};
use crate::xdiff::xprepare::prepare_env;
use crate::xdiff::xtypes::{
    Algorithm, Change, DiffData, EmitConf, Env, Params, XDF_IGNORE_BLANK_LINES, XDF_NEED_MINIMAL,
    XdResult,
};
use crate::xdiff::xutils::{blankline, bogosqrt};
use crate::xdiff::{xhistogram, xpatience};

mod compact;

pub(crate) use compact::change_compact;

/// Floor on the edit cost at which [`split`] gives up and takes the
/// furthest-reaching path it has.
const XDL_MAX_COST_MIN: i64 = 256;
/// Edit cost below which the "did we find a good snake?" heuristic is not
/// even consulted.
const XDL_HEUR_MIN_COST: i64 = 256;
/// How many consecutive matching lines count as a snake worth stopping at.
const XDL_SNAKE_CNT: i64 = 20;
/// How much better than the current edit cost a diagonal has to look before
/// the heuristic will cut the search short.
const XDL_K_HEUR: i64 = 4;
/// A line number no backward diagonal has reached yet.
const XDL_LINE_MAX: i64 = i64::MAX;

/// `xdalgoenv_t`: the three knobs [`split`]'s heuristics read.
struct AlgoEnv {
    /// Edit cost at which to stop searching and take the best path so far.
    mxcost: i64,
    /// Matching-line run length that counts as a snake.
    snake_cnt: i64,
    /// Edit cost above which the snake heuristic applies.
    heur_min: i64,
}

/// One of the Myers walk's two K-vectors: the furthest point reached on each
/// diagonal. Diagonals run from `off1 - lim2` to `lim1 - off2`, so the index
/// is signed and the storage is biased.
struct KVec {
    cells: Vec<i64>,
    bias: i64,
}

impl KVec {
    fn new(len: usize, bias: i64) -> Self {
        Self {
            cells: vec![0; len],
            bias,
        }
    }
}

impl Index<i64> for KVec {
    type Output = i64;

    fn index(&self, d: i64) -> &i64 {
        &self.cells[(d + self.bias) as usize]
    }
}

impl IndexMut<i64> for KVec {
    fn index_mut(&mut self, d: i64) -> &mut i64 {
        &mut self.cells[(d + self.bias) as usize]
    }
}

/// The sub-problem one recursion level works on: the half-open box
/// `off1 ..< lim1` by `off2 ..< lim2` over the two reduced files.
#[derive(Clone, Copy)]
struct Bounds {
    off1: i64,
    lim1: i64,
    off2: i64,
    lim2: i64,
}

/// The band of diagonals one frontier currently spans, plus the diagonal it
/// started from — the heuristics score a diagonal by its distance from that.
#[derive(Clone, Copy)]
struct Band {
    min: i64,
    max: i64,
    mid: i64,
}

/// Where [`split`] cut the box, and whether each half must be solved
/// minimally (it need not be when the cut was a heuristic guess).
struct Split {
    i1: i64,
    i2: i64,
    min_lo: bool,
    min_hi: bool,
}

/// Split the box at the point where the forward diagonal from its corner and
/// the backward diagonal from the far corner cross.
///
/// Scanning from both ends is what makes this O(ND) rather than O(N^2), and
/// the two heuristics are what stop it degenerating: one takes an early exit
/// when a long run of matching lines ("a snake") turns up on a promising
/// diagonal, the other gives up at `mxcost` and returns the
/// furthest-reaching path measured by `i1 + i2`.
fn split(
    ha1: &[u64],
    ha2: &[u64],
    b: Bounds,
    kvdf: &mut KVec,
    kvdb: &mut KVec,
    need_min: bool,
    xenv: &AlgoEnv,
) -> Split {
    let (dmin, dmax) = (b.off1 - b.lim2, b.lim1 - b.off2);
    let mut fwd = Band {
        min: b.off1 - b.off2,
        max: b.off1 - b.off2,
        mid: b.off1 - b.off2,
    };
    let mut back = Band {
        min: b.lim1 - b.lim2,
        max: b.lim1 - b.lim2,
        mid: b.lim1 - b.lim2,
    };
    let odd = (fwd.mid - back.mid) & 1 != 0;

    kvdf[fwd.mid] = b.off1;
    kvdb[back.mid] = b.lim1;

    let mut ec = 0i64;
    loop {
        ec += 1;
        let mut got_snake = false;

        // Extend the diagonal domain by one. When the next value would
        // leave the box we move the other end instead, because `max - min`
        // has to stay a power of two; the outermost cell is seeded so the
        // core loop below needs no extra bounds test.
        if fwd.min > dmin {
            fwd.min -= 1;
            kvdf[fwd.min - 1] = -1;
        } else {
            fwd.min += 1;
        }
        if fwd.max < dmax {
            fwd.max += 1;
            kvdf[fwd.max + 1] = -1;
        } else {
            fwd.max -= 1;
        }

        let mut d = fwd.max;
        while d >= fwd.min {
            let mut i1 = if kvdf[d - 1] >= kvdf[d + 1] {
                kvdf[d - 1] + 1
            } else {
                kvdf[d + 1]
            };
            let prev1 = i1;
            let mut i2 = i1 - d;
            while i1 < b.lim1 && i2 < b.lim2 && ha1[i1 as usize] == ha2[i2 as usize] {
                i1 += 1;
                i2 += 1;
            }
            if i1 - prev1 > xenv.snake_cnt {
                got_snake = true;
            }
            kvdf[d] = i1;
            if odd && back.min <= d && d <= back.max && kvdb[d] <= i1 {
                return Split {
                    i1,
                    i2,
                    min_lo: true,
                    min_hi: true,
                };
            }
            d -= 2;
        }

        if back.min > dmin {
            back.min -= 1;
            kvdb[back.min - 1] = XDL_LINE_MAX;
        } else {
            back.min += 1;
        }
        if back.max < dmax {
            back.max += 1;
            kvdb[back.max + 1] = XDL_LINE_MAX;
        } else {
            back.max -= 1;
        }

        let mut d = back.max;
        while d >= back.min {
            let mut i1 = if kvdb[d - 1] < kvdb[d + 1] {
                kvdb[d - 1]
            } else {
                kvdb[d + 1] - 1
            };
            let prev1 = i1;
            let mut i2 = i1 - d;
            while i1 > b.off1 && i2 > b.off2 && ha1[(i1 - 1) as usize] == ha2[(i2 - 1) as usize] {
                i1 -= 1;
                i2 -= 1;
            }
            if prev1 - i1 > xenv.snake_cnt {
                got_snake = true;
            }
            kvdb[d] = i1;
            if !odd && fwd.min <= d && d <= fwd.max && i1 <= kvdf[d] {
                return Split {
                    i1,
                    i2,
                    min_lo: true,
                    min_hi: true,
                };
            }
            d -= 2;
        }

        if need_min {
            continue;
        }

        // With the edit cost above the trigger and a good snake in hand,
        // sample the diagonals for one that has reached somewhere
        // interesting: distance from the box corner, penalised by distance
        // from the mid diagonal. Above `XDL_K_HEUR` times the edit cost we
        // call it interesting and stop.
        if got_snake && ec > xenv.heur_min {
            if let Some(spl) = forward_snake(ha1, ha2, b, kvdf, fwd, ec, xenv) {
                return spl;
            }
            if let Some(spl) = backward_snake(ha1, ha2, b, kvdb, back, ec, xenv) {
                return spl;
            }
        }

        // Enough is enough. Collect the furthest reaching path using the
        // `i1 + i2` measure and split there.
        if ec >= xenv.mxcost {
            return furthest_reaching(b, kvdf, fwd, kvdb, back);
        }
    }
}

/// The forward half of [`split`]'s snake heuristic.
fn forward_snake(
    ha1: &[u64],
    ha2: &[u64],
    b: Bounds,
    kvdf: &KVec,
    fwd: Band,
    ec: i64,
    xenv: &AlgoEnv,
) -> Option<Split> {
    let mut best = 0i64;
    let mut at = None;
    let mut d = fwd.max;
    while d >= fwd.min {
        let dd = (d - fwd.mid).abs();
        let i1 = kvdf[d];
        let i2 = i1 - d;
        let v = (i1 - b.off1) + (i2 - b.off2) - dd;

        if v > XDL_K_HEUR * ec
            && v > best
            && b.off1 + xenv.snake_cnt <= i1
            && i1 < b.lim1
            && b.off2 + xenv.snake_cnt <= i2
            && i2 < b.lim2
        {
            let mut k = 1i64;
            while ha1[(i1 - k) as usize] == ha2[(i2 - k) as usize] {
                if k == xenv.snake_cnt {
                    best = v;
                    at = Some((i1, i2));
                    break;
                }
                k += 1;
            }
        }
        d -= 2;
    }
    at.filter(|_| best > 0).map(|(i1, i2)| Split {
        i1,
        i2,
        min_lo: true,
        min_hi: false,
    })
}

/// The backward half of [`split`]'s snake heuristic.
fn backward_snake(
    ha1: &[u64],
    ha2: &[u64],
    b: Bounds,
    kvdb: &KVec,
    back: Band,
    ec: i64,
    xenv: &AlgoEnv,
) -> Option<Split> {
    let mut best = 0i64;
    let mut at = None;
    let mut d = back.max;
    while d >= back.min {
        let dd = (d - back.mid).abs();
        let i1 = kvdb[d];
        let i2 = i1 - d;
        let v = (b.lim1 - i1) + (b.lim2 - i2) - dd;

        if v > XDL_K_HEUR * ec
            && v > best
            && b.off1 < i1
            && i1 <= b.lim1 - xenv.snake_cnt
            && b.off2 < i2
            && i2 <= b.lim2 - xenv.snake_cnt
        {
            let mut k = 0i64;
            while ha1[(i1 + k) as usize] == ha2[(i2 + k) as usize] {
                if k == xenv.snake_cnt - 1 {
                    best = v;
                    at = Some((i1, i2));
                    break;
                }
                k += 1;
            }
        }
        d -= 2;
    }
    at.filter(|_| best > 0).map(|(i1, i2)| Split {
        i1,
        i2,
        min_lo: false,
        min_hi: true,
    })
}

/// [`split`]'s give-up path: take whichever frontier reached further, in
/// `i1 + i2`, and cut there.
fn furthest_reaching(b: Bounds, kvdf: &KVec, fwd: Band, kvdb: &KVec, back: Band) -> Split {
    let (mut fbest, mut fbest1) = (-1i64, -1i64);
    let mut d = fwd.max;
    while d >= fwd.min {
        let mut i1 = kvdf[d].min(b.lim1);
        let mut i2 = i1 - d;
        if b.lim2 < i2 {
            i1 = b.lim2 + d;
            i2 = b.lim2;
        }
        if fbest < i1 + i2 {
            fbest = i1 + i2;
            fbest1 = i1;
        }
        d -= 2;
    }

    let (mut bbest, mut bbest1) = (XDL_LINE_MAX, XDL_LINE_MAX);
    let mut d = back.max;
    while d >= back.min {
        let mut i1 = b.off1.max(kvdb[d]);
        let mut i2 = i1 - d;
        if i2 < b.off2 {
            i1 = b.off2 + d;
            i2 = b.off2;
        }
        if i1 + i2 < bbest {
            bbest = i1 + i2;
            bbest1 = i1;
        }
        d -= 2;
    }

    if (b.lim1 + b.lim2) - bbest < fbest - (b.off1 + b.off2) {
        Split {
            i1: fbest1,
            i2: fbest - fbest1,
            min_lo: true,
            min_hi: false,
        }
    } else {
        Split {
            i1: bbest1,
            i2: bbest - bbest1,
            min_lo: false,
            min_hi: true,
        }
    }
}

/// Divide et impera: split the box into sub-boxes and recurse. The real work
/// — marking changed lines — happens in the two boundary checks.
fn recs_cmp(
    dd1: &mut DiffData<'_>,
    dd2: &mut DiffData<'_>,
    mut b: Bounds,
    kvdf: &mut KVec,
    kvdb: &mut KVec,
    need_min: bool,
    xenv: &AlgoEnv,
) {
    let (ha1, ha2) = (dd1.ha, dd2.ha);

    // Shrink the box by walking through each diagonal snake (SW and NE).
    while b.off1 < b.lim1 && b.off2 < b.lim2 && ha1[b.off1 as usize] == ha2[b.off2 as usize] {
        b.off1 += 1;
        b.off2 += 1;
    }
    while b.off1 < b.lim1
        && b.off2 < b.lim2
        && ha1[(b.lim1 - 1) as usize] == ha2[(b.lim2 - 1) as usize]
    {
        b.lim1 -= 1;
        b.lim2 -= 1;
    }

    // If one dimension is empty then every record on the other one changed.
    if b.off1 == b.lim1 {
        for i in b.off2..b.lim2 {
            dd2.mark(i);
        }
    } else if b.off2 == b.lim2 {
        for i in b.off1..b.lim1 {
            dd1.mark(i);
        }
    } else {
        let spl = split(ha1, ha2, b, kvdf, kvdb, need_min, xenv);
        let lo = Bounds {
            off1: b.off1,
            lim1: spl.i1,
            off2: b.off2,
            lim2: spl.i2,
        };
        let hi = Bounds {
            off1: spl.i1,
            lim1: b.lim1,
            off2: spl.i2,
            lim2: b.lim2,
        };
        recs_cmp(dd1, dd2, lo, kvdf, kvdb, spl.min_lo, xenv);
        recs_cmp(dd1, dd2, hi, kvdf, kvdb, spl.min_hi, xenv);
    }
}

/// Prepare both files and run whichever engine the flags asked for.
pub(crate) fn do_diff<'a>(text1: &'a [u8], text2: &'a [u8], xpp: &Params<'_>) -> XdResult<Env<'a>> {
    match xpp.algorithm() {
        Algorithm::Patience => return xpatience::diff(text1, text2, xpp),
        Algorithm::Histogram => return xhistogram::diff(text1, text2, xpp),
        Algorithm::Myers => {}
    }

    let mut xe = prepare_env(text1, text2, xpp);

    // One K-vector for the forward path and one for the backward. Both are
    // indexed by diagonal, which runs from `-nreff2` to `nreff1`.
    let ndiags = xe.xdf1.nreff + xe.xdf2.nreff + 3;
    let bias = xe.xdf2.nreff + 1;
    let mut kvdf = KVec::new(ndiags as usize, bias);
    let mut kvdb = KVec::new(ndiags as usize, bias);

    let xenv = AlgoEnv {
        mxcost: bogosqrt(ndiags).max(XDL_MAX_COST_MIN),
        snake_cnt: XDL_SNAKE_CNT,
        heur_min: XDL_HEUR_MIN_COST,
    };
    let need_min = xpp.flags & XDF_NEED_MINIMAL != 0;

    let mut dd1 = xe.xdf1.diff_data();
    let mut dd2 = xe.xdf2.diff_data();
    let whole = Bounds {
        off1: 0,
        lim1: dd1.nrec,
        off2: 0,
        lim2: dd2.nrec,
    };
    recs_cmp(
        &mut dd1, &mut dd2, whole, &mut kvdf, &mut kvdb, need_min, &xenv,
    );

    Ok(xe)
}

/// Collect the runs of changed lines into an edit script.
///
/// Walks both files from the end, so a run is found at its last line and the
/// script comes out in increasing line order after the reverse.
fn build_script(xe: &Env<'_>) -> Vec<Change> {
    let mut script = Vec::new();
    let mut i1 = xe.xdf1.nrec();
    let mut i2 = xe.xdf2.nrec();

    while i1 >= 0 || i2 >= 0 {
        if xe.xdf1.rchg.get(i1 - 1) || xe.xdf2.rchg.get(i2 - 1) {
            let l1 = i1;
            while xe.xdf1.rchg.get(i1 - 1) {
                i1 -= 1;
            }
            let l2 = i2;
            while xe.xdf2.rchg.get(i2 - 1) {
                i2 -= 1;
            }
            script.push(Change {
                i1,
                i2,
                chg1: l1 - i1,
                chg2: l2 - i2,
                ignore: false,
            });
        }
        i1 -= 1;
        i2 -= 1;
    }

    script.reverse();
    script
}

/// Mark the changes whose every line is blank, for `XDF_IGNORE_BLANK_LINES`.
fn mark_ignorable_lines(script: &mut [Change], xe: &Env<'_>, flags: u64) {
    for ch in script {
        let mut ignore = true;
        let mut i = 0;
        while i < ch.chg1 && ignore {
            ignore = blankline(xe.xdf1.line(ch.i1 + i), flags);
            i += 1;
        }
        let mut i = 0;
        while i < ch.chg2 && ignore {
            ignore = blankline(xe.xdf2.line(ch.i2 + i), flags);
            i += 1;
        }
        ch.ignore = ignore;
    }
}

/// Report each hunk's extent through `xdemitconf_t.hunk_func` instead of
/// writing a diff. Both `:diffupdate` and `vim.diff{on_hunk=}` take this
/// path; nothing here reads a line.
fn call_hunk_func(script: &[Change], xecfg: &EmitConf, emit: &mut Emit<'_>) -> XdResult {
    let mut at = 0usize;
    while at < script.len() {
        let mut first = at;
        let Some(last) = get_hunk(script, &mut first, xecfg) else {
            break;
        };
        let (start, end) = (&script[first], &script[last]);
        emit.hunk(
            start.i1,
            end.i1 + end.chg1 - start.i1,
            start.i2,
            end.i2 + end.chg2 - start.i2,
        )?;
        at = last + 1;
    }
    Ok(())
}

/// `xdl_diff`'s body: diff the two texts and report through `emit`.
pub(crate) fn diff(
    text1: &[u8],
    text2: &[u8],
    xpp: &Params<'_>,
    xecfg: &EmitConf,
    emit: &mut Emit<'_>,
) -> XdResult {
    let mut xe = do_diff(text1, text2, xpp)?;

    change_compact(&mut xe.xdf1, &xe.xdf2, xpp.flags);
    change_compact(&mut xe.xdf2, &xe.xdf1, xpp.flags);
    let mut script = build_script(&xe);

    if script.is_empty() {
        return Ok(());
    }
    if xpp.flags & XDF_IGNORE_BLANK_LINES != 0 {
        mark_ignorable_lines(&mut script, &xe, xpp.flags);
    }
    if emit.has_hunk_func() {
        call_hunk_func(&script, xecfg, emit)
    } else {
        emit_diff(&xe, &script, xecfg, emit)
    }
}
