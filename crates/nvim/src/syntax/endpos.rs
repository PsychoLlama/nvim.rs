//! Where a region ends, and the low-level matching primitives.
//!
//! [`find_endpos`] is the search for a region's end: the first END pattern that
//! matches after the START, with any SKIP pattern's matches stepped over, and
//! the `matchgroup=` end highlight worked out. Around it sit the primitives the
//! rest of the state machine matches with -- [`syn_regexec`] (a timed
//! `vim_regexec_multi`), [`check_keyword_id`] (the keyword hash lookup), and
//! [`syn_add_start_off`]/[`syn_add_end_off`], which apply the seven `ms=`/`me=`/
//! `hs=`/`he=`/`rs=`/`re=`/`lc=` offsets to a match.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// What [`find_endpos`] found.
///
/// `m_endpos.lnum == 0` is the "no END pattern matched in this line" answer,
/// and then every other field is meaningless -- the region continues into the
/// next line. That spelling is upstream's, and callers test it directly.
pub(crate) struct RegionEnd {
    /// End of the match: where the region stops.
    pub(crate) m_endpos: lpos_T,
    /// End of the highlighting, which a `me=`/`he=` offset can pull in front
    /// of the match end.
    pub(crate) hl_endpos: lpos_T,
    /// End of the END pattern's own match, for the `matchgroup=` item that
    /// highlights it.
    pub(crate) eoe_pos: lpos_T,
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
        let zero = lpos_T { lnum: 0, col: 0 };
        RegionEnd {
            m_endpos: zero,
            hl_endpos: zero,
            eoe_pos: zero,
            end_idx: 0,
            flags: None,
        }
    }
}

/// The `synpat_T` at `idx` in the current syntax block's pattern array.
///
/// Run pattern `idx`'s program over `lnum` from `col`, into a fresh match.
///
/// The engine may hand back a *different* program (`vim_regexec_multi` can
/// recompile), so the answer is written back into the pattern, and each
/// pattern is timed into its own `sp_time`.
pub(crate) unsafe fn run_pattern(idx: c_int, lnum: linenr_T, col: colnr_T) -> (bool, regmmatch_T) {
    let mut regmatch = empty_regmmatch();
    let mut block = syn_block();
    let spp = block.pattern_mut(idx);
    regmatch.rmm_ic = spp.sp_ic;
    regmatch.regprog = spp.sp_prog;
    let time: *mut syn_time_T = &raw mut spp.sp_time;
    // SAFETY: `time` is this pattern's own timer, live across the call, and
    // `regmatch` is the local just built.
    let matched = unsafe { syn_regexec(&raw mut regmatch, lnum, col, time) };
    block.pattern_mut(idx).sp_prog = regmatch.regprog;
    (matched, regmatch)
}

/// Number of patterns in the current syntax block.
#[inline(always)]
pub(crate) fn syn_pattern_count() -> c_int {
    syn_block().patterns().len() as c_int
}

/// Find the end of the start/skip/end region `idx` after `startpos`.
///
/// Only looks in `startpos.lnum`; if no END pattern matches there the region
/// continues into the next line and the answer is [`RegionEnd::none`]. Also
/// handles a match item that continued from a previous line.
///
/// `start_ext` are the submatches of the START pattern, which the END and SKIP
/// patterns may refer to with `\1`..`\9`.
pub(crate) unsafe fn find_endpos(
    mut idx: c_int,
    startpos: lpos_T,
    start_ext: *mut reg_extmatch_T,
) -> RegionEnd {
    // Just in case we are invoked for a keyword.
    if idx < 0 {
        return RegionEnd::none();
    }

    // Check for being called with a START pattern. Can happen with a match
    // that continues to the next line because it contained a region.
    // Upstream answers `hl_endpos = startpos` here, which no caller reads:
    // both test `m_endpos.lnum` first, and it is 0.
    if syn_block().pattern(idx).sp_type as c_int != SPTYPE_START {
        let mut end = RegionEnd::none();
        end.hl_endpos = startpos;
        return end;
    }

    // Find the SKIP or first END pattern after the last START pattern. The
    // patterns of one `:syntax region` are stored consecutively, START(s)
    // then an optional SKIP then the ENDs.
    while syn_block().pattern(idx).sp_type as c_int == SPTYPE_START {
        idx += 1;
    }
    let skip_idx = if syn_block().pattern(idx).sp_type as c_int == SPTYPE_SKIP {
        idx += 1;
        Some(idx - 1)
    } else {
        None
    };

    // Set up the external matches for syn_regexec().
    unsafe { unref_extmatch(re_extmatch_in.get()) };
    re_extmatch_in.set(unsafe { ref_extmatch(start_ext) });

    let mut buf_chartab = [0u64; 4];
    save_chartab(&mut buf_chartab);

    let start_idx = idx;
    let mut matchcol = startpos.col;
    let answer = unsafe { find_endpos_scan(start_idx, skip_idx, startpos, &mut matchcol) };

    restore_chartab(&buf_chartab);
    unsafe { unref_extmatch(re_extmatch_in.get()) };
    re_extmatch_in.set(::core::ptr::null_mut());
    answer
}

/// The search loop of [`find_endpos`]: try every END pattern from `start_idx`,
/// step `matchcol` over anything the SKIP pattern claims, and repeat.
unsafe fn find_endpos_scan(
    start_idx: c_int,
    skip_idx: Option<c_int>,
    startpos: lpos_T,
    matchcol: &mut colnr_T,
) -> RegionEnd {
    loop {
        let Some((best_idx, best)) = (unsafe { best_end_match(start_idx, startpos, *matchcol) })
        else {
            // All end patterns tried with no match: the item continues
            // until end-of-line.
            return RegionEnd::none();
        };
        if let Some(skip_idx) = skip_idx {
            match unsafe { skip_past(skip_idx, startpos, best.startpos[0], *matchcol) } {
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
        return unsafe { end_positions(best_idx, &best, startpos) };
    }
}

/// The END pattern that matches first at or after `matchcol`, with its match.
unsafe fn best_end_match(
    start_idx: c_int,
    startpos: lpos_T,
    matchcol: colnr_T,
) -> Option<(c_int, regmmatch_T)> {
    let mut best: Option<(c_int, regmmatch_T)> = None;
    let mut idx = start_idx;
    while idx < syn_pattern_count() {
        let spp = syn_block();
        let spp = spp.pattern(idx);
        if spp.sp_type as c_int != SPTYPE_END {
            break; // past the last END pattern of this region
        }
        let lc_col = (matchcol as c_int - spp.sp_offsets[SPO_LC_OFF as usize]).max(0);

        let (matched, regmatch) = unsafe { run_pattern(idx, startpos.lnum, lc_col as colnr_T) };
        let col = regmatch.startpos[0].col;
        if matched && best.as_ref().is_none_or(|(_, b)| col < b.startpos[0].col) {
            best = Some((idx, regmatch));
        }
        idx += 1;
    }
    best
}

/// What the SKIP pattern did to the search position.
enum Skipped {
    /// It did not match before the end pattern; use the end pattern.
    No,
    /// It ran to the next line, or included the end of this one.
    PastLine,
    /// Resume the end-pattern search at this column.
    To(colnr_T),
}

/// Does the SKIP pattern match before the best END pattern's match?
unsafe fn skip_past(
    skip_idx: c_int,
    startpos: lpos_T,
    best_start: lpos_T,
    matchcol: colnr_T,
) -> Skipped {
    let offsets = syn_block().pattern(skip_idx).offsets();
    let lc_col = (matchcol as c_int - offsets.offsets[SPO_LC_OFF as usize]).max(0);
    let (matched, regmatch) = unsafe { run_pattern(skip_idx, startpos.lnum, lc_col as colnr_T) };
    if !matched || regmatch.startpos[0].col > best_start.col {
        return Skipped::No;
    }

    // Add the offset to the skip pattern's match.
    let pos = unsafe { syn_add_end_off(offsets, &regmatch, SPO_ME_OFF, 1) };
    if pos.lnum > startpos.lnum {
        // The skip pattern goes on to the next line, so there is no match
        // with an end pattern in this line.
        return Skipped::PastLine;
    }
    let line_len = unsafe { ml_get_buf_len(syn_buf.get(), startpos.lnum) };

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
unsafe fn end_positions(best_idx: c_int, best: &regmmatch_T, startpos: lpos_T) -> RegionEnd {
    let block = syn_block();
    let spp = block.pattern(best_idx);
    let offsets = spp.offsets();

    // Match from the start pattern to the end pattern, corrected for the
    // end pattern's match and highlight offsets. Neither may end before
    // the start.
    let mut m_endpos = unsafe { syn_add_end_off(offsets, best, SPO_ME_OFF, 1) };
    if m_endpos.lnum == startpos.lnum && m_endpos.col < startpos.col {
        m_endpos.col = startpos.col;
    }
    let mut eoe_pos = unsafe { syn_add_end_off(offsets, best, SPO_HE_OFF, 1) };
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
            let mut hl_endpos = lpos_T {
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

/// A zeroed `regmmatch_T`, which `vim_regexec_multi` fills in.
pub(crate) const fn empty_regmmatch() -> regmmatch_T {
    regmmatch_T {
        regprog: ::core::ptr::null_mut(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    }
}

/// Limit `pos` not to be after `limit`.
pub(crate) fn limit_pos(pos: &mut lpos_T, limit: lpos_T) {
    if pos.lnum > limit.lnum {
        *pos = limit;
    } else if pos.lnum == limit.lnum && pos.col > limit.col {
        pos.col = limit.col;
    }
}

/// [`limit_pos`], but a `pos` of line 0 -- "not set" -- takes the limit.
pub(crate) fn limit_pos_zero(pos: &mut lpos_T, limit: lpos_T) {
    if pos.lnum == 0 {
        *pos = limit;
    } else {
        limit_pos(pos, limit);
    }
}

/// Apply the `me=`/`he=`/`re=` end offset `idx` of `spp` to `regmatch`.
///
/// `extra` is added when the offset is measured from the *start* of the match
/// (`me=s+1`), which is how "one past" is spelled for a region's end.
pub(crate) unsafe fn syn_add_end_off(
    spp: PatOffsets,
    regmatch: &regmmatch_T,
    idx: c_int,
    extra: c_int,
) -> lpos_T {
    let flagged = spp.flags as c_int & (1 << idx) != 0;
    let base = if flagged {
        regmatch.startpos[0]
    } else {
        regmatch.endpos[0]
    };
    let off = spp.offsets[idx as usize] + if flagged { extra } else { 0 };

    let col = if base.lnum > syn_buf_line_count() {
        // Watch out for a match with the last NL in the buffer. Matters for
        // "rs=e+2" when there is a matchgroup.
        0
    } else {
        unsafe { walk_chars(base.lnum, base.col, off) }
    };
    lpos_T {
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
pub(crate) unsafe fn syn_add_start_off(
    spp: PatOffsets,
    regmatch: &regmmatch_T,
    idx: c_int,
    extra: c_int,
) -> lpos_T {
    let flagged = spp.flags as c_int & (1 << (idx + SPO_COUNT)) != 0;
    let base = if flagged {
        regmatch.endpos[0]
    } else {
        regmatch.startpos[0]
    };
    let off = spp.offsets[idx as usize] + if flagged { extra } else { 0 };

    let (lnum, col) = if base.lnum > syn_buf_line_count() {
        // A "\n" at the end of the pattern may take us below the last line.
        let lnum = syn_buf_line_count();
        (lnum, unsafe { ml_get_buf_len(syn_buf.get(), lnum) })
    } else {
        (base.lnum, base.col)
    };
    lpos_T {
        lnum,
        col: unsafe { walk_chars(lnum, col, off) },
    }
}

/// Step `off` characters forward (or backward) from `col` in line `lnum`,
/// stopping at the line's ends. Answers the resulting column.
unsafe fn walk_chars(lnum: linenr_T, col: colnr_T, off: c_int) -> colnr_T {
    if off == 0 {
        return col;
    }
    let base = unsafe { ml_get_buf(syn_buf.get(), lnum) };
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
    unsafe { p.offset_from(base) as colnr_T }
}

/// The current line of the syntax buffer.
///
/// NOTE: the *bytes* are invalid after anything that can look for a pattern
/// match -- the regexp engine may reach `ml_get_buf` for another line and
/// evict this one. Reading them is the caller's unsafe step; asking for the
/// pointer is not.
pub(crate) fn syn_getcurline() -> *mut c_char {
    // SAFETY: `syn_buf` is the buffer `syntax_start` pointed the parser at,
    // and `current_lnum` a line of it.
    unsafe { ml_get_buf(syn_buf.get(), current_lnum.get()) }
}

/// Length of the current line of the syntax buffer.
pub(crate) fn syn_getcurline_len() -> colnr_T {
    // SAFETY: as [`syn_getcurline`].
    unsafe { ml_get_buf_len(syn_buf.get(), current_lnum.get()) }
}

/// The byte at `col` of the line being parsed.
///
/// Every caller is testing for the NUL that ends the line, so `col` is at
/// most its length and the read stays inside what `ml_get_buf` answered.
pub(crate) fn syn_curline_byte(col: colnr_T) -> u8 {
    debug_assert!(col <= syn_getcurline_len());
    // SAFETY: `col` is within the line, its terminator included.
    unsafe { *syn_getcurline().offset(col as isize) as u8 }
}

/// Number of lines in the buffer being parsed.
pub(crate) fn syn_buf_line_count() -> linenr_T {
    // SAFETY: `syn_buf` is the buffer `syntax_start` pointed the parser at.
    unsafe { (*syn_buf.get()).b_ml.ml_line_count }
}

/// `vim_regexec_multi` in the syntax buffer, timed when `:syntime` is on.
///
/// Answers whether there was a match, and on a match shifts `rmp`'s positions
/// from pattern-relative to buffer-absolute line numbers.
pub(crate) unsafe fn syn_regexec(
    rmp: *mut regmmatch_T,
    lnum: linenr_T,
    col: colnr_T,
    st: *mut syn_time_T,
) -> bool {
    let timing = syn_time_on.get();
    let start = if timing { profile_start() } else { 0 };

    if unsafe { (*rmp).regprog }.is_null() {
        // A previous vim_regexec_multi() tried the NFA engine, got
        // NFA_TOO_EXPENSIVE, and compiling with the other engine failed.
        return false;
    }
    unsafe { (*rmp).rmm_maxcol = (*syn_buf.get()).b_p_smc as colnr_T };
    let mut timed_out: c_int = 0;
    let (win, buf, tm) = (syn_win.get(), syn_buf.get(), syn_tm.get());
    // SAFETY: the window and buffer the parser was started for.
    let r = unsafe { vim_regexec_multi(rmp, win, buf, lnum, col, tm, &raw mut timed_out) };

    if timing {
        let took = profile_end(start);
        unsafe { (*st).total = profile_add((*st).total, took) };
        // `profile_cmp(a, b)` is negative when `a` is the *larger* time, so
        // this really does keep the slowest.
        if profile_cmp(took, unsafe { (*st).slowest }) < 0 {
            unsafe { (*st).slowest = took };
        }
        unsafe { (*st).count += 1 };
        if r > 0 {
            unsafe { (*st).match_0 += 1 };
        }
    }
    if timed_out != 0 && !unsafe { (*(*syn_win.get()).w_s).b_syn_slow } {
        unsafe { (*(*syn_win.get()).w_s).b_syn_slow = true };
        msg(
            gettext(c"'redrawtime' exceeded, syntax highlighting disabled"),
            0,
        );
    }

    if r > 0 {
        unsafe { (*rmp).startpos[0].lnum += lnum };
        unsafe { (*rmp).endpos[0].lnum += lnum };
        return true;
    }
    false
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

/// Check one position in a line for a matching keyword.
///
/// The caller must have established that a keyword can start at `startcol`.
pub(crate) unsafe fn check_keyword_id(
    line: *mut c_char,
    startcol: c_int,
    cur_si: Option<Item>,
) -> Option<KeywordMatch> {
    // Find the first character after the keyword; the first character was
    // already checked by the caller.
    let kwp = unsafe { line.offset(startcol as isize) };
    let mut kwlen: c_int = 0;
    loop {
        kwlen += unsafe { utfc_ptr2len(kwp.offset(kwlen as isize)) };
        if !unsafe { vim_iswordp_buf(kwp.offset(kwlen as isize), syn_buf.get()) } {
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

    let mut kp = ::core::ptr::null_mut::<keyentry_T>();
    if syn_block().b_keywtab.ht_used != 0 {
        kp = unsafe { match_keyword(buf, syn_field!(syn_block(), b_keywtab), cur_si) };
    }
    if kp.is_null() && syn_block().b_keywtab_ic.ht_used != 0 {
        unsafe { str_foldcase(kwp, kwlen, buf, MAXKEYWLEN + 1) };
        kp = unsafe { match_keyword(buf, syn_field!(syn_block(), b_keywtab_ic), cur_si) };
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
/// chained through `ke_next`. `current_next_list` (a pending `nextgroup=`)
/// overrides everything; otherwise a keyword is accepted at the top level when
/// it is not `contained`, and inside an item when that item's `contains=` list
/// names it.
unsafe fn match_keyword(
    keyword: *mut c_char,
    ht: *mut hashtab_T,
    cur_si: Option<Item>,
) -> *mut keyentry_T {
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
            .offset(-(::core::mem::offset_of!(keyentry, keyword) as isize))
    } as *mut keyentry_T;
    while !kp.is_null() {
        // SAFETY: `kp` walks a chain of live keyword entries.
        let (syn, cont_in, flags) = unsafe { ((*kp).k_syn, (*kp).cont_in_list, (*kp).flags) };
        let ok = if !current_next_list.get().is_null() {
            let next = current_next_list.get();
            // SAFETY: the parser's own lists.
            unsafe { in_id_list(None, next, syn, cont_in, SynFlags::NONE) }
        } else if let Some(cur_si) = cur_si {
            let contains = cur_si.si_cont_list;
            // SAFETY: as above, inside the item the caller named.
            unsafe { in_id_list(Some(cur_si), contains, syn, cont_in, flags) }
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
