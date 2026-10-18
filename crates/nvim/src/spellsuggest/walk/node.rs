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

#![forbid(unsafe_code)]

use crate::ascii::ascii_iswhite;
use crate::cstr::until_nul;
use crate::mbyte::{c_cluster_len, char_count};
use crate::spell::WordFlags;
use crate::spell::{
    can_compound, captype, make_case_word, match_checkcompoundpattern, nofold_len, spell_iswordp,
    spell_iswordp_nmw, spell_valid_case, valid_word_prefix,
};
use crate::spellsuggest::collect::{List, add_banned, add_suggestion};
use crate::spellsuggest::score::score_wordcount_adj;
use crate::spellsuggest::soundalike::{add_sound_suggest, find_keepcap_word};
use crate::spellsuggest::walk::{
    FLAG_PREFIX_OK, PFD_NOTSPECIAL, PFD_PREFIXTREE, STACK_SIZE, State, Walk, c_prefix_eq, char_back,
};
use crate::spellsuggest::{
    MAXWLEN, SCORE_ICASE, SCORE_NONWORD, SCORE_RARE, SCORE_REGION, SugInfo, copy_terminated,
};
use crate::types::{NUL, SpellIdx};
use crate::winlayer::Win;
use core::ffi::c_int;

/// Where the last blank-separated word of `text` starts. `text` ends at its
/// end or at a NUL.
pub(super) fn last_word_start(text: &[u8]) -> usize {
    let text = until_nul(text);
    let is_white = |b: &u8| *b == b' ' || *b == b'\t';
    let mut p = 0;
    loop {
        let Some(white) = text[p..].iter().position(is_white) else {
            return p;
        };
        let white = p + white;
        p = white + text[white..].iter().take_while(|b| is_white(b)).count();
    }
}

impl Walk<'_> {
    /// At the start of a node: deal with the NUL bytes, which mean the
    /// good word may end here.
    pub(super) fn node_start(&mut self) {
        let level = self.depth as usize;
        let entry_state = self.stack[level].state;

        let node = self.stack[level].node;
        // The tree index is this level's node plus a child number the
        // node's own length byte bounds.
        let node_len = self.byte_at(node) as c_int; // bytes in this node
        let at = node + self.stack[level].child as SpellIdx; // the current byte

        if self.stack[level].prefix_depth == PFD_PREFIXTREE {
            self.prefix_tree_node(entry_state, node_len, at);
            return;
        }

        // The `||` short circuit is what keeps the read from happening once
        // the child number has run past the node's own length byte.
        if self.stack[level].child as c_int > node_len || self.byte_at(at) != 0 {
            // Past the bytes in the node and/or past its NUL bytes.
            self.stack[level].state = State::EndNul;
            self.stack[level].saved_badflags = self.su.su_badflags.bits() as u8;
            return;
        }

        self.word_end(at);
    }

    /// The same state, but inside the postponed-prefix tree.
    ///
    /// A NUL here means a prefix ends, so the word it applies to is looked
    /// for from the root of the case-folded tree. The prefix's own letters
    /// move into `preword` with the case they had in the bad word, and the
    /// caps type for the rest of the bad word becomes the one the word
    /// after the prefix is judged by.
    fn prefix_tree_node(&mut self, entry_state: State, node_len: c_int, at: SpellIdx) {
        let level = self.depth as usize;

        // Skip over the NUL bytes; they are used just below. The `&&` is
        // what stops the read once the count has reached the node's length.
        let mut nul_count = 0;
        while nul_count < node_len && self.byte_at(at + nul_count) == 0 {
            nul_count += 1;
        }
        self.stack[level].child += nul_count as i16;

        // Always past the NUL bytes now.
        self.stack[level].state = State::EndNul;
        self.stack[level].saved_badflags = self.su.su_badflags.bits() as u8;

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
            self.stack[level].bad_idx as usize,
            &self.su.su_bad,
        );
        let prefix_flags = self.su.captype_between(0, prefix_len);
        self.su.su_badflags = self
            .su
            .captype_between(prefix_len, self.su.su_badlen as usize);

        self.go_deeper(0);
        self.depth += 1;
        let child = self.depth as usize;
        self.stack[child].prefix_depth = (self.depth - 1) as u8;
        // The word after the prefix is in the case-folded tree.
        self.tree = self.word_tree;
        self.stack[child].node = 0;

        // Move the prefix to `preword` with the right case, which is
        // also what makes `find_keepcap_word` work later.
        self.tword[self.stack[child].good_len as usize] = NUL as u8;
        let split_off = self.stack[child].split_off as usize;
        let preword_len = self.stack[child].preword_len as usize;
        make_case_word(
            &self.tword[split_off..],
            &mut self.preword[preword_len..],
            prefix_flags,
        );
        self.stack[child].preword_len = self.preword_len() as u8;
        self.stack[child].split_off = self.stack[child].good_len;
    }

    /// Undo the changes a word split or a compound join made, and go back
    /// to looking for NUL bytes at this node.
    pub(super) fn split_undo(&mut self) {
        let level = self.depth as usize;
        let saved = WordFlags::from_bits(self.stack[level].saved_badflags.into());
        self.su.su_badflags = saved;

        self.stack[level].state = State::Start;

        // In case the split went into the prefix tree.
        self.tree = self.word_tree;
    }

    /// A word ends at this NUL byte: check it over and, if the bad word
    /// ends too, offer it.
    fn word_end(&mut self, at: SpellIdx) {
        let level = self.depth as usize;
        self.stack[level].child += 1; // eat one NUL byte

        // `at` is a NUL child of this level's node, so the entry beside it
        // is the ending word's flags.
        let mut flags = WordFlags::from_bits(self.idx_at(at) as c_int);
        if flags.has(WordFlags::NOSUGGEST) {
            return;
        }

        let bad_idx = self.stack[level].bad_idx as usize;
        // The bad word "ends" wherever a word can no longer continue,
        // not only at its terminator.
        let bad_word_ends = self.fword_at(bad_idx) == NUL
            || if self.soundfold {
                ascii_iswhite(self.fword_at(bad_idx))
            } else {
                !spell_iswordp(&self.fword[bad_idx..], Win::current())
            };
        self.tword[self.stack[level].good_len as usize] = NUL as u8;

        if !self.prefix_allows_word(&mut flags) {
            return;
        }

        // NEEDCOMPOUND: the word cannot stand on its own. Appending
        // another compound word to it is still worth trying, below.
        let mut good_word_ends = !(self.stack[level].comp_len == self.stack[level].comp_split
            && bad_word_ends
            && flags.has(WordFlags::NEEDCOMP));

        // Where in `preword` the last character of the word before this one
        // starts, once there is one to compound onto. `None` until then,
        // and that is meaningful: it is what says "nothing precedes this
        // word".
        let mut prev_word_tail = None;
        let mut compound_ok = true;
        if self.stack[level].comp_len > self.stack[level].comp_split {
            if self.slang.sl_nobreak {
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
            if flags.has(WordFlags::BANNED) {
                add_banned(self.su, &self.preword[preword_len..]);
                return;
            }
            // Both `&&` and `||` are load bearing only for the work they
            // save.
            let this_word_banned = self.stack[level].comp_len == self.stack[level].comp_split
                && was_banned(self.su, &self.preword[preword_len..]);
            if this_word_banned || was_banned(self.su, &self.preword) {
                if self.slang.sl_compprog.borrow().is_none() {
                    return;
                }
                // Banned so far, but compounding may still save it.
                good_word_ends = false;
            }
        }

        let mut newscore = 0;
        if !self.soundfold {
            // Sound-folded words have no flags. The `&&` keeps the region
            // read out of the common case.
            if flags.has(WordFlags::REGION)
                && (flags.bits() as u32 >> 16) & self.langp.region as u32 == 0
            {
                newscore += SCORE_REGION;
            }
            if flags.has(WordFlags::RARE) {
                newscore += SCORE_RARE;
            }
            let preword_len = self.stack[level].preword_len as usize;
            let caps = captype(&self.preword[preword_len..], usize::MAX);
            if !spell_valid_case(self.su.su_badflags, caps) {
                newscore += SCORE_ICASE;
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

    /// Check the postponed prefix in front of this word, if there is one.
    ///
    /// Returns false when the prefix cannot be used with the word, which
    /// ends this NUL byte's turn. A rare prefix makes the whole word rare.
    fn prefix_allows_word(&mut self, flags: &mut WordFlags) -> bool {
        let level = self.depth as usize;
        if self.stack[level].prefix_depth > PFD_NOTSPECIAL
            || self.stack[level].flags & FLAG_PREFIX_OK != 0
            || self.prefix_tree.is_empty()
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
        let mut node = self.stack[prefix_level].node as usize;
        let node_len = self.prefix_tree.node_len(node);
        node += 1;
        let prefix_count = self.prefix_tree.word_ends(node, node_len);
        if prefix_count == 0 {
            return true;
        }

        let split_off = self.stack[level].split_off as usize;
        let prefix_flags = valid_word_prefix(
            prefix_count as c_int,
            node,
            *flags,
            &self.tword[split_off..],
            self.slang,
            false,
        );
        if prefix_flags == 0 {
            return false;
        }
        if WordFlags::from_bits(prefix_flags).has(WordFlags::RAREPFX) {
            *flags |= WordFlags::RARE;
        }

        // Checking for a prefix and for compounding at once runs into
        // the prefix flag first; remember that it was accepted, so
        // that arriving at a compound flag does not reject it.
        self.stack[level].flags |= FLAG_PREFIX_OK;
        true
    }

    /// For a `NOBREAK` language: did the word before this one come through
    /// unchanged?
    fn nobreak_previous_word_matches(&mut self) -> bool {
        let level = self.depth as usize;
        let taken = self.stack[level].bad_idx as c_int - self.stack[level].split_bad_idx as c_int;
        if taken != self.stack[level].good_len as c_int - self.stack[level].split_off as c_int {
            return false;
        }
        let split_off = self.stack[level].split_off as usize;
        let split_bad_idx = self.stack[level].split_bad_idx as usize;
        // The test above is what holds the comparison to a length that both
        // stretches have.
        c_prefix_eq(
            self.fword.get(split_bad_idx..).unwrap_or_default(),
            &self.tword[split_off..],
            taken.max(0) as usize,
        )
    }

    /// For a `NOBREAK` language whose previous word was already correct:
    /// offer just that previous word.
    ///
    /// If this word was corrected too, then what has to be checked is
    /// whether a correct word follows -- which is what the rest of the
    /// walk goes on to do.
    fn suggest_previous_word(&mut self) {
        let level = self.depth as usize;
        let preword_len = self.stack[level].preword_len as usize;
        self.preword[preword_len] = NUL as u8;
        let score = score_wordcount_adj(
            self.slang,
            self.stack[level].score,
            &self.preword[preword_len..],
            self.stack[level].preword_len > 0,
        );
        if score <= self.su.su_maxscore {
            add_suggestion(
                self.su,
                List::Main,
                &self.preword,
                self.stack[level].split_bad_idx as c_int - self.repextra,
                score,
                0,
                false,
                self.langp.sallang,
                false,
            );
        }
    }

    /// Join this word onto the compound word before it.
    ///
    /// Returns `None` when the word cannot compound at all, which ends
    /// this NUL byte's turn -- splitting is still tried later for the same
    /// word without its compound flag. Otherwise it returns where the last
    /// character of the word before this one starts in `preword`, and
    /// whether the compound is allowed; a compound that is not allowed may
    /// still become one once another short word is appended, so the walk
    /// carries on.
    fn join_compound(
        &mut self,
        flags: WordFlags,
        bad_word_ends: bool,
    ) -> Option<(Option<usize>, bool)> {
        let level = self.depth as usize;
        let this_word_len =
            self.stack[level].good_len as c_int - self.stack[level].split_off as c_int;
        let split_off = self.stack[level].split_off as usize;

        // No compound flag, or too short to be a compound part.
        if (flags.bits() as u32) >> 24 == 0 || this_word_len < self.slang.sl_compminlen {
            return None;
        }
        // `COMPOUNDMIN` counts characters, not bytes. `tword` was
        // terminated before this ran.
        let compminlen = self.slang.sl_compminlen;
        if compminlen > 0 && (char_count(until_nul(&self.tword[split_off..])) as c_int) < compminlen
        {
            return None;
        }

        let comp_len = self.stack[level].comp_len as usize;
        self.compflags[comp_len] = ((flags.bits() as u32) >> 24) as u8;
        self.compflags[comp_len + 1] = NUL as u8;
        let preword_len = self.stack[level].preword_len as usize;
        // The word `tword` holds, after whatever a previous split left in
        // `preword`.
        let word = &self.tword[split_off..split_off + this_word_len as usize];
        copy_terminated(&mut self.preword[preword_len..], word);

        // CHECKCOMPOUNDPATTERN forbids some pairs of word endings and
        // beginnings outright.
        let forbidden_pair = match_checkcompoundpattern(
            &self.preword,
            self.stack[level].preword_len as usize,
            &self.slang.sl_comppat,
        );
        let mut compound_ok = !forbidden_pair;

        if compound_ok && bad_word_ends {
            let comp_split = self.stack[level].comp_split as usize;
            let last_word = last_word_start(&self.preword);
            let joins = can_compound(
                self.slang,
                until_nul(&self.preword[last_word..]),
                &self.compflags[comp_split..],
            );
            if !joins {
                // Not allowed as it stands; another short word may
                // still make it valid.
                compound_ok = false;
            }
        }

        // Step back from this word's start over the last character of the
        // one before it.
        let tail = (preword_len > 0).then(|| char_back(&self.preword, preword_len));
        Some((tail, compound_ok))
    }

    /// Put the word with its proper case into `preword`, after whatever a
    /// previous split or compound left there.
    fn build_preword(&mut self, flags: WordFlags, prev_word_tail: Option<usize>) {
        let level = self.depth as usize;
        let split_off = self.stack[level].split_off as usize;
        let preword_len = self.stack[level].preword_len as usize;
        // `tword` was terminated by the caller.
        let good = &self.tword[split_off..];

        if self.soundfold {
            // Sound-folded words have no case to get right.
            copy_terminated(&mut self.preword[preword_len..], until_nul(good));
        } else if flags.has(WordFlags::KEEPCAP) {
            // The spelling has to come from the keep-case tree.
            find_keepcap_word(self.slang, good, &mut self.preword[preword_len..]);
        } else {
            // Take the bad word's caps type: a one-cap or all-cap bad
            // word wants a good word to match. An all-cap bad word
            // one character long only says one-cap, though.
            let mut caps = self.su.su_badflags;
            if caps.has(WordFlags::ALLCAP)
                && self.su.su_badlen as usize == c_cluster_len(&self.su.su_bad)
            {
                caps = WordFlags::ONECAP;
            }
            caps |= flags;

            // A compound word appended after a word character must not
            // start with a capital.
            if let Some(tail) = prev_word_tail
                && spell_iswordp_nmw(&self.preword[tail..], Win::current())
            {
                caps.clear(WordFlags::ONECAP);
            }
            make_case_word(good, &mut self.preword[preword_len..], caps);
        }
    }

    /// The bad word ends here and so does a valid good word: offer it.
    ///
    /// Returns the score the walk carries on with, which the non-word
    /// penalty may have raised.
    fn offer_word(&mut self, mut newscore: c_int) -> c_int {
        let level = self.depth as usize;

        if self.soundfold {
            // A sound-folded match stands for real words, which have
            // to be found and scored separately.
            let score = self.stack[level].score;
            add_sound_suggest(self.su, self.slang, self.langp, &self.preword, score);
            return newscore;
        }
        if self.stack[level].bad_idx == 0 {
            return newscore;
        }

        // Penalise turning a non-word character into a word character,
        // as in "thes," -> "these". `bad_idx` is not zero -- tested just
        // above -- so there is a character before it to step back over.
        let p = char_back(self.fword, self.stack[level].bad_idx as usize);
        if !spell_iswordp(until_nul(&self.fword[p..]), Win::current()) && self.preword[0] != 0 {
            let end = self.preword_len();
            let q = char_back(&self.preword, end);
            if spell_iswordp(&self.preword[q..end], Win::current()) {
                newscore += SCORE_NONWORD;
            }
        }

        let preword_len = self.stack[level].preword_len as usize;
        let score = score_wordcount_adj(
            self.slang,
            self.stack[level].score + newscore,
            &self.preword[preword_len..],
            self.stack[level].preword_len > 0,
        );
        if score > self.su.su_maxscore {
            return newscore;
        }

        let replaced = self.stack[level].bad_idx as c_int - self.repextra;
        let sallang = self.langp.sallang;
        add_suggestion(
            self.su,
            List::Main,
            &self.preword,
            replaced,
            score,
            0,
            false,
            sallang,
            false,
        );

        if self.su.su_badflags.has(WordFlags::MIXCAP) {
            // With mixed case there is no telling whether the word
            // should be upper or lower case, so offer both.
            let caps = captype(&self.preword, usize::MAX);
            if caps.is_empty() || caps == WordFlags::ALLCAP {
                let split_off = self.stack[level].split_off as usize;
                // `tword` was terminated before this ran.
                make_case_word(
                    &self.tword[split_off..],
                    &mut self.preword[preword_len..],
                    WordFlags::ALLCAP.when(caps.is_empty()),
                );
                add_suggestion(
                    self.su,
                    List::Main,
                    &self.preword,
                    replaced,
                    score + SCORE_ICASE,
                    0,
                    false,
                    sallang,
                    false,
                );
            }
        }
        newscore
    }
}

/// Is this word, which ends at its end or at a NUL, one the walk has been
/// told never to suggest?
fn was_banned(su: &SugInfo, word: &[u8]) -> bool {
    !su.su_banned.is_empty() && su.su_banned.contains(until_nul(word))
}
