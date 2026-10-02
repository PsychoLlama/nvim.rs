//! The per-line driver.
//!
//! [`Win::syntax_start`] is the entry point every reader goes through: it
//! points the parser at a window/buffer, finds a state to start from (the
//! cache in `stack.rs`, or a `syn_sync` scan) and parses forward to the wanted
//! line. [`SynState::finish_line`] is one line of that walk,
//! [`SynState::start_line`] resets the per-line state, and
//! [`SynState::update_ends`] recomputes where the items on the stack end
//! after the state was loaded from the cache.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::*;
use crate::types::NUL;

impl Win {
    /// Start syntax recognition for a line.
    ///
    /// Normally called from the screen update, once per displayed line. The
    /// window and buffer are remembered in the parser, because
    /// [`get_syntax_attr`] is not given the window -- and careful: `curwin`
    /// and `curbuf` are likely to point somewhere else entirely.
    pub(crate) fn syntax_start(self, lnum: LineNr) {
        with_parser(|parser| parser.start(self, lnum));
    }
}

impl SynState {
    /// [`Win::syntax_start`]'s body.
    pub(crate) fn start(&mut self, window: Win, lnum: LineNr) {
        let buffer = window.buffer();
        self.buffer = buffer;
        self.current.sub_char = NUL;
        let changedtick = buf_get_changedtick(buffer);
        if self.parsed.map(SynBlockRef::raw) != Some(window.w_s)
            || self.buf != buffer.try_id()
            || self.changedtick != changedtick
        {
            self.invalidate_current_state();
            self.buf = buffer.try_id();
            self.parsed = Some(window.syntax());
        }
        self.changedtick = changedtick;
        self.win = Some(window.id());

        self.stack_alloc();
        let mut block = self.block();
        block.b_sst_lasttick = display_tick.get();

        // If the state at the end of the previous line is useful, store it.
        if self.stack_valid && self.lnum < lnum && self.lnum < buffer.line_count() {
            self.finish_line(false);
            if !self.state_stored {
                self.lnum += 1;
                self.store_current_state();
            }
            // If lnum is now "lnum", keep the current state -- which happens
            // very often. Otherwise work it out below.
            if self.lnum != lnum {
                self.invalidate_current_state();
            }
        } else {
            self.invalidate_current_state();
        }

        // Try to synchronise from a saved state, but only if "lnum" is
        // neither before one nor too far beyond one.
        let mut last_valid = None;
        if !self.stack_valid {
            let mut last_min_valid = None;
            let minlines = block.b_syn_sync_minlines;
            for id in block.b_sst.used() {
                let entry = block.b_sst.entry(id);
                if entry.lnum > lnum {
                    break;
                }
                if entry.change_lnum == 0 {
                    last_valid = Some(id);
                    if entry.lnum >= lnum - minlines {
                        last_min_valid = Some(id);
                    }
                }
            }
            if let Some(id) = last_min_valid {
                self.load_current_state(id);
            }
        }

        // Still nothing: re-synchronise.
        let first_stored = if self.stack_valid {
            self.lnum
        } else {
            self.sync(window, lnum, last_valid);
            if self.lnum == 1 {
                1 // the first line is always valid, whatever "minlines" says
            } else {
                // "minlines" lines have to be parsed before a state can be
                // considered valid enough to store.
                self.lnum + block.b_syn_sync_minlines
            }
        };

        // Advance from the sync point or the saved state to the wanted line,
        // saving some entries along the way to sync with later on.
        let dist = self.store_distance();
        let mut prev = None;
        while self.lnum < lnum {
            self.start_line();
            self.finish_line(false);
            self.lnum += 1;
            if self.lnum >= first_stored {
                prev = self.record_line(prev, lnum, dist);
            }

            // This can take a long time: stop when CTRL-C is pressed. The
            // current state is then wrong.
            line_breakcheck();
            if got_int.get() {
                self.lnum = lnum;
                break;
            }
        }
        self.start_line();
    }

    /// How many lines apart to store cache entries for lines that are not
    /// displayed. Displayed lines get one each; the rest share what is left.
    pub(crate) fn store_distance(&self) -> LineNr {
        let entries = c_int::try_from(self.block().b_sst.len())
            .expect("the cache is a few thousand entries at most");
        if entries <= Rows.get() {
            999999
        } else {
            self.line_count() / (entries - Rows.get()) + 1
        }
    }

    /// After parsing up to `lnum`, either adopt the cached state for this
    /// line or store the one just computed. Answers the cache entry to carry
    /// into the next line.
    ///
    /// When the cached entry for this line matches what we parsed, every
    /// entry below it that was only waiting on a change *before* this line
    /// becomes valid again -- which is what turns one re-parse into a whole
    /// valid tail.
    fn record_line(
        &mut self,
        mut prev: Option<EntryId>,
        lnum: LineNr,
        dist: LineNr,
    ) -> Option<EntryId> {
        let parsed_lnum = self.lnum;
        let mut block = self.block();
        if prev.is_none() {
            prev = block.b_sst.find(parsed_lnum - 1);
        }
        let cache = &block.b_sst;
        let mut sp = prev.or(cache.first());
        while let Some(id) = sp
            && cache.entry(id).lnum < parsed_lnum
        {
            sp = cache.next(id);
        }

        if let Some(id) = sp
            && block.b_sst.entry(id).lnum == parsed_lnum
            && self.syn_stack_equal(id)
        {
            let mut prev = id;
            let mut sp = Some(id);
            while let Some(id) = sp
                && block.b_sst.entry(id).change_lnum <= parsed_lnum
            {
                let entry = block.b_sst.entry_mut(id);
                if entry.lnum <= lnum {
                    prev = id; // a valid state before the desired line
                } else if entry.change_lnum == 0 {
                    break; // past the states that depend on a change
                }
                entry.change_lnum = 0;
                sp = block.b_sst.next(id);
            }
            self.load_current_state(prev);
            return Some(prev);
        }

        // Store the state at this line when it is the first one, the line we
        // are parsing for, or far enough from the last stored one.
        if prev.is_none_or(|prev| {
            parsed_lnum == lnum || parsed_lnum >= block.b_sst.entry(prev).lnum + dist
        }) {
            return self.store_current_state();
        }
        prev
    }

    /// Empty the state stack. It stays valid, as upstream's `ga_clear`
    /// leaves it.
    pub(crate) fn clear_current_state(&mut self) {
        self.stack.clear();
    }

    /// Reset the per-line state before parsing a line.
    pub(crate) fn start_line(&mut self) {
        self.finished = false;
        self.col = 0;

        // The end of a start/skip/end that continues from the previous line
        // needs updating, and so do regions with "keepend".
        if self.state_len() > 0 {
            self.update_ends(true);
            self.check_state_ends();
        }

        self.next_match.idx = -1;
        self.line_id += 1;
        self.next_seqnr = 1;
    }

    /// Recompute where the items on the stack end.
    ///
    /// `startofline` says we are at the start of a line, in which case the
    /// innermost item is always updated; otherwise the update is forced only
    /// on the items with "keepend", because they influence what they contain.
    pub(crate) fn update_ends(&mut self, startofline: bool) {
        let block = self.block();
        if startofline {
            // A match carried over from a previous line with a contained
            // region ends as soon as that region ends, so drop the end it has
            // and mark it as continued.
            let lnum = self.lnum;
            for cur_si in &mut self.stack {
                if cur_si.si_idx >= 0
                    && c_int::from(block.pattern(cur_si.si_idx).sp_type) == SPTYPE_MATCH
                    && cur_si.si_m_endpos.lnum < lnum
                {
                    cur_si.si_flags |= SynFlags::MATCHCONT;
                    cur_si.si_m_endpos = LPos { lnum: 0, col: 0 };
                    cur_si.si_h_endpos = cur_si.si_m_endpos;
                    cur_si.si_ends = 1;
                }
            }
        }

        // Start from the innermost "extend" item, as check_keepend does: a
        // "keepend" outside it does nothing. If "extend" has just been
        // removed (`!startofline`) the normal regions inside a "keepend" need
        // updating too, because "extend" could have extended those as well.
        let mut i = self.top();
        if self.keepend_level >= 0 {
            while i > self.keepend_level {
                if self.item(i).si_flags.has(SynFlags::EXTEND) {
                    break;
                }
                i -= 1;
            }
        }

        let mut seen_keepend = false;
        while i < self.state_len() {
            let innermost = i == self.top();
            let lnum = self.lnum;
            let cur_si = self.item_mut(i);
            if cur_si.si_flags.has(SynFlags::KEEPEND)
                || (seen_keepend && !startofline)
                || (innermost && startofline)
            {
                // Highlighting starts in column 0.
                cur_si.si_h_startpos.col = 0;
                cur_si.si_h_startpos.lnum = lnum;

                let (matchcont, keepend) = (
                    cur_si.si_flags.has(SynFlags::MATCHCONT),
                    cur_si.si_flags.has(SynFlags::KEEPEND),
                );
                if !matchcont {
                    self.update_si_end(i, self.col, !startofline);
                }
                if !startofline && keepend {
                    seen_keepend = true;
                }
            }
            i += 1;
        }
        self.check_keepend();
    }

    /// Throw the state away and mark it invalid.
    pub(crate) fn invalidate_current_state(&mut self) {
        self.clear_current_state();
        self.stack_valid = false;
        self.next_list = ::core::ptr::null_mut();
        self.keepend_level = -1;
    }

    /// Mark the state valid and ready to be pushed onto. A stack that is
    /// already valid keeps whatever it holds, as upstream's does.
    pub(crate) fn validate_current_state(&mut self) {
        self.stack_valid = true;
    }

    /// Parse to the end of the current line without answering any
    /// attributes; only the state at the end of the line is wanted.
    ///
    /// May start anywhere in the line, as long as the state is valid. While
    /// syncing, answers whether a sync point was found.
    pub(crate) fn finish_line(&mut self, syncing: bool) -> bool {
        while !self.finished {
            self.current_attr(syncing, false, None, false);

            if syncing && self.state_len() != 0 {
                // Check for a match with a sync item.
                let si_idx = self.item(self.top()).si_idx;
                if si_idx >= 0
                    && self
                        .block()
                        .pattern(si_idx)
                        .sp_flags
                        .has(SynFlags::SYNC_HERE | SynFlags::SYNC_THERE)
                {
                    return true;
                }

                // current_attr() skipped the check for an item that ends
                // here; do it now. Be careful not to go past the NUL.
                let prev_col = self.col;
                if c_int::from(self.curline_byte(self.col)) != NUL {
                    self.col += 1;
                }
                self.check_state_ends();
                self.col = prev_col;
            }
            self.col += 1;
        }
        false
    }
}

/// Stop parsing syntax above line `lnum`.
///
/// If the stored state at or below this line depended on a change before it,
/// it now depends on the line below the last parsed one. The window looks
/// like: the line which changed, the displayed lines, then `lnum` -- the line
/// below the window.
pub(crate) fn syntax_end_parsing(window: Win, lnum: LineNr) {
    with_parser(|parser| {
        let Some(mut block) = parser.parsed else {
            return;
        };
        if block.raw() != window.w_s {
            return; // not the right window
        }
        let mut sp = block.b_sst.find(lnum);
        if let Some(id) = sp
            && block.b_sst.entry(id).lnum < lnum
        {
            sp = block.b_sst.next(id);
        }
        if let Some(id) = sp
            && block.b_sst.entry(id).change_lnum != 0
        {
            block.b_sst.entry_mut(id).change_lnum = lnum;
        }
    });
}

/// Throw the parser's state away and mark it invalid: the items it was
/// parsing are about to change.
pub(crate) fn invalidate_current_state() {
    with_parser(SynState::invalidate_current_state);
}

/// Has the syntax at the start of `lnum` changed since last time?
///
/// Only called just after [`get_syntax_attr`] for the previous line, to
/// decide whether the next line has to be redrawn too.
pub(crate) fn syntax_check_changed(lnum: LineNr) -> bool {
    with_parser(|parser| {
        // Only worth checking when `lnum` is just below the line we last
        // parsed and there is a saved state for it.
        if !parser.stack_valid || lnum != parser.lnum + 1 {
            return true;
        }
        let Some(buffer) = parser.buf.and_then(BufId::get) else {
            return true;
        };
        parser.buffer = buffer;
        let block = parser.block();
        let Some(sp) = block
            .b_sst
            .find(lnum)
            .filter(|&id| block.b_sst.entry(id).lnum == lnum)
        else {
            return true;
        };

        // Finish the previous line, which is needed when not all of it was
        // drawn, and compare with the state saved for this one.
        parser.finish_line(false);
        let changed = !parser.syn_stack_equal(sp);

        // Store the current state for later use.
        parser.lnum += 1;
        parser.store_current_state();
        changed
    })
}
