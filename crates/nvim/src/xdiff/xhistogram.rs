// Ported from `xhistogram.c`, whose notice follows and continues to apply.
//
// Copyright (C) 2010, Google Inc.
// and other copyright owners as documented in JGit's IP log.
//
// This program and the accompanying materials are made available
// under the terms of the Eclipse Distribution License v1.0 which
// accompanies this distribution, is reproduced below, and is
// available at http://www.eclipse.org/org/documents/edl-v10.php
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or
// without modification, are permitted provided that the following
// conditions are met:
//
// - Redistributions of source code must retain the above copyright
//   notice, this list of conditions and the following disclaimer.
//
// - Redistributions in binary form must reproduce the above
//   copyright notice, this list of conditions and the following
//   disclaimer in the documentation and/or other materials provided
//   with the distribution.
//
// - Neither the name of the Eclipse Foundation, Inc. nor the
//   names of its contributors may be used to endorse or promote
//   products derived from this software without specific prior
//   written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND
// CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
// OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
// NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
// LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
// ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! `xhistogram.c`: the histogram algorithm.
//!
//! A refinement of patience. Instead of insisting a common line be unique in
//! both files, index every line of the "A" range by how often it occurs, and
//! take as the pivot the longest common run built from the *least* frequent
//! line available. Recurse either side of that run.
//!
//! Two things make it give up rather than answer. A hash bucket that reaches
//! [`MAX_CHAIN_LENGTH`] distinct lines aborts the whole diff — upstream's
//! way of refusing pathological input — and a run whose rarest line is still
//! more common than the chain limit is handed to the classic algorithm
//! instead.

#![forbid(unsafe_code)]

use crate::xdiff::xprepare::prepare_env;
use crate::xdiff::xtypes::{Aborted, Block, Env, Params, XdFile, XdResult};
use crate::xdiff::xutils::{fall_back_diff, hashbits, recmatch};

/// Largest line number the index can hold; upstream's `MAX_PTR`.
const MAX_PTR: i64 = i32::MAX as i64;
/// Ceiling on an occurrence count; upstream's `MAX_CNT`.
const MAX_CNT: u32 = i32::MAX as u32;
/// Distinct lines allowed in one hash bucket before the engine gives up.
const MAX_CHAIN_LENGTH: u32 = 64;

/// One distinct line of the A range, and every place it occurs.
///
/// `ptr` is the *most recently scanned* occurrence, and A is scanned
/// backwards, so it is the earliest line seen so far; the rest of the
/// occurrences hang off [`HistIndex::next_ptrs`] in increasing order.
struct Occurrence {
    /// Line number of the first known occurrence.
    ptr: i64,
    /// How many occurrences there are, capped at [`MAX_CNT`].
    cnt: u32,
    /// Next distinct line in the same hash bucket.
    next: Option<u32>,
}

/// `struct histindex`: the A range, indexed by line content.
struct HistIndex {
    /// Head of each hash bucket's chain.
    records: Vec<Option<u32>>,
    /// Per line of the A range, which [`Occurrence`] it belongs to.
    line_map: Vec<Option<u32>>,
    /// Per line of the A range, the next line with the same content; 0 for
    /// none, which is safe because line numbers here are 1-based.
    next_ptrs: Vec<i64>,
    /// Every distinct line; the index into this is the id used above.
    pool: Vec<Occurrence>,
    /// Width of [`Self::records`], in bits.
    table_bits: u32,
    /// Line number [`Self::line_map`] and [`Self::next_ptrs`] start at.
    ptr_shift: i64,
    /// Occurrence count of the current best run's rarest line. Starts one
    /// above the chain limit, so any run at all improves on it.
    cnt: u32,
    /// Did any line of B match a line of A?
    has_common: bool,
}

/// The common run [`find_lcs`] settled on, as inclusive line ranges.
#[derive(Clone, Copy, Default)]
struct Region {
    begin1: i64,
    end1: i64,
    begin2: i64,
    end2: i64,
}

/// Do line `l1` of `a` and line `l2` of `b` match? Both are 1-based.
fn cmp_recs(xpp: &Params<'_>, a: &XdFile<'_>, l1: i64, b: &XdFile<'_>, l2: i64) -> bool {
    a.ha_at(l1 - 1) == b.ha_at(l2 - 1) && recmatch(a.line(l1 - 1), b.line(l2 - 1), xpp.flags)
}

/// `XDL_HASHLONG` over a record's hash.
fn table_hash(side: &XdFile<'_>, line: i64, bits: u32) -> usize {
    let v = side.ha_at(line - 1);
    (v.wrapping_add(v >> bits) & ((1u64 << bits) - 1)) as usize
}

impl HistIndex {
    fn next_ptr(&self, ptr: i64) -> i64 {
        self.next_ptrs[(ptr - self.ptr_shift) as usize]
    }

    fn set_next_ptr(&mut self, ptr: i64, to: i64) {
        self.next_ptrs[(ptr - self.ptr_shift) as usize] = to;
    }

    /// How often line `ptr`'s content occurs in the A range.
    fn count_at(&self, ptr: i64) -> u32 {
        let id = self.line_map[(ptr - self.ptr_shift) as usize]
            .expect("every line of the A range is mapped by scan_a");
        self.pool[id as usize].cnt
    }

    /// Index the A range: one [`Occurrence`] per distinct line, with every
    /// repeat threaded onto [`Self::next_ptrs`].
    fn scan_a(&mut self, xpp: &Params<'_>, env: &Env<'_>, blk: Block) -> XdResult {
        let mut ptr = blk.end1();
        while blk.line1 <= ptr {
            let bucket = table_hash(&env.xdf1, ptr, self.table_bits);
            let mut chain_len = 0u32;
            let mut cursor = self.records[bucket];
            let mut matched = false;

            while let Some(id) = cursor {
                if cmp_recs(xpp, &env.xdf1, self.pool[id as usize].ptr, &env.xdf1, ptr) {
                    // Identical to a line already seen: put `ptr` on the
                    // front of that line's occurrence chain.
                    let head = self.pool[id as usize].ptr;
                    self.set_next_ptr(ptr, head);
                    self.pool[id as usize].ptr = ptr;
                    self.pool[id as usize].cnt = (self.pool[id as usize].cnt + 1).min(MAX_CNT);
                    self.line_map[(ptr - self.ptr_shift) as usize] = Some(id);
                    matched = true;
                    break;
                }
                cursor = self.pool[id as usize].next;
                chain_len += 1;
            }

            if !matched {
                if chain_len == MAX_CHAIN_LENGTH {
                    return Err(Aborted);
                }
                // First time we have seen this line; start a new chain.
                let id = self.pool.len() as u32;
                self.pool.push(Occurrence {
                    ptr,
                    cnt: 1,
                    next: self.records[bucket],
                });
                self.records[bucket] = Some(id);
                self.line_map[(ptr - self.ptr_shift) as usize] = Some(id);
            }

            ptr -= 1;
        }
        Ok(())
    }

    /// Extend every occurrence of line `b_ptr`'s content into the longest
    /// common run it can, keeping the best one found so far in `lcs`.
    ///
    /// Answers the next line of B worth trying, which is past the end of any
    /// run this call produced.
    fn try_lcs(
        &mut self,
        xpp: &Params<'_>,
        env: &Env<'_>,
        lcs: &mut Region,
        b_ptr: i64,
        blk: Block,
    ) -> i64 {
        let (end_a, end_b) = (blk.end1(), blk.end2());
        let mut b_next = b_ptr + 1;
        let mut cursor = self.records[table_hash(&env.xdf2, b_ptr, self.table_bits)];

        while let Some(id) = cursor {
            cursor = self.pool[id as usize].next;
            let rec_cnt = self.pool[id as usize].cnt;

            if rec_cnt > self.cnt {
                // Too common to be worth a run, but it still tells us the
                // two files have something in common.
                if !self.has_common {
                    self.has_common =
                        cmp_recs(xpp, &env.xdf1, self.pool[id as usize].ptr, &env.xdf2, b_ptr);
                }
                continue;
            }

            let mut a_at = self.pool[id as usize].ptr;
            if !cmp_recs(xpp, &env.xdf1, a_at, &env.xdf2, b_ptr) {
                continue;
            }
            self.has_common = true;

            loop {
                let mut np = self.next_ptr(a_at);
                let (mut as_, mut bs) = (a_at, b_ptr);
                let (mut ae, mut be) = (a_at, b_ptr);
                let mut rc = rec_cnt;

                while blk.line1 < as_
                    && blk.line2 < bs
                    && cmp_recs(xpp, &env.xdf1, as_ - 1, &env.xdf2, bs - 1)
                {
                    as_ -= 1;
                    bs -= 1;
                    if 1 < rc {
                        rc = rc.min(self.count_at(as_));
                    }
                }
                while ae < end_a
                    && be < end_b
                    && cmp_recs(xpp, &env.xdf1, ae + 1, &env.xdf2, be + 1)
                {
                    ae += 1;
                    be += 1;
                    if 1 < rc {
                        rc = rc.min(self.count_at(ae));
                    }
                }

                if b_next <= be {
                    b_next = be + 1;
                }
                // A longer run wins, and so does an equally long one built
                // from a rarer line.
                if lcs.end1 - lcs.begin1 < ae - as_ || rc < self.cnt {
                    *lcs = Region {
                        begin1: as_,
                        begin2: bs,
                        end1: ae,
                        end2: be,
                    };
                    self.cnt = rc;
                }

                if np == 0 {
                    break;
                }
                // Skip the occurrences this run already swallowed.
                while np <= ae {
                    np = self.next_ptr(np);
                    if np == 0 {
                        break;
                    }
                }
                if np == 0 {
                    break;
                }
                a_at = np;
            }
        }

        b_next
    }
}

/// Build the index for the A range and find the best common run in it.
///
/// Answers `Ok(true)` when even the best run's rarest line is too common to
/// trust, which is the caller's cue to fall back to the classic algorithm.
fn find_lcs(xpp: &Params<'_>, env: &Env<'_>, lcs: &mut Region, blk: Block) -> XdResult<bool> {
    let table_bits = hashbits(blk.count1 as u32);
    let mut index = HistIndex {
        records: vec![None; 1usize << table_bits],
        line_map: vec![None; blk.count1 as usize],
        next_ptrs: vec![0; blk.count1 as usize],
        // Upstream sizes its arena `count1 / 4 + 1` records at a time, "from
        // xprepare.c"; the same guess makes a decent capacity here.
        pool: Vec::with_capacity((blk.count1 / 4 + 1) as usize),
        table_bits,
        ptr_shift: blk.line1,
        cnt: 0,
        has_common: false,
    };

    index.scan_a(xpp, env, blk)?;
    index.cnt = MAX_CHAIN_LENGTH + 1;

    let mut b_ptr = blk.line2;
    while b_ptr <= blk.end2() {
        b_ptr = index.try_lcs(xpp, env, lcs, b_ptr, blk);
    }

    Ok(index.has_common && MAX_CHAIN_LENGTH < index.cnt)
}

/// Diff the two line ranges `blk` names.
fn histogram_diff(xpp: &Params<'_>, env: &mut Env<'_>, mut blk: Block) -> XdResult {
    loop {
        if blk.count1 <= 0 && blk.count2 <= 0 {
            return Ok(());
        }
        if blk.end1() >= MAX_PTR {
            return Err(Aborted);
        }
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

        let mut lcs = Region::default();
        if find_lcs(xpp, env, &mut lcs, blk)? {
            return fall_back_diff(env, &xpp.without_algorithm(), blk);
        }

        if lcs.begin1 == 0 && lcs.begin2 == 0 {
            for i in 0..blk.count1 {
                env.xdf1.rchg.set(blk.line1 + i - 1, true);
            }
            for i in 0..blk.count2 {
                env.xdf2.rchg.set(blk.line2 + i - 1, true);
            }
            return Ok(());
        }

        histogram_diff(
            xpp,
            env,
            Block {
                line1: blk.line1,
                count1: lcs.begin1 - blk.line1,
                line2: blk.line2,
                count2: lcs.begin2 - blk.line2,
            },
        )?;
        // Upstream hand-optimises the tail recursion into a `goto redo`;
        // so does this loop.
        blk = Block {
            line1: lcs.end1 + 1,
            count1: blk.end1() - lcs.end1,
            line2: lcs.end2 + 1,
            count2: blk.end2() - lcs.end2,
        };
    }
}

/// `xdl_do_histogram_diff`.
pub fn diff<'a>(text1: &'a [u8], text2: &'a [u8], xpp: &Params<'_>) -> XdResult<Env<'a>> {
    let mut env = prepare_env(text1, text2, xpp);
    let (d1s, d1e) = (env.xdf1.dstart, env.xdf1.dend);
    let (d2s, d2e) = (env.xdf2.dstart, env.xdf2.dend);
    histogram_diff(
        xpp,
        &mut env,
        Block {
            line1: d1s + 1,
            count1: d1e - d1s + 1,
            line2: d2s + 1,
            count2: d2e - d2s + 1,
        },
    )?;
    Ok(env)
}
