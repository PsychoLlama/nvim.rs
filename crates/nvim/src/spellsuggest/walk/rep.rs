//! Edits the language itself suggests: the `REP` items of the `.aff` file.
//!
//! A `REP` line names a stretch of text and what it is commonly mistyped
//! as -- "f" for "ph", "shun" for "tion" -- and applying one is a single
//! edit at [`SCORE_REP`] no matter how many characters it moves. That is
//! what lets a suggestion be found that no sequence of one-character edits
//! could reach within the score ceiling.
//!
//! Like the rearrangements in [`super::transpose`], a `REP` item rewrites
//! `fword` in place and is undone on the way back up, by the child level
//! whose state is [`State::RepUndo`]. Unlike them it can change the bad
//! word's *length*: `Walk::repextra` tracks by how much, so that a
//! suggestion still knows how much of the original bad word it replaces.
//!
//! The items are sorted by their first byte and indexed by it, so the scan
//! starts at the first item that can possibly match and stops at the first
//! one whose first byte no longer does.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::spellsuggest::SCORE_REP;
use crate::spellsuggest::walk::{State, Walk};
use crate::types::RepItem;
use core::ffi::c_int;

impl Walk<'_> {
    /// Decide whether trying `REP` items here is worth it at all, and if
    /// so where in the list to start.
    ///
    /// Runs on into [`Walk::rep`], as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's language and bad word must be valid.
    pub(super) unsafe fn rep_ini(&mut self) {
        let level = self.depth as usize;

        // Skip quickly when there are no REP items and this is not the
        // sound-fold tree, when the score would be too high anyway, or
        // when a REP item or a swap has already changed the text here.
        //
        // SAFETY: `lp` and `su` are the caller's, valid by the contract
        // above.
        if (unsafe { (*self.lp).lp_replang }.is_null() && !self.soundfold)
            || self.stack[level].score + SCORE_REP >= unsafe { (*self.su).su_maxscore }
            || self.stack[level].bad_idx < self.stack[level].change_from
        {
            self.stack[level].state = State::Final;
            return;
        }

        // SAFETY: `bad_idx` is a position inside the bad word, and the
        // first-byte index is a 256-entry table indexed by an unsigned
        // byte of it. `lp_replang` is non-null here unless this is the
        // sound-fold walk -- that is what the test above just settled.
        let first_byte = self.fword_at(self.stack[level].bad_idx as usize) as usize;
        self.stack[level].child = if self.soundfold {
            unsafe { (*self.slang).sl_repsal_first[first_byte] }
        } else {
            unsafe { (*(*self.lp).lp_replang).sl_rep_first[first_byte] }
        };

        if self.stack[level].child < 0 {
            // No item starts with this byte.
            self.stack[level].state = State::Final;
            return;
        }

        self.stack[level].state = State::Rep;
        // SAFETY: as above.
        unsafe { self.rep() };
    }

    /// Try the `REP` items in turn until one matches, replacing the text
    /// it matches and going a level deeper.
    ///
    /// This state is its own successor: [`State::RepUndo`] puts the bad
    /// word back and returns here for the next item.
    ///
    /// # Safety
    ///
    /// The walk's language and bad word must be valid.
    pub(super) unsafe fn rep(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;
        // SAFETY: the language is valid by the contract above and stays
        // loaded for as long as the walk.
        let items = unsafe { self.rep_items() };

        while (self.stack[level].child as usize) < items.len() {
            let item = &items[self.stack[level].child as usize];
            self.stack[level].child += 1;

            if item.from[0] != self.fword[bad_idx] {
                // Past every item that could match.
                self.stack[level].child = items.len() as i16;
                break;
            }
            if !self.fword[bad_idx..].starts_with(&item.from)
                // SAFETY: `su` is the caller's suggestion state.
                || !unsafe { self.try_deeper(SCORE_REP) }
            {
                continue;
            }

            self.go_deeper(SCORE_REP);
            // The replacement has to be undone when the walk returns.
            self.stack[level].state = State::RepUndo;
            self.depth += 1;

            // Change the "from" text into the "to" text, closing or
            // opening the gap between them first.
            let (from_len, to_len) = (item.from.len(), item.to.len());
            let to = item.to.clone();
            if from_len != to_len {
                move_tail(&mut self.fword[bad_idx..], from_len, to_len);
                self.repextra += to_len as c_int - from_len as c_int;
            }
            self.fword[bad_idx..bad_idx + to_len].copy_from_slice(&to);

            let child = self.depth as usize;
            self.stack[child].change_from = (bad_idx + to_len) as u8;
            self.stack[child].char_len = 0;
            break;
        }

        // The state test tells "the list ran out" apart from "an item
        // matched and pushed a level", which left it at `RepUndo`.
        if self.stack[level].child as usize >= items.len() && self.stack[level].state == State::Rep
        {
            self.stack[level].state = State::Final;
        }
    }

    /// Put the text a `REP` item replaced back, and go on to the next
    /// item.
    ///
    /// # Safety
    ///
    /// The walk's language and bad word must be valid.
    pub(super) unsafe fn rep_undo(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;
        // `child` still points just past the item that was applied.
        //
        // SAFETY: the language is valid by the contract above.
        let item = &unsafe { self.rep_items() }[self.stack[level].child as usize - 1];
        let (from_len, to_len) = (item.from.len(), item.to.len());
        let from = item.from.clone();

        if from_len != to_len {
            move_tail(&mut self.fword[bad_idx..], to_len, from_len);
            self.repextra -= to_len as c_int - from_len as c_int;
        }
        self.fword[bad_idx..bad_idx + from_len].copy_from_slice(&from);

        self.stack[level].state = State::Rep;
    }

    /// The `REP` list this walk uses: the sound-fold one when walking the
    /// sound-fold tree, otherwise the replacement language's.
    ///
    /// # Safety
    ///
    /// The walk's language must be valid, and `lp_replang` must be
    /// non-null unless this is the sound-fold walk. The list must outlive
    /// the borrow, which it does: nothing unloads a language mid-walk.
    unsafe fn rep_items<'a>(&self) -> &'a [RepItem] {
        // SAFETY: the caller guarantees the language; `rep_ini` is what
        // establishes the `lp_replang` precondition.
        if self.soundfold {
            unsafe { &(*self.slang).sl_repsal }
        } else {
            unsafe { &(*(*self.lp).lp_replang).sl_rep }
        }
    }
}

/// Shift the text after a replacement so that `to_len` bytes fit where
/// `from_len` were, terminator included.
///
/// # Safety
///
fn move_tail(word: &mut [u8], from_len: usize, to_len: usize) {
    // Everything up to and including the word's terminator moves. A
    // replacement that grows the word can push that past the buffer,
    // which the C let run; here the buffer's own end is the limit.
    let tail_end = word[from_len..]
        .iter()
        .position(|&b| b == 0)
        .map_or(word.len(), |n| from_len + n + 1);
    let room = word.len().saturating_sub(to_len) + from_len;
    word.copy_within(from_len..tail_end.min(room), to_len);
}
