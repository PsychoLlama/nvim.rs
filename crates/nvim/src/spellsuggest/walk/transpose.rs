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
use crate::mbyte::{utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len, utfc_ptr2len};
use crate::spell::spell_iswordp;
use crate::spellsuggest::walk::{State, Walk};
use crate::spellsuggest::{NUL, SCORE_SWAP, SCORE_SWAP3};
use core::ffi::c_int;
use core::ptr;

impl Walk {
    /// Swap two characters of the bad word: "12" -> "21".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn swap(&mut self) {
        // SAFETY: the bad word is the caller's NUL-terminated buffer, and
        // the rewrite below only moves characters it has just measured.
        unsafe {
            let level = self.depth as usize;
            let bad_idx = self.stack[level].bad_idx as usize;
            let p = self.fword_ptr(bad_idx);

            if self.fword_at(bad_idx) == NUL {
                // The end of the word: nothing to swap or replace.
                self.stack[level].state = State::Final;
                return;
            }

            // A leading non-word character rules out the three-character
            // rearrangements as well.
            if !self.soundfold && !spell_iswordp(p, curwin.get()) {
                self.stack[level].state = State::RepIni;
                return;
            }

            let first_len = utf_ptr2len(p);
            let first = utf_ptr2char(p);
            let second = if *p.offset(first_len as isize) == NUL as core::ffi::c_char {
                NUL
            } else if !self.soundfold && !spell_iswordp(p.offset(first_len as isize), curwin.get())
            {
                first // don't swap a non-word character
            } else {
                utf_ptr2char(p.offset(first_len as isize))
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

            if !self.try_deeper(SCORE_SWAP) {
                // If this swap is out of reach then SWAP3 is too.
                self.stack[level].state = State::RepIni;
                return;
            }

            self.go_deeper(SCORE_SWAP);
            self.stack[level].state = State::UnSwap;
            self.depth += 1;

            let second_len = utf_char2len(second);
            ptr::copy(p.offset(first_len as isize), p, second_len as usize);
            utf_char2bytes(first, p.offset(second_len as isize));
            self.stack[self.depth as usize].change_from =
                (self.stack[level].bad_idx as c_int + first_len + second_len) as u8;
        }
    }

    /// Undo the swap -- "21" -> "12" -- and go straight on to try swapping
    /// two characters over a third, as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_swap(&mut self) {
        // SAFETY: the two characters are the ones `swap` wrote, so their
        // lengths are readable from the bad word itself.
        unsafe {
            let level = self.depth as usize;
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);
            let first_len = utfc_ptr2len(p);
            let second = utf_ptr2char(p.offset(first_len as isize));
            ptr::copy(
                p,
                p.offset(utfc_ptr2len(p.offset(first_len as isize)) as isize),
                first_len as usize,
            );
            utf_char2bytes(second, p);

            // `swap3` names its own successor on every path, so there is
            // nothing to set here.
            self.swap3();
        }
    }

    /// Swap two characters over a third: "123" -> "321".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn swap3(&mut self) {
        // SAFETY: as `swap`.
        unsafe {
            let level = self.depth as usize;
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);

            let first_len = utf_ptr2len(p);
            let first = utf_ptr2char(p);
            let middle_len = utf_ptr2len(p.offset(first_len as isize));
            let middle = utf_ptr2char(p.offset(first_len as isize));
            let third_at = p.offset((first_len + middle_len) as isize);
            let third = if !self.soundfold && !spell_iswordp(third_at, curwin.get()) {
                first // don't swap a non-word character
            } else {
                utf_ptr2char(third_at)
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

            if !self.try_deeper(SCORE_SWAP3) {
                self.stack[level].state = State::RepIni;
                return;
            }

            self.go_deeper(SCORE_SWAP3);
            self.stack[level].state = State::UnSwap3;
            self.depth += 1;

            let third_len = utf_char2len(third);
            ptr::copy(third_at, p, third_len as usize);
            utf_char2bytes(middle, p.offset(third_len as isize));
            utf_char2bytes(first, p.offset((middle_len + third_len) as isize));
            self.stack[self.depth as usize].change_from =
                (self.stack[level].bad_idx as c_int + first_len + middle_len + third_len) as u8;
        }
    }

    /// Undo the three-character swap -- "321" -> "123" -- and go on to
    /// rotate the three left: "123" -> "231".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_swap3(&mut self) {
        // SAFETY: the three characters are the ones `swap3` wrote.
        unsafe {
            let level = self.depth as usize;
            let mut p = self.fword_ptr(self.stack[level].bad_idx as usize);

            let third_len = utfc_ptr2len(p);
            let middle = utf_ptr2char(p.offset(third_len as isize));
            let middle_len = utfc_ptr2len(p.offset(third_len as isize));
            let first = utf_ptr2char(p.offset((third_len + middle_len) as isize));
            let first_len = utfc_ptr2len(p.offset((third_len + middle_len) as isize));
            ptr::copy(
                p,
                p.offset((middle_len + first_len) as isize),
                third_len as usize,
            );
            utf_char2bytes(first, p);
            utf_char2bytes(middle, p.offset(first_len as isize));

            // The middle character was never checked: the first and third
            // were, at the swap and the three-way swap.
            p = p.offset(first_len as isize);
            if !self.soundfold && !spell_iswordp(p, curwin.get()) {
                self.stack[level].state = State::RepIni;
                return;
            }

            if !self.try_deeper(SCORE_SWAP3) {
                self.stack[level].state = State::RepIni;
                return;
            }

            self.go_deeper(SCORE_SWAP3);
            self.stack[level].state = State::UnRot3L;
            self.depth += 1;

            // Rotate left: "123" -> "231".
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);
            let first_len = utf_ptr2len(p);
            let first = utf_ptr2char(p);
            let mut rest_len = utf_ptr2len(p.offset(first_len as isize));
            rest_len += utf_ptr2len(p.offset((first_len + rest_len) as isize));
            ptr::copy(p.offset(first_len as isize), p, rest_len as usize);
            utf_char2bytes(first, p.offset(rest_len as isize));
            self.stack[self.depth as usize].change_from =
                (self.stack[level].bad_idx as c_int + first_len + rest_len) as u8;
        }
    }

    /// Undo the left rotation -- "231" -> "123" -- and go on to rotate the
    /// three right: "123" -> "312".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_rot3l(&mut self) {
        // SAFETY: the three characters are the ones `un_swap3` wrote.
        unsafe {
            let level = self.depth as usize;
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);

            let mut moved_len = utfc_ptr2len(p);
            moved_len += utfc_ptr2len(p.offset(moved_len as isize));
            let last = utf_ptr2char(p.offset(moved_len as isize));
            let last_len = utfc_ptr2len(p.offset(moved_len as isize));
            ptr::copy(p, p.offset(last_len as isize), moved_len as usize);
            utf_char2bytes(last, p);

            if !self.try_deeper(SCORE_SWAP3) {
                self.stack[level].state = State::RepIni;
                return;
            }

            self.go_deeper(SCORE_SWAP3);
            self.stack[level].state = State::UnRot3R;
            self.depth += 1;

            // Rotate right: "123" -> "312".
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);
            let mut moved_len = utf_ptr2len(p);
            moved_len += utf_ptr2len(p.offset(moved_len as isize));
            let last = utf_ptr2char(p.offset(moved_len as isize));
            let last_len = utf_ptr2len(p.offset(moved_len as isize));
            ptr::copy(p, p.offset(last_len as isize), moved_len as usize);
            utf_char2bytes(last, p);
            self.stack[self.depth as usize].change_from =
                (self.stack[level].bad_idx as c_int + moved_len + last_len) as u8;
        }
    }

    /// Undo the right rotation -- "312" -> "123" -- and go straight on to
    /// the `REP` items, as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_rot3r(&mut self) {
        // SAFETY: the three characters are the ones `un_rot3l` wrote.
        unsafe {
            let level = self.depth as usize;
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);

            let first = utf_ptr2char(p);
            let first_len = utfc_ptr2len(p);
            let mut rest_len = utfc_ptr2len(p.offset(first_len as isize));
            rest_len += utfc_ptr2len(p.offset((first_len + rest_len) as isize));
            ptr::copy(p.offset(first_len as isize), p, rest_len as usize);
            utf_char2bytes(first, p.offset(rest_len as isize));

            // `rep_ini` names its own successor on every path.
            self.rep_ini();
        }
    }
}
