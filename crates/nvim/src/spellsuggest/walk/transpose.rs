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
use crate::spellsuggest::{SCORE_SWAP, SCORE_SWAP3};
use crate::types::NUL;
use core::ffi::c_int;
use core::ptr;

impl Walk<'_> {
    /// Swap two characters of the bad word: "12" -> "21".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn swap(&mut self) {
        let level = self.depth as usize;
        let bad_idx = self.stack[level].bad_idx as usize;
        // SAFETY: `bad_idx` is a position the walk reached inside the bad
        // word, the caller's NUL-terminated buffer.
        let p = unsafe { self.fword_ptr(bad_idx) };

        if unsafe { self.fword_at(bad_idx) } == NUL {
            // The end of the word: nothing to swap or replace.
            self.stack[level].state = State::Final;
            return;
        }

        // A leading non-word character rules out the three-character
        // rearrangements as well.
        //
        // SAFETY: `p` is a character of the bad word, and the byte there
        // was just tested non-NUL.
        if !self.soundfold && !unsafe { spell_iswordp(p, curwin.get()) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        // SAFETY: `first_len` is the length of the character at `p`, so
        // `p + first_len` is the next character or the terminator; the
        // branches below read past it only after seeing it is not NUL.
        let first_len = unsafe { utf_ptr2len(p) };
        let first = unsafe { utf_ptr2char(p) };
        let second = if unsafe { *p.offset(first_len as isize) } == NUL as core::ffi::c_char {
            NUL
        } else if !self.soundfold
            && !unsafe { spell_iswordp(p.offset(first_len as isize), curwin.get()) }
        {
            first // don't swap a non-word character
        } else {
            unsafe { utf_ptr2char(p.offset(first_len as isize)) }
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

        let second_len = utf_char2len(second);
        // SAFETY: the rewrite only moves the two characters just measured,
        // so it stays within the bytes they already occupy in the bad
        // word.
        unsafe { ptr::copy(p.offset(first_len as isize), p, second_len as usize) };
        unsafe { utf_char2bytes(first, p.offset(second_len as isize)) };
        self.stack[self.depth as usize].change_from =
            (self.stack[level].bad_idx as c_int + first_len + second_len) as u8;
    }

    /// Undo the swap -- "21" -> "12" -- and go straight on to try swapping
    /// two characters over a third, as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_swap(&mut self) {
        let level = self.depth as usize;
        // SAFETY: the two characters are the ones `swap` wrote, so their
        // lengths are readable from the bad word itself and the rewrite
        // stays within the bytes they occupy.
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };
        let first_len = unsafe { utfc_ptr2len(p) };
        let second = unsafe { utf_ptr2char(p.offset(first_len as isize)) };
        unsafe {
            ptr::copy(
                p,
                p.offset(utfc_ptr2len(p.offset(first_len as isize)) as isize),
                first_len as usize,
            )
        };
        unsafe { utf_char2bytes(second, p) };

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

        // SAFETY: as `swap` -- each length is measured from the character
        // before it, so every offset lands on the next character or on the
        // terminator of the caller's NUL-terminated bad word.
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };

        let first_len = unsafe { utf_ptr2len(p) };
        let first = unsafe { utf_ptr2char(p) };
        let middle_len = unsafe { utf_ptr2len(p.offset(first_len as isize)) };
        let middle = unsafe { utf_ptr2char(p.offset(first_len as isize)) };
        let third_at = unsafe { p.offset((first_len + middle_len) as isize) };
        let third = if !self.soundfold && !unsafe { spell_iswordp(third_at, curwin.get()) } {
            first // don't swap a non-word character
        } else {
            unsafe { utf_ptr2char(third_at) }
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

        let third_len = utf_char2len(third);
        // SAFETY: the rewrite only moves the three characters just
        // measured, so it stays within the bytes they already occupy.
        unsafe { ptr::copy(third_at, p, third_len as usize) };
        unsafe { utf_char2bytes(middle, p.offset(third_len as isize)) };
        unsafe { utf_char2bytes(first, p.offset((middle_len + third_len) as isize)) };
        self.stack[self.depth as usize].change_from =
            (self.stack[level].bad_idx as c_int + first_len + middle_len + third_len) as u8;
    }

    /// Undo the three-character swap -- "321" -> "123" -- and go on to
    /// rotate the three left: "123" -> "231".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_swap3(&mut self) {
        let level = self.depth as usize;

        // SAFETY: the three characters are the ones `swap3` wrote, so
        // their lengths are readable from the bad word itself and the
        // rewrite stays within the bytes they occupy.
        let mut p = unsafe {
            let p = self.fword_ptr(self.stack[level].bad_idx as usize);

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
            p.offset(first_len as isize)
        };
        if !self.soundfold && !unsafe { spell_iswordp(p, curwin.get()) } {
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

        // Rotate left: "123" -> "231".
        //
        // SAFETY: as above -- the three characters are still where they
        // were, and only their own bytes are rewritten.
        p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };
        let first_len = unsafe { utf_ptr2len(p) };
        let first = unsafe { utf_ptr2char(p) };
        let mut rest_len = unsafe { utf_ptr2len(p.offset(first_len as isize)) };
        rest_len += unsafe { utf_ptr2len(p.offset((first_len + rest_len) as isize)) };
        unsafe { ptr::copy(p.offset(first_len as isize), p, rest_len as usize) };
        unsafe { utf_char2bytes(first, p.offset(rest_len as isize)) };
        self.stack[self.depth as usize].change_from =
            (self.stack[level].bad_idx as c_int + first_len + rest_len) as u8;
    }

    /// Undo the left rotation -- "231" -> "123" -- and go on to rotate the
    /// three right: "123" -> "312".
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_rot3l(&mut self) {
        let level = self.depth as usize;

        // SAFETY: the three characters are the ones `un_rot3l`'s caller
        // wrote, so their lengths are readable from the bad word itself
        // and the rewrite stays within the bytes they occupy.
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };

        let mut moved_len = unsafe { utfc_ptr2len(p) };
        moved_len += unsafe { utfc_ptr2len(p.offset(moved_len as isize)) };
        let last = unsafe { utf_ptr2char(p.offset(moved_len as isize)) };
        let last_len = unsafe { utfc_ptr2len(p.offset(moved_len as isize)) };
        unsafe { ptr::copy(p, p.offset(last_len as isize), moved_len as usize) };
        unsafe { utf_char2bytes(last, p) };

        // SAFETY: `su` is the caller's suggestion state.
        if !unsafe { self.try_deeper(SCORE_SWAP3) } {
            self.stack[level].state = State::RepIni;
            return;
        }

        self.go_deeper(SCORE_SWAP3);
        self.stack[level].state = State::UnRot3R;
        self.depth += 1;

        // Rotate right: "123" -> "312".
        //
        // SAFETY: as above -- the same three characters, rewritten in
        // place over their own bytes.
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };
        let mut moved_len = unsafe { utf_ptr2len(p) };
        moved_len += unsafe { utf_ptr2len(p.offset(moved_len as isize)) };
        let last = unsafe { utf_ptr2char(p.offset(moved_len as isize)) };
        let last_len = unsafe { utf_ptr2len(p.offset(moved_len as isize)) };
        unsafe { ptr::copy(p, p.offset(last_len as isize), moved_len as usize) };
        unsafe { utf_char2bytes(last, p) };
        self.stack[self.depth as usize].change_from =
            (self.stack[level].bad_idx as c_int + moved_len + last_len) as u8;
    }

    /// Undo the right rotation -- "312" -> "123" -- and go straight on to
    /// the `REP` items, as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    pub(super) unsafe fn un_rot3r(&mut self) {
        let level = self.depth as usize;

        // SAFETY: the three characters are the ones `un_rot3l` wrote, so
        // their lengths are readable from the bad word itself and the
        // rewrite stays within the bytes they occupy.
        let p = unsafe { self.fword_ptr(self.stack[level].bad_idx as usize) };

        let first = unsafe { utf_ptr2char(p) };
        let first_len = unsafe { utfc_ptr2len(p) };
        let mut rest_len = unsafe { utfc_ptr2len(p.offset(first_len as isize)) };
        rest_len += unsafe { utfc_ptr2len(p.offset((first_len + rest_len) as isize)) };
        unsafe { ptr::copy(p.offset(first_len as isize), p, rest_len as usize) };
        unsafe { utf_char2bytes(first, p.offset(rest_len as isize)) };

        // `rep_ini` names its own successor on every path.
        //
        // SAFETY: the walk's trees and bad word are valid by the contract
        // above.
        unsafe { self.rep_ini() };
    }
}
