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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::TabPage;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

/// One entry per token written into the fake file, per buffer.
type LineMap = [Vec<linemap_entry_T>; DB_COUNT as usize];

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
/// what this decides.  `Merge` unlinks `dp->df_next` into `dp`; `Keep` moves
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
unsafe fn merge_gaps(
    dp_orig: *mut diff_T,
    linemap: &LineMap,
    idx1: usize,
    entry_back: linenr_T,
    mut decide: impl FnMut(*mut diff_T, &linemap_entry_T, &linemap_entry_T) -> Gap,
) -> (bool, bool) {
    let (mut merged, mut unmerged) = (false, false);
    let map = &linemap[idx1];
    let mut dp = dp_orig;
    while !dp.is_null() && !unsafe { (*dp).df_next }.is_null() {
        let next = unsafe { (*dp).df_next };
        // Both indices are into the token file, so a block ending past the
        // map has no entry to compare and the gap cannot be judged.  The
        // bound upstream tests is the block's *last* token either way,
        // even where `entry_back` then steps one further back than that; a
        // negative index is upstream's own out-of-bounds read, refused
        // here rather than reproduced.
        let last = unsafe { (*dp).df_lnum[idx1] } + unsafe { (*dp).df_count[idx1] } - 1;
        let right = unsafe { (*next).df_lnum[idx1] } - 1;
        if last >= map.len() as linenr_T || right >= map.len() as linenr_T {
            dp = next;
            continue;
        }
        let left = last + 1 - entry_back;
        let (Some(entry1), Some(entry2)) = (
            map.get(usize::try_from(left).unwrap_or(usize::MAX)),
            map.get(usize::try_from(right).unwrap_or(usize::MAX)),
        ) else {
            dp = next;
            continue;
        };
        // Two tokens on different source lines are not a gap *within* a
        // line, which is the only thing either mode merges across.
        if entry1.lineoff != entry2.lineoff {
            dp = next;
            continue;
        }
        match decide(dp, entry1, entry2) {
            Gap::Merge => {
                for i in 0..DB_COUNT as usize {
                    unsafe {
                        (*dp).df_count[i] =
                            (*next).df_lnum[i] + (*next).df_count[i] - (*dp).df_lnum[i]
                    };
                }
                unsafe { (*dp).df_next = (*next).df_next };
                unsafe { clear_diffblock(next) };
                merged = true;
            }
            Gap::Keep(candidate) => {
                unmerged |= candidate;
                dp = next;
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
unsafe fn refine_inline_char(dp_orig: *mut diff_T, linemap: &LineMap, idx1: usize) {
    for _ in 0..4 {
        let (merged, unmerged) = unsafe {
            merge_gaps(dp_orig, linemap, idx1, 1, |dp, _, _| {
                let next = (*dp).df_next;
                let gap = (*next).df_lnum[idx1] - ((*dp).df_lnum[idx1] + (*dp).df_count[idx1]);
                if gap > 3 {
                    return Gap::Keep(false);
                }
                let longest = (0..DB_COUNT as usize)
                    .map(|i| (*dp).df_count[i] + (*next).df_count[i])
                    .max()
                    .unwrap_or(0);
                if longest >= gap * 4 {
                    Gap::Merge
                } else {
                    Gap::Keep(true)
                }
            })
        };
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
unsafe fn refine_inline_word(
    dp_orig: *mut diff_T,
    linemap: &LineMap,
    idx1: usize,
    start_lnum: linenr_T,
) {
    let buf = unsafe { (*curtab.get()).tp_diffbuf[idx1] };
    for _ in 0..4 {
        unsafe {
            merge_gaps(dp_orig, linemap, idx1, 2, |dp, entry1, entry2| {
                let gap_start = entry1.byte_start + entry1.num_bytes;
                let gap_size = entry2.byte_start - gap_start;
                if gap_size <= 0 || gap_size > DIFF_WORD_GAP {
                    return Gap::Keep(false);
                }
                // The gap is only worth swallowing if it is *punctuation*
                // between two changed words; a word in the gap is a real
                // unchanged word and splitting there is the point.
                let line = CStr::from_ptr(ml_get_buf(buf, start_lnum + entry1.lineoff as linenr_T))
                    .to_bytes();
                let gap = &line[(gap_start as usize).min(line.len())..];
                let gap = &gap[..(gap_size as usize).min(gap.len())];
                if gap.is_empty()
                    || gap.iter().enumerate().any(|(i, _)| {
                        mb_get_class_tab(gap.as_ptr().add(i).cast(), (*buf).b_chartab.as_ptr())
                            == CLASS_WORD
                    })
                {
                    return Gap::Keep(false);
                }
                // How much text the two blocks either side of the gap cover:
                // a gap is only worth swallowing next to a substantial change.
                let next = (*dp).df_next;
                let mut changed: i64 = 0;
                for (i, map) in linemap.iter().enumerate() {
                    if (*curtab.get()).tp_diffbuf[i].is_null() {
                        continue;
                    }
                    for block in [dp, next] {
                        for k in 0..(*block).df_count[i] {
                            let at = (*block).df_lnum[i] + k - 1;
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
            })
        };
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
/// a NUL byte) is written as the NUL it stands for.
unsafe fn tokenize_line(
    line: &CStr,
    off: c_int,
    chartab: *const uint64_t,
    word: bool,
    out: &mut Vec<u8>,
    map: &mut Vec<linemap_entry_T>,
) {
    let flags = diff_flags.get();
    let trim_eol = flags & (DIFF_IWHITEEOL | DIFF_IWHITE) != 0;
    let bytes = line.to_bytes();
    let mut in_keyword = false;
    // Where to rewind to if the line ends in white space: the state as of
    // the first byte of the trailing run.  `None` while the run has been
    // broken by a non-blank.
    let mut eol: Option<(usize, usize)> = None;
    let mut last_white = false;

    let mut i = 0;
    while i < bytes.len() {
        let at = unsafe { bytes.as_ptr().add(i) }.cast::<c_char>();
        let new_in_keyword = word && unsafe { mb_get_class_tab(at, chartab) } == CLASS_WORD;
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
            tok_len = unsafe { utfc_ptr2len(at) };
            if ascii_iswhite(bytes[i] as c_int) && flags & DIFF_IWHITE != 0 {
                // The whole run of white space is one token.
                tok_len = (bytes.len() - skip_white(&bytes[i..]).len() - i) as c_int;
            }
            if flags & DIFF_ICASE != 0 {
                // xdiff cannot ignore case, so fold the text instead.
                let c = unsafe { utf_ptr2char(at) };
                let c_len = utf_char2len(c);
                // MB_MAXBYTES + 1.
                let mut cbuf = [0u8; 22];
                let folded =
                    unsafe { utf_char2bytes(utf_fold(c), cbuf.as_mut_ptr().cast::<c_char>()) };
                out.extend_from_slice(&cbuf[..folded as usize]);
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
            map.push(linemap_entry_T {
                byte_start: i as colnr_T,
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
        map.push(linemap_entry_T {
            byte_start: bytes.len() as colnr_T,
            // Upstream writes `sizeof(NL)`, and `NL` is a *character
            // constant*, so this is 4 rather than 1.  Reproduced: the
            // width lands in `dc_end` and so in what gets highlighted.
            // See O-B15-16.
            num_bytes: ::core::mem::size_of::<c_int>() as colnr_T,
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
fn change_for(new_diff: &diff_T, linemap: &LineMap) -> diffline_change_T {
    let mut change = diffline_change_T {
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
                change.dc_start[i] = MAXCOL as colnr_T;
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
                    change.dc_end[i] = MAXCOL as colnr_T;
                    change.dc_end_lnum_off[i] = c_int::MAX;
                }
            }
        }
    }
    change
}

/// Diff one changed block again, token by token, and cache the column ranges.
///
/// The result is `dp->df_changes`, which [`diff_find_change`] then windows
/// per line.  The whole thing runs *inside* the tabpage's diff state: the sub
/// diff is built by pointing `tp_first_diff` at a fresh list and calling the
/// ordinary `diff_file_internal`/`diff_read` pair, so the real block list and
/// buffer table are saved and put back at the end.
pub(crate) unsafe fn diff_find_change_inline_diff(dp: *mut diff_T) {
    let save_diff_algorithm = diff_algorithm.get();
    let mut dio = diffio_T {
        dio_orig: DIFFIN_INIT,
        dio_new: DIFFIN_INIT,
        dio_diff: diffout_T {
            dout_fname: ::core::ptr::null_mut(),
            dout_ga: GA_EMPTY_INIT_VALUE,
        },
        // The inline diff only supports the internal algorithm.
        dio_internal: 1,
    };
    unsafe {
        ga_init(
            &raw mut dio.dio_diff.dout_ga,
            ::core::mem::size_of::<diffhunk_T>() as c_int,
            1000,
        )
    };
    // Always slide diff splits along whitespace.
    diff_algorithm.set(save_diff_algorithm | XDF_INDENT_HEURISTIC);

    // `diff_read` reads both of these: the list it appends to, and the
    // table that says which buffers are active.
    // SAFETY: `curtab` is set from startup to exit.
    let mut tp = unsafe { TabPage::current() };
    let orig_diff = tp.tp_first_diff;
    let orig_diffbuf = tp.tp_diffbuf;
    tp.tp_first_diff = ::core::ptr::null_mut();

    let mut linemap: LineMap = ::core::array::from_fn(|_| Vec::new());
    let (mut file1, mut file2) = (Vec::<u8>::new(), Vec::<u8>::new());
    let mut file1_idx = usize::MAX;

    'done: {
        for (i, map) in linemap.iter_mut().enumerate() {
            dio.dio_diff.dout_ga.ga_len = 0;
            let buf = tp.tp_diffbuf[i];
            if buf.is_null() || unsafe { (*buf).b_ml.ml_mfp }.is_null() {
                continue; // not loaded
            }
            if unsafe { (*dp).df_count[i] } == 0 {
                // A buffer with no text in this block must not be left in
                // the table, or the whole block reads as modified in it.
                tp.tp_diffbuf[i] = ::core::ptr::null_mut();
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
            let chartab = unsafe { (*tp.tp_diffbuf[file1_idx]).b_chartab.as_ptr() };
            for off in 0..unsafe { (*dp).df_count[i] } {
                let line = unsafe { CStr::from_ptr(ml_get_buf(buf, (*dp).df_lnum[i] + off)) };
                unsafe {
                    tokenize_line(
                        line,
                        off,
                        chartab,
                        diff_flags.get() & DIFF_INLINE_WORD != 0,
                        out,
                        map,
                    )
                };
            }
            if first {
                continue;
            }
            dio.dio_orig.din_mmfile = mmfile_t {
                ptr: file1.as_mut_ptr().cast(),
                size: file1.len() as c_int,
            };
            dio.dio_new.din_mmfile = mmfile_t {
                ptr: file2.as_mut_ptr().cast(),
                size: file2.len() as c_int,
            };
            if unsafe { diff_file_internal(&raw mut dio) }.is_err() {
                break 'done;
            }
            unsafe { diff_read(0, i as c_int, &raw mut dio) };
            unsafe { clear_diffout(&raw mut dio.dio_diff) };
        }

        let head = tp.tp_first_diff;
        if file1_idx != usize::MAX {
            if diff_flags.get() & DIFF_INLINE_WORD != 0 {
                unsafe { refine_inline_word(head, &linemap, file1_idx, (*dp).df_lnum[file1_idx]) };
            } else if diff_flags.get() & DIFF_INLINE_CHAR != 0 {
                unsafe { refine_inline_char(head, &linemap, file1_idx) };
            }
        }

        unsafe { (*dp).df_changes.ga_len = 0 }; // already zero
        let mut new_diff = head;
        while !new_diff.is_null() {
            let change = change_for(unsafe { &*new_diff }, &linemap);
            unsafe { ga_grow(&raw mut (*dp).df_changes, 1) };
            unsafe {
                *((*dp).df_changes.ga_data as *mut diffline_change_T)
                    .offset((*dp).df_changes.ga_len as isize) = change
            };
            unsafe { (*dp).df_changes.ga_len += 1 };
            new_diff = unsafe { (*new_diff).df_next };
        }
    }

    diff_algorithm.set(save_diff_algorithm);
    unsafe { (*dp).has_changes = true };
    diff_clear(tp);
    tp.tp_first_diff = orig_diff;
    tp.tp_diffbuf = orig_diffbuf;
    // `dio.dio_orig`/`dio_new` point into `file1`/`file2`, which go out of
    // scope here; only the hunk array is separately owned.
    unsafe { clear_diffout(&raw mut dio.dio_diff) };
}
