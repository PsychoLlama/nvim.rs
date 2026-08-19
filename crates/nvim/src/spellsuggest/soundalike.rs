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
//! words. [`suggest_try_soundalike_prep`] and
//! [`suggest_try_soundalike_finish`] bracket it around a per-language
//! table of soundfolds already handled, because the same soundfold is
//! reached many times over and the work below is far too slow to repeat.
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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::hashtab::{hash_add_item, hash_clear, hash_hash, hash_init, hash_lookup, hash_removed};
use crate::mbyte::{utf_ptr2char, utf_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmalloc};
use crate::message::internal_error;
use crate::spell::{allcap_copy, make_case_word, spell_soundfold};
use crate::spellsuggest::collect::add_suggestion;
use crate::spellsuggest::score::{
    EMPTY_SOUND, score_wordcount_adj, spell_edit_score, spell_edit_score_limit, spell_isupper,
    spell_tofold,
};
use crate::spellsuggest::walk::suggest_trie_walk;
use crate::spellsuggest::{
    MAXWLEN, SCORE_ICASE, SCORE_LIMITMAX, SCORE_MAXMAX, SCORE_REGION, SPS_DOUBLE, TAB, WF_CAPMASK,
    WF_KEEPCAP, WF_NOSUGGEST, WF_REGION, sps_flags, suginfo_T,
};
use crate::types::{NUL, hashitem_T, idx_T, int16_t, langp_T, linenr_T, slang_T, uint8_t};
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr};

/// The best score seen so far for one sound-folded word, with that word
/// stored inline after it. The hash table of soundfolds already handled
/// keys on the inline word.
#[repr(C)]
struct sftword_T {
    sft_score: int16_t,
    sft_word: [uint8_t; 0],
}

/// Where the inline soundfolded word sits inside a `sftword_T`; the hash
/// table keys on that field, so the record is recovered by stepping back.
const SFT_WORD_OFF: usize = mem::offset_of!(sftword_T, sft_word);

/// Does a language have both a sound-folding table and a loaded `.sug`?
/// Without both there is nothing to walk.
///
/// # Safety
///
/// `slang` must be a loaded language.
unsafe fn has_sound_tree(slang: *mut slang_T) -> bool {
    // SAFETY: the caller guarantees the language.
    unsafe { (*slang).sl_sal.ga_len > 0 && !(*slang).sl_sbyts.is_null() }
}

/// Prepare the per-language table of soundfolds already handled.
///
/// # Safety
///
/// The current window must have its languages loaded.
pub unsafe fn suggest_try_soundalike_prep() {
    // SAFETY: the caller guarantees the window's spell state.
    unsafe {
        for lp in crate::spellsuggest::window_langs() {
            if has_sound_tree(lp.lp_slang) {
                hash_init(&raw mut (*lp.lp_slang).sl_sounddone);
            }
        }
    }
}

/// Find suggestions by comparing the bad word in sound-a-like form.
///
/// Postponed prefixes are not supported here.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
pub unsafe fn suggest_try_soundalike(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees `su` and the window's spell state.
    unsafe {
        for lp in crate::spellsuggest::window_langs() {
            if !has_sound_tree(lp.lp_slang) {
                continue;
            }
            let mut salword = EMPTY_SOUND;
            spell_soundfold(
                lp.lp_slang,
                &raw mut (*su).su_fbadword as *mut c_char,
                true,
                salword.as_mut_ptr(),
            );
            // The same walker as the edit-distance search, told to treat
            // the tree it walks as sounds rather than letters.
            suggest_trie_walk(su, lp, salword.as_mut_ptr(), true);
        }
    }
}

/// Release the per-language table of soundfolds already handled.
///
/// # Safety
///
/// The current window must have its languages loaded.
pub unsafe fn suggest_try_soundalike_finish() {
    // SAFETY: the caller guarantees the window's spell state; every key in
    // the table is the inline word of a `sftword_T` this module allocated.
    unsafe {
        for lp in crate::spellsuggest::window_langs() {
            let slang = lp.lp_slang;
            if !has_sound_tree(slang) {
                continue;
            }

            let done = &raw mut (*slang).sl_sounddone;
            let mut todo = (*done).ht_used as c_int;
            let mut hi: *mut hashitem_T = (*done).ht_array;
            while todo > 0 {
                let key = (*hi).hi_key;
                if !(key.is_null() || core::ptr::eq(key, &raw const hash_removed)) {
                    xfree(key.sub(SFT_WORD_OFF) as *mut c_void);
                    todo -= 1;
                }
                hi = hi.add(1);
            }

            // Another region may reuse the table, so leave it empty rather
            // than freed.
            hash_clear(done);
            hash_init(done);
        }
    }
}

/// Read one word-number delta from a `.sug` line.
///
/// The numbers are stored in as few bytes as they fit, with the count in
/// the top bits of the first byte and every byte biased by one so that no
/// NUL can appear inside the line. Returns the delta and steps `pos` past
/// what it consumed.
fn bytes2offset(bytes: &[u8], pos: &mut usize) -> c_int {
    // A missing byte only happens on a damaged file; reading it as zero
    // keeps the walk inside the line where the C would have run on into
    // whatever followed it.
    let mut next = |pos: &mut usize| -> c_int {
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
/// `score` is the sound-a-like score the walk arrived at.
///
/// # Safety
///
/// `su` and `lp` must be valid and the language must have a loaded `.sug`.
pub unsafe fn add_sound_suggest(
    su: *mut suginfo_T,
    goodword: *mut c_char,
    score: c_int,
    lp: *mut langp_T,
) {
    // SAFETY: the caller guarantees the pointers; the tree walks below are
    // bounded by the counts the tree itself stores, as in the C.
    unsafe {
        let slang = (*lp).lp_slang;

        // The same soundfold turns up many times with different scores and
        // what follows is slow, so only the best score for each is done.
        let hash = hash_hash(goodword);
        let goodword_len = libc::strlen(goodword) as usize;
        let hi = hash_lookup(&raw mut (*slang).sl_sounddone, goodword, goodword_len, hash);
        let key = (*hi).hi_key;
        if key.is_null() || core::ptr::eq(key, &raw const hash_removed) {
            let sft = xmalloc(SFT_WORD_OFF + goodword_len + 1) as *mut sftword_T;
            (*sft).sft_score = score as int16_t;
            let word = (sft as *mut u8).add(SFT_WORD_OFF);
            ptr::copy_nonoverlapping(goodword as *const u8, word, goodword_len + 1);
            hash_add_item(
                &raw mut (*slang).sl_sounddone,
                hi,
                word as *mut c_char,
                hash,
            );
        } else {
            let sft = key.sub(SFT_WORD_OFF) as *mut sftword_T;
            if score >= (*sft).sft_score as c_int {
                return;
            }
            (*sft).sft_score = score as int16_t;
        }

        let sfwordnr = soundfold_find(slang, goodword);
        if sfwordnr < 0 {
            internal_error(c"add_sound_suggest()".as_ptr());
            return;
        }

        // Walk the list of word numbers that produce this soundfold.
        let nrline = ml_get_buf((*slang).sl_sugbuf, sfwordnr as linenr_T + 1);
        let deltas = core::ffi::CStr::from_ptr(nrline).to_bytes();
        let mut pos = 0;
        let mut orgnr = 0;

        while pos < deltas.len() {
            orgnr += bytes2offset(deltas, &mut pos);
            let (mut theword, n, i) = word_number_to_letters(slang, orgnr);
            emit_word(su, lp, slang, &mut theword, n, i, score);
        }
    }
}

/// Turn a word number back into its letters by walking the case-folded
/// tree, counting the words hanging below every sibling passed over.
///
/// Returns the word buffer, its length, the tree node the word ended at
/// and the sibling index to continue the flag scan from.
///
/// # Safety
///
/// `slang` must have a case-folded tree.
unsafe fn word_number_to_letters(
    slang: *mut slang_T,
    orgnr: c_int,
) -> ([c_char; MAXWLEN], usize, c_int) {
    // SAFETY: the caller guarantees the tree; the indices below stay inside
    // it because every node's first byte is its number of children.
    unsafe {
        let byts = (*slang).sl_fbyts;
        let idxs: *mut idx_T = (*slang).sl_fidxs;

        let mut theword = [0 as c_char; MAXWLEN];
        let mut n: usize = 0;
        let mut wordcount = 0;
        let mut wlen = 0;
        let mut i: c_int = 1;

        while wlen < MAXWLEN - 3 {
            i = 1;
            if wordcount == orgnr && *byts.add(n + 1) == 0 {
                break; // found the end of the word
            }
            if *byts.add(n + 1) == 0 {
                wordcount += 1;
            }

            // Skip the NUL bytes; there can be several.
            let mut bad = false;
            while *byts.add(n + i as usize) == 0 {
                if i > *byts.add(n) as c_int {
                    // Safety check: the tree disagrees with the count.
                    theword[wlen..wlen + 3].copy_from_slice(&[
                        b'B' as c_char,
                        b'A' as c_char,
                        b'D' as c_char,
                    ]);
                    wlen += 3;
                    bad = true;
                    break;
                }
                i += 1;
            }
            if bad {
                break;
            }

            // One of the siblings has the word under it.
            while i < *byts.add(n) as c_int {
                let wc = *idxs.offset(*idxs.add(n + i as usize) as isize) as c_int;
                if wordcount + wc > orgnr {
                    break;
                }
                wordcount += wc;
                i += 1;
            }

            theword[wlen] = *byts.add(n + i as usize) as c_char;
            n = *idxs.add(n + i as usize) as usize;
            wlen += 1;
        }
        theword[wlen] = NUL as c_char;
        (theword, n, i)
    }
}

/// Offer the word `theword` under every flag/region combination the tree
/// records for it.
///
/// # Safety
///
/// All pointers must be valid and `n`/`i` must come from
/// [`word_number_to_letters`].
#[allow(clippy::too_many_arguments)]
unsafe fn emit_word(
    su: *mut suginfo_T,
    lp: *mut langp_T,
    slang: *mut slang_T,
    theword: &mut [c_char; MAXWLEN],
    n: usize,
    mut i: c_int,
    score: c_int,
) {
    // SAFETY: the caller guarantees the pointers and the tree position.
    unsafe {
        let byts = (*slang).sl_fbyts;
        let idxs: *mut idx_T = (*slang).sl_fidxs;

        // The flags and regions are the NUL-byte children of this node. The
        // bound has to be tested before the byte is read.
        while i <= *byts.add(n) as c_int && *byts.add(n + i as usize) == 0 {
            let mut cword = [0 as c_char; MAXWLEN];
            let mut flags = *idxs.add(n + i as usize) as c_int;
            i += 1;

            if flags & WF_NOSUGGEST != 0 {
                continue;
            }

            let p = if flags & WF_KEEPCAP != 0 {
                // The letters came out of the case-folded tree, so the
                // real spelling has to be looked up in the keep-case one.
                find_keepcap_word(slang, theword.as_mut_ptr(), cword.as_mut_ptr());
                cword.as_mut_ptr()
            } else {
                flags |= (*su).su_badflags;
                if flags & WF_CAPMASK != 0 {
                    make_case_word(theword.as_mut_ptr(), cword.as_mut_ptr(), flags);
                    cword.as_mut_ptr()
                } else {
                    theword.as_mut_ptr()
                }
            };

            if sps_flags.get() & SPS_DOUBLE != 0 {
                if score <= (*su).su_maxscore {
                    add_suggestion(
                        su,
                        &raw mut (*su).su_sga,
                        p,
                        (*su).su_badlen,
                        score,
                        0,
                        false,
                        slang,
                        false,
                    );
                }
                continue;
            }

            // A word from another region is worth less.
            let mut goodscore =
                if flags & WF_REGION != 0 && (flags as u32 >> 16) & (*lp).lp_region as u32 == 0 {
                    SCORE_REGION
                } else {
                    0
                };

            // A small penalty for turning the first letter from lower to
            // upper case: "tath" -> "Kath" is less likely than "tath" ->
            // "path". Not when the letter is the same, which is counted
            // already.
            let gc = utf_ptr2char(p);
            if spell_isupper(gc) {
                let bc = utf_ptr2char(&raw const (*su).su_badword as *const c_char);
                if !spell_isupper(bc) && spell_tofold(bc) != spell_tofold(gc) {
                    goodscore += SCORE_ICASE / 2;
                }
            }

            // The edit distance for the good word. REP items are not
            // considered, which may leave the score a little high. A
            // ceiling makes it faster; past a high enough ceiling the
            // depth-first search costs more than filling the table.
            let limit = (4 * ((*su).su_sfmaxscore - goodscore) - score) / 3;
            goodscore += if limit > SCORE_LIMITMAX {
                spell_edit_score(
                    Some(&*slang),
                    &raw const (*su).su_badword as *const c_char,
                    p,
                )
            } else {
                spell_edit_score_limit(
                    Some(&*slang),
                    &raw const (*su).su_badword as *const c_char,
                    p,
                    limit,
                )
            };

            if goodscore >= SCORE_MAXMAX {
                continue;
            }

            goodscore = score_wordcount_adj(&*slang, goodscore, p, false);
            goodscore = (3 * goodscore + score) / 4;
            if goodscore <= (*su).su_sfmaxscore {
                add_suggestion(
                    su,
                    &raw mut (*su).su_ga,
                    p,
                    (*su).su_badlen,
                    goodscore,
                    score,
                    true,
                    slang,
                    true,
                );
            }
        }
    }
}

/// Find `word` in the sound-fold tree and return its word number, or -1
/// when it is not there.
///
/// # Safety
///
/// `slang` must have a loaded `.sug` and `word` must be NUL-terminated.
pub unsafe fn soundfold_find(slang: *mut slang_T, word: *mut c_char) -> c_int {
    // SAFETY: the caller guarantees the tree and the word; each node's
    // first byte is its child count, which bounds every index below.
    unsafe {
        let byts = (*slang).sl_sbyts;
        let idxs: *mut idx_T = (*slang).sl_sidxs;
        let ptr = word as *mut u8;

        let mut arridx: idx_T = 0;
        let mut wlen = 0;
        let mut wordnr = 0;

        loop {
            // The first byte of a node is how many bytes may follow.
            let mut len = *byts.offset(arridx as isize) as c_int;
            arridx += 1;

            // A leading zero byte means a word may end here.
            let mut c = *ptr.add(wlen) as c_int;
            if *byts.offset(arridx as isize) == 0 {
                if c == NUL {
                    return wordnr;
                }

                // Skip the zeros; there can be several.
                while len > 0 && *byts.offset(arridx as isize) == 0 {
                    arridx += 1;
                    len -= 1;
                }
                if len == 0 {
                    return -1; // no children, the word should have ended
                }
                wordnr += 1;
            }

            if c == NUL {
                return -1; // the word ends but the tree does not
            }

            // Linear search over the accepted bytes, counting the words
            // hanging below the ones passed over.
            if c == TAB {
                c = ' ' as c_int; // a tab counts as a space
            }
            while (*byts.offset(arridx as isize) as c_int) < c {
                wordnr += *idxs.offset(*idxs.offset(arridx as isize) as isize) as c_int;
                arridx += 1;
                len -= 1;
                if len == 0 {
                    return -1; // ran out of bytes without finding it
                }
            }
            if *byts.offset(arridx as isize) as c_int != c {
                return -1;
            }

            arridx = *idxs.offset(arridx as isize);
            wlen += 1;

            // One space in the good word may stand for several in the
            // word being checked.
            if c == ' ' as c_int {
                while *ptr.add(wlen) as c_int == ' ' as c_int || *ptr.add(wlen) as c_int == TAB {
                    wlen += 1;
                }
            }
        }
    }
}

/// One level of the keep-case search.
#[derive(Clone, Copy, Default)]
struct KeepCapLevel {
    /// Tree node this level starts from.
    arridx: idx_T,
    /// 0 before either case has been tried, 1 after folded, 2 after upper.
    round: c_int,
    /// How far into the folded and upper-case words this level is.
    fwordidx: usize,
    uwordidx: usize,
    /// How much of the answer has been written.
    kwordlen: usize,
}

/// Find the keep-case spelling of a case-folded word.
///
/// There could in theory be several keep-case words folding to the same
/// thing; this finds one of them. Each character is tried both folded and
/// upper-case, and changing case can change a character's byte length, so
/// the two words are tracked with separate offsets.
///
/// # Safety
///
/// `fword` must be NUL-terminated and `kword` must have room for
/// `MAXWLEN` bytes.
pub unsafe fn find_keepcap_word(slang: *mut slang_T, fword: *mut c_char, kword: *mut c_char) {
    // SAFETY: the caller guarantees the words; the tree indices are bounded
    // by each node's child count.
    unsafe {
        let byts = (*slang).sl_kbyts;
        let idxs: *mut idx_T = (*slang).sl_kidxs;
        if byts.is_null() {
            // The tree is empty: cannot happen.
            *kword = NUL as c_char;
            return;
        }

        let mut uword = [0 as c_char; MAXWLEN];
        allcap_copy(fword, uword.as_mut_ptr());

        let mut stack = [KeepCapLevel::default(); MAXWLEN];
        let mut depth: isize = 0;

        while depth >= 0 {
            let level = stack[depth as usize];
            if *fword.add(level.fwordidx) == NUL as c_char {
                // At the end of the folded word: if the tree lets a word
                // end here, this is the answer.
                if *byts.offset(level.arridx as isize + 1) == 0 {
                    *kword.add(level.kwordlen) = NUL as c_char;
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

            let flen = utf_ptr2len(fword.add(level.fwordidx)) as usize;
            let ulen = utf_ptr2len(uword.as_ptr().add(level.uwordidx)) as usize;
            let (mut p, mut l) = if round == 1 {
                (fword.add(level.fwordidx) as *const u8, flen)
            } else {
                (uword.as_ptr().add(level.uwordidx) as *const u8, ulen)
            };

            // Match the character's bytes one node at a time.
            let mut tryidx = level.arridx;
            while l > 0 {
                let len = *byts.offset(tryidx as isize) as idx_T;
                tryidx += 1;
                let c = *p as c_int;
                p = p.add(1);

                let mut lo = tryidx;
                let mut hi = tryidx + len - 1;
                while lo < hi {
                    let m = (lo + hi) / 2;
                    let b = *byts.offset(m as isize) as c_int;
                    if b > c {
                        hi = m - 1;
                    } else if b < c {
                        lo = m + 1;
                    } else {
                        lo = m;
                        hi = m;
                        break;
                    }
                }

                if hi < lo || *byts.offset(lo as isize) as c_int != c {
                    break;
                }

                tryidx = *idxs.offset(lo as isize);
                l -= 1;
            }

            if l != 0 {
                continue;
            }

            // The whole character matched: keep it and go a level deeper.
            let (src, taken) = if round == 1 {
                (fword.add(level.fwordidx) as *const c_char, flen)
            } else {
                (uword.as_ptr().add(level.uwordidx), ulen)
            };
            ptr::copy_nonoverlapping(src, kword.add(level.kwordlen), taken);

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
        *kword = NUL as c_char;
    }
}
