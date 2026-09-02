//! The three cheapest edits: take the byte, delete one, insert one.
//!
//! These are the states that actually move down the tree. [`State::Plain`]
//! is the ordinary case -- take each child byte of the node in turn, for
//! free if it matches the bad word and at [`SCORE_SUBST`] if it does not
//! -- and it is where the vast majority of the walk's time goes.
//! [`State::Del`] skips a character of the bad word, [`State::Ins`] adds
//! one from the tree without consuming any.
//!
//! Every handler here is `#[inline(always)]`: these are the states the
//! walk spends most of its rounds in, and leaving them as calls out of the
//! driver loop measurably slows the whole search down.
//!
//! # Multi-byte characters
//!
//! The tree stores bytes, but the scoring has to be per character: two
//! characters differ if *any* of their bytes do, and the lengths can then
//! differ as well. `Frame::char_len` and `Frame::char_idx` count the bytes
//! of the character currently being assembled in `tword`, and
//! `Frame::diff` remembers whether it has turned out to be a substitution,
//! an insertion or an exact match. Only when the last byte arrives is the
//! score settled -- and possibly discounted, because changing a composing
//! character, substituting a character the language's `MAP` lines call
//! similar, or doubling a character all cost less than the plain edit.
//! Until then no delete, insert or swap may be tried, which is what
//! [`State::Del`]'s first test enforces.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::main::got_int;
use crate::mbyte::{utf_iscomposing_legacy, utf_ptr2char, utf8len_tab, utfc_ptr2len};
use crate::spell::WordFlags;
use crate::spellsuggest::score::similar_chars;
use crate::spellsuggest::walk::{DIFF_INSERT, DIFF_NONE, DIFF_YES, FLAG_DID_DEL, State, Walk};
use crate::spellsuggest::{
    SCORE_DEL, SCORE_DELCOMP, SCORE_DELDUP, SCORE_INS, SCORE_INSCOMP, SCORE_INSDUP, SCORE_SIMILAR,
    SCORE_SUBCOMP, SCORE_SUBST,
};
use crate::types::{NUL, idx_T};
use core::ffi::{c_char, c_int};

/// The sound-fold marker for a word starting with a vowel. Adding or
/// dropping one is cheaper than a real edit, as `soundalike_score` also
/// assumes.
const SOUND_VOWEL: c_int = b'*' as c_int;

/// How many bytes a character starting with this byte has, from the
/// encoding's own table.
#[inline]
fn byte2len(c: c_int) -> u8 {
    utf8len_tab[c as usize]
}

impl Walk {
    /// Past the NUL bytes of the node: start taking its real bytes, unless
    /// the bad word has run out.
    ///
    /// Runs on into [`Walk::plain`] rather than going back round the
    /// driver loop, exactly as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    #[inline(always)]
    pub(super) unsafe fn end_nul(&mut self) {
        let level = self.depth as usize;
        // SAFETY: `su` is the caller's suggestion state.
        let saved = WordFlags::from_bits(self.stack[level].saved_badflags.into());
        unsafe { (*self.su).su_badflags = saved };

        // SAFETY: `bad_idx` is a position the walk reached inside the bad
        // word, the caller's NUL-terminated buffer.
        if unsafe { self.fword_at(self.stack[level].bad_idx as usize) } == NUL
            && self.stack[level].char_len == 0
        {
            // The bad word ends, so there is no byte to match against.
            self.stack[level].state = State::Del;
            return;
        }
        self.stack[level].state = State::Plain;
        // SAFETY: the trees and the bad word are valid by the contract
        // above, which is all `plain` asks for.
        unsafe { self.plain() };
    }

    /// Take one byte of this node: add it to the good word and go a level
    /// deeper, for free if it matches the bad word and at the cost of a
    /// substitution if it does not.
    ///
    /// This state is its own successor: each round takes one more byte,
    /// until the node runs out.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    #[inline(always)]
    pub(super) unsafe fn plain(&mut self) {
        let level = self.depth as usize;
        let node = self.stack[level].node;

        // SAFETY: `node` is a node the walk arrived at, and a node's first
        // byte is the count of children that follow it.
        if self.stack[level].child as c_int > unsafe { self.byte_at(node) } as c_int {
            // Every byte of this node has been taken. Where the bad
            // word has already been changed, skip the other tricks.
            self.stack[level].state = if self.stack[level].bad_idx >= self.stack[level].change_from
            {
                State::Del
            } else {
                State::Final
            };
            return;
        }

        let at = node + self.stack[level].child as idx_T;
        self.stack[level].child += 1;
        // SAFETY: `at` is this node's start plus a child number the count
        // just read bounds.
        let byte = unsafe { self.byte_at(at) } as c_int;

        let bad_idx = self.stack[level].bad_idx as usize;
        // Matching costs nothing. So does a byte in the middle of a
        // character that has already been paid for.
        //
        // SAFETY: `bad_idx` is a position the walk reached inside the bad
        // word, the caller's NUL-terminated buffer.
        let newscore = if byte == unsafe { self.fword_at(bad_idx) }
            || (self.stack[level].char_len > 0 && self.stack[level].diff != DIFF_NONE)
        {
            0
        } else {
            SCORE_SUBST
        };

        // Don't substitute where the bad word has already been
        // changed, and don't substitute a byte that was just deleted:
        // accepting it is always cheaper than delete plus substitute.
        //
        // SAFETY: `del_idx` is a bad-word position too, and the `&&` is
        // what keeps it from being read before a delete has set it.
        let undoing_a_delete = self.stack[level].flags & FLAG_DID_DEL != 0
            && byte == unsafe { self.fword_at(self.stack[level].del_idx as usize) };
        let allowed = newscore == 0
            || (self.stack[level].bad_idx >= self.stack[level].change_from && !undoing_a_delete);
        // SAFETY: `su` is the caller's suggestion state.
        if !allowed || !unsafe { self.try_deeper(newscore) } {
            return;
        }

        self.go_deeper(newscore);
        self.depth += 1;
        let child = self.depth as usize;

        // SAFETY: a bad-word position again, as above.
        if unsafe { self.fword_at(self.stack[child].bad_idx as usize) } != NUL {
            self.stack[child].bad_idx += 1;
        }
        self.tword[self.stack[child].good_len as usize] = byte as c_char;
        self.stack[child].good_len += 1;
        // SAFETY: `at` is a child byte of the node, so the entry beside it
        // is where that child's own node starts.
        self.stack[child].node = unsafe { self.idx_at(at) };
        if newscore == SCORE_SUBST {
            self.stack[child].diff = DIFF_YES;
        }

        if self.stack[child].char_len == 0 {
            // The first byte of a character.
            self.stack[child].char_idx = 0;
            self.stack[child].char_len = byte2len(byte);
            // Cannot underflow: `end_nul` sent a bad word that ends
            // here to STATE_DEL instead, so `bad_idx` was just
            // advanced above.
            self.stack[child].bad_char_start = (self.stack[child].bad_idx as c_int - 1) as u8;
            self.stack[child].diff = if newscore != 0 { DIFF_YES } else { DIFF_NONE };
        } else if self.stack[child].diff == DIFF_INSERT && self.stack[child].bad_idx > 0 {
            // Inserting the trail bytes of a character does not
            // advance in the bad word.
            self.stack[child].bad_idx -= 1;
        }

        self.stack[child].char_idx += 1;
        if self.stack[child].char_idx == self.stack[child].char_len {
            // SAFETY: `depth` is still the level the character was added
            // at, which is what `settle_character` asks for.
            unsafe { self.settle_character() };
            // Starting a new character.
            self.stack[child].char_len = 0;
        }
    }

    /// The last byte of a `tword` character has arrived: correct the
    /// position in the bad word and apply whatever discount the character
    /// as a whole has earned.
    ///
    /// # Safety
    ///
    /// `self.depth` must be the level the character was added at.
    #[inline(always)]
    unsafe fn settle_character(&mut self) {
        let level = self.depth as usize;
        let char_start = self.stack[level].bad_char_start as usize;
        let good_char = self.stack[level].good_len as usize - self.stack[level].char_len as usize;

        if self.stack[level].diff == DIFF_YES {
            // The characters differ, so the bad word's one may be a
            // different length than the bytes matched; that was not
            // checked while the bytes were coming in.
            //
            // SAFETY: `char_start` is where this character begins in the
            // bad word, a NUL-terminated buffer, so measuring the
            // character there stays inside it.
            self.stack[level].bad_idx = (self.stack[level].bad_char_start as c_int
                + unsafe { utfc_ptr2len(self.fword_ptr(char_start)) })
                as u8;

            // SAFETY: both are the first byte of a character, one in
            // `tword` -- this walk's own buffer -- and one in the bad
            // word; `slang` is the language of the walk's own trees.
            let good = unsafe { utf_ptr2char(self.tword.as_ptr().add(good_char)) };
            let bad = unsafe { utf_ptr2char(self.fword_ptr(char_start)) };
            if utf_iscomposing_legacy(good) && utf_iscomposing_legacy(bad) {
                // Changing a composing character counts for less.
                self.stack[level].score -= SCORE_SUBST - SCORE_SUBCOMP;
            } else if !self.soundfold
                && unsafe { (*self.slang).sl_has_map }
                && similar_chars(unsafe { &*self.slang }, good, bad)
            {
                // So does substituting a character the language's MAP
                // lines call similar.
                self.stack[level].score -= SCORE_SUBST - SCORE_SIMILAR;
            }
        } else if self.stack[level].diff == DIFF_INSERT
            && self.stack[level].good_len > self.stack[level].char_len
        {
            // SAFETY: `good_char` is where the character just added begins
            // in `tword`, this walk's own buffer.
            let mut p = unsafe { self.tword.as_mut_ptr().add(good_char) };
            let inserted = unsafe { utf_ptr2char(p) };
            if utf_iscomposing_legacy(inserted) {
                // Inserting a composing character does not count for
                // much.
                self.stack[level].score -= SCORE_INS - SCORE_INSCOMP;
            } else {
                // Doubling a character earns a bonus. Illogical for
                // the sound-fold tree, but it does score better there
                // too.
                //
                // SAFETY: the branch above tested that `good_len` is past
                // this character, so `p` is past `tword`'s start and there
                // is a character before it to step back over.
                p = unsafe { Walk::char_back(self.tword.as_ptr(), p) };
                if inserted == unsafe { utf_ptr2char(p) } {
                    self.stack[level].score -= SCORE_INS - SCORE_INSDUP;
                }
            }
        }
    }

    /// Skip one character of the bad word: delete it.
    ///
    /// Runs on into [`Walk::ins_prep`] when there was nothing to delete,
    /// exactly as the C fell through.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    #[inline(always)]
    pub(super) unsafe fn delete(&mut self) {
        let level = self.depth as usize;

        if self.stack[level].char_len > 0 {
            // Past the first byte of a character: delete, insert and
            // swap all have to wait for the character to finish.
            self.stack[level].state = State::Final;
            return;
        }

        self.stack[level].state = State::InsPrep;
        self.stack[level].child = 1;

        let bad_idx = self.stack[level].bad_idx as usize;
        // SAFETY: `bad_idx` is a position the walk reached inside the bad
        // word, the caller's NUL-terminated buffer.
        let newscore = if self.soundfold
            && self.stack[level].bad_idx == 0
            && unsafe { self.fword_at(bad_idx) } == SOUND_VOWEL
        {
            // Deleting a leading vowel counts less; `soundalike_score`
            // charges the same.
            2 * SCORE_DEL / 3
        } else {
            SCORE_DEL
        };

        // SAFETY: as above, and `su` is the caller's suggestion state.
        if !(unsafe { self.fword_at(bad_idx) } != NUL && unsafe { self.try_deeper(newscore) }) {
            // SAFETY: the walk's trees are valid by the contract above.
            unsafe { self.ins_prep() };
            return;
        }

        self.go_deeper(newscore);
        self.depth += 1;
        let child = self.depth as usize;

        // Remember the deleted character so that it is not inserted
        // again straight away.
        self.stack[child].flags |= FLAG_DID_DEL;
        self.stack[child].del_idx = self.stack[level].bad_idx;

        // Advance over the character. Deleting one of a doubled pair
        // -- "nn" -> "n" -- earns a bonus; that is a little illogical
        // for the sound-fold tree but it scores better there too.
        //
        // SAFETY: `bad_idx` is the first byte of a character of the bad
        // word, and stepping over that character lands on the next one or
        // on the terminator.
        let deleted = unsafe { utf_ptr2char(self.fword_ptr(bad_idx)) };
        self.stack[child].bad_idx = (self.stack[child].bad_idx as c_int
            + unsafe { utfc_ptr2len(self.fword_ptr(bad_idx)) })
            as u8;
        if utf_iscomposing_legacy(deleted) {
            self.stack[child].score -= SCORE_DEL - SCORE_DELCOMP;
        } else if deleted
            == unsafe { utf_ptr2char(self.fword_ptr(self.stack[child].bad_idx as usize)) }
        {
            self.stack[child].score -= SCORE_DEL - SCORE_DELDUP;
        }
    }

    /// Find the first byte of this node worth inserting, or give up on
    /// inserting here.
    ///
    /// # Safety
    ///
    /// The walk's trees must be valid.
    #[inline(always)]
    pub(super) unsafe fn ins_prep(&mut self) {
        let level = self.depth as usize;

        if self.stack[level].flags & FLAG_DID_DEL != 0 {
            // A byte was just deleted, so inserting one makes no
            // sense: a substitution is always cheaper.
            self.stack[level].state = State::Swap;
            return;
        }

        // Skip over the NUL bytes.
        let node = self.stack[level].node;
        loop {
            // SAFETY: `node` is a node the walk arrived at, whose first
            // byte is the count of children that follow it; the loop reads
            // a child only after finding the child number within it.
            if self.stack[level].child as c_int > unsafe { self.byte_at(node) } as c_int {
                // Only NUL bytes at this node.
                self.stack[level].state = State::Swap;
                return;
            }
            if unsafe { self.byte_at(node + self.stack[level].child as idx_T) } != NUL as u8 {
                // Found a byte to insert.
                self.stack[level].state = State::Ins;
                return;
            }
            self.stack[level].child += 1;
        }
    }

    /// Insert one byte of this node into the bad word.
    ///
    /// This state is its own successor: each round inserts the next byte
    /// of the node instead.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    #[inline(always)]
    pub(super) unsafe fn insert(&mut self) {
        let level = self.depth as usize;
        let node = self.stack[level].node;

        // SAFETY: `node` is a node the walk arrived at, and a node's first
        // byte is the count of children that follow it.
        if self.stack[level].child as c_int > unsafe { self.byte_at(node) } as c_int {
            // Every byte of this node has been tried.
            self.stack[level].state = State::Swap;
            return;
        }

        let at = node + self.stack[level].child as idx_T;
        self.stack[level].child += 1;

        // A bounds check the tree itself should have made unnecessary;
        // giving up is how the C reacted to a tree that disagrees with
        // its own length.
        //
        // SAFETY: `slang` is the language of the walk's own trees.
        if self.byts == unsafe { (*self.slang).sl_fbyts }
            && at >= unsafe { (*self.slang).sl_fbyts_len }
        {
            got_int.set(true);
            return;
        }

        // SAFETY: `at` is the node's start plus a child number the count
        // above bounds, and for the case-folded tree the check just above
        // has bounded it by the tree's own length as well.
        let byte = unsafe { self.byte_at(at) } as c_int;
        let newscore = if self.soundfold && self.stack[level].good_len == 0 && byte == SOUND_VOWEL {
            // Inserting a leading vowel counts less; see
            // `soundalike_score`.
            2 * SCORE_INS / 3
        } else {
            SCORE_INS
        };

        // Skip a byte equal to the bad word's: accepting it, which
        // STATE_PLAIN already did, is always better.
        //
        // SAFETY: `bad_idx` is a position inside the bad word, and `su` is
        // the caller's suggestion state.
        if byte == unsafe { self.fword_at(self.stack[level].bad_idx as usize) }
            || !unsafe { self.try_deeper(newscore) }
        {
            return;
        }

        self.go_deeper(newscore);
        self.depth += 1;
        let child = self.depth as usize;

        self.tword[self.stack[child].good_len as usize] = byte as c_char;
        self.stack[child].good_len += 1;
        // SAFETY: `at` is a child byte of the node, so the entry beside it
        // is where that child's own node starts.
        self.stack[child].node = unsafe { self.idx_at(at) };

        let char_len = byte2len(byte);
        if char_len > 1 {
            // More bytes of the same character follow; they all have
            // to arrive before any other edit is tried.
            self.stack[child].char_len = char_len;
            self.stack[child].char_idx = 1;
            self.stack[child].diff = DIFF_INSERT;
        } else {
            // Doubling a character earns a bonus, in the sound-fold
            // tree as well.
            let good_len = self.stack[child].good_len as usize;
            if good_len >= 2 && self.tword[good_len - 2] as u8 as c_int == byte {
                self.stack[child].score -= SCORE_INS - SCORE_INSDUP;
            }
        }
    }
}
