//! Where a region ends, and the low-level matching primitives.
//!
//! [`SynState::find_endpos`] is the search for a region's end: the first END pattern that
//! matches after the START, with any SKIP pattern's matches stepped over, and
//! the `matchgroup=` end highlight worked out. Around it sit the primitives the
//! rest of the state machine matches with -- [`SynState::syn_regexec`] (a timed
//! `vim_regexec_multi`), [`SynState::check_keyword_id`] (the keyword hash
//! lookup), and [`SynState::syn_add_start_off`]/[`SynState::syn_add_end_off`], which apply the seven `ms=`/`me=`/
//! `hs=`/`he=`/`rs=`/`re=`/`lc=` offsets to a match.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// What [`SynState::find_endpos`] found.
///
/// `m_endpos.lnum == 0` is the "no END pattern matched in this line" answer,
/// and then every other field is meaningless -- the region continues into the
/// next line. That spelling is upstream's, and callers test it directly.
pub(crate) struct RegionEnd {
    /// End of the match: where the region stops.
    pub(crate) m_endpos: LPos,
    /// End of the highlighting, which a `me=`/`he=` offset can pull in front
    /// of the match end.
    pub(crate) hl_endpos: LPos,
    /// End of the END pattern's own match, for the `matchgroup=` item that
    /// highlights it.
    pub(crate) eoe_pos: LPos,
    /// Index of the END pattern when it has a `matchgroup=` of its own, else
    /// 0.
    pub(crate) end_idx: c_int,
    /// The flags of the END pattern that matched. `None` when none did; the
    /// caller keeps whatever flags it already had.
    pub(crate) flags: Option<SynFlags>,
}

impl RegionEnd {
    /// The "no end in this line" answer.
    const fn none() -> Self {
        let zero = LPos { lnum: 0, col: 0 };
        RegionEnd {
            m_endpos: zero,
            hl_endpos: zero,
            eoe_pos: zero,
            end_idx: 0,
            flags: None,
        }
    }
}

impl SynState {
    /// Run pattern `idx`'s program over `lnum` from `col`, into a fresh match.
    ///
    /// `ext` is what the pattern's `\z1`..`\z9` match: the captures of the
    /// region's start pattern, when this is one of its skip or end patterns.
    /// Answers whether it matched, the match, and the pattern's own `\z(`
    /// captures when it is a start pattern that has some.
    ///
    /// The engine may hand back a *different* program (`vim_regexec_multi` can
    /// recompile), so the answer is written back into the pattern, and each
    /// pattern is timed into its own `sp_time`.
    pub(crate) fn run_pattern(
        &mut self,
        idx: c_int,
        lnum: LineNr,
        col: ColNr,
        ext: Option<&ExtMatch>,
    ) -> (bool, RegMMatch, Option<ExtMatchRef>) {
        let mut regmatch = empty_regmmatch();
        let mut block = self.block();
        let spp = block.pattern_mut(idx);
        regmatch.rmm_ic = spp.sp_ic;
        regmatch.regprog = spp.sp_prog;
        let time = &mut spp.sp_time;
        let mut captures = None;
        let io = ExtMatchIo {
            input: ext,
            output: &mut captures,
        };
        let matched = self.syn_regexec(&mut regmatch, lnum, col, time, io);
        block.pattern_mut(idx).sp_prog = regmatch.regprog;
        (matched, regmatch, captures)
    }

    /// Find the end of the start/skip/end region `idx` after `startpos`.
    ///
    /// Only looks in `startpos.lnum`; if no END pattern matches there the region
    /// continues into the next line and the answer is [`RegionEnd::none`]. Also
    /// handles a match item that continued from a previous line.
    ///
    /// `start_ext` are the `\z(` submatches of the START pattern, which the END
    /// and SKIP patterns may refer to with `\z1`..`\z9`.
    pub(crate) fn find_endpos(
        &mut self,
        mut idx: c_int,
        startpos: LPos,
        start_ext: Option<&ExtMatch>,
    ) -> RegionEnd {
        // Just in case we are invoked for a keyword.
        if idx < 0 {
            return RegionEnd::none();
        }

        // Check for being called with a START pattern. Can happen with a match
        // that continues to the next line because it contained a region.
        // Upstream answers `hl_endpos = startpos` here, which no caller reads:
        // both test `m_endpos.lnum` first, and it is 0.
        if self.block().pattern(idx).sp_type as c_int != SPTYPE_START {
            let mut end = RegionEnd::none();
            end.hl_endpos = startpos;
            return end;
        }

        // Find the SKIP or first END pattern after the last START pattern. The
        // patterns of one `:syntax region` are stored consecutively, START(s)
        // then an optional SKIP then the ENDs.
        while self.block().pattern(idx).sp_type as c_int == SPTYPE_START {
            idx += 1;
        }
        let skip_idx = if self.block().pattern(idx).sp_type as c_int == SPTYPE_SKIP {
            idx += 1;
            Some(idx - 1)
        } else {
            None
        };

        let mut buf_chartab = [0u64; 4];
        self.install_syntax_chartab(&mut buf_chartab);

        let start_idx = idx;
        let mut matchcol = startpos.col;
        let answer = self.find_endpos_scan(start_idx, skip_idx, startpos, &mut matchcol, start_ext);

        self.restore_chartab(&buf_chartab);
        answer
    }

    /// The search loop of [`find_endpos`]: try every END pattern from `start_idx`,
    /// step `matchcol` over anything the SKIP pattern claims, and repeat.
    fn find_endpos_scan(
        &mut self,
        start_idx: c_int,
        skip_idx: Option<c_int>,
        startpos: LPos,
        matchcol: &mut ColNr,
        ext: Option<&ExtMatch>,
    ) -> RegionEnd {
        loop {
            let Some((best_idx, best)) = self.best_end_match(start_idx, startpos, *matchcol, ext)
            else {
                // All end patterns tried with no match: the item continues
                // until end-of-line.
                return RegionEnd::none();
            };
            if let Some(skip_idx) = skip_idx {
                match self.skip_past(skip_idx, startpos, best.startpos[0], *matchcol, ext) {
                    Skipped::No => {}
                    // The skip match reaches the end of the line (or the next
                    // one): no end pattern can match in this line after all.
                    Skipped::PastLine => return RegionEnd::none(),
                    Skipped::To(col) => {
                        *matchcol = col;
                        continue; // start with the first end pattern again
                    }
                }
            }
            return self.end_positions(best_idx, &best, startpos);
        }
    }

    /// The END pattern that matches first at or after `matchcol`, with its match.
    fn best_end_match(
        &mut self,
        start_idx: c_int,
        startpos: LPos,
        matchcol: ColNr,
        ext: Option<&ExtMatch>,
    ) -> Option<(c_int, RegMMatch)> {
        let mut best: Option<(c_int, RegMMatch)> = None;
        let mut idx = start_idx;
        while idx < self.block().patterns().len() as c_int {
            let spp = self.block();
            let spp = spp.pattern(idx);
            if spp.sp_type as c_int != SPTYPE_END {
                break; // past the last END pattern of this region
            }
            let lc_col = (matchcol as c_int - spp.sp_offsets[SPO_LC_OFF as usize]).max(0);

            let (matched, regmatch, _) = self.run_pattern(idx, startpos.lnum, lc_col as ColNr, ext);
            let col = regmatch.startpos[0].col;
            if matched && best.as_ref().is_none_or(|(_, b)| col < b.startpos[0].col) {
                best = Some((idx, regmatch));
            }
            idx += 1;
        }
        best
    }
}

/// What the SKIP pattern did to the search position.
enum Skipped {
    /// It did not match before the end pattern; use the end pattern.
    No,
    /// It ran to the next line, or included the end of this one.
    PastLine,
    /// Resume the end-pattern search at this column.
    To(ColNr),
}

impl SynState {
    /// Does the SKIP pattern match before the best END pattern's match?
    fn skip_past(
        &mut self,
        skip_idx: c_int,
        startpos: LPos,
        best_start: LPos,
        matchcol: ColNr,
        ext: Option<&ExtMatch>,
    ) -> Skipped {
        let offsets = self.block().pattern(skip_idx).offsets();
        let lc_col = (matchcol as c_int - offsets.offsets[SPO_LC_OFF as usize]).max(0);
        let (matched, regmatch, _) =
            self.run_pattern(skip_idx, startpos.lnum, lc_col as ColNr, ext);
        if !matched || regmatch.startpos[0].col > best_start.col {
            return Skipped::No;
        }

        // Add the offset to the skip pattern's match.
        let pos = self.syn_add_end_off(offsets, &regmatch, SPO_ME_OFF, 1);
        if pos.lnum > startpos.lnum {
            // The skip pattern goes on to the next line, so there is no match
            // with an end pattern in this line.
            return Skipped::PastLine;
        }
        let line_len = ml_get_buf_len(self.buffer, startpos.lnum);

        // Take care of an empty match or a negative offset.
        let col = if pos.col <= matchcol {
            matchcol + 1
        } else if pos.col <= regmatch.endpos[0].col {
            pos.col
        } else {
            // Be careful not to jump over the NUL at the end of the line.
            let mut col = regmatch.endpos[0].col;
            while col < line_len && col < pos.col {
                col += 1;
            }
            col
        };
        if col >= line_len {
            // The skip pattern includes end-of-line.
            Skipped::PastLine
        } else {
            Skipped::To(col)
        }
    }

    /// Turn the winning END match into the four positions the caller wants.
    fn end_positions(&self, best_idx: c_int, best: &RegMMatch, startpos: LPos) -> RegionEnd {
        let block = self.block();
        let spp = block.pattern(best_idx);
        let offsets = spp.offsets();

        // Match from the start pattern to the end pattern, corrected for the
        // end pattern's match and highlight offsets. Neither may end before
        // the start.
        let mut m_endpos = self.syn_add_end_off(offsets, best, SPO_ME_OFF, 1);
        if m_endpos.lnum == startpos.lnum && m_endpos.col < startpos.col {
            m_endpos.col = startpos.col;
        }
        let mut eoe_pos = self.syn_add_end_off(offsets, best, SPO_HE_OFF, 1);
        if eoe_pos.lnum == startpos.lnum && eoe_pos.col < startpos.col {
            eoe_pos.col = startpos.col;
        }
        limit_pos(&mut eoe_pos, m_endpos);

        let (hl_endpos, end_idx, m_endpos) =
            if spp.sp_syn_match_id != spp.sp_syn.id && spp.sp_syn_match_id != 0 {
                // The end group is highlighted differently: the highlighting
                // stops where the `matchgroup=` item takes over, and the match
                // is then turned into that item.
                let flagged = offsets.flags as c_int & (1 << (SPO_RE_OFF + SPO_COUNT)) != 0;
                let base = if flagged {
                    best.endpos[0]
                } else {
                    best.startpos[0]
                };
                let mut hl_endpos = LPos {
                    lnum: base.lnum,
                    col: base.col + offsets.offsets[SPO_RE_OFF as usize],
                };
                if hl_endpos.lnum == startpos.lnum && hl_endpos.col < startpos.col {
                    hl_endpos.col = startpos.col;
                }
                limit_pos(&mut hl_endpos, m_endpos);
                (hl_endpos, best_idx, hl_endpos)
            } else {
                (eoe_pos, 0, m_endpos)
            };

        RegionEnd {
            m_endpos,
            hl_endpos,
            eoe_pos,
            end_idx,
            flags: Some(spp.sp_flags),
        }
    }
}

/// A zeroed `RegMMatch`, which `vim_regexec_multi` fills in.
pub(crate) const fn empty_regmmatch() -> RegMMatch {
    RegMMatch {
        regprog: ::core::ptr::null_mut(),
        startpos: [LPos { lnum: 0, col: 0 }; 10],
        endpos: [LPos { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    }
}

/// Limit `pos` not to be after `limit`.
pub(crate) fn limit_pos(pos: &mut LPos, limit: LPos) {
    if pos.lnum > limit.lnum {
        *pos = limit;
    } else if pos.lnum == limit.lnum && pos.col > limit.col {
        pos.col = limit.col;
    }
}

/// [`limit_pos`], but a `pos` of line 0 -- "not set" -- takes the limit.
pub(crate) fn limit_pos_zero(pos: &mut LPos, limit: LPos) {
    if pos.lnum == 0 {
        *pos = limit;
    } else {
        limit_pos(pos, limit);
    }
}

impl SynState {
    /// Apply the `me=`/`he=`/`re=` end offset `idx` of `spp` to `regmatch`.
    ///
    /// `extra` is added when the offset is measured from the *start* of the match
    /// (`me=s+1`), which is how "one past" is spelled for a region's end.
    pub(crate) fn syn_add_end_off(
        &self,
        spp: PatOffsets,
        regmatch: &RegMMatch,
        idx: c_int,
        extra: c_int,
    ) -> LPos {
        let flagged = spp.flags as c_int & (1 << idx) != 0;
        let base = if flagged {
            regmatch.startpos[0]
        } else {
            regmatch.endpos[0]
        };
        let off = spp.offsets[idx as usize] + if flagged { extra } else { 0 };

        let col = if base.lnum > self.line_count() {
            // Watch out for a match with the last NL in the buffer. Matters for
            // "rs=e+2" when there is a matchgroup.
            0
        } else {
            self.walk_chars(base.lnum, base.col, off)
        };
        LPos {
            lnum: base.lnum,
            col,
        }
    }

    /// Apply the `ms=`/`hs=`/`rs=` start offset `idx` of `spp` to `regmatch`.
    ///
    /// Differs from [`syn_add_end_off`] in three ways, all upstream's: the offset
    /// flag lives in the *upper* half of `sp_off_flags`, a set flag means the
    /// offset is measured from the match *end* rather than its start, and a
    /// position past the last line is clamped to the end of the last line instead
    /// of to column 0.
    pub(crate) fn syn_add_start_off(
        &self,
        spp: PatOffsets,
        regmatch: &RegMMatch,
        idx: c_int,
        extra: c_int,
    ) -> LPos {
        let flagged = spp.flags as c_int & (1 << (idx + SPO_COUNT)) != 0;
        let base = if flagged {
            regmatch.endpos[0]
        } else {
            regmatch.startpos[0]
        };
        let off = spp.offsets[idx as usize] + if flagged { extra } else { 0 };

        let (lnum, col) = if base.lnum > self.line_count() {
            // A "\n" at the end of the pattern may take us below the last line.
            let lnum = self.line_count();
            (lnum, ml_get_buf_len(self.buffer, lnum))
        } else {
            (base.lnum, base.col)
        };
        LPos {
            lnum,
            col: self.walk_chars(lnum, col, off),
        }
    }

    /// Step `off` characters forward (or backward) from `col` in line `lnum`,
    /// stopping at the line's ends. Answers the resulting column.
    fn walk_chars(&self, lnum: LineNr, col: ColNr, off: c_int) -> ColNr {
        if off == 0 {
            return col;
        }
        let base = unsafe { ml_get_buf(self.buffer, lnum) };
        let mut p = unsafe { base.offset(col as isize) };
        let mut left = off;
        if off > 0 {
            while left > 0 && unsafe { *p } as c_int != NUL {
                p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
                left -= 1;
            }
        } else {
            while left < 0 && base < p {
                p = unsafe { p.offset(-((utf_head_off(base, p.offset(-1)) + 1) as isize)) };
                left += 1;
            }
        }
        unsafe { p.offset_from(base) as ColNr }
    }

    /// The current line of the syntax buffer.
    ///
    /// NOTE: the *bytes* are invalid after anything that can look for a pattern
    /// match -- the regexp engine may reach `ml_get_buf` for another line and
    /// evict this one. Reading them is the caller's unsafe step; asking for the
    /// pointer is not.
    ///
    /// The buffer is the one the entry point resolved for this call, which the
    /// per-column path reads rather than asking the registry per character.
    pub(crate) fn curline(&self) -> *mut c_char {
        // SAFETY: `buffer` is the buffer the parse was started for, and `lnum`
        // a line of it.
        unsafe { ml_get_buf(self.buffer, self.lnum) }
    }

    /// Length of the current line of the syntax buffer.
    pub(crate) fn curline_len(&self) -> ColNr {
        ml_get_buf_len(self.buffer, self.lnum)
    }

    /// The byte at `col` of the line being parsed.
    ///
    /// Every caller is testing for the NUL that ends the line, so `col` is at
    /// most its length and the read stays inside what `ml_get_buf` answered.
    pub(crate) fn curline_byte(&self, col: ColNr) -> u8 {
        debug_assert!(col <= self.curline_len());
        // SAFETY: `col` is within the line, its terminator included.
        unsafe { *self.curline().offset(col as isize) as u8 }
    }

    /// Number of lines in the buffer being parsed.
    pub(crate) fn line_count(&self) -> LineNr {
        self.buffer.line_count()
    }

    /// `vim_regexec_multi` in the syntax buffer, timed into `time` when
    /// `:syntime` is on.
    ///
    /// Answers whether there was a match, and on a match shifts `regmatch`'s
    /// positions from pattern-relative to buffer-absolute line numbers.
    pub(crate) fn syn_regexec(
        &mut self,
        regmatch: &mut RegMMatch,
        lnum: LineNr,
        col: ColNr,
        time: &mut SynTime,
        io: ExtMatchIo<'_>,
    ) -> bool {
        let timing = syn_time_on.get();
        let start = if timing { profile_start() } else { 0 };

        if regmatch.regprog.is_null() {
            // A previous vim_regexec_multi() tried the NFA engine, got
            // NFA_TOO_EXPENSIVE, and compiling with the other engine failed.
            return false;
        }
        // The window and buffer the parser was started for -- the window may
        // have gone, the buffer never has once the parse has begun.
        let (win, buf) = (self.win.and_then(WinId::get), self.buffer);
        regmatch.rmm_maxcol = buf.b_p_smc as ColNr;
        let mut timed_out: c_int = 0;
        let tm = self
            .deadline
            .as_mut()
            .map_or(::core::ptr::null_mut(), |tm| tm as *mut ProfTime);
        // SAFETY: the caller's match holds a live program; the deadline and the
        // flag are this frame's.
        let r = unsafe {
            vim_regexec_syntax(regmatch, win, buf, lnum, col, tm, &raw mut timed_out, io)
        };

        if timing {
            let took = profile_end(start);
            time.total = profile_add(time.total, took);
            // `profile_cmp(a, b)` is negative when `a` is the *larger* time, so
            // this really does keep the slowest.
            if profile_cmp(took, time.slowest) < 0 {
                time.slowest = took;
            }
            time.count += 1;
            if r > 0 {
                time.match_0 += 1;
            }
        }
        if timed_out != 0
            && let Some(mut block) = win.map(Win::syntax)
            && !block.b_syn_slow
        {
            block.b_syn_slow = true;
            msg(
                gettext(c"'redrawtime' exceeded, syntax highlighting disabled"),
                0,
            );
        }

        if r > 0 {
            regmatch.startpos[0].lnum += lnum;
            regmatch.endpos[0].lnum += lnum;
            return true;
        }
        false
    }
}

/// A keyword the hash tables claimed.
pub(crate) struct KeywordMatch {
    /// Highlight group id of the keyword.
    pub(crate) id: c_int,
    /// Column of the character after the keyword.
    pub(crate) endcol: c_int,
    /// The keyword's `HL_*` flags.
    pub(crate) flags: SynFlags,
    /// Its `nextgroup=` list.
    pub(crate) next_list: *mut int16_t,
    /// Its `cchar=` conceal substitution character.
    pub(crate) cchar: c_int,
}

impl SynState {
    /// Check one position in a line for a matching keyword.
    ///
    /// The caller must have established that a keyword can start at `startcol`.
    ///
    /// # Safety
    ///
    /// `line` must point at a NUL-terminated line with a word character at
    /// `startcol` — the caller has already established that a keyword can start
    /// there, and the scan for its end reads forwards from it. `cur_si`, when it
    /// is `Some`, must still be live: nothing may have pushed to, popped from or
    /// cleared the syntax state stack since it was taken. `line` is a line of
    /// `buffer`, whose 'iskeyword' says where the keyword ends.
    pub(crate) unsafe fn check_keyword_id(
        &self,
        line: *mut c_char,
        startcol: c_int,
        cur_si: Option<c_int>,
    ) -> Option<KeywordMatch> {
        // Find the first character after the keyword; the first character was
        // already checked by the caller.
        let kwp = unsafe { line.offset(startcol as isize) };
        let mut kwlen: c_int = 0;
        loop {
            kwlen += unsafe { utfc_ptr2len(kwp.offset(kwlen as isize)) };
            if !unsafe { vim_iswordp_buf(kwp.offset(kwlen as isize), self.buffer) } {
                break;
            }
        }
        if kwlen > MAXKEYWLEN {
            return None;
        }

        // A copy, so it can be NUL-terminated and lowercased.
        let mut keyword: [c_char; MAXKEYWLEN as usize + 1] = [0; MAXKEYWLEN as usize + 1];
        let buf = &raw mut keyword as *mut c_char;
        unsafe { xmemcpyz(buf.cast(), kwp.cast(), kwlen as size_t) };

        let mut kp = ::core::ptr::null_mut::<KeyEntry>();
        if self.block().b_keywtab.ht_used != 0 {
            kp = unsafe { self.match_keyword(buf, syn_field!(self.block(), b_keywtab), cur_si) };
        }
        if kp.is_null() && self.block().b_keywtab_ic.ht_used != 0 {
            unsafe { str_foldcase(kwp, kwlen, buf, MAXKEYWLEN + 1) };
            kp = unsafe { self.match_keyword(buf, syn_field!(self.block(), b_keywtab_ic), cur_si) };
        }
        if kp.is_null() {
            return None;
        }
        Some(KeywordMatch {
            id: unsafe { (*kp).k_syn.id } as c_int,
            endcol: startcol + kwlen,
            flags: unsafe { (*kp).flags },
            next_list: unsafe { (*kp).next_list },
            cchar: unsafe { (*kp).k_char },
        })
    }

    /// The first keyword entry for `keyword` in `ht` that the containment rules
    /// admit here.
    ///
    /// There can be several entries with the same text and different attributes,
    /// chained through `ke_next`. `next_list` (a pending `nextgroup=`)
    /// overrides everything; otherwise a keyword is accepted at the top level when
    /// it is not `contained`, and inside an item when that item's `contains=` list
    /// names it.
    ///
    /// # Safety
    ///
    /// `keyword` must point at a NUL-terminated string, unaliased for the call.
    /// `ht` must point at a live hash table, unaliased for the call. `cur_si`
    /// must still be live: nothing may have pushed to, popped from or cleared the
    /// syntax state stack since it was taken, when it is `Some`.
    unsafe fn match_keyword(
        &self,
        keyword: *mut c_char,
        ht: *mut HashTab,
        cur_si: Option<c_int>,
    ) -> *mut KeyEntry {
        let hi = unsafe { hash_find(ht, keyword) };
        if !hi.is_kept() {
            return ::core::ptr::null_mut();
        }
        // The hash key IS the entry's trailing `keyword[]` array, so the entry
        // starts that many bytes before it.
        // SAFETY: `hash_find` answered a live item, and the key is the entry's
        // own trailing array, so the subtraction stays inside the allocation.
        let mut kp = unsafe {
            hi.hi_key
                .offset(-(::core::mem::offset_of!(KeyEntry, keyword) as isize))
        } as *mut KeyEntry;
        while !kp.is_null() {
            // SAFETY: `kp` walks a chain of live keyword entries.
            let (syn, cont_in, flags) = unsafe { ((*kp).k_syn, (*kp).cont_in_list, (*kp).flags) };
            let ok = if !self.next_list.is_null() {
                let next = self.next_list;
                // SAFETY: the parser's own lists.
                unsafe { self.in_id_list(None, next, syn, cont_in, SynFlags::NONE) }
            } else if let Some(cur_si) = cur_si {
                let contains = self.item(cur_si).si_cont_list;
                // SAFETY: as above, inside the item the caller named.
                unsafe { self.in_id_list(Some(cur_si), contains, syn, cont_in, flags) }
            } else {
                !flags.has(SynFlags::CONTAINED)
            };
            if ok {
                return kp;
            }
            kp = unsafe { (*kp).ke_next };
        }
        ::core::ptr::null_mut()
    }
}
