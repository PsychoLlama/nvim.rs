//! `xdiffi.c`: the classic algorithm, hunk compaction, and the entry point.
//!
//! [`split`] and [`recs_cmp`] are Myers' O(ND) walk — "An O(ND) Difference
//! Algorithm and its Variations", Eugene Myers — run as divide and conquer:
//! a box is split at the point where the forward and backward frontiers
//! meet, and the two halves are split again. The heuristics in [`split`] are
//! what keep it out of the pathological cases, at the cost of a suboptimal
//! script.
//!
//! [`change_compact`] is the second half and is not about correctness at
//! all: a run of changed lines that can be slid up or down without changing
//! the edit script is slid to wherever it reads best, which is where
//! `'diffopt'`'s `indent-heuristic` lives.

#![forbid(unsafe_code)]

use core::ops::{Index, IndexMut};

use crate::src::xdiff::ffi::{Emit, is_space};
use crate::src::xdiff::xemit::{emit_diff, get_hunk};
use crate::src::xdiff::xprepare::prepare_env;
use crate::src::xdiff::xtypes::{
    Algorithm, Change, DiffData, EmitConf, Env, Params, XDF_IGNORE_BLANK_LINES,
    XDF_INDENT_HEURISTIC, XDF_NEED_MINIMAL, XdFile, XdResult,
};
use crate::src::xdiff::xutils::{blankline, bogosqrt, recmatch};
use crate::src::xdiff::{xhistogram, xpatience};

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

/// If a line is indented more than this, [`line_indent`] just answers this.
/// Keeps the work bounded on data that is not human-readable text.
const MAX_INDENT: i32 = 200;
/// The same idea for runs of blank lines around a candidate split.
const MAX_BLANKS: i32 = 20;
/// How far a hunk may be slid in search of a better-looking split.
const INDENT_HEURISTIC_MAX_SLIDING: i64 = 100;

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
pub fn do_diff<'a>(text1: &'a [u8], text2: &'a [u8], xpp: &Params<'_>) -> XdResult<Env<'a>> {
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

/// Do lines `a` and `b` of the same file match?
fn recs_match(xdf: &XdFile<'_>, a: i64, b: i64, flags: u64) -> bool {
    xdf.ha_at(a) == xdf.ha_at(b) && recmatch(xdf.line(a), xdf.line(b), flags)
}

/// A run of changed lines, or the empty run above an unchanged line.
///
/// If the first line of a group equals the line following it, the group can
/// be slid down; if its last line equals the line preceding it, it can be
/// slid up. Neither move changes the edit script's size — which is exactly
/// why the choice is free, and why [`change_compact`] gets to make it on
/// aesthetic grounds.
#[derive(Clone, Copy)]
struct Group {
    /// First changed line, or the unchanged line an empty group sits above.
    start: i64,
    /// First unchanged line after the group; equal to `start` when empty.
    end: i64,
}

/// The first group in `xdf`.
fn group_init(xdf: &XdFile<'_>) -> Group {
    let mut g = Group { start: 0, end: 0 };
    while xdf.rchg.get(g.end) {
        g.end += 1;
    }
    g
}

/// Move `g` to the next group. False when it is already the last.
fn group_next(xdf: &XdFile<'_>, g: &mut Group) -> bool {
    if g.end == xdf.nrec() {
        return false;
    }
    g.start = g.end + 1;
    g.end = g.start;
    while xdf.rchg.get(g.end) {
        g.end += 1;
    }
    true
}

/// Move `g` to the previous group. False when it is already the first.
fn group_previous(xdf: &XdFile<'_>, g: &mut Group) -> bool {
    if g.start == 0 {
        return false;
    }
    g.end = g.start - 1;
    g.start = g.end;
    while xdf.rchg.get(g.start - 1) {
        g.start -= 1;
    }
    true
}

/// Slide `g` one line toward the end of the file, absorbing any group it
/// bumps into. False when it cannot be slid.
fn group_slide_down(xdf: &mut XdFile<'_>, g: &mut Group, flags: u64) -> bool {
    if g.end < xdf.nrec() && recs_match(xdf, g.start, g.end, flags) {
        xdf.rchg.set(g.start, false);
        g.start += 1;
        xdf.rchg.set(g.end, true);
        g.end += 1;
        while xdf.rchg.get(g.end) {
            g.end += 1;
        }
        true
    } else {
        false
    }
}

/// Slide `g` one line toward the start of the file. False when it cannot.
fn group_slide_up(xdf: &mut XdFile<'_>, g: &mut Group, flags: u64) -> bool {
    if g.start > 0 && recs_match(xdf, g.start - 1, g.end - 1, flags) {
        g.start -= 1;
        xdf.rchg.set(g.start, true);
        g.end -= 1;
        xdf.rchg.set(g.end, false);
        while xdf.rchg.get(g.start - 1) {
            g.start -= 1;
        }
        true
    } else {
        false
    }
}

/// The two files' group walks have gone out of step, which they cannot: they
/// are stepped together and every slide is matched. Upstream prints and
/// exits rather than carrying on with a corrupt script; so do we.
fn bug(msg: &str) -> ! {
    eprintln!("BUG: {msg}");
    std::process::exit(1);
}

/// Amount of indentation on this line, with TAB at 8 columns; -1 if the line
/// is empty or nothing but whitespace, clamped at [`MAX_INDENT`].
fn line_indent(line: &[u8]) -> i32 {
    let mut ret = 0i32;
    for &c in line {
        if !is_space(c) {
            return ret;
        } else if c == b' ' {
            ret += 1;
        } else if c == b'\t' {
            ret += 8 - ret % 8;
        }
        // Other whitespace characters are ignored.
        if ret >= MAX_INDENT {
            return MAX_INDENT;
        }
    }
    -1
}

/// What a hypothetical split above line `at` looks like.
struct SplitMeasurement {
    /// Is the split at the end of the file, blank lines aside?
    end_of_file: bool,
    /// Indent of the line immediately after the split, or -1 if it is blank.
    indent: i32,
    /// How many consecutive lines above the split are blank.
    pre_blank: i32,
    /// Indent of the nearest non-blank line above, or -1 if there is none.
    pre_indent: i32,
    /// How many lines after the line following the split are blank.
    post_blank: i32,
    /// Indent of the nearest non-blank line after that, or -1.
    post_indent: i32,
}

/// A split's badness. Smaller is better on both counts.
#[derive(Clone, Copy)]
struct SplitScore {
    effective_indent: i32,
    penalty: i32,
}

fn measure_split(xdf: &XdFile<'_>, at: i64) -> SplitMeasurement {
    let mut m = SplitMeasurement {
        end_of_file: at >= xdf.nrec(),
        indent: if at >= xdf.nrec() {
            -1
        } else {
            line_indent(xdf.line(at))
        },
        pre_blank: 0,
        pre_indent: -1,
        post_blank: 0,
        post_indent: -1,
    };

    let mut i = at - 1;
    while i >= 0 {
        m.pre_indent = line_indent(xdf.line(i));
        if m.pre_indent != -1 {
            break;
        }
        m.pre_blank += 1;
        if m.pre_blank == MAX_BLANKS {
            m.pre_indent = 0;
            break;
        }
        i -= 1;
    }

    let mut i = at + 1;
    while i < xdf.nrec() {
        m.post_indent = line_indent(xdf.line(i));
        if m.post_indent != -1 {
            break;
        }
        m.post_blank += 1;
        if m.post_blank == MAX_BLANKS {
            m.post_indent = 0;
            break;
        }
        i += 1;
    }

    m
}

/// Penalty if there are no non-blank lines before the split.
const START_OF_FILE_PENALTY: i32 = 1;
/// Penalty if there are no non-blank lines after the split.
const END_OF_FILE_PENALTY: i32 = 21;
/// Multiplier for the number of blank lines around the split.
const TOTAL_BLANK_WEIGHT: i32 = -30;
/// Multiplier for the number of blank lines after the split.
const POST_BLANK_WEIGHT: i32 = 6;
/// Penalty if the line is indented more than its predecessor.
const RELATIVE_INDENT_PENALTY: i32 = -4;
/// The same, with blank lines nearby.
const RELATIVE_INDENT_WITH_BLANK_PENALTY: i32 = 10;
/// Penalty if the line is indented less than both neighbours.
const RELATIVE_OUTDENT_PENALTY: i32 = 24;
/// The same, with blank lines nearby.
const RELATIVE_OUTDENT_WITH_BLANK_PENALTY: i32 = 17;
/// Penalty if the line is indented less than its predecessor but not less
/// than its successor.
const RELATIVE_DEDENT_PENALTY: i32 = 23;
/// The same, with blank lines nearby.
const RELATIVE_DEDENT_WITH_BLANK_PENALTY: i32 = 17;
/// How much the sign of the effective-indent difference weighs against the
/// accumulated penalty in [`score_cmp`].
const INDENT_WEIGHT: i32 = 60;

/// Add `m`'s badness to `s`.
///
/// The weights were determined empirically against the corpus at
/// <https://github.com/mhagger/diff-slider-tools>; only their ratios matter,
/// since scores are never compared to anything but each other.
fn score_add_split(m: &SplitMeasurement, s: &mut SplitScore) {
    if m.pre_indent == -1 && m.pre_blank == 0 {
        s.penalty += START_OF_FILE_PENALTY;
    }
    if m.end_of_file {
        s.penalty += END_OF_FILE_PENALTY;
    }

    // Blank lines following the split, the line right after it included.
    let post_blank = if m.indent == -1 { 1 + m.post_blank } else { 0 };
    let total_blank = m.pre_blank + post_blank;
    s.penalty += TOTAL_BLANK_WEIGHT * total_blank;
    s.penalty += POST_BLANK_WEIGHT * post_blank;

    let indent = if m.indent != -1 {
        m.indent
    } else {
        m.post_indent
    };
    let any_blanks = total_blank != 0;

    // Note that the effective indent is -1 at the end of the file.
    s.effective_indent += indent;

    if indent == -1 || m.pre_indent == -1 || indent == m.pre_indent {
        // No additional adjustment.
    } else if indent > m.pre_indent {
        s.penalty += if any_blanks {
            RELATIVE_INDENT_WITH_BLANK_PENALTY
        } else {
            RELATIVE_INDENT_PENALTY
        };
    } else if m.post_indent != -1 && m.post_indent > indent {
        // The following line is indented more, so this line is likely the
        // start of a block rather than the end of one.
        s.penalty += if any_blanks {
            RELATIVE_OUTDENT_WITH_BLANK_PENALTY
        } else {
            RELATIVE_OUTDENT_PENALTY
        };
    } else {
        // That was probably the end of a block.
        s.penalty += if any_blanks {
            RELATIVE_DEDENT_WITH_BLANK_PENALTY
        } else {
            RELATIVE_DEDENT_PENALTY
        };
    }
}

/// Negative if `s1` is the better split.
fn score_cmp(s1: &SplitScore, s2: &SplitScore) -> i32 {
    let cmp_indents = i32::from(s1.effective_indent > s2.effective_indent)
        - i32::from(s1.effective_indent < s2.effective_indent);
    INDENT_WEIGHT * cmp_indents + (s1.penalty - s2.penalty)
}

/// Slide every group of changed lines in `xdf` to the position that reads
/// best, keeping `xdfo`'s group walk in step.
///
/// Preference order: line up with a group in the other file if one is
/// reachable; otherwise, if `indent-heuristic` is on, take the
/// lowest-scoring shift; otherwise leave the group where the slide-down
/// loop left it.
pub fn change_compact(xdf: &mut XdFile<'_>, xdfo: &XdFile<'_>, flags: u64) {
    let mut g = group_init(xdf);
    let mut go = group_init(xdfo);

    loop {
        if g.end != g.start {
            let mut groupsize;
            let mut earliest_end;
            let mut end_matching_other;
            loop {
                groupsize = g.end - g.start;
                // The last `end` at which this group lines up with a group
                // of changed lines in the other file; -1 for "none yet".
                end_matching_other = -1i64;

                while group_slide_up(xdf, &mut g, flags) {
                    if !group_previous(xdfo, &mut go) {
                        bug("group sync broken sliding up");
                    }
                }

                // This is the highest the group can be shifted.
                earliest_end = g.end;
                if go.end > go.start {
                    end_matching_other = g.end;
                }

                while group_slide_down(xdf, &mut g, flags) {
                    if !group_next(xdfo, &mut go) {
                        bug("group sync broken sliding down");
                    }
                    if go.end > go.start {
                        end_matching_other = g.end;
                    }
                }

                if groupsize == g.end - g.start {
                    break;
                }
            }

            // The group is now as far down as it goes, so everything below
            // only has to handle upward shifts.
            if g.end == earliest_end {
                // No shifting was possible.
            } else if end_matching_other != -1 {
                // Move the (possibly merged) group back to line up with the
                // last group of changes from the other file it can reach.
                while go.end == go.start {
                    if !group_slide_up(xdf, &mut g, flags) {
                        bug("match disappeared");
                    }
                    if !group_previous(xdfo, &mut go) {
                        bug("group sync broken sliding to match");
                    }
                }
            } else if flags & XDF_INDENT_HEURISTIC != 0 {
                let best_shift = best_indent_shift(xdf, &g, earliest_end, groupsize);
                while g.end > best_shift {
                    if !group_slide_up(xdf, &mut g, flags) {
                        bug("best shift unreached");
                    }
                    if !group_previous(xdfo, &mut go) {
                        bug("group sync broken sliding to blank line");
                    }
                }
            }
        }

        // Move past the just-processed group.
        if !group_next(xdf, &mut g) {
            break;
        }
        if !group_next(xdfo, &mut go) {
            bug("group sync broken moving to next group");
        }
    }

    if group_next(xdfo, &mut go) {
        bug("group sync broken at end of file");
    }
}

/// The `end` the indent heuristic would rather `g` sat at.
///
/// A group of pure adds or deletes implies two splits — one above it and one
/// below — so each candidate position is scored as the sum of the two, and
/// the lowest wins. Ties go to the *latest* shift, because the comparison is
/// `<=`.
fn best_indent_shift(xdf: &XdFile<'_>, g: &Group, earliest_end: i64, groupsize: i64) -> i64 {
    let mut shift = earliest_end;
    shift = shift.max(g.end - groupsize - 1);
    shift = shift.max(g.end - INDENT_HEURISTIC_MAX_SLIDING);

    let mut best_shift = -1i64;
    let mut best_score = SplitScore {
        effective_indent: 0,
        penalty: 0,
    };
    while shift <= g.end {
        let mut score = SplitScore {
            effective_indent: 0,
            penalty: 0,
        };
        score_add_split(&measure_split(xdf, shift), &mut score);
        score_add_split(&measure_split(xdf, shift - groupsize), &mut score);
        if best_shift == -1 || score_cmp(&score, &best_score) <= 0 {
            best_score = score;
            best_shift = shift;
        }
        shift += 1;
    }
    best_shift
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
pub fn diff(
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
