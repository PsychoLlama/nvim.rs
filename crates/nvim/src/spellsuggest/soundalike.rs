//! Suggestions found by how the word sounds.
//!
//! The edit-distance search can only find words spelled nearly the same.
//! This one finds words *pronounced* nearly the same, which is what turns
//! "nashun" into "nation". It needs the language's `.sug` companion file:
//! a second word tree holding every sound-folded form the dictionary
//! produces, and, for each of them, the numbers of the real words that
//! fold to it.
//!
//! One pass is: sound-fold the bad word, walk the sound-fold tree with the
//! same trie walker the edit-distance search uses (so that inserts,
//! deletes and swaps of *sounds* are tried), and for every sound-folded
//! word it reaches call [`add_sound_suggest`] to turn that back into real
//! words. The passes share a per-language
//! table of soundfolds already handled, `SugInfo::su_sounddone`, because
//! the same soundfold is reached many times over and the work below is far
//! too slow to repeat.
//!
//! # From a soundfold back to words
//!
//! The `.sug` file stores, per soundfold, the *word numbers* of the words
//! that produce it -- as increasing deltas, each in as few bytes as it
//! fits ([`bytes2offset`]). A word number is turned back into letters by
//! walking the case-folded tree and counting, at every branch, how many
//! words hang below the siblings passed over; that count is what the
//! reader stashed in the first index entry of each child.
//!
//! Words flagged `KEEPCAP` cannot be reconstructed that way -- the tree
//! walked is the case-folded one -- so [`find_keepcap_word`] searches the
//! keep-case tree for a word that folds to the same thing, trying each
//! character both folded and upper-case.

#![forbid(unsafe_code)]

use crate::cstr::until_nul;
use crate::mbyte::{char_at, char_len};
use crate::memline::Lines;
use crate::message::internal_error;
use crate::spell::{
    WordFlags, WordHasher, allcap_copy, loaded_langs, make_case_word, spell_soundfold,
};
use crate::spellsuggest::collect::{List, add_suggestion};
use crate::spellsuggest::score::{
    EMPTY_SOUND, score_wordcount_adj, spell_edit_score, spell_edit_score_limit, spell_isupper,
    spell_tofold,
};
use crate::spellsuggest::walk::suggest_trie_walk;
use crate::spellsuggest::{
    MAXWLEN, SCORE_ICASE, SCORE_LIMITMAX, SCORE_MAXMAX, SCORE_REGION, SPS_DOUBLE, SugInfo, TAB,
    sps_flags, window_langs,
};
use crate::types::{LangId, LangP, LineNr, NUL, SpellLang, int16_t};
use core::ffi::c_int;
use core::hash::BuildHasherDefault;
use std::collections::HashMap;

/// The best score seen so far for each sound-folded word one language's
/// passes have handled.
pub(crate) type SoundDone = HashMap<Box<[u8]>, int16_t, BuildHasherDefault<WordHasher>>;

/// Does a language have both a sound-folding table and a loaded `.sug`?
/// Without both there is nothing to walk.
fn has_sound_tree(slang: &SpellLang) -> bool {
    slang.has_soundfold() && !slang.sl_sound_tree.is_empty()
}

/// Find suggestions by comparing the bad word in sound-a-like form.
///
/// Postponed prefixes are not supported here.
pub(super) fn suggest_try_soundalike(su: &mut SugInfo) {
    for langp in window_langs() {
        loaded_langs.with(|langs| {
            let Some(slang) = langs.get(langp.slang).filter(|slang| has_sound_tree(slang)) else {
                return;
            };
            let mut salword = EMPTY_SOUND;
            spell_soundfold(slang, &su.su_fbadword, true, &mut salword);
            // The same walker as the edit-distance search, told to treat
            // the tree it walks as sounds rather than letters.
            suggest_trie_walk(su, langp, langs, &mut salword, true);
        });
    }
}

/// Read one word-number delta from a `.sug` line.
///
/// The numbers are stored in as few bytes as they fit, with the count in
/// the top bits of the first byte and every byte biased by one so that no
/// NUL can appear inside the line. Returns the delta and steps `pos` past
/// what it consumed.
pub(super) fn bytes2offset(bytes: &[u8], pos: &mut usize) -> c_int {
    // A missing byte only happens on a damaged file; reading it as zero
    // keeps the walk inside the line where the C would have run on into
    // whatever followed it.
    let next = |pos: &mut usize| -> c_int {
        let b = bytes.get(*pos).copied().unwrap_or(0) as c_int;
        *pos += 1;
        b - 1
    };

    let c = bytes.get(*pos).copied().unwrap_or(0) as c_int;
    *pos += 1;
    let (mut nr, extra) = if c & 0x80 == 0 {
        (c - 1, 0)
    } else if c & 0xc0 == 0x80 {
        ((c & 0x3f) - 1, 1)
    } else if c & 0xe0 == 0xc0 {
        ((c & 0x1f) - 1, 2)
    } else {
        ((c & 0x0f) - 1, 3)
    };
    for _ in 0..extra {
        nr = nr * 255 + next(pos);
    }
    nr
}

/// A match with a sound-folded word was found: add the real word or words
/// that produce it.
///
/// `goodword` ends at its end or at a NUL; `score` is the sound-a-like score
/// the walk arrived at; `slang` is `langp`'s language, with a loaded `.sug`.
pub(super) fn add_sound_suggest(
    su: &mut SugInfo,
    slang: &SpellLang,
    langp: LangP,
    goodword: &[u8],
    score: c_int,
) {
    let goodword = until_nul(goodword);

    // The same soundfold turns up many times with different scores and
    // what follows is slow, so only the best score for each is done.
    let done = sounddone(su, langp.slang);
    match done.get_mut(goodword) {
        None => {
            done.insert(goodword.into(), score as int16_t);
        }
        Some(best) => {
            if score >= c_int::from(*best) {
                return;
            }
            *best = score as int16_t;
        }
    }

    let sfwordnr = soundfold_find(slang, goodword);
    if sfwordnr < 0 {
        internal_error(c"add_sound_suggest()");
        return;
    }

    // Walk the list of word numbers that produce this soundfold.
    let sugbuf = slang
        .sl_sugbuf
        .as_ref()
        .expect("a loaded `.sug` has its buffer");
    let mut lines = Lines::in_buffer(sugbuf.buf());
    let deltas = lines.line(sfwordnr as LineNr + 1);
    let mut pos = 0;
    let mut orgnr = 0;

    while pos < deltas.len() {
        orgnr += bytes2offset(deltas, &mut pos);
        let (theword, n, i) = word_number_to_letters(slang, orgnr);
        emit_word(su, langp, slang, &theword, n, i, score);
    }
}

/// The table of soundfolds already handled for language `id`, made empty on
/// first use in a run.
fn sounddone(su: &mut SugInfo, id: LangId) -> &mut SoundDone {
    let at = match su.su_sounddone.iter().position(|(lang, _)| *lang == id) {
        Some(at) => at,
        None => {
            su.su_sounddone.push((id, SoundDone::default()));
            su.su_sounddone.len() - 1
        }
    };
    &mut su.su_sounddone[at].1
}

/// Turn a word number back into its letters by walking the case-folded
/// tree, counting the words hanging below every sibling passed over.
///
/// Returns the NUL-terminated word, the tree node the word ended at and the
/// sibling index to continue the flag scan from.
pub(super) fn word_number_to_letters(
    slang: &SpellLang,
    orgnr: c_int,
) -> ([u8; MAXWLEN], usize, usize) {
    let tree = slang.sl_fold_tree.view();

    let mut theword = [0u8; MAXWLEN];
    let mut n: usize = 0;
    let mut wordcount = 0;
    let mut wlen = 0;
    let mut i: usize = 1;

    while wlen < MAXWLEN - 3 {
        i = 1;
        if wordcount == orgnr && tree.ends_word(n + 1) {
            break; // found the end of the word
        }
        if tree.ends_word(n + 1) {
            wordcount += 1;
        }

        // Skip the NUL bytes; there can be several.
        let mut bad = false;
        while tree.ends_word(n + i) {
            if i > tree.node_len(n) {
                // Safety check: the tree disagrees with the count.
                theword[wlen..wlen + 3].copy_from_slice(b"BAD");
                wlen += 3;
                bad = true;
                break;
            }
            i += 1;
        }
        if bad {
            break;
        }

        // One of the siblings has the word under it. The index each
        // sibling holds is a node start, and that node's own entry is the
        // number of words below it.
        while i < tree.node_len(n) {
            let wc = tree.idx(tree.child_node(n + i));
            if wordcount + wc > orgnr {
                break;
            }
            wordcount += wc;
            i += 1;
        }

        theword[wlen] = tree.byte(n + i);
        n = tree.child_node(n + i);
        wlen += 1;
    }
    theword[wlen] = NUL as u8;
    (theword, n, i)
}

/// Offer the word `theword` under every flag/region combination the tree
/// records for it. `n`/`i` come from [`word_number_to_letters`].
#[allow(clippy::too_many_arguments)]
fn emit_word(
    su: &mut SugInfo,
    langp: LangP,
    slang: &SpellLang,
    theword: &[u8; MAXWLEN],
    n: usize,
    mut i: usize,
    score: c_int,
) {
    let tree = slang.sl_fold_tree.view();

    // The flags and regions are the NUL-byte children of this node, so the
    // scan stops at the node's child count.
    while i <= tree.node_len(n) && tree.ends_word(n + i) {
        let mut cword = [0u8; MAXWLEN];
        let mut flags = WordFlags::from_bits(tree.idx(n + i));
        i += 1;

        if flags.has(WordFlags::NOSUGGEST) {
            continue;
        }

        let p: &[u8] = if flags.has(WordFlags::KEEPCAP) {
            // The letters came out of the case-folded tree, so the
            // real spelling has to be looked up in the keep-case one.
            find_keepcap_word(slang, theword, &mut cword);
            until_nul(&cword)
        } else {
            flags |= su.su_badflags;
            if flags.has(WordFlags::CAPMASK) {
                make_case_word(theword, &mut cword, flags);
                until_nul(&cword)
            } else {
                until_nul(theword)
            }
        };

        if sps_flags.get() & SPS_DOUBLE != 0 {
            if score <= su.su_maxscore {
                let badlen = su.su_badlen;
                let id = Some(langp.slang);
                add_suggestion(su, List::Sound, p, badlen, score, 0, false, id, false);
            }
            continue;
        }

        // A word from another region is worth less.
        let mut goodscore = if flags.has(WordFlags::REGION)
            && (flags.bits() as u32 >> 16) & langp.region as u32 == 0
        {
            SCORE_REGION
        } else {
            0
        };

        // A small penalty for turning the first letter from lower to
        // upper case: "tath" -> "Kath" is less likely than "tath" ->
        // "path". Not when the letter is the same, which is counted
        // already.
        let gc = char_at(p);
        if spell_isupper(gc) {
            let bc = char_at(su.badword());
            if !spell_isupper(bc) && spell_tofold(bc) != spell_tofold(gc) {
                goodscore += SCORE_ICASE / 2;
            }
        }

        // The edit distance for the good word. REP items are not
        // considered, which may leave the score a little high. A
        // ceiling makes it faster; past a high enough ceiling the
        // depth-first search costs more than filling the table.
        let limit = (4 * (su.su_sfmaxscore - goodscore) - score) / 3;
        goodscore += if limit > SCORE_LIMITMAX {
            spell_edit_score(Some(slang), su.badword(), p)
        } else {
            spell_edit_score_limit(Some(slang), su.badword(), p, limit)
        };

        if goodscore >= SCORE_MAXMAX {
            continue;
        }

        goodscore = score_wordcount_adj(slang, goodscore, p, false);
        goodscore = (3 * goodscore + score) / 4;
        if goodscore <= su.su_sfmaxscore {
            let badlen = su.su_badlen;
            let id = Some(langp.slang);
            add_suggestion(su, List::Main, p, badlen, goodscore, score, true, id, true);
        }
    }
}

/// Find `word` in the sound-fold tree and return its word number, or -1
/// when it is not there. `word` ends at its end or at a NUL.
pub(super) fn soundfold_find(slang: &SpellLang, word: &[u8]) -> c_int {
    let tree = slang.sl_sound_tree.view();
    let byte = |at: usize| word.get(at).copied().unwrap_or(0);

    let mut arridx: usize = 0;
    let mut wlen = 0;
    let mut wordnr = 0;

    // The walk stops at the word's terminator: every read of it past the
    // first is behind a test on the byte before.
    loop {
        // The first byte of a node is how many bytes may follow.
        let mut len = tree.node_len(arridx);
        arridx += 1;

        // A leading zero byte means a word may end here.
        let mut c = byte(wlen);
        if tree.ends_word(arridx) {
            if c == NUL as u8 {
                return wordnr;
            }

            // Skip the zeros; there can be several.
            let ends = tree.word_ends(arridx, len);
            arridx += ends;
            len -= ends;
            if len == 0 {
                return -1; // no children, the word should have ended
            }
            wordnr += 1;
        }

        if c == NUL as u8 {
            return -1; // the word ends but the tree does not
        }

        // Linear search over the accepted bytes, counting the words
        // hanging below the ones passed over. The index a sibling holds
        // is itself a node start, and that node's own entry is the
        // number of words below it.
        if c == TAB as u8 {
            c = b' '; // a tab counts as a space
        }
        while tree.byte(arridx) < c {
            wordnr += tree.idx(tree.child_node(arridx));
            arridx += 1;
            len -= 1;
            if len == 0 {
                return -1; // ran out of bytes without finding it
            }
        }
        if tree.byte(arridx) != c {
            return -1;
        }

        arridx = tree.child_node(arridx);
        wlen += 1;

        // One space in the good word may stand for several in the
        // word being checked.
        if c == b' ' {
            while byte(wlen) == b' ' || byte(wlen) == TAB as u8 {
                wlen += 1;
            }
        }
    }
}

/// One level of the keep-case search.
#[derive(Clone, Copy, Default)]
struct KeepCapLevel {
    /// Tree node this level starts from.
    arridx: usize,
    /// 0 before either case has been tried, 1 after folded, 2 after upper.
    round: c_int,
    /// How far into the folded and upper-case words this level is.
    fwordidx: usize,
    uwordidx: usize,
    /// How much of the answer has been written.
    kwordlen: usize,
}

/// `utf_ptr2len` over the bytes of a C string: 0 at its end.
fn c_char_len(bytes: &[u8]) -> usize {
    match bytes.first() {
        None | Some(0) => 0,
        Some(_) => char_len(bytes),
    }
}

/// Find the keep-case spelling of a case-folded word, NUL-terminated into
/// `kword`. `fword` ends at its end or at a NUL.
///
/// There could in theory be several keep-case words folding to the same
/// thing; this finds one of them. Each character is tried both folded and
/// upper-case, and changing case can change a character's byte length, so
/// the two words are tracked with separate offsets.
pub(super) fn find_keepcap_word(slang: &SpellLang, fword: &[u8], kword: &mut [u8]) {
    let tree = slang.sl_keep_tree.view();
    if tree.is_empty() {
        // The tree is empty: cannot happen.
        kword[0] = NUL as u8;
        return;
    }
    let fword = until_nul(fword);

    let mut uword = [0u8; MAXWLEN];
    allcap_copy(fword, &mut uword);
    let uword = until_nul(&uword);

    let mut stack = [KeepCapLevel::default(); MAXWLEN];
    let mut depth: isize = 0;

    // Every level's offsets are byte counts already matched inside `fword`
    // and `uword`; every tree index is a node start the tree handed back,
    // stepped forward at most that node's child count.
    while depth >= 0 {
        let level = stack[depth as usize];
        if level.fwordidx >= fword.len() {
            // At the end of the folded word: if the tree lets a word
            // end here, this is the answer.
            if tree.ends_word(level.arridx + 1) {
                kword[level.kwordlen] = NUL as u8;
                return;
            }
            // Otherwise the answer would be too long; back up.
            depth -= 1;
            continue;
        }

        stack[depth as usize].round += 1;
        let round = stack[depth as usize].round;
        if round > 2 {
            // Both cases tried; back up.
            depth -= 1;
            continue;
        }

        let folded = &fword[level.fwordidx..];
        let upper = uword.get(level.uwordidx..).unwrap_or_default();
        let (flen, ulen) = (c_char_len(folded), c_char_len(upper));
        let (chars, taken) = if round == 1 {
            (folded, flen)
        } else {
            (upper, ulen)
        };

        // Match the character's bytes one node at a time.
        let mut tryidx = level.arridx;
        let mut matched = 0;
        while matched < taken {
            let len = tree.node_len(tryidx);
            tryidx += 1;
            let Some(at) = tree.child(tryidx, len, chars[matched]) else {
                break;
            };
            tryidx = tree.child_node(at);
            matched += 1;
        }

        // An answer longer than the buffer cannot be written down; the C
        // wrote on past it.
        if matched != taken || level.kwordlen + taken >= kword.len() {
            continue;
        }

        // The whole character matched: keep it and go a level deeper.
        kword[level.kwordlen..level.kwordlen + taken].copy_from_slice(&chars[..taken]);
        depth += 1;
        stack[depth as usize] = KeepCapLevel {
            arridx: tryidx,
            round: 0,
            fwordidx: level.fwordidx + flen,
            uwordidx: level.uwordidx + ulen,
            kwordlen: level.kwordlen + taken,
        };
    }

    // Not found: cannot happen.
    kword[0] = NUL as u8;
}
