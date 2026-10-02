//! The per-cell attribute lookup — [`SynState::current_attr`].
//!
//! [`get_syntax_attr`] moves the state machine to a column and answers the
//! highlight attribute for it; [`SynState::current_attr`] is the step that
//! does the work. It repeatedly looks for a keyword and then for a pattern
//! that can match at the current column and is admitted by the containment
//! rules, pushes what it finds onto the state stack, and finally walks the
//! stack down to the innermost item whose highlight range covers the column.
//!
//! This is the hottest path in the module: it runs once per cell of every
//! highlighted line.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::c_int;

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

/// Highlight attributes for the character at `col`.
///
/// [`Win::syntax_start`] must have been called for the line first. `col` is
/// normally 0 for the first use in a line and increments by one each time;
/// skipping characters and stopping before the end of the line are both
/// allowed, but `col` may never go backwards.
///
/// `can_spell`, when given, is set to whether spell checking applies there.
/// `keep_state` keeps the state stack as it stands at `col` rather than
/// closing the items that end there, which is what `synstack()` needs.
///
/// `buffer` is the buffer of the window `syntax_start` was given: the caller
/// has it in hand, and passing it spares a registry lookup per column.
///
/// Answers [`get_syntax_info`]'s two as well, which the draw path wants for
/// the same cell: asking for them separately would cost a second trip to the
/// parser per column.
// Out of line: inlined, the parser's take and put-back cost the draw loop
// its register allocation (scrbench +0.04 % in `win_line` alone).
#[inline(never)]
pub(crate) fn get_syntax_attr(
    buffer: Buf,
    col: ColNr,
    can_spell: Option<&mut bool>,
    keep_state: bool,
) -> SyntaxCell {
    with_parser(|parser| {
        debug_assert!(
            parser.buf == buffer.try_id(),
            "`buffer` is the buffer being parsed"
        );
        parser.buffer = buffer;
        let attr = parser.attr_at(col, can_spell, keep_state);
        SyntaxCell {
            attr,
            flags: parser.current.flags,
            seqnr: parser.current.seqnr,
        }
    })
}

/// What [`get_syntax_attr`] answers for one cell.
pub(crate) struct SyntaxCell {
    /// The highlight attribute.
    pub(crate) attr: c_int,
    /// The `HL_*` flags of the item the cell is in.
    pub(crate) flags: SynFlags,
    /// That item's sequence number, which tells two runs of the same group
    /// apart.
    pub(crate) seqnr: c_int,
}

impl SynState {
    /// [`get_syntax_attr`]'s body. Inlined into it, so the per-column call
    /// pays one prologue rather than two.
    #[inline(always)]
    pub(crate) fn attr_at(
        &mut self,
        col: ColNr,
        mut can_spell: Option<&mut bool>,
        keep_state: bool,
    ) -> c_int {
        let Some(block) = self.parsed else {
            return 0; // never started
        };
        if let Some(can_spell) = can_spell.as_deref_mut() {
            *can_spell = default_can_spell(block);
        }
        if !block.b_sst.is_allocated() {
            return 0; // out of memory
        }

        // After 'synmaxcol' the attribute is always zero.
        let synmaxcol = self.buffer.b_p_smc;
        if synmaxcol > 0 && col >= synmaxcol as ColNr {
            self.clear_current_state();
            self.current.id = 0;
            self.current.trans_id = 0;
            self.current.flags = SynFlags::NONE;
            self.current.seqnr = 0;
            return 0;
        }
        if !self.stack_valid {
            self.validate_current_state();
        }

        // Skip from the current column to "col", answering the attributes
        // there.
        let mut attr = 0;
        while self.col <= col {
            let last = self.col == col;
            attr = self.current_attr(false, true, can_spell.as_deref_mut(), last && keep_state);
            self.col += 1;
        }
        attr
    }

    /// Syntax attributes for `lnum`/`col`, advancing the state stack to that
    /// column.
    ///
    /// `syncing` restricts matching to `:syntax sync` items; `displaying`
    /// says the answer will be drawn, which admits `display` items;
    /// `keep_state` leaves the items that end here on the stack.
    pub(crate) fn current_attr(
        &mut self,
        syncing: bool,
        displaying: bool,
        can_spell: Option<&mut bool>,
        keep_state: bool,
    ) -> c_int {
        let block = self.block();

        // No character, no attributes -- past end of line? Do try matching an
        // empty line, which could be the start of a region.
        let line = self.curline();
        // SAFETY: the parser's column is inside its line, terminator
        // included, and nothing has run a pattern since the line was got.
        let here = unsafe { *line.offset(self.col as isize) } as c_int;
        if here == NUL && self.col != 0 {
            // If we found a match after the last column, use it.
            if self.next_match.idx >= 0
                && self.next_match.col >= self.col
                && self.next_match.col != MAXCOL as c_int
            {
                self.push_next_match();
            }
            self.finished = true;
            self.state_stored = false;
            return 0;
        }
        // If the current or the next character is NUL, this finishes the line.
        // SAFETY: as above; the byte after a non-NUL one is still inside.
        if here == NUL || unsafe { *line.offset(self.col as isize + 1) } as c_int == NUL {
            self.finished = true;
            self.state_stored = false;
        }
        if self.try_next_column {
            self.next_match.idx = -1;
            self.try_next_column = false;
        }

        // Only check for keywords when not syncing and there are some.
        let do_keywords =
            !syncing && (block.b_keywtab.ht_used > 0 || block.b_keywtab_ic.ht_used > 0);

        // Zero-width matches with a nextlist already used here, so the same
        // one cannot match twice in this column and loop forever.
        let mut zero_width: Vec<c_int> = Vec::new();

        // Use the `syntax iskeyword` option while matching.
        let mut buf_chartab = [0u64; 4];
        self.install_syntax_chartab(&mut buf_chartab);

        let mut zero_width_next_list = false;
        let mut cur_si: Option<c_int>;

        // Repeat matching keywords and patterns to find contained items at
        // the same column. Stops when there is no extra match here.
        loop {
            let mut found_match = false;
            let mut keep_next_list = false;
            let mut found_keyword = false;

            // 1. Only when there is no current state, or the current state
            //    may contain other things, do we need to look for keywords
            //    and patterns. Always look for contained items when some item
            //    has a `containedin=` (which costs extra time).
            cur_si = (self.state_len() != 0).then(|| self.top());
            if block.b_syn_containedin != 0
                || cur_si.is_none_or(|si| !self.item(si).si_cont_list.is_null())
            {
                // 2. A keyword, if we are on a keyword character after a
                //    non-keyword one. Never while syncing.
                if do_keywords && let Some(si) = self.try_keyword(cur_si) {
                    cur_si = Some(si);
                    found_keyword = true;
                }

                // 3. A pattern, only if no keyword was found.
                if !found_keyword && !block.patterns().is_empty() {
                    // If we have not looked yet, or we are past what we
                    // found, look for a match with any pattern.
                    if self.next_match.idx < 0 || self.next_match.col < self.col {
                        self.scan_patterns(syncing, displaying, cur_si, &zero_width);
                    }
                    // If we found a match at the current column, use it.
                    if self.next_match.idx >= 0 && self.next_match.col == self.col {
                        let lspp = block.pattern(self.next_match.idx);
                        if self.next_match.m_endpos.lnum == self.lnum
                            && self.next_match.m_endpos.col == self.col
                            && !lspp.sp_next_list.is_none()
                        {
                            // A zero-width item with a nextgroup: do not push
                            // it, just set the nextgroup -- and remember it,
                            // so it cannot match here again.
                            self.next_list = lspp.sp_next_list.as_ptr();
                            self.next_flags = lspp.sp_flags;
                            keep_next_list = true;
                            zero_width_next_list = true;
                            zero_width.push(self.next_match.idx);
                            self.next_match.idx = -1;
                        } else {
                            cur_si = Some(self.push_next_match());
                        }
                        found_match = true;
                    }
                }
            }

            // Handle searching for a nextgroup match.
            if !self.next_list.is_null() && !keep_next_list {
                // If a nextgroup was not found, keep looking for one when
                // this is an empty line and "skipempty" was given, or we are
                // on white space and "skipwhite" was given.
                if !found_match {
                    let white = self.next_flags.has(SynFlags::SKIPWHITE)
                        && ascii_iswhite(c_int::from(self.curline_byte(self.col)));
                    let empty = self.next_flags.has(SynFlags::SKIPEMPTY)
                        && c_int::from(self.curline_byte(0)) == NUL;
                    if white || empty {
                        break;
                    }
                }
                // Found: use it and keep looking for contained matches. Not
                // found: keep looking for a normal match. When the nextgroup
                // came from a zero-width item and nothing matched, do not
                // loop -- we would get stuck.
                self.next_list = ::core::ptr::null_mut();
                self.next_match.idx = -1;
                if !zero_width_next_list {
                    found_match = true;
                }
            }
            if !found_match {
                break;
            }
        }

        self.restore_chartab(&buf_chartab);

        let sip = self.pick_current_attr(cur_si);
        if let Some(can_spell) = can_spell {
            *can_spell = match sip {
                Some(sip) if cur_si.is_some() => self.item_can_spell(sip),
                _ => default_can_spell(block),
            };
        }
        if cur_si.is_some() && !syncing && !keep_state {
            // Check whether the current state -- and the states before it --
            // end at the next column. Not while syncing: we would miss a
            // single-character match. The current column is checked first:
            // the item here may be an empty match, and a containing item
            // might end in this column too.
            self.check_state_ends();
            if self.state_len() > 0 && c_int::from(self.curline_byte(self.col)) != NUL {
                self.col += 1;
                self.check_state_ends();
                self.col -= 1;
            }
        }

        // A nextgroup ends at end of line, unless "skipnl" or "skipempty".
        if !self.next_list.is_null() && !self.next_flags.has(SynFlags::SKIPNL | SynFlags::SKIPEMPTY)
        {
            let line = self.curline();
            // SAFETY: as at the top: inside the line, and the byte after a
            // non-NUL one is too.
            let here = unsafe { *line.offset(self.col as isize) } as c_int;
            if here != NUL && unsafe { *line.offset(self.col as isize + 1) } as c_int == NUL {
                self.next_list = ::core::ptr::null_mut();
            }
        }

        self.current.attr
    }

    /// Try to match a keyword at the current column and push it if one
    /// matches.
    ///
    /// Answers the pushed item, or `None` when the column does not start a
    /// keyword or no keyword matches there.
    fn try_keyword(&mut self, cur_si: Option<c_int>) -> Option<c_int> {
        // Only on a keyword character that follows a non-keyword one.
        let buffer = self.buffer;
        let line = self.curline();
        // SAFETY: the column is inside the line, terminator included.
        let cur_pos = unsafe { line.offset(self.col as isize) };
        // SAFETY: as above; the buffer the parser was started for.
        if !unsafe { vim_iswordp_buf(cur_pos, buffer) } {
            return None;
        }
        if self.col != 0 {
            // SAFETY: `prev` is inside the line the parser is on.
            let prev = unsafe { cur_pos.offset(-1) };
            let head = unsafe { prev.offset(-(utf_head_off(line, prev) as isize)) };
            // SAFETY: as above.
            if unsafe { vim_iswordp_buf(head, buffer) } {
                return None;
            }
        }
        // SAFETY: a keyword character starts at the column, in the parser's
        // line of the parser's buffer.
        let kw = unsafe { self.check_keyword_id(line, self.col, cur_si) }?;

        self.push_current_state(KEYWORD_IDX);
        let top = self.top();
        let seqnr = self.take_seqnr();
        let (lnum, col) = (self.lnum, self.col);
        let outer = (top > 0).then(|| {
            let outer = self.item(top - 1);
            (outer.si_flags, outer.si_attr, outer.si_trans_id)
        });
        let si = self.item_mut(top);
        si.si_m_startcol = col;
        si.si_h_startpos.lnum = lnum;
        si.si_h_startpos.col = 0; // starts right away
        si.si_m_endpos.lnum = lnum;
        si.si_m_endpos.col = kw.endcol;
        si.si_h_endpos.lnum = lnum;
        si.si_h_endpos.col = kw.endcol;
        si.si_ends = 1;
        si.si_end_idx = 0;
        si.si_flags = kw.flags;
        si.si_seqnr = seqnr;
        si.si_cchar = kw.cchar;
        if let Some((flags, _, _)) = outer {
            si.si_flags |= flags.masked(SynFlags::CONCEAL);
        }
        si.si_id = kw.id;
        si.si_trans_id = kw.id;
        if kw.flags.has(SynFlags::TRANSP) {
            // Transparent: take the attributes of the item around it.
            let (attr, trans_id) = outer.map_or((0, 0), |(_, attr, trans_id)| (attr, trans_id));
            si.si_attr = attr;
            si.si_trans_id = trans_id;
        } else {
            si.si_attr = syn_id2attr(kw.id);
        }
        si.si_cont_list = ::core::ptr::null_mut();
        si.si_next_list = kw.next_list;
        self.check_keepend();
        Some(top)
    }

    /// Look for the pattern that matches earliest at or after the current
    /// column and record it in `next_match`.
    ///
    /// Matching with a pattern takes a good deal of time, so this remembers
    /// per pattern where it last matched in this line
    /// (`sp_startcol`/`sp_line_id`) and skips any pattern that cannot beat
    /// the best match so far.
    fn scan_patterns(
        &mut self,
        syncing: bool,
        displaying: bool,
        cur_si: Option<c_int>,
        zero_width: &[c_int],
    ) {
        let mut block = self.block();
        self.next_match.idx = 0; // no match in this line yet
        self.next_match.col = MAXCOL as c_int;

        let mut idx = block.patterns().len() as c_int;
        while idx > 0 {
            idx -= 1;
            // Everything the loop needs is copied out first: `find_endpos`
            // below reaches the pattern array again and writes into it, so
            // no borrow of it may be live across this body.
            let scan = PatScan::of(block, idx);
            if !self.pattern_admitted(&scan, cur_si, syncing, displaying) {
                continue;
            }
            // Already tried in this line, and it cannot match before the
            // best match so far.
            if scan.line_id == self.line_id && scan.startcol >= self.next_match.col {
                continue;
            }
            block.pattern_mut(idx).sp_line_id = self.line_id;

            let lc_col = (self.col - scan.offsets.offsets[SPO_LC_OFF as usize]).max(0);
            let (matched, regmatch, captures) = self.run_pattern(idx, self.lnum, lc_col, None);
            if !matched {
                // No match in this line; try another pattern.
                block.pattern_mut(idx).sp_startcol = MAXCOL as c_int;
                continue;
            }

            // The first column of the match.
            let pos = self.syn_add_start_off(scan.offsets, &regmatch, SPO_MS_OFF, -1);
            if pos.lnum > self.lnum {
                // Must have used the end of the match in a following line,
                // which we cannot handle.
                block.pattern_mut(idx).sp_startcol = MAXCOL as c_int;
                continue;
            }
            let startcol = pos.col;
            // Remember the next column where this pattern matches in this
            // line.
            block.pattern_mut(idx).sp_startcol = startcol;
            // A previously found match starts earlier: keep that one.
            if startcol >= self.next_match.col {
                continue;
            }
            // Matched this pattern here before: skip it, and retry in the
            // next column, because it may match from there.
            if self.did_match_already(idx, zero_width) {
                self.try_next_column = true;
                continue;
            }

            let mut endpos = regmatch.endpos[0];
            let mut hl_startpos = self.syn_add_start_off(scan.offsets, &regmatch, SPO_HS_OFF, -1);
            // The region start defaults to the end of the start match.
            let eos_pos = self.syn_add_end_off(scan.offsets, &regmatch, SPO_RS_OFF, 0);

            let mut flags = SynFlags::NONE;
            let mut eoe_pos = LPos { lnum: 0, col: 0 };
            let mut end_idx = 0;
            let mut hl_endpos = LPos { lnum: 0, col: 0 };

            if scan.ty == SPTYPE_START && scan.flags.has(SynFlags::ONELINE) {
                // A "oneline" must end in this line too. Look for the end
                // after the start match, and set every resulting position at
                // once.
                let end = self.find_endpos(idx, endpos, captures.as_deref());
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
                hl_endpos = self.syn_add_end_off(scan.offsets, &regmatch, SPO_HE_OFF, 0);
                endpos = self.syn_add_end_off(scan.offsets, &regmatch, SPO_ME_OFF, 0);
                if endpos.lnum == self.lnum && endpos.col + c_int::from(syncing) < startcol {
                    // An empty match: may need to try again in the next
                    // column.
                    if regmatch.startpos[0].col == regmatch.endpos[0].col {
                        self.try_next_column = true;
                    }
                    continue;
                }
            }

            // Keep the best match so far. Highlighting must start after
            // startpos and end before endpos.
            if hl_startpos.lnum == self.lnum && hl_startpos.col < startcol {
                hl_startpos.col = startcol;
            }
            limit_pos_zero(&mut hl_endpos, endpos);

            self.next_match = NextMatch {
                idx,
                col: startcol,
                m_endpos: endpos,
                h_startpos: hl_startpos,
                h_endpos: hl_endpos,
                eos_pos,
                eoe_pos,
                end_idx,
                flags,
                extmatch: captures,
            };
        }
    }

    /// Can the pattern `spp` describes match here at all: is it the right
    /// kind of item, and do the containment rules admit it?
    ///
    /// This is one `if (A && B && C && D)` upstream, and every operand short
    /// circuits: `in_id_list` is the expensive one and runs last.
    #[inline]
    fn pattern_admitted(
        &self,
        spp: &PatScan,
        cur_si: Option<c_int>,
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
        if !self.next_list.is_null() {
            // A pending `nextgroup=` admits only what it names.
            // SAFETY: the parser's own lists.
            unsafe {
                self.in_id_list(
                    None,
                    self.next_list,
                    spp.syn,
                    spp.cont_in_list,
                    SynFlags::NONE,
                )
            }
        } else if let Some(cur_si) = cur_si {
            // Inside an item, only what its `contains=` names.
            let contains = self.item(cur_si).si_cont_list;
            // SAFETY: as above, inside the item the caller named.
            unsafe { self.in_id_list(Some(cur_si), contains, spp.syn, spp.cont_in_list, spp.flags) }
        } else {
            // At the top level, anything that is not `contained`.
            !spp.flags.has(SynFlags::CONTAINED)
        }
    }

    /// Publish the attributes of the innermost item whose highlight range
    /// covers the current column into `current`, and answer that item.
    ///
    /// Answers `None` when `cur_si` is `None`, i.e. nothing matched here at
    /// all.
    fn pick_current_attr(&mut self, cur_si: Option<c_int>) -> Option<c_int> {
        let sub_char = self.current.sub_char;
        self.current = CurrentAttr {
            sub_char,
            ..CurrentAttr::NONE
        };
        cur_si?;
        // Use the attributes of the innermost item if we are inside its
        // highlighting; if not, of the item around it, and so on.
        let (lnum, col) = (self.lnum, self.col);
        let mut walked = None;
        let mut idx = self.top();
        while idx >= 0 {
            let sip = self.item(idx);
            walked = Some(idx);
            let started = lnum > sip.si_h_startpos.lnum
                || (lnum == sip.si_h_startpos.lnum && col >= sip.si_h_startpos.col);
            let not_ended = sip.si_h_endpos.lnum == 0
                || lnum < sip.si_h_endpos.lnum
                || (lnum == sip.si_h_endpos.lnum && col < sip.si_h_endpos.col);
            if started && not_ended {
                self.current = CurrentAttr {
                    attr: sip.si_attr,
                    id: sip.si_id,
                    trans_id: sip.si_trans_id,
                    flags: sip.si_flags,
                    seqnr: sip.si_seqnr,
                    sub_char: sip.si_cchar,
                };
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

    /// Whether spell checking should be done in the item the attribute walk
    /// left at `sip`.
    fn item_can_spell(&self, sip: c_int) -> bool {
        let block = self.block();
        let mut sps = sp_syn { inc_tag: 0, id: 0 };
        // The two cluster ids are looked up as bare groups: no `containedin=`.
        let no_cont_in = ::core::ptr::null_mut();
        let contains = self.item(sip).si_cont_list;
        let trans_id = self.current.trans_id;
        if block.b_spell_cluster_id == 0 {
            // There is no @Spell cluster: spell check items without a
            // @NoSpell cluster.
            if block.b_nospell_cluster_id == 0 || trans_id == 0 {
                return block.b_syn_spell != SYNSPL_NOTOP;
            }
            sps.id = block.b_nospell_cluster_id as int16_t;
            // SAFETY: the parser's own state stack and lists.
            return !unsafe {
                self.in_id_list(Some(sip), contains, sps, no_cont_in, SynFlags::NONE)
            };
        }
        // The @Spell cluster is defined: spell check in items carrying it,
        // but not when @NoSpell is there too. At the top level only spell
        // check when `:syntax spell toplevel` was used.
        if trans_id == 0 {
            return block.b_syn_spell == SYNSPL_TOP;
        }
        sps.id = block.b_spell_cluster_id as int16_t;
        // SAFETY: as above.
        let mut can =
            unsafe { self.in_id_list(Some(sip), contains, sps, no_cont_in, SynFlags::NONE) };
        if block.b_nospell_cluster_id != 0 {
            sps.id = block.b_nospell_cluster_id as int16_t;
            // SAFETY: as above.
            if unsafe { self.in_id_list(Some(sip), contains, sps, no_cont_in, SynFlags::NONE) } {
                can = false;
            }
        }
        can
    }

    /// Have we already matched pattern `idx` at the current column?
    ///
    /// Two places to look: an item already on the state stack that started
    /// here, and the list of zero-width items with a `nextgroup=` used in
    /// this column.
    fn did_match_already(&self, idx: c_int, gap: &[c_int]) -> bool {
        let (lnum, col) = (self.lnum, self.col);
        self.stack
            .iter()
            .rev()
            .any(|si| si.si_m_startcol == col && si.si_m_lnum == lnum && si.si_idx == idx)
            || gap.contains(&idx)
    }
}

/// Whether spell checking is done outside every syntax item: only when there
/// is no `@Spell` cluster, or when `:syntax spell toplevel` was used.
fn default_can_spell(block: SynBlockRef) -> bool {
    if block.b_syn_spell == SYNSPL_DEFAULT {
        block.b_spell_cluster_id == 0
    } else {
        block.b_syn_spell == SYNSPL_TOP
    }
}

/// What [`SynState::scan_patterns`] needs from one pattern, copied out.
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
    fn of(block: SynBlockRef, idx: c_int) -> PatScan {
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
