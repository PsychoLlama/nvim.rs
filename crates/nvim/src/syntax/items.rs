//! The state stack's item operations.
//!
//! The stack is a `Vec` of `StateItem`, innermost last, and an item is named
//! by its index -- a push can move every item, so nothing holds one across
//! a step. These are the operations on it: pushing what a match found
//! ([`SynState::push_next_match`]), working out an item's highlight attributes
//! and containment ([`SynState::update_si_attr`]) and where it ends
//! ([`SynState::update_si_end`]), applying `keepend`/`extend`
//! ([`SynState::check_keepend`]), and popping items whose end the driver has
//! reached ([`SynState::check_state_ends`]).

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

impl SynState {
    /// Number of items on the state stack.
    #[inline(always)]
    pub(crate) fn state_len(&self) -> c_int {
        self.stack.len() as c_int
    }

    /// The item at `i` on the state stack (0 is the outermost).
    #[inline(always)]
    pub(crate) fn item(&self, i: c_int) -> &StateItem {
        &self.stack[i as usize]
    }

    /// The item at `i`, to change.
    #[inline(always)]
    pub(crate) fn item_mut(&mut self, i: c_int) -> &mut StateItem {
        &mut self.stack[i as usize]
    }

    /// The index of the innermost item. Only meaningful when the stack is not
    /// empty.
    #[inline(always)]
    pub(crate) fn top(&self) -> c_int {
        self.state_len() - 1
    }

    /// Push what `next_match` found onto the state stack.
    ///
    /// Answers the index of the item now on top, which is the `matchgroup=`
    /// item for a region's start pattern when there is one and the region
    /// itself otherwise.
    pub(crate) fn push_next_match(&mut self) -> c_int {
        let idx = self.next_match.idx;
        let block = self.block();
        let spp = block.pattern(idx);

        self.push_current_state(idx);
        let top = self.top();
        let (lnum, col) = (self.lnum, self.col);
        let seqnr = self.take_seqnr();
        let outer_conceal = if top > 0 {
            // A concealed item conceals what it contains.
            self.item(top - 1).si_flags.masked(SynFlags::CONCEAL)
        } else {
            SynFlags::NONE
        };
        let extmatch = self.next_match.extmatch.clone();
        let h_startpos = self.next_match.h_startpos;
        let cur_si = self.item_mut(top);
        cur_si.si_h_startpos = h_startpos;
        cur_si.si_m_startcol = col;
        cur_si.si_m_lnum = lnum;
        cur_si.si_flags = spp.sp_flags | outer_conceal;
        cur_si.si_seqnr = seqnr;
        cur_si.si_cchar = spp.sp_cchar;
        cur_si.si_next_list = spp.sp_next_list.as_ptr();
        cur_si.si_extmatch = extmatch;

        if spp.sp_type as c_int == SPTYPE_START && !spp.sp_flags.has(SynFlags::ONELINE) {
            // A start-skip-end that may cross lines: work out how much of it
            // is in this line.
            let endcol = self.next_match.m_endpos.col;
            self.update_si_end(top, endcol, true);
            self.check_keepend();
        } else {
            let next = &self.next_match;
            let (m_endpos, h_endpos, flags, eoe_pos, end_idx) = (
                next.m_endpos,
                next.h_endpos,
                next.flags,
                next.eoe_pos,
                next.end_idx,
            );
            let cur_si = self.item_mut(top);
            cur_si.si_m_endpos = m_endpos;
            cur_si.si_h_endpos = h_endpos;
            cur_si.si_ends = 1;
            cur_si.si_flags |= flags;
            cur_si.si_eoe_pos = eoe_pos;
            cur_si.si_end_idx = end_idx;
        }
        if self.keepend_level < 0 && self.item(top).si_flags.has(SynFlags::KEEPEND) {
            self.keepend_level = top;
        }
        self.check_keepend();
        self.update_si_attr(top);

        let save_flags = self
            .item(top)
            .si_flags
            .masked(SynFlags::CONCEAL | SynFlags::CONCEALENDS);

        // If the start pattern has a `matchgroup=` of its own, push a second
        // item for it, ending where the start match ends.
        if spp.sp_type as c_int == SPTYPE_START && spp.sp_syn_match_id != 0 {
            self.push_current_state(idx);
            let top = self.top();
            let seqnr = self.take_seqnr();
            let (h_startpos, eos_pos) = (self.next_match.h_startpos, self.next_match.eos_pos);
            let cur_si = self.item_mut(top);
            cur_si.si_h_startpos = h_startpos;
            cur_si.si_m_startcol = col;
            cur_si.si_m_lnum = lnum;
            cur_si.si_m_endpos = eos_pos;
            cur_si.si_h_endpos = eos_pos;
            cur_si.si_ends = 1;
            cur_si.si_end_idx = 0;
            cur_si.si_flags = SynFlags::MATCH | save_flags;
            cur_si.si_seqnr = seqnr;
            if cur_si.si_flags.has(SynFlags::CONCEALENDS) {
                cur_si.si_flags |= SynFlags::CONCEAL;
            }
            cur_si.si_next_list = ::core::ptr::null_mut();
            self.check_keepend();
            self.update_si_attr(top);
        }

        self.next_match.idx = -1; // try another match next time
        self.top()
    }

    /// The next item sequence number, post-incrementing the counter.
    ///
    /// `si_seqnr` orders items that begin at the same column; `synstack()`
    /// reports it and the state-stack equality test compares it.
    #[inline]
    pub(crate) fn take_seqnr(&mut self) -> c_int {
        let n = self.next_seqnr;
        self.next_seqnr = n + 1;
        n
    }

    /// Pop every item on the stack whose end the driver has now reached.
    pub(crate) fn check_state_ends(&mut self) {
        let mut cur = self.top();
        loop {
            let (lnum, col) = (self.lnum, self.col);
            let cur_si = self.item(cur);
            if cur_si.si_ends == 0
                || cur_si.si_m_endpos.lnum > lnum
                || (cur_si.si_m_endpos.lnum == lnum && cur_si.si_m_endpos.col > col)
            {
                return;
            }

            // If the end pattern has a highlight group of its own and it
            // continues beyond this position, highlight it now. The item
            // stays on the stack, standing in for the end match.
            if cur_si.si_end_idx != 0
                && (cur_si.si_eoe_pos.lnum > lnum
                    || (cur_si.si_eoe_pos.lnum == lnum && cur_si.si_eoe_pos.col > col))
            {
                let seqnr = self.take_seqnr();
                let cur_si = self.item_mut(cur);
                cur_si.si_idx = cur_si.si_end_idx;
                cur_si.si_end_idx = 0;
                cur_si.si_m_endpos = cur_si.si_eoe_pos;
                cur_si.si_h_endpos = cur_si.si_eoe_pos;
                cur_si.si_flags |= SynFlags::MATCH;
                cur_si.si_seqnr = seqnr;
                if cur_si.si_flags.has(SynFlags::CONCEALENDS) {
                    cur_si.si_flags |= SynFlags::CONCEAL;
                }
                self.update_si_attr(cur);

                // `nextgroup=` should not match in the end pattern, and what
                // matches next may be different now.
                self.next_list = ::core::ptr::null_mut();
                self.next_match.idx = 0;
                self.next_match.col = MAXCOL as c_int;
                return;
            }

            // Hand the ended item's `nextgroup=` to the driver, unless we are
            // at end of line and it has neither "skipnl" nor "skipempty".
            let (next_list, next_flags) = (cur_si.si_next_list, cur_si.si_flags);
            self.next_list = next_list;
            self.next_flags = next_flags;
            if !self.next_flags.has(SynFlags::SKIPNL | SynFlags::SKIPEMPTY)
                && self.curline_byte(self.col) as c_int == NUL
            {
                self.next_list = ::core::ptr::null_mut();
            }

            // When the ended item has "extend", another item with "keepend"
            // now needs to check for its end.
            let had_extend = self.item(cur).si_flags.has(SynFlags::EXTEND);

            self.pop_current_state();
            if self.state_len() <= 0 {
                return;
            }
            if had_extend && self.keepend_level >= 0 {
                self.update_ends(false);
                if self.state_len() <= 0 {
                    return;
                }
            }
            cur = self.top();

            // Only for a region does the search for the end continue after
            // the end of the contained item. If the contained match included
            // the end of the line, stop here and let the region continue. Not
            // when "keepend" is used for the contained item, not when we are
            // away from the end of the line (the end could be
            // `end="x$"me=e-1`), and not when "excludenl" is used
            // (SynFlags::HAS_EOL will not be set).
            let cur_si = self.item(cur);
            if cur_si.si_idx >= 0
                && self.block().pattern(cur_si.si_idx).sp_type as c_int == SPTYPE_START
                && !cur_si.si_flags.has(SynFlags::MATCH | SynFlags::KEEPEND)
            {
                self.update_si_end(cur, self.col, true);
                self.check_keepend();
                if self.next_flags.has(SynFlags::HAS_EOL)
                    && self.keepend_level < 0
                    && self.curline_byte(self.col) as c_int == NUL
                {
                    return;
                }
            }
        }
    }

    /// Fill in `si_id`, `si_attr`, `si_trans_id` and `si_cont_list` for the
    /// item at `idx`, from the pattern it came from.
    pub(crate) fn update_si_attr(&mut self, idx: c_int) {
        let si_idx = self.item(idx).si_idx;
        if si_idx < 0 {
            return; // a keyword; should not happen
        }
        let block = self.block();
        let spp = block.pattern(si_idx);
        let is_match = self.item(idx).si_flags.has(SynFlags::MATCH);

        let id = if is_match {
            spp.sp_syn_match_id as c_int
        } else {
            spp.sp_syn.id as c_int
        };
        let cont_list = if is_match {
            ::core::ptr::null_mut()
        } else {
            spp.sp_cont_list.as_ptr()
        };
        let sip = self.item_mut(idx);
        sip.si_id = id;
        sip.si_attr = syn_id2attr(id);
        sip.si_trans_id = id;
        sip.si_cont_list = cont_list;

        // A transparent item takes its attributes from the item around it,
        // and its containment too when it has none of its own. Not for the
        // matchgroup of a start or end pattern.
        if !spp.sp_flags.has(SynFlags::TRANSP) || is_match {
            return;
        }
        if idx == 0 {
            let sip = self.item_mut(idx);
            sip.si_attr = 0;
            sip.si_trans_id = 0;
            if sip.si_cont_list.is_null() {
                sip.si_cont_list = ID_LIST_ALL;
            }
        } else {
            let outer = self.item(idx - 1);
            let (attr, trans_id, cont_list) =
                (outer.si_attr, outer.si_trans_id, outer.si_cont_list);
            let sip = self.item_mut(idx);
            sip.si_attr = attr;
            sip.si_trans_id = trans_id;
            if sip.si_cont_list.is_null() {
                sip.si_flags |= SynFlags::TRANS_CONT;
                sip.si_cont_list = cont_list;
            }
        }
    }

    /// Propagate the end of every "keepend" item on the stack to the items
    /// it contains, so none of them can reach past it.
    pub(crate) fn check_keepend(&mut self) {
        // This check can consume a lot of time; only do it from the level
        // where there really is a keepend.
        if self.keepend_level < 0 {
            return;
        }

        // Find the innermost "extend" item: "keepend" items outside it do
        // nothing. With no "extend" item this stops at `keepend_level` and
        // every "keepend" works normally.
        let mut i = self.top();
        while i > self.keepend_level {
            if self.item(i).si_flags.has(SynFlags::EXTEND) {
                break;
            }
            i -= 1;
        }

        let mut maxpos = LPos { lnum: 0, col: 0 };
        let mut maxpos_h = LPos { lnum: 0, col: 0 };
        while i < self.state_len() {
            let sip = self.item_mut(i);
            if maxpos.lnum != 0 {
                limit_pos_zero(&mut sip.si_m_endpos, maxpos);
                limit_pos_zero(&mut sip.si_h_endpos, maxpos_h);
                limit_pos_zero(&mut sip.si_eoe_pos, maxpos);
                sip.si_ends = 1;
            }
            if sip.si_ends != 0 && sip.si_flags.has(SynFlags::KEEPEND) {
                if maxpos.lnum == 0 || pos_after(maxpos, sip.si_m_endpos) {
                    maxpos = sip.si_m_endpos;
                }
                if maxpos_h.lnum == 0 || pos_after(maxpos_h, sip.si_h_endpos) {
                    maxpos_h = sip.si_h_endpos;
                }
            }
            i += 1;
        }
    }

    /// Find where the start-skip-end item at `at` ends, if it ends in this
    /// line.
    ///
    /// `startcol` is where to start looking; `force` overrules an end the
    /// item already has.
    pub(crate) fn update_si_end(&mut self, at: c_int, startcol: c_int, force: bool) {
        let sip = self.item(at);
        if sip.si_idx < 0 {
            return; // a keyword has no end pattern
        }
        // Don't update when it is already done. Can be a match of an end
        // pattern that started in a previous line -- but watch out, it can
        // also be a "keepend" from a containing item.
        if !force && sip.si_m_endpos.lnum >= self.lnum {
            return;
        }

        let startpos = LPos {
            lnum: self.lnum,
            col: startcol as ColNr,
        };
        let (si_idx, start_ext) = (sip.si_idx, sip.si_extmatch.clone());
        let end = self.find_endpos(si_idx, startpos, start_ext.as_deref());
        // A "oneline" that found no end never continues in the next line:
        // it ends with this one. Only then is the line's length wanted.
        let oneline_end = (end.m_endpos.lnum == 0
            && self.block().pattern(si_idx).sp_flags.has(SynFlags::ONELINE))
        .then(|| self.curline_len());
        let lnum = self.lnum;

        let sip = self.item_mut(at);
        if let Some(flags) = end.flags {
            sip.si_flags = flags;
        }
        if end.m_endpos.lnum == 0 {
            // No end pattern matched.
            if let Some(line_len) = oneline_end {
                sip.si_ends = 1;
                sip.si_m_endpos.lnum = lnum;
                sip.si_m_endpos.col = line_len;
            } else {
                sip.si_ends = 0;
                sip.si_m_endpos.lnum = 0;
            }
            sip.si_h_endpos = sip.si_m_endpos;
        } else {
            sip.si_m_endpos = end.m_endpos;
            sip.si_h_endpos = end.hl_endpos;
            sip.si_eoe_pos = end.eoe_pos;
            sip.si_ends = 1;
            sip.si_end_idx = end.end_idx;
        }
    }

    /// Push a cleared item for pattern `idx` onto the state stack.
    ///
    /// A push on an invalid stack is dropped; upstream would grow the garray
    /// it had just declared dead.
    pub(crate) fn push_current_state(&mut self, idx: c_int) {
        if self.stack_valid {
            self.stack.push(StateItem {
                si_idx: idx,
                ..EMPTY_STATE_ITEM
            });
        }
    }

    /// Pop the innermost item off the state stack.
    pub(crate) fn pop_current_state(&mut self) {
        self.stack.pop();
        // After the end of a pattern, try matching a keyword or pattern
        // again.
        self.next_match.idx = -1;
        // If the first "keepend" item was the one popped, there is no keepend
        // level any more.
        if self.keepend_level >= self.state_len() {
            self.keepend_level = -1;
        }
    }
}

/// Is `a` strictly after `b`?
#[inline]
fn pos_after(a: LPos, b: LPos) -> bool {
    a.lnum > b.lnum || (a.lnum == b.lnum && a.col > b.col)
}
