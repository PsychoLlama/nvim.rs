//! `xpatience.c`: the patience algorithm.
//!
//! Find the lines that are unique in *both* files — intuitively the ones a
//! reader would call the common lines — and take the longest ordered
//! sequence of such pairs as the initial set of matches. Grow each match
//! outward while the neighbouring lines agree, recurse into the gaps, and
//! hand whatever has no unique pair left to the classic algorithm.
//!
//! Ordering matters twice here. The entries are visited in file-1 order, so
//! the longest increasing subsequence is computed over `line2` alone
//! ([`binary_search`]); and an *anchor* pins its position in that sequence,
//! which is why [`find_longest_common_sequence`] carries `anchor_i`.

#![forbid(unsafe_code)]

use crate::src::xdiff::xprepare::prepare_env;
use crate::src::xdiff::xtypes::{Block, Env, Params, XdResult};
use crate::src::xdiff::xutils::fall_back_diff;

/// `line2` for a line that is not unique in one of the two files.
const NON_UNIQUE: u64 = u64::MAX;

/// One slot of [`HashMap`]: a line hash and the (at most one) line on each
/// side that carries it.
#[derive(Clone, Default)]
struct Entry {
    /// The record hash, meaningful only when [`Self::line1`] is set.
    hash: u64,
    /// 1-based line in file 1, or 0 for an unused slot.
    line1: u64,
    /// 1-based line in file 2, 0 for none, [`NON_UNIQUE`] when the line
    /// occurs more than once in either file.
    line2: u64,
    /// Next entry, in file-1 order to begin with and in longest-sequence
    /// order once [`find_longest_common_sequence`] has rewritten it.
    next: Option<u32>,
    /// Previous entry in the longest sequence being built.
    previous: Option<u32>,
    /// This line is one of `xpparam_t.anchors` and must stay in the
    /// sequence.
    anchor: bool,
}

/// The open-addressed table `insert_record` fills, plus the file-1-order
/// chain through the slots it used.
struct HashMap {
    entries: Vec<Entry>,
    /// Head of the file-1-order chain.
    first: Option<u32>,
    /// Tail of it, so an insert is O(1).
    last: Option<u32>,
    /// How many slots are in use.
    nr: usize,
    /// Did any line of file 2 hash to a slot file 1 had used?
    has_matches: bool,
}

/// Does `line` start with one of the caller's anchor strings?
///
/// Upstream compares with `strncmp` against a NUL-terminated anchor, so an
/// anchor longer than the line reads on into the next one; comparing slices
/// stops at the line's end instead. Unobservable in nvim, which never fills
/// `xpparam_t.anchors` — `'diffanchors'` is implemented in `diff.rs` by
/// splitting the buffers before they get here.
fn is_anchor(xpp: &Params<'_>, line: &[u8]) -> bool {
    xpp.anchors.iter().any(|anchor| line.starts_with(anchor))
}

/// Record line `line` of file `pass` in the table.
///
/// After `prepare_env`, a record's `ha` is a *linearized* class id rather
/// than a hash: it starts at 0 and each new class is one higher. Upstream
/// doubles it before taking the modulus, "in the hope that the hashing was
/// unique enough".
fn insert_record(xpp: &Params<'_>, env: &Env<'_>, map: &mut HashMap, line: i64, pass: u32) {
    let side = if pass == 1 { &env.xdf1 } else { &env.xdf2 };
    let ha = side.ha_at(line - 1);
    let alloc = map.entries.len();
    let mut index = ((ha << 1) % alloc as u64) as usize;

    while map.entries[index].line1 != 0 {
        if map.entries[index].hash != ha {
            index += 1;
            if index >= alloc {
                index = 0;
            }
            continue;
        }
        if pass == 2 {
            map.has_matches = true;
        }
        map.entries[index].line2 = if pass == 1 || map.entries[index].line2 != 0 {
            NON_UNIQUE
        } else {
            line as u64
        };
        return;
    }
    if pass == 2 {
        return;
    }

    let anchor = is_anchor(xpp, env.xdf1.line(line - 1));
    let entry = &mut map.entries[index];
    entry.line1 = line as u64;
    entry.hash = ha;
    entry.anchor = anchor;

    let slot = index as u32;
    if map.first.is_none() {
        map.first = Some(slot);
    }
    if let Some(last) = map.last {
        map.entries[last as usize].next = Some(slot);
        map.entries[slot as usize].previous = Some(last);
    }
    map.last = Some(slot);
    map.nr += 1;
}

/// Build the table for one recursion level.
///
/// Called per level rather than once, because a line that is not unique
/// across the whole file may well be unique within the range being looked
/// at.
fn fill_hashmap(xpp: &Params<'_>, env: &Env<'_>, blk: Block) -> HashMap {
    // The size is exact: at most `count1` slots are ever used.
    let alloc = (blk.count1 * 2) as usize;
    let mut map = HashMap {
        entries: vec![Entry::default(); alloc],
        first: None,
        last: None,
        nr: 0,
        has_matches: false,
    };

    for i in 0..blk.count1 {
        insert_record(xpp, env, &mut map, blk.line1 + i, 1);
    }
    for i in 0..blk.count2 {
        insert_record(xpp, env, &mut map, blk.line2 + i, 2);
    }

    map
}

/// Index in `sequence` of the longest run whose last element has a smaller
/// `line2` than `entry`'s, or -1 if there is none.
fn binary_search(entries: &[Entry], sequence: &[u32], longest: i64, line2: u64) -> i64 {
    let mut left = -1i64;
    let mut right = longest;
    while left + 1 < right {
        let middle = left + (right - left) / 2;
        // By construction no two entries can be equal.
        if entries[sequence[middle as usize] as usize].line2 > line2 {
            right = middle;
        } else {
            left = middle;
        }
    }
    left
}

/// The longest ordered sequence of unique line pairs, as the head of a
/// `next` chain.
///
/// The entries arrive in file-1 order, so the sequence is patience sorting
/// over `line2`: one pile per length, each holding the smallest last element
/// seen for that length.
fn find_longest_common_sequence(map: &mut HashMap) -> Option<u32> {
    let mut sequence: Vec<u32> = vec![0; map.nr];
    let mut longest = 0i64;
    // Once an anchor claims a position, nothing may override it — nor
    // anything before it, which would have no effect anyway.
    let mut anchor_i = -1i64;

    let mut cursor = map.first;
    while let Some(id) = cursor {
        cursor = map.entries[id as usize].next;
        let line2 = map.entries[id as usize].line2;
        if line2 == 0 || line2 == NON_UNIQUE {
            continue;
        }
        let i = binary_search(&map.entries, &sequence, longest, line2);
        map.entries[id as usize].previous = if i < 0 {
            None
        } else {
            Some(sequence[i as usize])
        };
        let i = i + 1;
        if i <= anchor_i {
            continue;
        }
        sequence[i as usize] = id;
        if map.entries[id as usize].anchor {
            anchor_i = i;
            longest = anchor_i + 1;
        } else if i == longest {
            longest += 1;
        }
    }

    if longest == 0 {
        return None;
    }

    // Walk back from the last element, turning `previous` into `next`.
    let mut id = sequence[(longest - 1) as usize];
    map.entries[id as usize].next = None;
    while let Some(prev) = map.entries[id as usize].previous {
        map.entries[prev as usize].next = Some(id);
        id = prev;
    }
    Some(id)
}

/// Do these two lines have the same class id?
fn lines_match(env: &Env<'_>, line1: i64, line2: i64) -> bool {
    env.xdf1.ha_at(line1 - 1) == env.xdf2.ha_at(line2 - 1)
}

/// Walk the common sequence, growing each match outward and recursing into
/// the gaps between them.
fn walk_common_sequence(
    xpp: &Params<'_>,
    env: &mut Env<'_>,
    map: &HashMap,
    mut first: Option<u32>,
    blk: Block,
) -> XdResult {
    let (end1, end2) = (blk.line1 + blk.count1, blk.line2 + blk.count2);
    let (mut line1, mut line2) = (blk.line1, blk.line2);

    loop {
        // Try to grow the line ranges of common lines.
        let (mut next1, mut next2) = match first {
            Some(id) => (
                map.entries[id as usize].line1 as i64,
                map.entries[id as usize].line2 as i64,
            ),
            None => (end1, end2),
        };
        if first.is_some() {
            while next1 > line1 && next2 > line2 && lines_match(env, next1 - 1, next2 - 1) {
                next1 -= 1;
                next2 -= 1;
            }
        }
        while line1 < next1 && line2 < next2 && lines_match(env, line1, line2) {
            line1 += 1;
            line2 += 1;
        }

        if next1 > line1 || next2 > line2 {
            recurse(
                xpp,
                env,
                Block {
                    line1,
                    count1: next1 - line1,
                    line2,
                    count2: next2 - line2,
                },
            )?;
        }

        let Some(mut id) = first else {
            return Ok(());
        };

        // Absorb the run of consecutive pairs that follows.
        while let Some(next) = map.entries[id as usize].next {
            let (a, b) = (&map.entries[id as usize], &map.entries[next as usize]);
            if b.line1 != a.line1 + 1 || b.line2 != a.line2 + 1 {
                break;
            }
            id = next;
        }

        line1 = map.entries[id as usize].line1 as i64 + 1;
        line2 = map.entries[id as usize].line2 as i64 + 1;
        first = map.entries[id as usize].next;
    }
}

/// Recursively find the longest common sequence of unique lines, and ask the
/// classic algorithm when there is none.
fn recurse(xpp: &Params<'_>, env: &mut Env<'_>, blk: Block) -> XdResult {
    // Trivial case: one side is empty.
    if blk.count1 == 0 {
        for i in 0..blk.count2 {
            env.xdf2.rchg.set(blk.line2 + i - 1, true);
        }
        return Ok(());
    } else if blk.count2 == 0 {
        for i in 0..blk.count1 {
            env.xdf1.rchg.set(blk.line1 + i - 1, true);
        }
        return Ok(());
    }

    let mut map = fill_hashmap(xpp, env, blk);

    // Are there any matching lines at all?
    if !map.has_matches {
        for i in 0..blk.count1 {
            env.xdf1.rchg.set(blk.line1 + i - 1, true);
        }
        for i in 0..blk.count2 {
            env.xdf2.rchg.set(blk.line2 + i - 1, true);
        }
        return Ok(());
    }

    match find_longest_common_sequence(&mut map) {
        Some(first) => walk_common_sequence(xpp, env, &map, Some(first), blk),
        None => fall_back_diff(env, &xpp.without_algorithm(), blk),
    }
}

/// `xdl_do_patience_diff`.
pub fn diff<'a>(text1: &'a [u8], text2: &'a [u8], xpp: &Params<'_>) -> XdResult<Env<'a>> {
    let mut env = prepare_env(text1, text2, xpp);
    let (n1, n2) = (env.xdf1.nrec(), env.xdf2.nrec());
    recurse(
        xpp,
        &mut env,
        Block {
            line1: 1,
            count1: n1,
            line2: 1,
            count2: n2,
        },
    )?;
    Ok(env)
}
