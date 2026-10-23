//! The `inline:char` and `inline:word` sub-diff.
//!
//! Under those two `'diffopt'` values a changed block is diffed *again*, one
//! character or one word per "line", and the resulting hunks become the column
//! ranges.  [`diff_find_change_inline_diff`] builds that sub-problem and runs
//! the internal engine over it; the two `refine_inline_*` functions merge
//! sub-blocks that are too close together to be worth showing apart, and
//! [`change_for`] maps the result back through the line map onto real columns.
//!
//! ## The line map
//!
//! xdiff takes newline-delimited text, so a changed block is rewritten with
//! one token -- a character, or a word under `inline:word` -- per line.  A
//! [`LineMap`] entry per token records where in the *real* buffer that token
//! came from, and is what turns an answer in token-line numbers back into
//! `(line offset, byte range)`.  Because the tokens are the diff's unit, the
//! `'diffopt'` ignore flags are applied while building them: `icase` folds
//! each character, `iwhite` glues a run of white space into one token,
//! `iwhiteall` drops white space entirely, and `iwhiteeol` rewinds the
//! trailing run at end of line by truncating both arrays.

#![forbid(unsafe_code)]

use super::*;
use crate::mbyte::{c_cluster_len, char_at, class_in, class_of, encode_char};
use crate::winlayer::BufId;
use core::ffi::c_int;

/// One entry per token written into the fake file, per buffer.
type LineMap = [Vec<LinemapEntry>; DB_COUNT as usize];

/// The character class `mb_get_class_tab` gives an alphanumeric character.
///
/// Only this class counts as "word" for `inline:word`: emoji and CJK
/// ideographs get their own classes, and word-diffing them is not useful
/// because nvim has no way to segment them.
const CLASS_WORD: c_int = 2;

/// Whether the two blocks either side of a gap should be merged into one.
///
/// Both `inline:` modes run the same walk over the sub-diff's block list;
/// they differ only in what makes a gap small enough to swallow, which is
/// what this decides.  `Merge` folds the next block into this one; `Keep` moves
/// on; the `bool` is "this gap was a candidate but was left alone", which
/// only `inline:char` uses.
enum Gap {
    Merge,
    Keep(bool),
}

/// Walk a sub-diff's block list, merging each pair `decide` says to merge.
///
/// Answers whether any pair was merged and whether any candidate was left
/// alone.  `entry_back` is how far back from the *next* block's first line the
/// mode's first line-map entry sits: `inline:char` compares the last token of
/// the left block, `inline:word` the one before it.
fn merge_gaps(
    blocks: &mut Vec<DiffBlock>,
    linemap: &LineMap,
    idx1: usize,
    entry_back: LineNr,
    mut decide: impl FnMut(&DiffBlock, &DiffBlock, &LinemapEntry, &LinemapEntry) -> Gap,
) -> (bool, bool) {
    let (mut merged, mut unmerged) = (false, false);
    let map = &linemap[idx1];
    let mut at = 0;
    while at + 1 < blocks.len() {
        let (dp, next) = (&blocks[at], &blocks[at + 1]);
        // Both indices are into the token file, so a block ending past the
        // map has no entry to compare and the gap cannot be judged.  The
        // bound upstream tests is the block's *last* token either way,
        // even where `entry_back` then steps one further back than that; a
        // negative index is upstream's own out-of-bounds read, refused
        // here rather than reproduced.
        let last = dp.end(idx1) - 1;
        let right = next.df_lnum[idx1] - 1;
        if last >= map.len() as LineNr || right >= map.len() as LineNr {
            at += 1;
            continue;
        }
        let left = last + 1 - entry_back;
        let (Some(entry1), Some(entry2)) = (
            map.get(usize::try_from(left).unwrap_or(usize::MAX)),
            map.get(usize::try_from(right).unwrap_or(usize::MAX)),
        ) else {
            at += 1;
            continue;
        };
        // Two tokens on different source lines are not a gap *within* a
        // line, which is the only thing either mode merges across.
        if entry1.lineoff != entry2.lineoff {
            at += 1;
            continue;
        }
        match decide(dp, next, entry1, entry2) {
            Gap::Merge => {
                let next = blocks.remove(at + 1);
                let dp = &mut blocks[at];
                for i in 0..DB_COUNT as usize {
                    dp.df_count[i] = next.end(i) - dp.df_lnum[i];
                }
                merged = true;
            }
            Gap::Keep(candidate) => {
                unmerged |= candidate;
                at += 1;
            }
        }
    }
    (merged, unmerged)
}

/// `inline:char`: swallow a gap of at most three tokens when the two blocks
/// around it are at least four times as long as the gap.
///
/// Repeated until nothing more merges, because merging two blocks can make
/// the next gap worth swallowing too -- but at most four passes, and only
/// while the last pass both merged something and left something alone.
fn refine_inline_char(blocks: &mut Vec<DiffBlock>, linemap: &LineMap, idx1: usize) {
    for _ in 0..4 {
        let (merged, unmerged) = merge_gaps(blocks, linemap, idx1, 1, |dp, next, _, _| {
            let gap = next.df_lnum[idx1] - dp.end(idx1);
            if gap > 3 {
                return Gap::Keep(false);
            }
            let longest = (0..DB_COUNT as usize)
                .map(|i| dp.df_count[i] + next.df_count[i])
                .max()
                .unwrap_or(0);
            if longest >= gap * 4 {
                Gap::Merge
            } else {
                Gap::Keep(true)
            }
        });
        if !unmerged || !merged {
            break;
        }
    }
}

/// `inline:word`: swallow a gap of non-word text shorter than
/// `'diffopt'`'s `inline-word-gap`, when the words either side of it are
/// together at least twice as long as the gap.
///
/// Always four passes: unlike `inline:char` there is no cheap test for
/// "nothing left to do", and merging can expose a new short gap.
fn refine_inline_word(
    blocks: &mut Vec<DiffBlock>,
    slots: &DiffSlots,
    linemap: &LineMap,
    idx1: usize,
    start_lnum: LineNr,
) {
    let buf = used_slot(slots, idx1);
    for _ in 0..4 {
        merge_gaps(blocks, linemap, idx1, 2, |dp, next, entry1, entry2| {
            let gap_start = entry1.byte_start + entry1.num_bytes;
            let gap_size = entry2.byte_start - gap_start;
            if gap_size <= 0 || gap_size > DIFF_WORD_GAP {
                return Gap::Keep(false);
            }
            // The gap is only worth swallowing if it is *punctuation*
            // between two changed words; a word in the gap is a real
            // unchanged word and splitting there is the point.
            let mut lines = buf.lines();
            let line = lines.line(start_lnum + entry1.lineoff as LineNr);
            let from = (gap_start as usize).min(line.len());
            let gap_len = (gap_size as usize).min(line.len() - from);
            // Each character is classed as the line holds it, so one that
            // runs past the gap's end still decodes whole.
            if gap_len == 0
                || (from..from + gap_len).any(|i| class_in(&line[i..], buf) == CLASS_WORD)
            {
                return Gap::Keep(false);
            }
            // How much text the two blocks either side of the gap cover:
            // a gap is only worth swallowing next to a substantial change.
            let mut changed: i64 = 0;
            for (i, map) in linemap.iter().enumerate() {
                if slots[i].is_none() {
                    continue;
                }
                for block in [dp, next] {
                    for k in 0..block.df_count[i] {
                        let at = block.df_lnum[i] + k - 1;
                        if let Ok(at) = usize::try_from(at)
                            && let Some(e) = map.get(at)
                        {
                            changed += e.num_bytes as i64;
                        }
                    }
                }
            }
            if changed >= (gap_size * 2) as i64 {
                Gap::Merge
            } else {
                Gap::Keep(false)
            }
        });
    }
}

/// Split one buffer line into tokens, appending them to `out` and their
/// provenance to `map`.
///
/// `off` is the line's offset within the diff block, and `chartab` the
/// *first* buffer's `'iskeyword'` table -- deliberately, so that all buffers
/// are segmented the same way.  `word` selects `inline:word`.
///
/// The tokens are NL-separated because that is what xdiff reads; a real NL in
/// the text (which cannot occur in a line, but `ml_get_buf` can answer one for
/// a NUL byte) is written as the NUL it stands for. Each character is read
/// from the line's tail as the C string it is the end of.
fn tokenize_line(
    bytes: &[u8],
    off: c_int,
    chartab: &[uint64_t; 4],
    word: bool,
    out: &mut Vec<u8>,
    map: &mut Vec<LinemapEntry>,
) {
    let flags = diff_flags.get();
    let trim_eol = flags & (DIFF_IWHITEEOL | DIFF_IWHITE) != 0;
    let mut in_keyword = false;
    // Where to rewind to if the line ends in white space: the state as of
    // the first byte of the trailing run.  `None` while the run has been
    // broken by a non-blank.
    let mut eol: Option<(usize, usize)> = None;
    let mut last_white = false;

    let mut i = 0;
    while i < bytes.len() {
        let rest = &bytes[i..];
        let new_in_keyword = word && class_of(rest[0], || char_at(rest), chartab) == CLASS_WORD;
        if in_keyword && !new_in_keyword {
            out.push(NL as u8);
        }

        if ascii_iswhite(bytes[i] as c_int) {
            if flags & DIFF_IWHITEALL != 0 {
                in_keyword = false;
                i = bytes.len() - skip_white(&bytes[i..]).len();
                continue;
            }
            if trim_eol && !last_white {
                eol = Some((out.len(), map.len()));
                last_white = true;
            }
        } else if trim_eol {
            last_white = false;
            eol = None;
        }

        let mut tok_len = 1;
        if bytes[i] == NL as u8 {
            // NL is the internal stand-in for NUL.
            out.push(0);
        } else {
            tok_len = c_cluster_len(rest) as c_int;
            if ascii_iswhite(bytes[i] as c_int) && flags & DIFF_IWHITE != 0 {
                // The whole run of white space is one token.
                tok_len = (bytes.len() - skip_white(&bytes[i..]).len() - i) as c_int;
            }
            if flags & DIFF_ICASE != 0 {
                // xdiff cannot ignore case, so fold the text instead.
                let c = char_at(rest);
                let c_len = utf_char2len(c);
                // MB_MAXBYTES + 1.
                let mut cbuf = [0u8; 22];
                let folded = encode_char(utf_fold(c), &mut cbuf);
                out.extend_from_slice(&cbuf[..folded]);
                if tok_len > c_len {
                    // Composing characters follow, and are not folded.
                    out.extend_from_slice(&bytes[i + c_len as usize..i + tok_len as usize]);
                }
            } else {
                out.extend_from_slice(&bytes[i..i + tok_len as usize]);
            }
        }

        if !new_in_keyword {
            out.push(NL as u8);
        }
        if !new_in_keyword || !in_keyword {
            map.push(LinemapEntry {
                byte_start: i as ColNr,
                num_bytes: tok_len,
                lineoff: off,
            });
        } else {
            // Still inside a keyword: grow the entry rather than add one.
            map.last_mut()
                .expect("a keyword has a first character")
                .num_bytes += tok_len;
        }
        in_keyword = new_in_keyword;
        i += tok_len as usize;
    }
    if in_keyword {
        out.push(NL as u8);
    }
    if let (true, Some((out_len, map_len))) = (trim_eol, eol) {
        out.truncate(out_len);
        map.truncate(map_len);
    }
    if flags & DIFF_IWHITEALL == 0 {
        // An empty token for the line ending, so that a difference in
        // newlines is visible -- with `'list'` the eol listchar takes the
        // highlight.
        out.push(NL as u8);
        map.push(LinemapEntry {
            byte_start: bytes.len() as ColNr,
            // Upstream writes `sizeof(NL)`, and `NL` is a *character
            // constant*, so this is 4 rather than 1.  Reproduced: the
            // width lands in `dc_end` and so in what gets highlighted.
            // See O-B15-16.
            num_bytes: ::core::mem::size_of::<c_int>() as ColNr,
            lineoff: off,
        });
    }
}

/// One sub-diff block's column range in every buffer, read back through the
/// line map.
///
/// A block whose token range starts past the end of a buffer's map is an
/// insertion as far as that buffer is concerned, which is spelled `MAXCOL`
/// and an `INT_MAX` line offset -- the marker `diff_find_change` reads to
/// decide a line is an addition.
fn change_for(new_diff: &DiffBlock, linemap: &LineMap) -> DiffLineChange {
    let mut change = DiffLineChange {
        dc_start: [0; 8],
        dc_end: [0; 8],
        dc_start_lnum_off: [0; 8],
        dc_end_lnum_off: [0; 8],
    };
    for (i, map) in linemap.iter().enumerate() {
        // Never negative; the test is for safety only.
        if new_diff.df_lnum[i] <= 0 {
            continue;
        }
        let start = (new_diff.df_lnum[i] - 1) as usize; // zero-indexed
        let end = start + new_diff.df_count[i] as usize;
        match map.get(start) {
            Some(e) => {
                change.dc_start[i] = e.byte_start;
                change.dc_start_lnum_off[i] = e.lineoff;
            }
            None => {
                change.dc_start[i] = MAXCOL as ColNr;
                change.dc_start_lnum_off[i] = c_int::MAX;
            }
        }
        if start == end {
            change.dc_end[i] = change.dc_start[i];
            change.dc_end_lnum_off[i] = change.dc_start_lnum_off[i];
        } else {
            match map.get(end - 1) {
                Some(e) => {
                    change.dc_end[i] = e.byte_start + e.num_bytes;
                    change.dc_end_lnum_off[i] = e.lineoff;
                }
                None => {
                    change.dc_end[i] = MAXCOL as ColNr;
                    change.dc_end_lnum_off[i] = c_int::MAX;
                }
            }
        }
    }
    change
}

/// Diff one changed block again, token by token, and cache the column ranges.
///
/// The result is `dp.df_changes`, which [`diff_find_change`] then windows
/// per line.  The sub-diff runs the ordinary `diff_file_internal`/
/// [`diff_read`] pair over a scratch block list of its own, with `slots` --
/// the tab page's buffer table -- copied so that a buffer with no text in
/// the block can be left out of it.
pub(crate) fn diff_find_change_inline_diff(dp: &mut DiffBlock, slots: &DiffSlots) {
    let save_diff_algorithm = diff_algorithm.get();
    // The inline diff only supports the internal algorithm; its two sides
    // are `file1`/`file2` below rather than the run's own images.
    let mut dio = DiffIo {
        internal: true,
        ..DiffIo::default()
    };
    // Always slide diff splits along whitespace.
    diff_algorithm.set(save_diff_algorithm | XDF_INDENT_HEURISTIC);

    // The table `diff_read` and the refinement read: which buffers take part.
    let mut slots = *slots;
    let mut blocks = Vec::new();
    let mut linemap: LineMap = ::core::array::from_fn(|_| Vec::new());
    let (mut file1, mut file2) = (Vec::<u8>::new(), Vec::<u8>::new());
    let mut file1_idx = usize::MAX;

    'done: {
        for (i, map) in linemap.iter_mut().enumerate() {
            dio.diff.hunks.clear();
            let Some(buf) = slots[i]
                .and_then(BufId::get)
                .filter(|b| b.b_ml.ml_mfp.is_some())
            else {
                continue; // not loaded
            };
            if dp.df_count[i] == 0 {
                // A buffer with no text in this block must not be left in
                // the table, or the whole block reads as modified in it.
                slots[i] = None;
                continue;
            }
            if file1_idx == usize::MAX {
                file1_idx = i;
            }
            let first = file1_idx == i;
            let out = if first { &mut file1 } else { &mut file2 };
            out.clear();
            // Deliberately the *first* buffer's 'iskeyword', so that
            // every buffer is segmented the same way.
            let chartab = used_slot(&slots, file1_idx).b_chartab;
            let mut lines = buf.lines();
            for off in 0..dp.df_count[i] {
                let line = lines.line(dp.df_lnum[i] + off);
                let word = diff_flags.get() & DIFF_INLINE_WORD != 0;
                tokenize_line(line, off, &chartab, word, out, map);
            }
            if first {
                continue;
            }
            if diff_file_internal(&file1, &file2, &mut dio.diff.hunks).is_err() {
                break 'done;
            }
            if let Some(hunks) = HunkSource::open(&dio) {
                diff_read(&mut blocks, &slots, 0, i, hunks);
            }
        }

        if file1_idx != usize::MAX {
            if diff_flags.get() & DIFF_INLINE_WORD != 0 {
                let start = dp.df_lnum[file1_idx];
                refine_inline_word(&mut blocks, &slots, &linemap, file1_idx, start);
            } else if diff_flags.get() & DIFF_INLINE_CHAR != 0 {
                refine_inline_char(&mut blocks, &linemap, file1_idx);
            }
        }

        dp.df_changes.clear(); // already empty
        dp.df_changes
            .extend(blocks.iter().map(|new_diff| change_for(new_diff, &linemap)));
    }

    diff_algorithm.set(save_diff_algorithm);
    dp.has_changes = true;
}
