//! Arriving at a node, and what to do when a word ends there.
//!
//! A node's first byte says how many children follow it, and a child byte
//! of NUL means "a word may end here" -- the entry beside it holds that
//! word's flags rather than a child node. [`Walk::node_start`] is what
//! runs at each of those NUL bytes, once per round, and it is by far the
//! busiest state: everything that turns the letters collected so far into
//! an actual suggestion happens here.
//!
//! In order, a word that ends here has to get past
//!
//! 1. the `NOSUGGEST` flag,
//! 2. the postponed prefix in front of it, if any, being valid with it,
//! 3. `NEEDCOMPOUND`, and, when a compound word came before it, the
//!    compounding rules -- `COMPOUNDMIN`, `CHECKCOMPOUNDPATTERN` and
//!    `COMPOUNDRULE`,
//! 4. not being a banned word,
//!
//! and only then, if the bad word ends here too, is it offered. Whether
//! it was offered or not, the walk goes on to try continuing it: as a
//! compound, or as a split into two words. That part is in [`super::split`].
//!
//! [`Walk::prefix_tree_node`] is the same state entered while walking the
//! postponed-prefix tree, where a NUL byte means the end of a prefix
//! rather than the end of a word, and the walk switches over to the
//! case-folded tree to look for the word the prefix goes in front of.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::hashtab::{hash_find, hash_removed};
use crate::src::nvim::main::curwin;
use crate::src::nvim::mbyte::{mb_charlen, utfc_ptr2len};
use crate::src::nvim::memory::xmemcpyz;
use crate::src::nvim::os::libc::{strcpy, strncmp};
use crate::src::nvim::spell::{
    can_compound, captype, make_case_word, match_checkcompoundpattern, nofold_len, spell_iswordp,
    spell_iswordp_nmw, spell_valid_case, valid_word_prefix,
};
use crate::src::nvim::spellsuggest::collect::{add_banned, add_suggestion};
use crate::src::nvim::spellsuggest::score::score_wordcount_adj;
use crate::src::nvim::spellsuggest::soundalike::{add_sound_suggest, find_keepcap_word};
use crate::src::nvim::spellsuggest::walk::{
    FLAG_PREFIX_OK, PFD_NOTSPECIAL, PFD_PREFIXTREE, STACK_SIZE, State, Walk,
};
use crate::src::nvim::spellsuggest::{
    MAXWLEN, NUL, SCORE_ICASE, SCORE_NONWORD, SCORE_RARE, SCORE_REGION, WF_ALLCAP, WF_BANNED,
    WF_KEEPCAP, WF_MIXCAP, WF_NEEDCOMP, WF_NOSUGGEST, WF_ONECAP, WF_RARE, WF_RAREPFX, WF_REGION,
    badword_captype, suginfo_T,
};
use crate::src::nvim::types::{idx_T, size_t};
use core::ffi::{c_char, c_int};
use core::ptr;

impl Walk {
    /// At the start of a node: deal with the NUL bytes, which mean the
    /// good word may end here.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    pub(super) unsafe fn node_start(&mut self) {
        // SAFETY: the tree index is this level's node plus a child number
        // the node's own length byte bounds.
        unsafe {
            let level = self.depth as usize;
            let entry_state = self.stack[level].state;

            let node = self.stack[level].node;
            let node_len = self.byte_at(node) as c_int; // bytes in this node
            let at = node + self.stack[level].child as idx_T; // the current byte

            if self.stack[level].prefix_depth == PFD_PREFIXTREE {
                self.prefix_tree_node(entry_state, node_len, at);
                return;
            }

            if self.stack[level].child as c_int > node_len || self.byte_at(at) != 0 {
                // Past the bytes in the node and/or past its NUL bytes.
                self.stack[level].state = State::EndNul;
                self.stack[level].saved_badflags = (*self.su).su_badflags as u8;
                return;
            }

            self.word_end(at);
        }
    }

    /// The same state, but inside the postponed-prefix tree.
    ///
    /// A NUL here means a prefix ends, so the word it applies to is looked
    /// for from the root of the case-folded tree. The prefix's own letters
    /// move into `preword` with the case they had in the bad word, and the
    /// caps type for the rest of the bad word becomes the one the word
    /// after the prefix is judged by.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    unsafe fn prefix_tree_node(&mut self, entry_state: State, node_len: c_int, at: idx_T) {
        // SAFETY: `at` and the bytes after it are inside the node, which
        // the node's length byte bounds.
        unsafe {
            let level = self.depth as usize;

            // Skip over the NUL bytes; they are used just below.
            let mut nul_count = 0;
            while nul_count < node_len && self.byte_at(at + nul_count) == 0 {
                nul_count += 1;
            }
            self.stack[level].child += nul_count as i16;

            // Always past the NUL bytes now.
            self.stack[level].state = State::EndNul;
            self.stack[level].saved_badflags = (*self.su).su_badflags as u8;

            // At the end of a prefix, or at the very start of the prefix
            // tree: check for a word following. `at` is still the byte the
            // node was entered at, so a NUL there is what says a prefix
            // ended.
            let at_prefix_end = self.byte_at(at) == 0 || entry_state == State::NoPrefix;
            if self.depth >= MAXWLEN as c_int - 1 || !at_prefix_end {
                return;
            }

            // Set `su_badflags` to the caps type at this position; the
            // prefix itself keeps the caps type up to here.
            let prefix_len = nofold_len(
                self.fword,
                self.stack[level].bad_idx as c_int,
                (*self.su).su_badptr,
            );
            let prefix_flags = badword_captype(
                (*self.su).su_badptr,
                (*self.su).su_badptr.offset(prefix_len as isize),
            );
            (*self.su).su_badflags = badword_captype(
                (*self.su).su_badptr.offset(prefix_len as isize),
                (*self.su).su_badptr.offset((*self.su).su_badlen as isize),
            );

            self.go_deeper(0);
            self.depth += 1;
            let child = self.depth as usize;
            self.stack[child].prefix_depth = (self.depth - 1) as u8;
            // The word after the prefix is in the case-folded tree.
            self.byts = self.fbyts;
            self.idxs = self.fidxs;
            self.stack[child].node = 0;

            // Move the prefix to `preword` with the right case, which is
            // also what makes `find_keepcap_word` work later.
            self.tword[self.stack[child].good_len as usize] = NUL as c_char;
            let split_off = self.stack[child].split_off as usize;
            let preword_len = self.stack[child].preword_len as usize;
            make_case_word(
                self.tword.as_mut_ptr().add(split_off),
                self.preword.as_mut_ptr().add(preword_len),
                prefix_flags,
            );
            self.stack[child].preword_len = self.preword_len() as u8;
            self.stack[child].split_off = self.stack[child].good_len;
        }
    }

    /// Undo the changes a word split or a compound join made, and go back
    /// to looking for NUL bytes at this node.
    pub(super) unsafe fn split_undo(&mut self) {
        let level = self.depth as usize;
        // SAFETY: `su` is the caller's suggestion state.
        unsafe { (*self.su).su_badflags = self.stack[level].saved_badflags as c_int };

        self.stack[level].state = State::Start;

        // In case the split went into the prefix tree.
        self.byts = self.fbyts;
        self.idxs = self.fidxs;
    }

    /// A word ends at this NUL byte: check it over and, if the bad word
    /// ends too, offer it.
    ///
    /// # Safety
    ///
    /// The walk's trees and bad word must be valid.
    unsafe fn word_end(&mut self, at: idx_T) {
        // SAFETY: `at` is a NUL child of this level's node, so the entry
        // beside it is the ending word's flags; every buffer written below
        // is this walk's own.
        unsafe {
            let level = self.depth as usize;
            self.stack[level].child += 1; // eat one NUL byte

            let mut flags = self.idx_at(at) as c_int;
            if flags & WF_NOSUGGEST as c_int != 0 {
                return;
            }

            let bad_idx = self.stack[level].bad_idx as usize;
            // The bad word "ends" wherever a word can no longer continue,
            // not only at its terminator.
            let bad_word_ends = self.fword_at(bad_idx) == NUL
                || if self.soundfold {
                    ascii_iswhite(self.fword_at(bad_idx))
                } else {
                    !spell_iswordp(self.fword_ptr(bad_idx), curwin.get())
                };
            self.tword[self.stack[level].good_len as usize] = NUL as c_char;

            if !self.prefix_allows_word(&mut flags) {
                return;
            }

            // NEEDCOMPOUND: the word cannot stand on its own. Appending
            // another compound word to it is still worth trying, below.
            let mut good_word_ends = !(self.stack[level].comp_len == self.stack[level].comp_split
                && bad_word_ends
                && flags & WF_NEEDCOMP as c_int != 0);

            // The last character of the word before this one, once there
            // is one to compound onto. Null until then, and the null is
            // meaningful: it is what says "nothing precedes this word".
            let mut prev_word_tail: *mut c_char = ptr::null_mut();
            let mut compound_ok = true;
            if self.stack[level].comp_len > self.stack[level].comp_split {
                if (*self.slang).sl_nobreak {
                    if self.nobreak_previous_word_matches() {
                        self.suggest_previous_word();
                        return;
                    }
                } else {
                    match self.join_compound(flags, bad_word_ends) {
                        None => return,
                        Some((tail, ok)) => {
                            prev_word_tail = tail;
                            compound_ok = ok;
                        }
                    }
                }
            }

            self.build_preword(flags, prev_word_tail);

            if !self.soundfold {
                // A banned word must not be suggested. It may turn up
                // again as a good word, so remember it.
                let preword_len = self.stack[level].preword_len as usize;
                if flags & WF_BANNED as c_int != 0 {
                    add_banned(self.su, self.preword.as_mut_ptr().add(preword_len));
                    return;
                }
                let this_word_banned = self.stack[level].comp_len == self.stack[level].comp_split
                    && was_banned(self.su, self.preword.as_ptr().add(preword_len));
                if this_word_banned || was_banned(self.su, self.preword.as_ptr()) {
                    if (*self.slang).sl_compprog.is_null() {
                        return;
                    }
                    // Banned so far, but compounding may still save it.
                    good_word_ends = false;
                }
            }

            let mut newscore = 0;
            if !self.soundfold {
                // Sound-folded words have no flags.
                if flags & WF_REGION as c_int != 0
                    && (flags as u32 >> 16) & (*self.lp).lp_region as u32 == 0
                {
                    newscore += SCORE_REGION as c_int;
                }
                if flags & WF_RARE as c_int != 0 {
                    newscore += SCORE_RARE as c_int;
                }
                let preword_len = self.stack[level].preword_len as usize;
                if !spell_valid_case(
                    (*self.su).su_badflags,
                    captype(self.preword.as_ptr().add(preword_len), ptr::null()),
                ) {
                    newscore += SCORE_ICASE as c_int;
                }
            }

            if bad_word_ends
                && good_word_ends
                && self.stack[level].bad_idx >= self.stack[level].change_from
                && compound_ok
            {
                newscore = self.offer_word(newscore);
            }

            self.try_split_or_compound(flags, bad_word_ends, good_word_ends, newscore);
        }
    }

    /// Check the postponed prefix in front of this word, if there is one.
    ///
    /// Returns false when the prefix cannot be used with the word, which
    /// ends this NUL byte's turn. A rare prefix makes the whole word rare.
    ///
    /// # Safety
    ///
    /// The walk's trees must be valid.
    unsafe fn prefix_allows_word(&mut self, flags: &mut c_int) -> bool {
        // SAFETY: the prefix node index came out of the prefix tree and is
        // bounded by that node's own length byte.
        unsafe {
            let level = self.depth as usize;
            if self.stack[level].prefix_depth > PFD_NOTSPECIAL
                || self.stack[level].flags & FLAG_PREFIX_OK != 0
                || self.pbyts.is_null()
            {
                return true;
            }

            // `prefix_depth` is a real stack depth here, guarded by the
            // test above: `PFD_NOTSPECIAL` is 253 and the stack holds 254
            // frames, so it fits by exactly one.
            let prefix_level = self.stack[level].prefix_depth as usize;
            assert!(
                prefix_level < STACK_SIZE,
                "prefix depth {prefix_level} is past the walk's stack"
            );

            // Count the NUL bytes of the prefix node. None at all means
            // this is the first try, the one without a prefix.
            let mut node = self.stack[prefix_level].node;
            let node_len = *self.pbyts.offset(node as isize) as c_int;
            node += 1;
            let mut prefix_count = 0;
            while prefix_count < node_len && *self.pbyts.offset((node + prefix_count) as isize) == 0
            {
                prefix_count += 1;
            }
            if prefix_count == 0 {
                return true;
            }

            let split_off = self.stack[level].split_off as usize;
            let prefix_flags = valid_word_prefix(
                prefix_count,
                node,
                *flags,
                self.tword.as_mut_ptr().add(split_off),
                self.slang,
                false,
            );
            if prefix_flags == 0 {
                return false;
            }
            if prefix_flags & WF_RAREPFX as c_int != 0 {
                *flags |= WF_RARE as c_int;
            }

            // Checking for a prefix and for compounding at once runs into
            // the prefix flag first; remember that it was accepted, so
            // that arriving at a compound flag does not reject it.
            self.stack[level].flags |= FLAG_PREFIX_OK;
            true
        }
    }

    /// For a `NOBREAK` language: did the word before this one come through
    /// unchanged?
    ///
    /// # Safety
    ///
    /// The walk's bad word must be valid.
    unsafe fn nobreak_previous_word_matches(&self) -> bool {
        // SAFETY: both stretches compared are inside this walk's buffers.
        unsafe {
            let level = self.depth as usize;
            let taken =
                self.stack[level].bad_idx as c_int - self.stack[level].split_bad_idx as c_int;
            taken == self.stack[level].good_len as c_int - self.stack[level].split_off as c_int
                && strncmp(
                    self.fword_ptr(self.stack[level].split_bad_idx as usize),
                    self.tword
                        .as_ptr()
                        .add(self.stack[level].split_off as usize),
                    taken as size_t,
                ) == 0
        }
    }

    /// For a `NOBREAK` language whose previous word was already correct:
    /// offer just that previous word.
    ///
    /// If this word was corrected too, then what has to be checked is
    /// whether a correct word follows -- which is what the rest of the
    /// walk goes on to do.
    ///
    /// # Safety
    ///
    /// The walk's state must be valid.
    unsafe fn suggest_previous_word(&mut self) {
        // SAFETY: `preword` is this walk's buffer and is terminated here
        // before being handed on.
        unsafe {
            let level = self.depth as usize;
            let preword_len = self.stack[level].preword_len as usize;
            self.preword[preword_len] = NUL as c_char;
            let score = score_wordcount_adj(
                &*self.slang,
                self.stack[level].score,
                self.preword.as_mut_ptr().add(preword_len),
                self.stack[level].preword_len > 0,
            );
            if score <= (*self.su).su_maxscore {
                add_suggestion(
                    self.su,
                    &raw mut (*self.su).su_ga,
                    self.preword.as_ptr(),
                    self.stack[level].split_bad_idx as c_int - self.repextra,
                    score,
                    0,
                    false,
                    (*self.lp).lp_sallang,
                    false,
                );
            }
        }
    }

    /// Join this word onto the compound word before it.
    ///
    /// Returns `None` when the word cannot compound at all, which ends
    /// this NUL byte's turn -- splitting is still tried later for the same
    /// word without its compound flag. Otherwise it returns the last
    /// character of the word before this one and whether the compound is
    /// allowed; a compound that is not allowed may still become one once
    /// another short word is appended, so the walk carries on.
    ///
    /// # Safety
    ///
    /// The walk's state must be valid.
    unsafe fn join_compound(
        &mut self,
        flags: c_int,
        bad_word_ends: bool,
    ) -> Option<(*mut c_char, bool)> {
        // SAFETY: every buffer touched is this walk's own, and the flag
        // byte is the top byte of the tree's flags word.
        unsafe {
            let level = self.depth as usize;
            let this_word_len =
                self.stack[level].good_len as c_int - self.stack[level].split_off as c_int;
            let split_off = self.stack[level].split_off as usize;

            // No compound flag, or too short to be a compound part.
            if (flags as u32) >> 24 == 0 || this_word_len < (*self.slang).sl_compminlen {
                return None;
            }
            // `COMPOUNDMIN` counts characters, not bytes.
            if (*self.slang).sl_compminlen > 0
                && mb_charlen(self.tword.as_ptr().add(split_off)) < (*self.slang).sl_compminlen
            {
                return None;
            }

            let comp_len = self.stack[level].comp_len as usize;
            self.compflags[comp_len] = ((flags as u32) >> 24) as u8;
            self.compflags[comp_len + 1] = NUL as u8;
            let preword_len = self.stack[level].preword_len as usize;
            xmemcpyz(
                self.preword.as_mut_ptr().add(preword_len) as *mut _,
                self.tword.as_ptr().add(split_off) as *const _,
                this_word_len as size_t,
            );

            // CHECKCOMPOUNDPATTERN forbids some pairs of word endings and
            // beginnings outright.
            let mut compound_ok = !match_checkcompoundpattern(
                self.preword.as_mut_ptr(),
                self.stack[level].preword_len as c_int,
                &raw mut (*self.slang).sl_comppat,
            );

            if compound_ok && bad_word_ends {
                let comp_split = self.stack[level].comp_split as usize;
                let last_word = self.last_word_of_preword();
                if !can_compound(
                    self.slang,
                    last_word,
                    self.compflags.as_ptr().add(comp_split),
                ) {
                    // Not allowed as it stands; another short word may
                    // still make it valid.
                    compound_ok = false;
                }
            }

            let mut tail = self.preword.as_mut_ptr().add(preword_len);
            tail = Walk::char_back(self.preword.as_ptr(), tail);
            Some((tail, compound_ok))
        }
    }

    /// The start of the last whitespace-separated word in `preword`.
    ///
    /// # Safety
    ///
    /// `preword` must be NUL-terminated.
    unsafe fn last_word_of_preword(&mut self) -> *mut c_char {
        // SAFETY: `preword` is this walk's own NUL-terminated buffer.
        unsafe {
            let mut p = self.preword.as_mut_ptr();
            while *skiptowhite(p) != NUL as c_char {
                p = skipwhite(skiptowhite(p));
            }
            p
        }
    }

    /// Put the word with its proper case into `preword`, after whatever a
    /// previous split or compound left there.
    ///
    /// # Safety
    ///
    /// The walk's state must be valid.
    unsafe fn build_preword(&mut self, flags: c_int, prev_word_tail: *mut c_char) {
        // SAFETY: both buffers are this walk's own and `tword` was
        // terminated by the caller.
        unsafe {
            let level = self.depth as usize;
            let split_off = self.stack[level].split_off as usize;
            let preword_len = self.stack[level].preword_len as usize;
            let good = self.tword.as_mut_ptr().add(split_off);
            let out = self.preword.as_mut_ptr().add(preword_len);

            if self.soundfold {
                // Sound-folded words have no case to get right.
                strcpy(out, good);
            } else if flags & WF_KEEPCAP as c_int != 0 {
                // The spelling has to come from the keep-case tree.
                find_keepcap_word(self.slang, good, out);
            } else {
                // Take the bad word's caps type: a one-cap or all-cap bad
                // word wants a good word to match. An all-cap bad word
                // one character long only says one-cap, though.
                let mut caps = (*self.su).su_badflags;
                if caps & WF_ALLCAP as c_int != 0
                    && (*self.su).su_badlen == utfc_ptr2len((*self.su).su_badptr)
                {
                    caps = WF_ONECAP as c_int;
                }
                caps |= flags;

                // A compound word appended after a word character must not
                // start with a capital.
                if !prev_word_tail.is_null() && spell_iswordp_nmw(prev_word_tail, curwin.get()) {
                    caps &= !(WF_ONECAP as c_int);
                }
                make_case_word(good, out, caps);
            }
        }
    }

    /// The bad word ends here and so does a valid good word: offer it.
    ///
    /// Returns the score the walk carries on with, which the non-word
    /// penalty may have raised.
    ///
    /// # Safety
    ///
    /// The walk's state must be valid.
    unsafe fn offer_word(&mut self, mut newscore: c_int) -> c_int {
        // SAFETY: every buffer read is this walk's own or the caller's bad
        // word, and all are NUL-terminated at this point.
        unsafe {
            let level = self.depth as usize;

            if self.soundfold {
                // A sound-folded match stands for real words, which have
                // to be found and scored separately.
                add_sound_suggest(
                    self.su,
                    self.preword.as_mut_ptr(),
                    self.stack[level].score,
                    self.lp,
                );
                return newscore;
            }
            if self.stack[level].bad_idx == 0 {
                return newscore;
            }

            // Penalise turning a non-word character into a word character,
            // as in "thes," -> "these".
            let mut p = self.fword_ptr(self.stack[level].bad_idx as usize);
            p = Walk::char_back(self.fword, p);
            if !spell_iswordp(p, curwin.get()) && self.preword[0] != NUL as c_char {
                let end = self.preword_len();
                let mut q = self.preword.as_mut_ptr().add(end);
                q = Walk::char_back(self.preword.as_ptr(), q);
                if spell_iswordp(q, curwin.get()) {
                    newscore += SCORE_NONWORD as c_int;
                }
            }

            let preword_len = self.stack[level].preword_len as usize;
            let score = score_wordcount_adj(
                &*self.slang,
                self.stack[level].score + newscore,
                self.preword.as_mut_ptr().add(preword_len),
                self.stack[level].preword_len > 0,
            );
            if score > (*self.su).su_maxscore {
                return newscore;
            }

            let replaced = self.stack[level].bad_idx as c_int - self.repextra;
            add_suggestion(
                self.su,
                &raw mut (*self.su).su_ga,
                self.preword.as_ptr(),
                replaced,
                score,
                0,
                false,
                (*self.lp).lp_sallang,
                false,
            );

            if (*self.su).su_badflags & WF_MIXCAP != 0 {
                // With mixed case there is no telling whether the word
                // should be upper or lower case, so offer both.
                let caps = captype(self.preword.as_ptr(), ptr::null());
                if caps == 0 || caps == WF_ALLCAP as c_int {
                    let split_off = self.stack[level].split_off as usize;
                    make_case_word(
                        self.tword.as_mut_ptr().add(split_off),
                        self.preword.as_mut_ptr().add(preword_len),
                        if caps == 0 { WF_ALLCAP as c_int } else { 0 },
                    );
                    add_suggestion(
                        self.su,
                        &raw mut (*self.su).su_ga,
                        self.preword.as_ptr(),
                        replaced,
                        score + SCORE_ICASE as c_int,
                        0,
                        false,
                        (*self.lp).lp_sallang,
                        false,
                    );
                }
            }
            newscore
        }
    }
}

/// Is this word one the walk has been told never to suggest?
///
/// # Safety
///
/// `su` must be valid and `word` NUL-terminated.
unsafe fn was_banned(su: *mut suginfo_T, word: *const c_char) -> bool {
    // SAFETY: the caller guarantees both; a miss returns an empty item
    // rather than null.
    unsafe {
        let key = (*hash_find(&raw const (*su).su_banned, word)).hi_key;
        !(key.is_null() || key == &raw const hash_removed as *mut c_char)
    }
}
