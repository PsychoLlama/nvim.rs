//! Continuing past a word that ended: as a compound, or as a split.
//!
//! This runs at the same NUL byte [`super::node`] does, straight after it,
//! and it is what lets one suggestion be built out of more than one
//! dictionary word. There are two ways to carry on:
//!
//! - **compounding** -- the language's `COMPOUND*` rules allow this word
//!   to be glued directly onto the next, and
//! - **splitting** -- a space is inserted and the rest of the bad word is
//!   matched as a fresh word, which is what finds "thequick" -> "the
//!   quick" and also what finds a correction in the *second* word of "the
//!   teh".
//!
//! Either way the walk restarts at the root of the tree one level down,
//! with `preword` carrying what has been built so far, and the parent
//! level is left in [`State::SplitUndo`] so that the bad word's caps type
//! is put back when the walk comes home.
//!
//! # Doing the same NUL twice
//!
//! When a word could both compound and split, the split is tried first and
//! the compound afterwards. That is arranged by winding this level's child
//! counter *back* one so the same NUL byte comes round again, with
//! [`FLAG_DID_SPLIT`] set to say the split has been done -- which is the
//! only thing stopping the two from taking turns forever.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::charset::{skiptowhite, skipwhite};
use crate::main::curwin;
use crate::mbyte::{mb_charlen, utfc_ptr2len};
use crate::memory::xstrlcat;
use crate::spell::WordFlags;
use crate::spell::{byte_in_str, can_compound, match_compoundrule, nofold_len, spell_iswordp_nmw};
use crate::spellsuggest::score::score_wordcount_adj;
use crate::spellsuggest::walk::{FLAG_DID_SPLIT, PFD_NOPREFIX, PFD_PREFIXTREE, State, Walk};
use crate::spellsuggest::{MAXWLEN, SCORE_SPLIT, SCORE_SPLIT_NO, SCORE_SUBST, badword_captype};
use crate::types::NUL;
use core::ffi::{c_char, c_int};
use core::ptr;

impl Walk<'_> {
    /// Try continuing the word that just ended, by compounding or by
    /// splitting the bad word here.
    ///
    /// `newscore` is what the word cost so far; a split adds its own
    /// penalty to it.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    pub(super) unsafe fn try_split_or_compound(
        &mut self,
        flags: WordFlags,
        bad_word_ends: bool,
        good_word_ends: bool,
        mut newscore: c_int,
    ) {
        let level = self.depth as usize;

        // Only where a change is still allowed, and never in the
        // middle of a character.
        if (self.stack[level].bad_idx < self.stack[level].change_from && !bad_word_ends)
            || self.stack[level].char_len != 0
        {
            return;
        }

        // Past the end of the bad word there is nothing left to split
        // off. Otherwise a split lets the *next* word be changed, as
        // in "the the" where it is the second "the" that is wrong.
        //
        // SAFETY: `su` is the caller's suggestion state, valid by the
        // contract above.
        let try_split = (self.stack[level].bad_idx as c_int - self.repextra)
            < unsafe { (*self.su).su_badlen }
            && !self.soundfold;

        // SAFETY: the walk's language is valid by the contract above.
        let mut try_compound = unsafe { self.may_compound(flags) };
        if try_compound {
            let comp_len = self.stack[level].comp_len as usize;
            self.compflags[comp_len] = ((flags.bits() as u32) >> 24) as u8;
            self.compflags[comp_len + 1] = NUL as u8;
        }

        // SAFETY: `slang` is the language of the walk's own trees, and the
        // `&&` keeps the second read to the NOBREAK case.
        if unsafe { (*self.slang).sl_nobreak } && !unsafe { (*self.slang).sl_nocompoundsugs } {
            // With NOBREAK a split can never make a word valid, so
            // compounding is the only way to check what follows.
            try_compound = true;
        } else if !bad_word_ends && try_compound && self.stack[level].flags & FLAG_DID_SPLIT == 0 {
            // Both are possible here: do the split now and come back
            // for the compound, without looping between the two.
            try_compound = false;
            self.stack[level].flags |= FLAG_DID_SPLIT;
            self.stack[level].child -= 1; // do the same NUL again
            let comp_len = self.stack[level].comp_len as usize;
            self.compflags[comp_len] = NUL as u8;
        } else {
            self.stack[level].flags &= !FLAG_DID_SPLIT;
        }

        if !try_split && !try_compound {
            return;
        }

        if !try_compound && (!bad_word_ends || !good_word_ends) {
            // SAFETY: the walk's state is valid by the contract above.
            match unsafe { self.split_penalty(flags, newscore) } {
                None => return,
                Some(score) => newscore = score,
            }
        }

        // SAFETY: `su` is the caller's, as above.
        if !unsafe { self.try_deeper(newscore) } {
            return;
        }
        self.go_deeper(newscore);

        // Saved so that STATE_SPLITUNDO can put it back.
        //
        // SAFETY: as above.
        self.stack[level].saved_badflags = unsafe { (*self.su).su_badflags }.bits() as u8;
        self.stack[level].state = State::SplitUndo;

        self.depth += 1;
        let child = self.depth as usize;

        if !try_compound && !bad_word_ends {
            // SAFETY: `preword` is this walk's own NUL-terminated buffer,
            // with room for the separator after the word it holds.
            let room = self.preword.len();
            unsafe { xstrlcat(self.preword.as_mut_ptr(), c" ".as_ptr(), room) };
        }
        self.stack[child].preword_len = self.preword_len() as u8;
        self.stack[child].split_off = self.stack[child].good_len;
        self.stack[child].split_bad_idx = self.stack[child].bad_idx;

        // SAFETY: the bad word is valid by the contract above.
        unsafe { self.skip_split_character(try_compound, bad_word_ends, good_word_ends) };

        // Compounding keeps collecting flags; splitting may start
        // compounding over from here.
        if try_compound {
            self.stack[child].comp_len += 1;
        } else {
            self.stack[child].comp_split = self.stack[child].comp_len;
        }
        self.stack[child].prefix_depth = PFD_NOPREFIX;

        // The caps type for what is left of the bad word.
        //
        // SAFETY: `su` and its bad word are valid by the contract above,
        // and `consumed` is a length `nofold_len` measured out of that
        // word, so both offsets stay inside it.
        unsafe {
            let consumed = nofold_len(
                self.fword.as_mut_ptr().cast(),
                self.stack[child].bad_idx as c_int,
                (*self.su).su_badptr,
            );
            (*self.su).su_badflags = badword_captype(
                (*self.su).su_badptr.offset(consumed as isize),
                (*self.su).su_badptr.offset((*self.su).su_badlen as isize),
            );
        }

        // Restart at the top of the tree.
        self.stack[child].node = 0;

        // Postponed prefixes apply to the new word too.
        if !self.prefix_tree.is_empty() {
            self.tree = self.prefix_tree;
            self.stack[child].prefix_depth = PFD_PREFIXTREE;
            self.stack[child].state = State::NoPrefix;
        }
    }

    /// Do the language's compounding rules allow this word to be glued to
    /// the next one?
    ///
    /// # Safety
    ///
    /// The walk's language must be valid.
    unsafe fn may_compound(&mut self, flags: WordFlags) -> bool {
        let level = self.depth as usize;
        let split_off = self.stack[level].split_off as usize;
        let this_word_len =
            self.stack[level].good_len as c_int - self.stack[level].split_off as c_int;

        // SAFETY: the language's compound settings are plain fields of the
        // walk's own `slang`, and `tword` is its own NUL-terminated
        // buffer; every `&&` below guards only the work to its right, not
        // the validity of what it reads.
        !self.soundfold
            && !unsafe { (*self.slang).sl_nocompoundsugs }
            && !unsafe { (*self.slang).sl_compprog }.is_null()
            && (flags.bits() as u32) >> 24 != 0
            && this_word_len >= unsafe { (*self.slang).sl_compminlen }
            && (unsafe { (*self.slang).sl_compminlen } == 0
                || unsafe { mb_charlen(self.tword.as_ptr().add(split_off)) }
                    >= unsafe { (*self.slang).sl_compminlen })
            && (unsafe { (*self.slang).sl_compsylmax } < MAXWLEN as c_int
                || (self.stack[level].comp_len as c_int + 1
                    - self.stack[level].comp_split as c_int)
                    < unsafe { (*self.slang).sl_compmax })
            && unsafe { self.can_be_compound(((flags.bits() as u32) >> 24) as c_int) }
    }

    /// What a split costs, and whether it is allowed at all.
    ///
    /// Returns `None` when the words collected so far could not stand as
    /// separate words, which ends this NUL byte's turn.
    ///
    /// # Safety
    ///
    /// The walk's state must be valid.
    unsafe fn split_penalty(&mut self, flags: WordFlags, mut newscore: c_int) -> Option<c_int> {
        let level = self.depth as usize;

        // Splitting means the words so far have to be valid on their
        // own. A single word must not carry NEEDCOMPOUND.
        if self.stack[level].comp_len == self.stack[level].comp_split
            && flags.has(WordFlags::NEEDCOMP)
        {
            return None;
        }
        let mut last_word = self.preword.as_mut_ptr();
        // SAFETY: `preword` is this walk's own NUL-terminated buffer, so
        // `skiptowhite` stops inside it and the byte it stops at is one of
        // the buffer's own; `skipwhite` then stops at the next word.
        while unsafe { *skiptowhite(last_word) } != NUL as c_char {
            last_word = unsafe { skipwhite(skiptowhite(last_word)) };
        }
        if self.stack[level].comp_len > self.stack[level].comp_split {
            let comp_split = self.stack[level].comp_split as usize;
            // SAFETY: `slang` is the walk's own language, `last_word` a
            // NUL-terminated word of `preword`, and `comp_split` indexes
            // `compflags`, this walk's own array.
            let joins = unsafe {
                can_compound(
                    self.slang,
                    last_word,
                    self.compflags.as_ptr().add(comp_split),
                )
            };
            if !joins {
                return None;
            }
        }

        // SAFETY: `slang` is the walk's own language.
        newscore += if unsafe { (*self.slang).sl_nosplitsugs } {
            SCORE_SPLIT_NO
        } else {
            SCORE_SPLIT
        };

        // Give a bonus to words seen before.
        let preword_len = self.stack[level].preword_len as usize;
        // SAFETY: as above, and `preword_len` is a length this walk
        // measured out of its own buffer.
        Some(unsafe {
            score_wordcount_adj(
                &*self.slang,
                newscore,
                self.preword.as_mut_ptr().add(preword_len),
                true,
            )
        })
    }

    /// Step the new level over the character the split lands on.
    ///
    /// A non-word character at the split point is replaced by the space,
    /// and when the bad word ends the character is kept instead: it is
    /// copied into `preword` so that it survives into the suggestion.
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    unsafe fn skip_split_character(
        &mut self,
        try_compound: bool,
        bad_word_ends: bool,
        good_word_ends: bool,
    ) {
        let child = self.depth as usize;
        let bad_idx = self.stack[child].bad_idx as usize;

        // SAFETY: `bad_idx` is a position the walk reached inside the bad
        // word, the caller's NUL-terminated buffer.
        let replacing_nonword =
            !try_compound && !unsafe { spell_iswordp_nmw(self.fword_ptr(bad_idx), curwin.get()) };
        if !((replacing_nonword || bad_word_ends)
            && self.fword_at(bad_idx) != NUL
            && good_word_ends)
        {
            return;
        }

        // SAFETY: as above; the length comes from the encoding itself, so
        // the character it measures is inside the bad word, and `preword`
        // is this walk's own buffer with room for it.
        let taken = unsafe { utfc_ptr2len(self.fword_ptr(bad_idx)) };
        if bad_word_ends {
            // Keep the character: copy it into `preword`.
            let preword_len = self.stack[child].preword_len as usize;
            unsafe {
                ptr::copy_nonoverlapping(
                    self.fword_ptr(bad_idx),
                    self.preword.as_mut_ptr().add(preword_len),
                    taken as usize,
                );
            }
            self.stack[child].preword_len = (self.stack[child].preword_len as c_int + taken) as u8;
            self.preword[self.stack[child].preword_len as usize] = NUL as c_char;
        } else {
            // Replacing a non-word character with a space is a
            // substitution, not a split.
            self.stack[child].score -= SCORE_SPLIT - SCORE_SUBST;
        }
        self.stack[child].bad_idx = (self.stack[child].bad_idx as c_int + taken) as u8;
    }

    /// Could the compound flags collected so far, plus `flag`, still form
    /// a valid compound word?
    ///
    /// This also checks the `COMPOUNDRULE` lines, but only when they carry
    /// no wildcards -- with wildcards a partial sequence says nothing.
    ///
    /// # Safety
    ///
    /// The walk's language must be valid.
    unsafe fn can_be_compound(&mut self, flag: c_int) -> bool {
        let level = self.depth as usize;
        let comp_len = self.stack[level].comp_len as usize;
        let comp_split = self.stack[level].comp_split as usize;

        // A flag in neither the start-flag nor the all-flag set cannot
        // possibly compound.
        //
        // SAFETY: `slang` is the walk's own language and its flag strings
        // are NUL-terminated.
        let allowed = if comp_len == comp_split {
            unsafe { (*self.slang).sl_compstartflags }
        } else {
            unsafe { (*self.slang).sl_compallflags }
        };
        if !unsafe { byte_in_str(allowed, flag) } {
            return false;
        }

        // Without wildcards the flags so far can be matched against
        // the COMPOUNDRULE patterns, which only says anything once
        // there are two or more words.
        //
        // SAFETY: as above.
        if unsafe { (*self.slang).sl_comprules }.is_null() || comp_len <= comp_split {
            return true;
        }
        self.compflags[comp_len] = flag as u8;
        self.compflags[comp_len + 1] = NUL as u8;
        // SAFETY: as above, and `compflags` is this walk's own array,
        // NUL-terminated by the two lines above.
        let matched =
            unsafe { match_compoundrule(self.slang, self.compflags.as_ptr().add(comp_split)) };
        self.compflags[comp_len] = NUL as u8;
        matched
    }
}
