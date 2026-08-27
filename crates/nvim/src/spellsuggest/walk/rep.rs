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

use crate::os::cshim::strncmp;
use crate::spellsuggest::SCORE_REP;
use crate::spellsuggest::walk::{State, Walk};
use crate::types::{fromto_T, garray_T};
use ::libc::strlen;
use core::ffi::c_int;
use core::ptr;

impl Walk {
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
        let first_byte = unsafe { self.fword_at(self.stack[level].bad_idx as usize) } as usize;
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
        // SAFETY: `bad_idx` is a position inside the bad word, and the
        // language is valid by the contract above.
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };
        let gap = unsafe { self.rep_items() };

        // SAFETY: `gap` is the language's own garray, so its length is
        // what bounds the item index, and both sides of every item are
        // NUL-terminated strings the language owns.
        while (self.stack[level].child as c_int) < unsafe { (*gap).ga_len } {
            let item = unsafe {
                ((*gap).ga_data as *mut fromto_T).offset(self.stack[level].child as isize)
            };
            self.stack[level].child += 1;

            if unsafe { *(*item).ft_from != *p } {
                // Past every item that could match.
                self.stack[level].child = unsafe { (*gap).ga_len } as i16;
                break;
            }
            if unsafe { strncmp((*item).ft_from, p, strlen((*item).ft_from)) } != 0
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
            let from_len = unsafe { strlen((*item).ft_from) } as c_int;
            let to_len = unsafe { strlen((*item).ft_to) } as c_int;
            if from_len != to_len {
                unsafe { move_tail(p, from_len, to_len) };
                self.repextra += to_len - from_len;
            }
            unsafe { ptr::copy((*item).ft_to, p, to_len as usize) };

            let child = self.depth as usize;
            self.stack[child].change_from = (self.stack[level].bad_idx as c_int + to_len) as u8;
            self.stack[child].char_len = 0;
            break;
        }

        // The state test tells "the list ran out" apart from "an item
        // matched and pushed a level", which left it at `RepUndo`.
        //
        // SAFETY: as above.
        if self.stack[level].child as c_int >= unsafe { (*gap).ga_len }
            && self.stack[level].state == State::Rep
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
        // SAFETY: `child` still points just past the item that was
        // applied, whose two sides are NUL-terminated strings the language
        // owns, and `bad_idx` is a position inside the bad word.
        let gap = unsafe { self.rep_items() };
        let item = unsafe {
            ((*gap).ga_data as *mut fromto_T).offset(self.stack[level].child as isize - 1)
        };

        let from_len = unsafe { strlen((*item).ft_from) } as c_int;
        let to_len = unsafe { strlen((*item).ft_to) } as c_int;
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };
        if from_len != to_len {
            unsafe { move_tail(p, to_len, from_len) };
            self.repextra -= to_len - from_len;
        }
        unsafe { ptr::copy((*item).ft_from, p, from_len as usize) };

        self.stack[level].state = State::Rep;
    }

    /// The `REP` list this walk uses: the sound-fold one when walking the
    /// sound-fold tree, otherwise the replacement language's.
    ///
    /// # Safety
    ///
    /// The walk's language must be valid, and `lp_replang` must be
    /// non-null unless this is the sound-fold walk.
    unsafe fn rep_items(&self) -> *mut garray_T {
        // SAFETY: the caller guarantees the language; `rep_ini` is what
        // establishes the `lp_replang` precondition.
        if self.soundfold {
            unsafe { &raw mut (*self.slang).sl_repsal }
        } else {
            unsafe { &raw mut (*(*self.lp).lp_replang).sl_rep }
        }
    }
}

/// Shift the text after a replacement so that `to_len` bytes fit where
/// `from_len` were, terminator included.
///
/// # Safety
///
/// `p` must point into a NUL-terminated buffer with room for the shift.
unsafe fn move_tail(p: *mut core::ffi::c_char, from_len: c_int, to_len: c_int) {
    // SAFETY: the caller guarantees the buffer; the regions overlap, so
    // this has to be a move rather than a copy.
    let src = unsafe { p.offset(from_len as isize) };
    unsafe { ptr::copy(src, p.offset(to_len as isize), strlen(src) as usize + 1) };
}
