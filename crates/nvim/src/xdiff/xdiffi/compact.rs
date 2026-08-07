//! `xdiffi.c`'s second half: hunk compaction and the indent heuristic.
//!
//! None of this is about correctness. A run of changed lines whose first
//! line equals the line following it can be slid down, and one whose last
//! line equals the line preceding it can be slid up, without changing the
//! edit script's size — so where the run *sits* is free, and
//! [`change_compact`] spends that freedom on making the diff read better.
//!
//! Preference order, per group: line up with a group in the other file if
//! one is reachable at all; otherwise, under `'diffopt'`'s
//! `indent-heuristic`, take the shift whose two split positions score
//! lowest; otherwise leave it where the slide-down loop left it.

#![forbid(unsafe_code)]

use crate::src::xdiff::ffi::is_space;
use crate::src::xdiff::xtypes::{XDF_INDENT_HEURISTIC, XdFile};
use crate::src::xdiff::xutils::recmatch;

/// If a line is indented more than this, [`line_indent`] just answers this.
/// Keeps the work bounded on data that is not human-readable text.
const MAX_INDENT: i32 = 200;
/// The same idea for runs of blank lines around a candidate split.
const MAX_BLANKS: i32 = 20;
/// How far a hunk may be slid in search of a better-looking split.
const INDENT_HEURISTIC_MAX_SLIDING: i64 = 100;

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
