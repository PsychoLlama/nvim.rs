//! The `linematch` diff refinement: given one diff block whose lines
//! differ across two or more buffers, choose which of those lines to
//! align against each other so the rendered diff reads well.
//!
//! The algorithm walks an n-dimensional tensor whose axes are the buffers
//! and whose extent along each axis is that buffer's line count in the
//! block, plus one. Moving from one cell to a neighbour is a *decision*:
//! a bitmask naming the buffers whose next line is consumed by this step.
//! Each decision scores the characters the consumed lines have in common,
//! and the best path from the origin to the far corner is the alignment.
//! Among equally scored paths the one with the fewest changes of decision
//! wins, so runs of the same decision stay contiguous.
//!
//! For questions about the algorithm itself please contact its author,
//! Jonathon White (jonathonwhite@protonmail.com).
//!
//! # Boundary
//!
//! Nothing here touches C. Callers hand in the block text of each buffer
//! as a byte slice — the `mmfile_t`s the diff machinery passes around
//! convert at the call site — and get the decisions back as a `Vec`.

#![forbid(unsafe_code)]

use crate::types::linenr_T;
use core::ffi::c_int;

/// Buffers a diff block can span (`DB_COUNT`).
const LN_MAX_BUFS: usize = 8;
/// Distinct decisions: one per non-empty subset of the buffers, so
/// `2^LN_MAX_BUFS - 1`. Also the longest path recorded per tensor cell.
const LN_DECISION_MAX: usize = 255;
/// Only this many leading bytes of a line take part in character
/// matching, capping the quadratic comparison below.
const MATCH_CHAR_MAX_LEN: usize = 800;

/// One cell of the tensor: the best score reached here, and every
/// equally-scored way of reaching it.
#[derive(Clone)]
struct PathNode {
    /// Total score of the paths recorded below.
    score: c_int,
    /// How many entries of `choice`/`predecessor` are populated.
    path_n: usize,
    /// Memoized [`Tensor::min_turns`] result, indexed by the decision
    /// taken *out* of this cell. `-1` means "not computed yet".
    choice_mem: [c_int; LN_DECISION_MAX + 1],
    /// The decision taken to arrive here along each recorded path.
    choice: [c_int; LN_DECISION_MAX],
    /// The cell each recorded path came from.
    predecessor: [usize; LN_DECISION_MAX],
    /// Index into `choice`/`predecessor` of the path that reaches the end
    /// with the fewest changes of decision.
    optimal_choice: usize,
}

impl PathNode {
    const fn new() -> Self {
        Self {
            score: 0,
            path_n: 0,
            choice_mem: [-1; LN_DECISION_MAX + 1],
            choice: [0; LN_DECISION_MAX],
            predecessor: [0; LN_DECISION_MAX],
            optimal_choice: 0,
        }
    }
}

/// Length of the first line of `block`, excluding its newline.
fn line_len(block: &[u8]) -> usize {
    block
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(block.len())
}

/// The first line of `block`, truncated to what the matcher looks at.
fn leading_line(block: &[u8]) -> &[u8] {
    &block[..line_len(block).min(MATCH_CHAR_MAX_LEN - 1)]
}

/// Characters `a` and `b` have in common, in order — the length of their
/// longest common subsequence. Only the first line of each is compared.
///
/// Examples:
///   `matching_chars("aabc", "acba")`               -> 2, 'a' and 'b'
///   `matching_chars("123hello567", "he123ll567o")` -> 8, '123', 'll', '567'
///   `matching_chars("abcdefg", "gfedcba")`         -> 1, every character
///                                                     is common but no
///                                                     two are in order
fn matching_chars(a: &[u8], b: &[u8]) -> c_int {
    let s1 = leading_line(a);
    let s2 = leading_line(b);
    // Only two rows of the table are ever live, so they alternate.
    let mut buf = ([0 as c_int; MATCH_CHAR_MAX_LEN], [0; MATCH_CHAR_MAX_LEN]);
    let mut prev: &mut [c_int] = &mut buf.0;
    let mut cur: &mut [c_int] = &mut buf.1;
    for &c1 in s1 {
        for (j, &c2) in s2.iter().enumerate() {
            // Skip a character in either string, or consume one from
            // both when they agree.
            let mut best = prev[j + 1].max(cur[j]);
            if c1 == c2 {
                best = best.max(prev[j] + 1);
            }
            cur[j + 1] = best;
        }
        core::mem::swap(&mut prev, &mut cur);
    }
    prev[s2.len()]
}

/// [`matching_chars`] with spaces and tabs squeezed out first.
///
/// The newline terminating each line is copied along with the text, so
/// the compacted lines still look like lines to [`matching_chars`]. A
/// final line with no newline contributes nothing in its place: upstream
/// read one byte past the end of the block there, which this cannot and
/// does not reproduce.
fn matching_chars_iwhite(a: &[u8], b: &[u8]) -> c_int {
    let mut buf = [[0u8; MATCH_CHAR_MAX_LEN]; 2];
    let mut lens = [0usize; 2];
    for (k, block) in [a, b].into_iter().enumerate() {
        let take = (line_len(block).min(MATCH_CHAR_MAX_LEN - 1) + 1).min(block.len());
        for &byte in &block[..take] {
            if byte != b' ' && byte != b'\t' {
                buf[k][lens[k]] = byte;
                lens[k] += 1;
            }
        }
    }
    let (first, second) = buf.split_at(1);
    matching_chars(&first[0][..lens[0]], &second[0][..lens[1]])
}

/// Score one decision: the characters shared by every pair of lines it
/// consumes. `lines` holds `None` for the buffers it skips.
///
/// A match across three or more buffers is worth the same as a match
/// across two, so a wide decision doesn't beat a narrow one on breadth
/// alone.
fn count_matched_chars(lines: &[Option<&[u8]>], iwhite: bool) -> c_int {
    let mut matched_chars = 0;
    let mut matched = 0;
    for (i, first) in lines.iter().enumerate() {
        for second in &lines[i + 1..] {
            if let (Some(a), Some(b)) = (first, second) {
                matched += 1;
                matched_chars += if iwhite {
                    matching_chars_iwhite(a, b)
                } else {
                    matching_chars(a, b)
                };
            }
        }
    }
    if matched >= 2 {
        matched_chars = matched_chars * 2 / matched;
    }
    matched_chars
}

/// The block text from line `lnum` (1-based) onward, or `None` when the
/// block ends before that line starts.
pub fn block_from_lnum(block: &[u8], lnum: linenr_T) -> Option<&[u8]> {
    let mut rest = block;
    for _ in 1..lnum {
        match rest.iter().position(|&b| b == b'\n') {
            Some(end) => rest = &rest[end + 1..],
            None => return None,
        }
    }
    Some(rest)
}

/// Flatten a per-buffer line index into an index of the flattened tensor.
fn unwrap_indexes(values: &[c_int], diff_len: &[c_int]) -> usize {
    let mut stride: usize = diff_len.iter().map(|&len| len as usize + 1).product();
    let mut idx = 0;
    for (k, &n) in values.iter().enumerate() {
        stride /= diff_len[k] as usize + 1;
        idx += stride * n as usize;
    }
    idx
}

/// The flattened tensor, plus the inputs every step of the walk needs.
struct Tensor<'a> {
    nodes: Vec<PathNode>,
    /// Block text of each buffer, one entry per axis.
    blocks: &'a [&'a [u8]],
    /// Lines each buffer contributes, i.e. the extent of each axis less one.
    diff_len: &'a [c_int],
    iwhite: bool,
}

impl Tensor<'_> {
    fn ndiffs(&self) -> usize {
        self.diff_len.len()
    }

    /// Fill every cell along axis `dim` and everything below it, then
    /// score the decisions that lead into the cell `df_iters` names.
    fn populate(&mut self, df_iters: &mut [c_int], dim: usize) {
        if dim < self.ndiffs() {
            for i in 0..=self.diff_len[dim] {
                df_iters[dim] = i;
                self.populate(df_iters, dim + 1);
            }
            return;
        }
        // A buffer can only be consumed if it still has a line here.
        let mut paths = [0usize; LN_MAX_BUFS];
        let mut npaths = 0;
        for (j, &iter) in df_iters.iter().enumerate() {
            if iter > 0 {
                paths[npaths] = j;
                npaths += 1;
            }
        }
        let to = unwrap_indexes(df_iters, self.diff_len);
        self.nodes[to].score = -1;
        self.try_possible_paths(df_iters, &paths[..npaths], 0, &mut 0);
    }

    /// Walk every subset of `paths` — every decision that could lead into
    /// the cell `df_iters` names — and score each one.
    fn try_possible_paths(
        &mut self,
        df_iters: &[c_int],
        paths: &[usize],
        path_idx: usize,
        choice: &mut c_int,
    ) {
        let Some(&bit) = paths.get(path_idx) else {
            if *choice > 0 {
                self.record_path(df_iters, *choice);
            }
            return;
        };
        *choice |= 1 << bit;
        self.try_possible_paths(df_iters, paths, path_idx + 1, choice);
        *choice &= !(1 << bit);
        self.try_possible_paths(df_iters, paths, path_idx + 1, choice);
    }

    /// Score `choice` as a way into the cell `df_iters` names, and keep it
    /// if it ties or beats what is already recorded there.
    fn record_path(&mut self, df_iters: &[c_int], choice: c_int) {
        let (blocks, diff_len) = (self.blocks, self.diff_len);
        let mut from_vals = [0 as c_int; LN_MAX_BUFS];
        let mut lines = [None; LN_MAX_BUFS];
        for (k, &iter) in df_iters.iter().enumerate() {
            from_vals[k] = iter;
            if choice & (1 << k) != 0 {
                from_vals[k] -= 1;
                lines[k] = block_from_lnum(blocks[k], iter);
            }
        }
        let n = df_iters.len();
        let from = unwrap_indexes(&from_vals[..n], diff_len);
        let to = unwrap_indexes(df_iters, diff_len);
        let score = self.nodes[from].score + count_matched_chars(&lines[..n], self.iwhite);
        let node = &mut self.nodes[to];
        if score > node.score {
            node.path_n = 1;
            node.score = score;
            node.choice[0] = choice;
            node.predecessor[0] = from;
        } else if score == node.score {
            let k = node.path_n;
            node.path_n += 1;
            node.choice[k] = choice;
            node.predecessor[k] = from;
        }
    }

    /// Fewest changes of decision on any path from `idx` back to the
    /// origin, given that the step *out* of `idx` was `last_decision`.
    /// Records the winning path in the cell's `optimal_choice`.
    fn min_turns(&mut self, idx: usize, last_decision: c_int) -> usize {
        let memo = self.nodes[idx].choice_mem[last_decision as usize];
        if memo >= 0 {
            return memo as usize;
        }
        let mut minimum = 0;
        for i in 0..self.nodes[idx].path_n {
            let (choice, from) = (self.nodes[idx].choice[i], self.nodes[idx].predecessor[i]);
            let turns = self.min_turns(from, choice) + usize::from(last_decision != choice);
            if i == 0 || turns < minimum {
                self.nodes[idx].optimal_choice = i;
                minimum = turns;
            }
        }
        self.nodes[idx].choice_mem[last_decision as usize] = minimum as c_int;
        minimum
    }
}

/// Align the lines of one diff block across `diff_blk.len()` buffers.
///
/// `diff_blk` is each buffer's block text, `diff_len` the number of lines
/// each contributes. The result is one decision per output line, in
/// order: a bitmask of the buffers whose next line that output line
/// shows.
pub fn linematch_nbuffers(diff_blk: &[&[u8]], diff_len: &[c_int], iwhite: bool) -> Vec<c_int> {
    let ndiffs = diff_len.len();
    debug_assert!(ndiffs <= LN_MAX_BUFS);
    assert_eq!(diff_blk.len(), ndiffs);

    let mut memsize = 1usize;
    let mut max_decisions = 0usize;
    for &len in diff_len {
        debug_assert!(len >= 0);
        memsize *= len as usize + 1;
        max_decisions += len as usize;
    }

    let mut tensor = Tensor {
        nodes: vec![PathNode::new(); memsize],
        blocks: diff_blk,
        diff_len,
        iwhite,
    };
    let mut df_iters = [0 as c_int; LN_MAX_BUFS];
    tensor.populate(&mut df_iters[..ndiffs], 0);

    // Walk back from the far corner along the cheapest of the best paths.
    let mut node = unwrap_indexes(diff_len, diff_len);
    tensor.min_turns(node, 0);
    let mut decisions = Vec::with_capacity(max_decisions);
    while tensor.nodes[node].path_n > 0 {
        let j = tensor.nodes[node].optimal_choice;
        decisions.push(tensor.nodes[node].choice[j]);
        node = tensor.nodes[node].predecessor[j];
    }
    decisions.reverse();
    decisions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_len_stops_at_the_newline() {
        assert_eq!(line_len(b"abc\ndef\n"), 3);
        assert_eq!(line_len(b"\n"), 0);
        assert_eq!(line_len(b"no newline"), 10);
        assert_eq!(line_len(b""), 0);
    }

    #[test]
    fn matching_chars_counts_the_longest_common_subsequence() {
        // The examples from the algorithm's own documentation.
        assert_eq!(matching_chars(b"aabc\n", b"acba\n"), 2);
        assert_eq!(matching_chars(b"123hello567\n", b"he123ll567o\n"), 8);
        assert_eq!(matching_chars(b"abcdefg\n", b"gfedcba\n"), 1);
    }

    #[test]
    fn matching_chars_looks_at_the_first_line_only() {
        assert_eq!(matching_chars(b"ab\nxyz\n", b"ab\nxyz\n"), 2);
        assert_eq!(matching_chars(b"", b"anything\n"), 0);
    }

    #[test]
    fn matching_chars_truncates_long_lines() {
        // A character past the cap cannot match, one just inside it can.
        let filler = |n| {
            let mut line = vec![b'a'; n];
            line.extend_from_slice(b"z\n");
            line
        };
        assert_eq!(matching_chars(&filler(MATCH_CHAR_MAX_LEN - 1), b"z\n"), 0);
        assert_eq!(matching_chars(&filler(MATCH_CHAR_MAX_LEN - 2), b"z\n"), 1);
    }

    #[test]
    fn matching_chars_iwhite_ignores_spaces_and_tabs() {
        // Without it the shared indent alone scores a match.
        assert_eq!(matching_chars(b" a\n", b" b\n"), 1);
        assert_eq!(matching_chars_iwhite(b" a\n", b" b\n"), 0);
        // The newline travels with the text, so it is not compared.
        assert_eq!(matching_chars_iwhite(b"  \n", b"\t\n"), 0);
    }

    #[test]
    fn block_from_lnum_walks_lines() {
        let block = b"one\ntwo\nthree\n";
        assert_eq!(block_from_lnum(block, 1), Some(&block[..]));
        assert_eq!(block_from_lnum(block, 3), Some(&b"three\n"[..]));
        // The trailing newline leaves an empty — but present — remainder.
        assert_eq!(block_from_lnum(block, 4), Some(&b""[..]));
        assert_eq!(block_from_lnum(block, 5), None);
        assert_eq!(block_from_lnum(b"", 2), None);
    }

    #[test]
    fn unwrap_indexes_flattens_row_major() {
        assert_eq!(unwrap_indexes(&[0, 0], &[2, 3]), 0);
        assert_eq!(unwrap_indexes(&[0, 1], &[2, 3]), 1);
        assert_eq!(unwrap_indexes(&[1, 0], &[2, 3]), 4);
        assert_eq!(unwrap_indexes(&[2, 3], &[2, 3]), 11);
    }
}
