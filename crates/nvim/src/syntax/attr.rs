//! The per-cell attribute lookup — `syn_current_attr`.
//!
//! [`get_syntax_attr`] moves the state machine to a column and answers the
//! highlight attribute for it; [`syn_current_attr`] is the step that does the
//! work. It repeatedly looks for a keyword and then for a pattern that can match
//! at the current column and is admitted by the containment rules, pushes what
//! it finds onto the state stack, and finally walks the stack down to the
//! innermost item whose highlight range covers the column.
//!
//! This is the hottest path in the module: it runs once per cell of every
//! highlighted line.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

/// Highlight attributes for the character at `col`.
///
/// [`syntax_start`] must have been called for the line first. `col` is normally
/// 0 for the first use in a line and increments by one each time; skipping
/// characters and stopping before the end of the line are both allowed, but
/// `col` may never go backwards.
///
/// `keep_state` keeps the state stack as it stands at `col` rather than closing
/// the items that end there, which is what `synstack()` needs.
pub(crate) unsafe fn get_syntax_attr(
    col: colnr_T,
    can_spell: *mut bool,
    keep_state: bool,
) -> c_int {
    if !can_spell.is_null() {
        unsafe { *can_spell = default_can_spell() };
    }
    if syn_block().b_sst_array.is_null() {
        return 0; // out of memory
    }

    // After 'synmaxcol' the attribute is always zero.
    if unsafe { (*syn_buf.get()).b_p_smc } > 0
        && col >= unsafe { (*syn_buf.get()).b_p_smc } as colnr_T
    {
        clear_current_state();
        current_id.set(0);
        current_trans_id.set(0);
        current_flags.set(SynFlags::NONE);
        current_seqnr.set(0);
        return 0;
    }
    if !current_state_valid() {
        validate_current_state();
    }

    // Skip from the current column to "col", answering the attributes there.
    let mut attr = 0;
    while current_col.get() <= col {
        let last = current_col.get() == col;
        attr = unsafe { syn_current_attr(false, true, can_spell, last && keep_state) };
        current_col.set(current_col.get() + 1);
    }
    attr
}

/// Whether spell checking is done outside every syntax item: only when there is
/// no `@Spell` cluster, or when `:syntax spell toplevel` was used.
fn default_can_spell() -> bool {
    let mut block = syn_block();
    if block.b_syn_spell == SYNSPL_DEFAULT {
        block.b_spell_cluster_id == 0
    } else {
        block.b_syn_spell == SYNSPL_TOP
    }
}

/// Syntax attributes for `current_lnum`/`current_col`, advancing the state
/// stack to that column.
///
/// `syncing` restricts matching to `:syntax sync` items; `displaying` says the
/// answer will be drawn, which admits `display` items; `keep_state` leaves the
/// items that end here on the stack.
pub(crate) unsafe fn syn_current_attr(
    syncing: bool,
    displaying: bool,
    can_spell: *mut bool,
    keep_state: bool,
) -> c_int {
    // Must try again in the next column when the previous column found a
    // match it could not use (an empty match, or one already matched here).
    static try_next_column: GlobalCell<bool> = GlobalCell::new(false);

    // No character, no attributes -- past end of line? Do try matching an
    // empty line, which could be the start of a region.
    let line = syn_getcurline();
    if unsafe { *line.offset(current_col.get() as isize) } as c_int == NUL && current_col.get() != 0
    {
        // If we found a match after the last column, use it.
        if next_match_idx.get() >= 0
            && next_match_col.get() >= current_col.get()
            && next_match_col.get() != MAXCOL as c_int
        {
            push_next_match();
        }
        current_finished.set(true);
        current_state_stored.set(false);
        return 0;
    }
    // If the current or the next character is NUL, this finishes the line.
    if unsafe { *line.offset(current_col.get() as isize) } as c_int == NUL
        || unsafe { *line.offset(current_col.get() as isize + 1) } as c_int == NUL
    {
        current_finished.set(true);
        current_state_stored.set(false);
    }
    if try_next_column.get() {
        next_match_idx.set(-1);
        try_next_column.set(false);
    }

    // Only check for keywords when not syncing and there are some.
    let do_keywords =
        !syncing && (syn_block().b_keywtab.ht_used > 0 || syn_block().b_keywtab_ic.ht_used > 0);

    // Zero-width matches with a nextlist already used here, so the same one
    // cannot match twice in this column and loop forever.
    let mut zero_width: Vec<c_int> = Vec::new();

    // Use the `syntax iskeyword` option while matching.
    let mut buf_chartab = [0u64; 4];
    save_chartab(&mut buf_chartab);

    let mut cur_extmatch: *mut reg_extmatch_T = ::core::ptr::null_mut();
    let mut zero_width_next_list = false;
    let mut cur_si: Option<Item>;

    // Repeat matching keywords and patterns to find contained items at the
    // same column. Stops when there is no extra match here.
    loop {
        let mut found_match = false;
        let mut keep_next_list = false;
        let mut found_keyword = false;

        // 1. Only when there is no current state, or the current state may
        //    contain other things, do we need to look for keywords and
        //    patterns. Always look for contained items when some item has
        //    a `containedin=` (which costs extra time).
        cur_si = if state_len() != 0 {
            Some(unsafe { state_top() })
        } else {
            None
        };
        if syn_block().b_syn_containedin != 0 || cur_si.is_none_or(|si| !si.si_cont_list.is_null())
        {
            // 2. A keyword, if we are on a keyword character after a
            //    non-keyword one. Never while syncing.
            if do_keywords && let Some(si) = try_keyword(cur_si) {
                cur_si = Some(si);
                found_keyword = true;
            }

            // 3. A pattern, only if no keyword was found.
            if !found_keyword && syn_pattern_count() != 0 {
                // If we have not looked yet, or we are past what we found,
                // look for a match with any pattern.
                if next_match_idx.get() < 0 || next_match_col.get() < current_col.get() {
                    let (zw, ext) = (&zero_width, &mut cur_extmatch);
                    // SAFETY: the parser's own state.
                    unsafe {
                        scan_patterns(syncing, displaying, cur_si, zw, ext, &try_next_column)
                    };
                }
                // If we found a match at the current column, use it.
                if next_match_idx.get() >= 0 && next_match_col.get() == current_col.get() {
                    let block = syn_block();
                    let lspp = block.pattern(next_match_idx.get());
                    if next_match_m_endpos.get().lnum == current_lnum.get()
                        && next_match_m_endpos.get().col == current_col.get()
                        && !lspp.sp_next_list.is_none()
                    {
                        // A zero-width item with a nextgroup: do not push
                        // it, just set the nextgroup -- and remember it, so
                        // it cannot match here again.
                        current_next_list.set(lspp.sp_next_list.as_ptr());
                        current_next_flags.set(lspp.sp_flags);
                        keep_next_list = true;
                        zero_width_next_list = true;
                        zero_width.push(next_match_idx.get());
                        next_match_idx.set(-1);
                    } else {
                        cur_si = Some(push_next_match());
                    }
                    found_match = true;
                }
            }
        }

        // Handle searching for a nextgroup match.
        if !current_next_list.get().is_null() && !keep_next_list {
            // If a nextgroup was not found, keep looking for one when this
            // is an empty line and "skipempty" was given, or we are on
            // white space and "skipwhite" was given.
            if !found_match {
                let line = syn_getcurline();
                let white = current_next_flags.get().has(SynFlags::SKIPWHITE)
                    && ascii_iswhite(unsafe { *line.offset(current_col.get() as isize) } as c_int);
                let empty = current_next_flags.get().has(SynFlags::SKIPEMPTY)
                    && unsafe { *line } as c_int == NUL;
                if white || empty {
                    break;
                }
            }
            // Found: use it and keep looking for contained matches. Not
            // found: keep looking for a normal match. When the nextgroup
            // came from a zero-width item and nothing matched, do not loop
            // -- we would get stuck.
            current_next_list.set(::core::ptr::null_mut());
            next_match_idx.set(-1);
            if !zero_width_next_list {
                found_match = true;
            }
        }
        if !found_match {
            break;
        }
    }

    restore_chartab(&buf_chartab);

    let sip = pick_current_attr(cur_si);
    if !can_spell.is_null() {
        unsafe {
            *can_spell = match sip {
                Some(sip) if cur_si.is_some() => item_can_spell(sip),
                _ => default_can_spell(),
            }
        };
    }
    if cur_si.is_some() && !syncing && !keep_state {
        // Check whether the current state -- and the states before it --
        // end at the next column. Not while syncing: we would miss a
        // single-character match. The current column is checked first: the
        // item here may be an empty match, and a containing item might end
        // in this column too.
        check_state_ends();
        if state_len() > 0 && syn_curline_byte(current_col.get()) as c_int != NUL {
            current_col.set(current_col.get() + 1);
            check_state_ends();
            current_col.set(current_col.get() - 1);
        }
    }

    // A nextgroup ends at end of line, unless "skipnl" or "skipempty".
    if !current_next_list.get().is_null()
        && !current_next_flags
            .get()
            .has(SynFlags::SKIPNL | SynFlags::SKIPEMPTY)
    {
        let line = syn_getcurline();
        if unsafe { *line.offset(current_col.get() as isize) } as c_int != NUL
            && unsafe { *line.offset(current_col.get() as isize + 1) } as c_int == NUL
        {
            current_next_list.set(::core::ptr::null_mut());
        }
    }

    // No longer need the external matches -- but keep next_match_extmatch.
    unsafe { unref_extmatch(re_extmatch_out.get()) };
    re_extmatch_out.set(::core::ptr::null_mut());
    unsafe { unref_extmatch(cur_extmatch) };

    current_attr.get()
}

/// Try to match a keyword at the current column and push it if one matches.
///
/// Answers the pushed item, or `None` when the column does not start a keyword
/// or no keyword matches there.
fn try_keyword(cur_si: Option<Item>) -> Option<Item> {
    // Only on a keyword character that follows a non-keyword one.
    let line = syn_getcurline();
    let cur_pos = unsafe { line.offset(current_col.get() as isize) };
    if !unsafe { vim_iswordp_buf(cur_pos, syn_buf.get()) } {
        return None;
    }
    if current_col.get() != 0 {
        let prev = unsafe { cur_pos.offset(-1) };
        // SAFETY: `prev` is inside the line the parser is on.
        let head = unsafe { prev.offset(-(utf_head_off(line, prev) as isize)) };
        // SAFETY: the buffer the parser was started for.
        if unsafe { vim_iswordp_buf(head, syn_buf.get()) } {
            return None;
        }
    }
    let kw = unsafe { check_keyword_id(line, current_col.get(), cur_si) }?;

    push_current_state(KEYWORD_IDX);
    let mut si = unsafe { state_top() };
    si.si_m_startcol = current_col.get();
    si.si_h_startpos.lnum = current_lnum.get();
    si.si_h_startpos.col = 0; // starts right away
    si.si_m_endpos.lnum = current_lnum.get();
    si.si_m_endpos.col = kw.endcol;
    si.si_h_endpos.lnum = current_lnum.get();
    si.si_h_endpos.col = kw.endcol;
    si.si_ends = 1;
    si.si_end_idx = 0;
    si.si_flags = kw.flags;
    si.si_seqnr = next_seqnr.get();
    next_seqnr.set(next_seqnr.get() + 1);
    si.si_cchar = kw.cchar;
    if state_len() > 1 {
        unsafe { si.si_flags |= state_at(state_len() - 2).si_flags.masked(SynFlags::CONCEAL) };
    }
    si.si_id = kw.id;
    si.si_trans_id = kw.id;
    if kw.flags.has(SynFlags::TRANSP) {
        // Transparent: take the attributes of the item around it.
        if state_len() < 2 {
            si.si_attr = 0;
            si.si_trans_id = 0;
        } else {
            let outer = unsafe { state_at(state_len() - 2) };
            si.si_attr = outer.si_attr;
            si.si_trans_id = outer.si_trans_id;
        }
    } else {
        unsafe { si.si_attr = syn_id2attr(kw.id) };
    }
    si.si_cont_list = ::core::ptr::null_mut();
    si.si_next_list = kw.next_list;
    check_keepend();
    Some(si)
}

/// Look for the pattern that matches earliest at or after the current column
/// and record it in the `next_match_*` globals.
///
/// Matching with a pattern takes a good deal of time, so this remembers per
/// pattern where it last matched in this line (`sp_startcol`/`sp_line_id`) and
/// skips any pattern that cannot beat the best match so far.
unsafe fn scan_patterns(
    syncing: bool,
    displaying: bool,
    cur_si: Option<Item>,
    zero_width: &[c_int],
    cur_extmatch: &mut *mut reg_extmatch_T,
    try_next_column: &GlobalCell<bool>,
) {
    next_match_idx.set(0); // no match in this line yet
    next_match_col.set(MAXCOL as c_int);

    let mut idx = syn_pattern_count();
    while idx > 0 {
        idx -= 1;
        // Everything the loop needs is copied out first: `find_endpos`
        // below reaches the pattern array again and writes into it, so no
        // borrow of it may be live across this body.
        let scan = PatScan::of(idx);
        if !unsafe { pattern_admitted(&scan, cur_si, syncing, displaying) } {
            continue;
        }
        // Already tried in this line, and it cannot match before the best
        // match so far.
        if scan.line_id == current_line_id.get() && scan.startcol >= next_match_col.get() {
            continue;
        }
        syn_block().pattern_mut(idx).sp_line_id = current_line_id.get();

        let lc_col = (current_col.get() - scan.offsets.offsets[SPO_LC_OFF as usize]).max(0);
        let lnum = current_lnum.get();
        // SAFETY: the parser's own pattern, timed into its own `sp_time`.
        let (matched, regmatch) = unsafe { run_pattern(idx, lnum, lc_col) };
        if !matched {
            // No match in this line; try another pattern.
            syn_block().pattern_mut(idx).sp_startcol = MAXCOL as c_int;
            continue;
        }

        // The first column of the match.
        let pos = unsafe { syn_add_start_off(scan.offsets, &regmatch, SPO_MS_OFF, -1) };
        if pos.lnum > current_lnum.get() {
            // Must have used the end of the match in a following line,
            // which we cannot handle.
            syn_block().pattern_mut(idx).sp_startcol = MAXCOL as c_int;
            continue;
        }
        let startcol = pos.col;
        // Remember the next column where this pattern matches in this line.
        syn_block().pattern_mut(idx).sp_startcol = startcol;
        // A previously found match starts earlier: keep that one.
        if startcol >= next_match_col.get() {
            continue;
        }
        // Matched this pattern here before: skip it, and retry in the next
        // column, because it may match from there.
        if did_match_already(idx, zero_width) {
            try_next_column.set(true);
            continue;
        }

        let mut endpos = regmatch.endpos[0];
        let mut hl_startpos = unsafe { syn_add_start_off(scan.offsets, &regmatch, SPO_HS_OFF, -1) };
        // The region start defaults to the end of the start match.
        let eos_pos = unsafe { syn_add_end_off(scan.offsets, &regmatch, SPO_RS_OFF, 0) };

        // Grab the external submatches before they get overwritten. The
        // reference count does not change.
        unsafe { unref_extmatch(*cur_extmatch) };
        *cur_extmatch = re_extmatch_out.get();
        re_extmatch_out.set(::core::ptr::null_mut());

        let mut flags = SynFlags::NONE;
        let mut eoe_pos = lpos_T { lnum: 0, col: 0 };
        let mut end_idx = 0;
        let mut hl_endpos = lpos_T { lnum: 0, col: 0 };

        if scan.ty == SPTYPE_START && scan.flags.has(SynFlags::ONELINE) {
            // A "oneline" must end in this line too. Look for the end after
            // the start match, and set every resulting position at once.
            let end = unsafe { find_endpos(idx, endpos, *cur_extmatch) };
            if end.m_endpos.lnum == 0 {
                continue; // not found
            }
            endpos = end.m_endpos;
            hl_endpos = end.hl_endpos;
            eoe_pos = end.eoe_pos;
            end_idx = end.end_idx;
            if let Some(f) = end.flags {
                flags = f;
            }
        } else if scan.ty == SPTYPE_MATCH {
            // For a "match" the size must be > 0 once the end offset has
            // been added -- except when syncing.
            hl_endpos = unsafe { syn_add_end_off(scan.offsets, &regmatch, SPO_HE_OFF, 0) };
            endpos = unsafe { syn_add_end_off(scan.offsets, &regmatch, SPO_ME_OFF, 0) };
            if endpos.lnum == current_lnum.get() && endpos.col + c_int::from(syncing) < startcol {
                // An empty match: may need to try again in the next column.
                if regmatch.startpos[0].col == regmatch.endpos[0].col {
                    try_next_column.set(true);
                }
                continue;
            }
        }

        // Keep the best match so far. Highlighting must start after
        // startpos and end before endpos.
        if hl_startpos.lnum == current_lnum.get() && hl_startpos.col < startcol {
            hl_startpos.col = startcol;
        }
        limit_pos_zero(&mut hl_endpos, endpos);

        next_match_idx.set(idx);
        next_match_col.set(startcol);
        next_match_m_endpos.set(endpos);
        next_match_h_endpos.set(hl_endpos);
        next_match_h_startpos.set(hl_startpos);
        next_match_flags.set(flags);
        next_match_eos_pos.set(eos_pos);
        next_match_eoe_pos.set(eoe_pos);
        next_match_end_idx.set(end_idx);
        unsafe { unref_extmatch(next_match_extmatch.get()) };
        next_match_extmatch.set(*cur_extmatch);
        *cur_extmatch = ::core::ptr::null_mut();
    }
}

/// What [`scan_patterns`] needs from one pattern, copied out.
///
/// The loop calls `find_endpos`, which reaches the pattern array again and
/// writes into it, so no borrow of the array may be live across the body.
struct PatScan {
    syncing: bool,
    flags: SynFlags,
    ty: c_int,
    line_id: c_int,
    startcol: c_int,
    syn: sp_syn,
    /// The pattern's `containedin=` list, borrowed. The pattern owns it and
    /// nothing here can free it.
    cont_in_list: *mut int16_t,
    offsets: PatOffsets,
}

impl PatScan {
    fn of(idx: c_int) -> PatScan {
        let block = syn_block();
        let spp = block.pattern(idx);
        PatScan {
            syncing: spp.sp_syncing,
            flags: spp.sp_flags,
            ty: spp.sp_type as c_int,
            line_id: spp.sp_line_id,
            startcol: spp.sp_startcol,
            syn: spp.sp_syn,
            cont_in_list: spp.sp_cont_in_list.as_ptr(),
            offsets: spp.offsets(),
        }
    }
}

/// Can pattern `spp` match here at all: is it the right kind of item, and do
/// the containment rules admit it?
///
/// This is one `if (A && B && C && D)` upstream, and every operand short
/// circuits: `in_id_list` is the expensive one and runs last.
#[inline]
unsafe fn pattern_admitted(
    spp: &PatScan,
    cur_si: Option<Item>,
    syncing: bool,
    displaying: bool,
) -> bool {
    if spp.syncing != syncing {
        return false;
    }
    if !displaying && spp.flags.has(SynFlags::DISPLAY) {
        return false;
    }
    if spp.ty != SPTYPE_MATCH && spp.ty != SPTYPE_START {
        return false;
    }
    if !current_next_list.get().is_null() {
        // A pending `nextgroup=` admits only what it names.
        let next = current_next_list.get();
        // SAFETY: the caller's pattern and the parser's own lists.
        unsafe { in_id_list(None, next, spp.syn, spp.cont_in_list, SynFlags::NONE) }
    } else if let Some(cur_si) = cur_si {
        // Inside an item, only what its `contains=` names.
        let contains = cur_si.si_cont_list;
        // SAFETY: as above, inside the item the caller named.
        unsafe { in_id_list(Some(cur_si), contains, spp.syn, spp.cont_in_list, spp.flags) }
    } else {
        // At the top level, anything that is not `contained`.
        !spp.flags.has(SynFlags::CONTAINED)
    }
}

/// Publish the attributes of the innermost item whose highlight range covers
/// the current column into the `current_*` globals, and answer that item.
///
/// Answers `None` when `cur_si` is `None`, i.e. nothing matched here at all.
fn pick_current_attr(cur_si: Option<Item>) -> Option<Item> {
    current_attr.set(0);
    current_id.set(0);
    current_trans_id.set(0);
    current_flags.set(SynFlags::NONE);
    current_seqnr.set(0);
    cur_si?;
    // Use the attributes of the innermost item if we are inside its
    // highlighting; if not, of the item around it, and so on.
    let mut walked = None;
    let mut idx = state_len() - 1;
    while idx >= 0 {
        let sip = unsafe { state_at(idx) };
        walked = Some(sip);
        let started = current_lnum.get() > sip.si_h_startpos.lnum
            || (current_lnum.get() == sip.si_h_startpos.lnum
                && current_col.get() >= sip.si_h_startpos.col);
        let not_ended = sip.si_h_endpos.lnum == 0
            || current_lnum.get() < sip.si_h_endpos.lnum
            || (current_lnum.get() == sip.si_h_endpos.lnum
                && current_col.get() < sip.si_h_endpos.col);
        if started && not_ended {
            current_attr.set(sip.si_attr);
            current_id.set(sip.si_id);
            current_trans_id.set(sip.si_trans_id);
            current_flags.set(sip.si_flags);
            current_seqnr.set(sip.si_seqnr);
            current_sub_char.set(sip.si_cchar);
            break;
        }
        idx -= 1;
    }
    // When no item covered the column this is the outermost one the walk
    // touched -- upstream's `sip` after the loop, which the spell test
    // below reads `si_cont_list` from. Kept exactly, including the `None`
    // it holds when the stack is empty.
    walked
}

/// Whether spell checking should be done in the item the attribute walk left
/// in `sip`.
fn item_can_spell(sip: Item) -> bool {
    let mut block = syn_block();
    let mut sps = sp_syn { inc_tag: 0, id: 0 };
    // The two cluster ids are looked up as bare groups: no `containedin=`.
    let no_cont_in = ::core::ptr::null_mut();
    if block.b_spell_cluster_id == 0 {
        // There is no @Spell cluster: spell check items without a @NoSpell
        // cluster.
        if block.b_nospell_cluster_id == 0 || current_trans_id.get() == 0 {
            return block.b_syn_spell != SYNSPL_NOTOP;
        }
        sps.id = block.b_nospell_cluster_id as int16_t;
        // SAFETY: the parser's own state stack and lists.
        return !unsafe {
            in_id_list(Some(sip), sip.si_cont_list, sps, no_cont_in, SynFlags::NONE)
        };
    }
    // The @Spell cluster is defined: spell check in items carrying it, but
    // not when @NoSpell is there too. At the top level only spell check
    // when `:syntax spell toplevel` was used.
    if current_trans_id.get() == 0 {
        return block.b_syn_spell == SYNSPL_TOP;
    }
    sps.id = block.b_spell_cluster_id as int16_t;
    // SAFETY: as above.
    let mut can =
        unsafe { in_id_list(Some(sip), sip.si_cont_list, sps, no_cont_in, SynFlags::NONE) };
    if block.b_nospell_cluster_id != 0 {
        sps.id = block.b_nospell_cluster_id as int16_t;
        // SAFETY: as above.
        if unsafe { in_id_list(Some(sip), sip.si_cont_list, sps, no_cont_in, SynFlags::NONE) } {
            can = false;
        }
    }
    can
}

/// Have we already matched pattern `idx` at the current column?
///
/// Two places to look: an item already on the state stack that started here,
/// and the list of zero-width items with a `nextgroup=` used in this column.
pub(crate) fn did_match_already(idx: c_int, gap: &[c_int]) -> bool {
    let mut i = state_len();
    while i > 0 {
        i -= 1;
        let si = unsafe { state_at(i) };
        if si.si_m_startcol == current_col.get()
            && si.si_m_lnum == current_lnum.get()
            && si.si_idx == idx
        {
            return true;
        }
    }
    gap.contains(&idx)
}
