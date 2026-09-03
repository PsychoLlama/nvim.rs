//! Edits that move characters of the bad word around.
//!
//! Four rearrangements are tried, in this order:
//!
//! | from  | to    | state |
//! |-------|-------|-------|
//! | `12`  | `21`  | [`State::Swap`] |
//! | `123` | `321` | [`State::Swap3`] |
//! | `123` | `231` | rotate left, set up by [`State::UnSwap3`] |
//! | `123` | `312` | rotate right, set up by [`State::UnRot3L`] |
//!
//! Unlike every other edit, these change `fword` -- the bad word itself --
//! in place, because the tree is matched against it byte by byte and there
//! is nowhere else to put the rearranged text. Each therefore comes in a
//! pair: the state that makes the change pushes a level whose own state is
//! the one that puts the bad word back, so the change is undone exactly
//! when the walk returns to this level. The undo states then go straight
//! on to set up the next rearrangement, which is why one handler here
//! usually does two things.
//!
//! `Frame::change_from` is set on the child to just past the rearranged
//! text: everything before it has been altered already and must not be
//! altered again.
//!
//! # What is worth rearranging
//!
//! Swapping is only tried where the characters are word characters and
//! actually differ -- swapping a character with itself, or with something
//! that is not part of a word, cannot produce a word the tree has. For the
//! three-character forms, equal outer characters make all three
//! rearrangements either identical to the original or identical to a plain
//! swap, so they are skipped wholesale. The middle character may be
//! anything: "a.b" -> "b.a" is a legitimate swap.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::main::curwin;
use crate::mbyte::{char_at, char_len, cluster_len, encode_char, utf_char2len};
use crate::spell::spell_iswordp;
use crate::spellsuggest::walk::{State, Walk};
use crate::spellsuggest::{SCORE_SWAP, SCORE_SWAP3};
use crate::types::NUL;
use core::ffi::c_int;

impl Walk<'_> {
    /// The character at the front of `s`, and how many bytes it takes.
    ///
    /// Zero bytes at the word's terminator, which is what `utf_ptr2len`
    /// answered and what every step below is guarded by: a slice's own end
    /// is not the word's end, so the NUL has to be tested for by hand.
    #[inline]
    fn head(s: &[u8]) -> (c_int, usize) {
        match s.first() {
            None | Some(&0) => (NUL, 0),
            Some(_) => (char_at(s), char_len(s)),
        }
    }

    /// Swap two characters of the bad word: "12" -> "21".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn swap(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;

        if self.fword_at(bad_idx) == NUL {
            // The end of the word: nothing to swap or replace.
            self.stack[level].state = State::Final;
            return;
        }

        // A leading non-word character rules out the three-character
        // rearrangements as well.
        //
        // SAFETY: the byte at `bad_idx` was just tested non-NUL, so the
        // character there is inside the bad word.
        let p = self.fword_ptr(bad_idx);
        if !self.soundfold && !unsafe { spell_iswordp(p, curwin.get()) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        let word = self.fword_from(bad_idx);
        let (first, first_len) = Walk::head(word);
        let second = if word[first_len] == NUL as u8 {
            NUL
        } else if !self.soundfold
            // SAFETY: the byte after the first character was just tested
            // non-NUL, so the character there is inside the bad word too.
            && !unsafe { spell_iswordp(self.fword_ptr(bad_idx + first_len), curwin.get()) }
        {
            first // don't swap a non-word character
        } else {
            char_at(&self.fword[bad_idx + first_len..])
        };

        if second == NUL {
            // Only one character left.
            self.stack[level].state = State::RepIni;
            return;
        }
        if first == second {
            // Swapping identical characters changes nothing. This is
            // also where a non-word second character lands.
            self.stack[level].state = State::Swap3;
            return;
        }

        // SAFETY: `su` is the caller's suggestion state.
        if !unsafe { self.try_deeper(SCORE_SWAP) } {
            // If this swap is out of reach then SWAP3 is too.
            self.stack[level].state = State::RepIni;
            return;
        }

        self.go_deeper(SCORE_SWAP);
        self.stack[level].state = State::UnSwap;
        self.depth += 1;

        let second_len = utf_char2len(second) as usize;
        let word = self.fword_from(bad_idx);
        word.copy_within(first_len..first_len + second_len, 0);
        encode_char(first, &mut word[second_len..]);
        self.stack[self.depth as usize].change_from = (bad_idx + first_len + second_len) as u8;
    }

    /// Undo the swap -- "21" -> "12" -- and go straight on to try swapping
    /// two characters over a third, as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_swap(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;
        // The two characters are the ones `swap` wrote, so their lengths
        // are readable from the bad word itself and the rewrite stays
        // within the bytes they occupy.
        let word = self.fword_from(bad_idx);
        let first_len = cluster_len(word);
        let second = char_at(&word[first_len..]);
        let second_len = cluster_len(&word[first_len..]);
        word.copy_within(..first_len, second_len);
        encode_char(second, word);

        // `swap3` names its own successor on every path, so there is
        // nothing to set here.
        //
        // SAFETY: the bad word is valid by the contract above.
        unsafe { self.swap3() };
    }

    /// Swap two characters over a third: "123" -> "321".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn swap3(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;

        // Each length is measured from the character before it, so every
        // offset lands on the next character or on the terminator.
        let word = self.fword_from(bad_idx);
        let (first, first_len) = Walk::head(word);
        let (middle, middle_len) = Walk::head(&word[first_len..]);
        let third_at = first_len + middle_len;
        // SAFETY: `third_at` is a character boundary at or before the
        // terminator, so the character there is inside the bad word.
        let third = if !self.soundfold
            && !unsafe { spell_iswordp(self.fword_ptr(bad_idx + third_at), curwin.get()) }
        {
            first // don't swap a non-word character
        } else {
            Walk::head(&self.fword[bad_idx + third_at..]).0
        };

        // With "121" the result of this swap is the original, a left
        // rotation gives "211" which the plain swap already tried, and
        // a right rotation gives "112" which the plain swap at the
        // next character will try. So skip all three. A non-word third
        // character lands here too.
        if first == third || third == NUL {
            self.stack[level].state = State::RepIni;
            return;
        }

        // SAFETY: `su` is the caller's suggestion state.
        if !unsafe { self.try_deeper(SCORE_SWAP3) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        self.go_deeper(SCORE_SWAP3);
        self.stack[level].state = State::UnSwap3;
        self.depth += 1;

        let third_len = utf_char2len(third) as usize;
        let word = self.fword_from(bad_idx);
        word.copy_within(third_at..third_at + third_len, 0);
        encode_char(middle, &mut word[third_len..]);
        encode_char(first, &mut word[middle_len + third_len..]);
        self.stack[self.depth as usize].change_from =
            (bad_idx + first_len + middle_len + third_len) as u8;
    }

    /// Undo the three-character swap -- "321" -> "123" -- and go on to
    /// rotate the three left: "123" -> "231".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_swap3(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;

        // The three characters are the ones `swap3` wrote, so their
        // lengths are readable from the bad word itself and the rewrite
        // stays within the bytes they occupy.
        let word = self.fword_from(bad_idx);
        let third_len = cluster_len(word);
        let middle = char_at(&word[third_len..]);
        let middle_len = cluster_len(&word[third_len..]);
        let first = char_at(&word[third_len + middle_len..]);
        let first_len = cluster_len(&word[third_len + middle_len..]);
        word.copy_within(..third_len, middle_len + first_len);
        encode_char(first, word);
        encode_char(middle, &mut word[first_len..]);

        // The middle character was never checked: the first and third
        // were, at the swap and the three-way swap.
        //
        // SAFETY: `first_len` is a character boundary inside the bad word.
        let middle_at = self.fword_ptr(bad_idx + first_len);
        if !self.soundfold && !unsafe { spell_iswordp(middle_at, curwin.get()) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        // SAFETY: `su` is the caller's suggestion state.
        if !unsafe { self.try_deeper(SCORE_SWAP3) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        self.go_deeper(SCORE_SWAP3);
        self.stack[level].state = State::UnRot3L;
        self.depth += 1;

        // Rotate left: "123" -> "231". The three characters are still
        // where they were, and only their own bytes are rewritten.
        let word = self.fword_from(bad_idx);
        let (first, first_len) = Walk::head(word);
        let mut rest_len = Walk::head(&word[first_len..]).1;
        rest_len += Walk::head(&word[first_len + rest_len..]).1;
        word.copy_within(first_len..first_len + rest_len, 0);
        encode_char(first, &mut word[rest_len..]);
        self.stack[self.depth as usize].change_from = (bad_idx + first_len + rest_len) as u8;
    }

    /// Undo the left rotation -- "231" -> "123" -- and go on to rotate the
    /// three right: "123" -> "312".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_rot3l(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;

        // The three characters are the ones `un_rot3l`'s caller wrote, so
        // their lengths are readable from the bad word itself and the
        // rewrite stays within the bytes they occupy.
        let word = self.fword_from(bad_idx);
        let mut moved_len = cluster_len(word);
        moved_len += cluster_len(&word[moved_len..]);
        let last = char_at(&word[moved_len..]);
        let last_len = cluster_len(&word[moved_len..]);
        word.copy_within(..moved_len, last_len);
        encode_char(last, word);

        // SAFETY: `su` is the caller's suggestion state.
        if !unsafe { self.try_deeper(SCORE_SWAP3) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        self.go_deeper(SCORE_SWAP3);
        self.stack[level].state = State::UnRot3R;
        self.depth += 1;

        // Rotate right: "123" -> "312". The same three characters,
        // rewritten in place over their own bytes.
        let word = self.fword_from(bad_idx);
        let mut moved_len = Walk::head(word).1;
        moved_len += Walk::head(&word[moved_len..]).1;
        let (last, last_len) = Walk::head(&word[moved_len..]);
        word.copy_within(..moved_len, last_len);
        encode_char(last, word);
        self.stack[self.depth as usize].change_from = (bad_idx + moved_len + last_len) as u8;
    }

    /// Undo the right rotation -- "312" -> "123" -- and go straight on to
    /// the `REP` items, as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_rot3r(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;

        // The three characters are the ones `un_rot3l` wrote, so their
        // lengths are readable from the bad word itself and the rewrite
        // stays within the bytes they occupy.
        let word = self.fword_from(bad_idx);
        let first = char_at(word);
        let first_len = cluster_len(word);
        let mut rest_len = cluster_len(&word[first_len..]);
        rest_len += cluster_len(&word[first_len + rest_len..]);
        word.copy_within(first_len..first_len + rest_len, 0);
        encode_char(first, &mut word[rest_len..]);

        // `rep_ini` names its own successor on every path.
        //
        // SAFETY: the walk's trees and bad word are valid by the contract
        // above.
        unsafe { self.rep_ini() };
    }
}
