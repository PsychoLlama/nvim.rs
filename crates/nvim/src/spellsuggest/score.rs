//! How far one word is from another.
//!
//! Every suggestion carries a score, and the lower it is the earlier the
//! word is offered. Two quite different measures produce one:
//!
//! - **Edit distance** — how many characters have to be deleted, inserted,
//!   substituted or swapped to turn the bad word into the good one.
//!   [`spell_edit_score`] is the full dynamic-programming table;
//!   [`spell_edit_score_limit`] is the same measure computed depth-first
//!   with a cutoff, which is much faster when the answer is going to be
//!   "too far" anyway.
//! - **Sound-a-like distance** — the same idea over the two words'
//!   sound-folded forms, in [`soundalike_score`]. It permits at most two
//!   edits and is written out case by case rather than as a loop, because
//!   it runs once per candidate word of the whole dictionary.
//!
//! [`stp_sal_score`] is the bridge: it sound-folds a suggestion (and, when
//! the suggestion replaces a different number of characters than the bad
//! word occupies, the matching stretch of the bad word) and scores the
//! pair.
//!
//! Two adjustments sit alongside: [`score_wordcount_adj`] discounts words
//! the dictionary marks as common, and [`similar_chars`] lets a language's
//! `MAP` lines declare two characters near-equivalent so substituting one
//! for the other costs less.
//!
//! # Reading past the terminator
//!
//! [`soundalike_score`] takes whole `MAXWLEN` buffers rather than the
//! strings inside them. That is deliberate: several of its comparisons
//! look one or two positions beyond a NUL — `ps2[1]` where `ps2[0]` is the
//! terminator, say — and the C did the same, reading whatever the caller's
//! stack buffer happened to hold there. Taking the buffer keeps those
//! reads in bounds and keeps the answer identical.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::ascii_iswhite;
use crate::charset::skiptowhite;
use crate::hashtab::{hash_find, hash_removed};
use crate::main::curwin;
use crate::mbyte::{mb_cptr2char_adv, mb_isupper, utf_char2bytes, utf_fold, utf_ptr2char};
use crate::memory::xmemcpyz;
use crate::spell::{WC_KEY_OFF, spell_casefold, spell_soundfold, spelltab};
use crate::spellsuggest::{
    MAXWLEN, SCORE_COMMON1, SCORE_COMMON2, SCORE_COMMON3, SCORE_DEL, SCORE_ICASE, SCORE_INS,
    SCORE_MAXMAX, SCORE_SIMILAR, SCORE_SUBST, SCORE_SWAP, SCORE_THRES2, SCORE_THRES3, suggest_T,
    suginfo_T,
};
use crate::types::{MB_MAXCHAR, NUL, hashitem_T, size_t, slang_T, wordcount_T};
use ::libc::{strcpy, strlen};
use core::ffi::{c_char, c_int};

/// A sound-folded word and the room around it, as the callers keep it.
pub type SoundBuf = [c_char; MAXWLEN];

/// An empty sound-fold buffer, ready to be filled by `spell_soundfold`.
pub const EMPTY_SOUND: SoundBuf = [0; MAXWLEN];

/// The character that sound-folding puts at the front of a word starting
/// with a vowel. Adding or dropping one is cheaper than a real edit.
const SOUND_VOWEL: c_char = b'*' as c_char;

/// Fold a character for comparison, mirroring the `SPELL_TOFOLD` macro:
/// the byte table covers Latin-1, everything above it goes through the
/// Unicode fold.
pub fn spell_tofold(c: c_int) -> c_int {
    if c >= 128 {
        utf_fold(c)
    } else {
        // SAFETY: `spelltab` is main-thread editor state and this reads one
        // byte out of it without keeping a reference (see `GlobalCell`).
        // Indices are 0..128 by the branch above.
        unsafe { (*spelltab.ptr()).st_fold[c as usize] as c_int }
    }
}

/// Is this character upper-case? The byte table covers Latin-1,
/// everything above it goes through Unicode.
pub fn spell_isupper(c: c_int) -> bool {
    if c >= 128 {
        mb_isupper(c)
    } else {
        // SAFETY: reads one byte of main-thread editor state without
        // keeping a reference.
        unsafe { (*spelltab.ptr()).st_isu[c as usize] }
    }
}

/// Decode a NUL-terminated word into code points, terminator included.
///
/// The edit-distance measures index characters at arbitrary positions, and
/// doing that over UTF-8 bytes would mean re-scanning from the start every
/// time.
///
/// # Safety
///
/// `word` must point at a NUL-terminated string of at most `MAXWLEN - 1`
/// characters.
unsafe fn word_chars(word: *const c_char, out: &mut [c_int; MAXWLEN]) -> usize {
    let mut len = 0;
    let mut p = word;
    // SAFETY: the caller guarantees a NUL-terminated string that fits.
    unsafe {
        while *p != 0 {
            out[len] = mb_cptr2char_adv(&raw mut p);
            len += 1;
        }
    }
    out[len] = 0;
    len + 1
}

/// Returns true if `c1` and `c2` are similar characters according to the
/// `MAP` lines in the .aff file.
pub fn similar_chars(slang: &slang_T, c1: c_int, c2: c_int) -> bool {
    let m1 = map_class(slang, c1);
    // A character with no MAP entry is similar to nothing, not even to
    // another character with no entry.
    m1 != 0 && m1 == map_class(slang, c2)
}

/// The `MAP` group a character belongs to, or 0 for none.
fn map_class(slang: &slang_T, c: c_int) -> c_int {
    if c < 256 {
        return slang.sl_map_array[c as usize];
    }
    let mut buf = [0 as c_char; MB_MAXCHAR + 1];
    // SAFETY: `buf` holds any single character plus a terminator, the hash
    // key is that NUL-terminated buffer, and the value stored with a key is
    // the string just past its terminator.
    unsafe {
        let len = utf_char2bytes(c, buf.as_mut_ptr());
        buf[len as usize] = 0;
        let hi: *mut hashitem_T = hash_find(&raw const slang.sl_map_hash, buf.as_ptr());
        let key = (*hi).hi_key;
        if key.is_null() || key == &raw const hash_removed as *mut c_char {
            0
        } else {
            utf_ptr2char(key.add(strlen(key) as usize + 1))
        }
    }
}

/// Discount the score of a word the dictionary counted as common.
///
/// A `COMMON` word, or one a `.sug` build saw often enough, is more likely
/// to be what the user meant, so it is allowed to sit closer to the top
/// than its raw edit distance says.
///
/// # Safety
///
/// `word` must be a NUL-terminated string.
pub unsafe fn score_wordcount_adj(
    slang: &slang_T,
    score: c_int,
    word: *mut c_char,
    split: bool,
) -> c_int {
    // SAFETY: the caller guarantees a NUL-terminated word; the hash table
    // stores `wordcount_T`s whose key is an inline field at `WC_KEY_OFF`.
    let count = unsafe {
        let hi = hash_find(&raw const slang.sl_wordcount, word);
        let key = (*hi).hi_key;
        if key.is_null() || key == &raw const hash_removed as *mut c_char {
            return score;
        }
        let wc = key.sub(WC_KEY_OFF) as *mut wordcount_T;
        (*wc).wc_count as c_int
    };

    let bonus = if count < SCORE_THRES2 {
        SCORE_COMMON1
    } else if count < SCORE_THRES3 {
        SCORE_COMMON2
    } else {
        SCORE_COMMON3
    };

    // Halve the bonus for a word that only makes up part of the
    // replacement: it did not have to be the common one.
    let newscore = if split {
        score - bonus / 2
    } else {
        score - bonus
    };
    newscore.max(0)
}

/// One byte of a sound-fold buffer, zero past its end.
///
/// The comparisons below deliberately look just past a terminator; the
/// clamp only matters for a buffer that is full to the last byte, which no
/// sound-folded word is.
fn at(buf: &SoundBuf, i: usize) -> c_char {
    if i < buf.len() { buf[i] } else { 0 }
}

/// Are the two buffers equal from these positions to their terminators?
fn tails_equal(a: &SoundBuf, mut ai: usize, b: &SoundBuf, mut bi: usize) -> bool {
    loop {
        let (x, y) = (at(a, ai), at(b, bi));
        if x != y {
            return false;
        }
        if x == 0 {
            return true;
        }
        ai += 1;
        bi += 1;
    }
}

/// Advance both positions while the buffers agree, stopping at a
/// terminator on either side.
fn skip_equal(a: &SoundBuf, ai: &mut usize, b: &SoundBuf, bi: &mut usize) {
    while at(a, *ai) == at(b, *bi) && at(a, *ai) != 0 {
        *ai += 1;
        *bi += 1;
    }
}

/// Advance both positions while the buffers agree, without stopping at a
/// terminator.
///
/// The callers only use this where one side is known to be strictly longer
/// than the other, so the shorter side's terminator ends the walk; the
/// length bound is belt and braces.
fn skip_equal_unchecked(a: &SoundBuf, ai: &mut usize, b: &SoundBuf, bi: &mut usize) {
    while *ai < a.len() && at(a, *ai) == at(b, *bi) {
        *ai += 1;
        *bi += 1;
    }
}

/// The length of the NUL-terminated string starting at `from`.
fn tail_len(buf: &SoundBuf, from: usize) -> usize {
    let mut i = from;
    while at(buf, i) != 0 {
        i += 1;
    }
    i - from
}

/// Compute a score for two sound-a-like words.
///
/// At most two inserts/deletes/swaps/substitutes are permitted, which is
/// what keeps this fast enough to run against every word reachable in the
/// sound-fold tree. Anything further apart scores `SCORE_MAXMAX`.
///
/// Both arguments are whole buffers rather than strings; see the module
/// docs for why.
pub fn soundalike_score(goodstart: &SoundBuf, badstart: &SoundBuf) -> c_int {
    let mut gi = 0;
    let mut bi = 0;
    let mut score = 0;

    // Adding or inserting the leading "*" (the word starts with a vowel)
    // should not count for much; vowels in the middle are not counted at
    // all by the sound folding itself.
    if (at(badstart, 0) == SOUND_VOWEL || at(goodstart, 0) == SOUND_VOWEL)
        && at(badstart, 0) != at(goodstart, 0)
    {
        if (at(badstart, 0) == 0 && at(goodstart, 1) == 0)
            || (at(goodstart, 0) == 0 && at(badstart, 1) == 0)
        {
            // Changing a word with a vowel to a word without a sound.
            return SCORE_DEL;
        }
        if at(badstart, 0) == 0 || at(goodstart, 0) == 0 {
            // More than two changes.
            return SCORE_MAXMAX;
        }
        let like_substitute = at(badstart, 1) == at(goodstart, 1)
            || (at(badstart, 1) != 0
                && at(goodstart, 1) != 0
                && at(badstart, 2) == at(goodstart, 2));
        if !like_substitute {
            score = 2 * SCORE_DEL / 3;
            if at(badstart, 0) == SOUND_VOWEL {
                bi += 1;
            } else {
                gi += 1;
            }
        }
    }

    let goodlen = tail_len(goodstart, gi) as isize;
    let badlen = tail_len(badstart, bi) as isize;

    // Too different in length to be fixed by two changes.
    let n = goodlen - badlen;
    if !(-2..=2).contains(&n) {
        return SCORE_MAXMAX;
    }

    // "pl" walks the longer word, "ps" the shorter one.
    let (pl, ps) = if n > 0 {
        (goodstart, badstart)
    } else {
        (badstart, goodstart)
    };
    let (mut li, mut si) = if n > 0 { (gi, bi) } else { (bi, gi) };

    skip_equal(pl, &mut li, ps, &mut si);

    match n {
        -2 | 2 => {
            // Two characters must be deleted from the longer word.
            li += 1; // first delete
            skip_equal_unchecked(pl, &mut li, ps, &mut si);
            // The rest must match after the second delete.
            if tails_equal(pl, li + 1, ps, si) {
                return score + SCORE_DEL * 2;
            }
        }
        -1 | 1 => {
            // At least one delete from the longer word is required.

            // 1: delete
            let mut li2 = li + 1;
            let mut si2 = si;
            loop {
                if at(pl, li2) != at(ps, si2) {
                    break;
                }
                if at(pl, li2) == 0 {
                    return score + SCORE_DEL;
                }
                li2 += 1;
                si2 += 1;
            }

            // 2: delete, then swap, then the rest must be equal
            if at(pl, li2) == at(ps, si2 + 1)
                && at(pl, li2 + 1) == at(ps, si2)
                && tails_equal(pl, li2 + 2, ps, si2 + 2)
            {
                return score + SCORE_DEL + SCORE_SWAP;
            }

            // 3: delete, then substitute, then the rest must be equal
            if tails_equal(pl, li2 + 1, ps, si2 + 1) {
                return score + SCORE_DEL + SCORE_SUBST;
            }

            // 4: swap first, then delete
            if at(pl, li) == at(ps, si + 1) && at(pl, li + 1) == at(ps, si) {
                let mut li3 = li + 2;
                let mut si3 = si + 2;
                skip_equal_unchecked(pl, &mut li3, ps, &mut si3);
                if tails_equal(pl, li3 + 1, ps, si3) {
                    return score + SCORE_SWAP + SCORE_DEL;
                }
            }

            // 5: substitute first, then delete
            let mut li4 = li + 1;
            let mut si4 = si + 1;
            skip_equal_unchecked(pl, &mut li4, ps, &mut si4);
            if tails_equal(pl, li4 + 1, ps, si4) {
                return score + SCORE_SUBST + SCORE_DEL;
            }
        }
        _ => {
            // Equal lengths, so the changes have to preserve the length: an
            // insert is only possible together with a delete.

            // 1: identical
            if at(pl, li) == 0 {
                return score;
            }

            // 2: swap
            if at(pl, li) == at(ps, si + 1) && at(pl, li + 1) == at(ps, si) {
                let mut li2 = li + 2;
                let mut si2 = si + 2;
                loop {
                    if at(pl, li2) != at(ps, si2) {
                        break;
                    }
                    if at(pl, li2) == 0 {
                        return score + SCORE_SWAP;
                    }
                    li2 += 1;
                    si2 += 1;
                }

                // 3: swap and swap again
                if at(pl, li2) == at(ps, si2 + 1)
                    && at(pl, li2 + 1) == at(ps, si2)
                    && tails_equal(pl, li2 + 2, ps, si2 + 2)
                {
                    return score + SCORE_SWAP + SCORE_SWAP;
                }

                // 4: swap and substitute
                if tails_equal(pl, li2 + 1, ps, si2 + 1) {
                    return score + SCORE_SWAP + SCORE_SUBST;
                }
            }

            // 5: substitute
            let mut li2 = li + 1;
            let mut si2 = si + 1;
            loop {
                if at(pl, li2) != at(ps, si2) {
                    break;
                }
                if at(pl, li2) == 0 {
                    return score + SCORE_SUBST;
                }
                li2 += 1;
                si2 += 1;
            }

            // 6: substitute and swap
            if at(pl, li2) == at(ps, si2 + 1)
                && at(pl, li2 + 1) == at(ps, si2)
                && tails_equal(pl, li2 + 2, ps, si2 + 2)
            {
                return score + SCORE_SUBST + SCORE_SWAP;
            }

            // 7: substitute and substitute
            if tails_equal(pl, li2 + 1, ps, si2 + 1) {
                return score + SCORE_SUBST + SCORE_SUBST;
            }

            // 8: insert then delete
            let mut li3 = li;
            let mut si3 = si + 1;
            skip_equal_unchecked(pl, &mut li3, ps, &mut si3);
            if tails_equal(pl, li3 + 1, ps, si3) {
                return score + SCORE_INS + SCORE_DEL;
            }

            // 9: delete then insert
            let mut li4 = li + 1;
            let mut si4 = si;
            skip_equal_unchecked(pl, &mut li4, ps, &mut si4);
            if tails_equal(pl, li4, ps, si4 + 1) {
                return score + SCORE_INS + SCORE_DEL;
            }
        }
    }

    SCORE_MAXMAX
}

/// Compute the edit distance to turn `badword` into `goodword`: the fewer
/// deletes, inserts, substitutes and swaps required, the lower the score.
///
/// The algorithm is described by Du and Chang, 1992; the implementation
/// follows Aspell's `editdist.cpp`.
///
/// # Safety
///
/// Both words must be NUL-terminated and shorter than `MAXWLEN`
/// characters.
pub unsafe fn spell_edit_score(
    slang: Option<&slang_T>,
    badword: *const c_char,
    goodword: *const c_char,
) -> c_int {
    let mut wbadword = [0; MAXWLEN];
    let mut wgoodword = [0; MAXWLEN];
    // SAFETY: the caller guarantees NUL-terminated words that fit.
    let (badlen, goodlen) = unsafe {
        (
            word_chars(badword, &mut wbadword),
            word_chars(goodword, &mut wgoodword),
        )
    };

    // `cnt` is a (badlen + 1) x (goodlen + 1) table addressed column-major,
    // so that a row of it is one character of the bad word against every
    // prefix of the good one.
    let stride = badlen + 1;
    let mut cnt = vec![0 as c_int; stride * (goodlen + 1)];
    let idx = |a: usize, b: usize| a + b * stride;

    cnt[idx(0, 0)] = 0;
    for j in 1..=goodlen {
        cnt[idx(0, j)] = cnt[idx(0, j - 1)] + SCORE_INS;
    }

    for i in 1..=badlen {
        cnt[idx(i, 0)] = cnt[idx(i - 1, 0)] + SCORE_DEL;
        for j in 1..=goodlen {
            let bc = wbadword[i - 1];
            let gc = wgoodword[j - 1];
            if bc == gc {
                cnt[idx(i, j)] = cnt[idx(i - 1, j - 1)];
                continue;
            }

            let mut best = substitute_cost(slang, bc, gc) + cnt[idx(i - 1, j - 1)];
            if i > 1 && j > 1 && bc == wgoodword[j - 2] && wbadword[i - 2] == gc {
                best = best.min(SCORE_SWAP + cnt[idx(i - 2, j - 2)]);
            }
            best = best.min(SCORE_DEL + cnt[idx(i - 1, j)]);
            best = best.min(SCORE_INS + cnt[idx(i, j - 1)]);
            cnt[idx(i, j)] = best;
        }
    }

    cnt[idx(badlen - 1, goodlen - 1)]
}

/// What replacing `bc` with `gc` costs: a case difference is cheap, a
/// difference the language's `MAP` lines call similar is cheaper than a
/// plain substitution.
fn substitute_cost(slang: Option<&slang_T>, bc: c_int, gc: c_int) -> c_int {
    if spell_tofold(bc) == spell_tofold(gc) {
        return SCORE_ICASE;
    }
    match slang {
        Some(slang) if slang.sl_has_map && similar_chars(slang, gc, bc) => SCORE_SIMILAR,
        _ => SCORE_SUBST,
    }
}

/// One alternative still to be tried by [`spell_edit_score_limit`].
#[derive(Clone, Copy, Default)]
struct Alternative {
    badi: usize,
    goodi: usize,
    score: c_int,
}

/// The cheapest edit there is; nothing below this can bring a score down.
const SCORE_EDIT_MIN: c_int = SCORE_SIMILAR;

/// Like [`spell_edit_score`], but stops as soon as the answer is known to
/// exceed `limit`, in which case it returns `SCORE_MAXMAX`.
///
/// Rather than filling a table this walks the two words together, taking
/// the free path while they agree and branching where they differ; the
/// branches it does not take right away go on a small stack. The idea
/// comes from Aspell's `leditdist.cpp`.
///
/// # Safety
///
/// Both words must be NUL-terminated and shorter than `MAXWLEN`
/// characters.
pub unsafe fn spell_edit_score_limit(
    slang: Option<&slang_T>,
    badword: *const c_char,
    goodword: *const c_char,
    limit: c_int,
) -> c_int {
    let mut wbadword = [0; MAXWLEN];
    let mut wgoodword = [0; MAXWLEN];
    // SAFETY: the caller guarantees NUL-terminated words that fit.
    unsafe {
        word_chars(badword, &mut wbadword);
        word_chars(goodword, &mut wgoodword);
    }

    // Room for over three times two edits, which is more than the cutoff
    // can ever leave outstanding.
    let mut stack = [Alternative::default(); 10];
    let mut stackidx = 0;
    let mut bi = 0;
    let mut gi = 0;
    let mut score = 0;
    let mut minscore = limit + 1;

    'alternatives: loop {
        // Walk the equal part; it costs nothing, so it is always the best
        // move available.
        let (bc, gc) = loop {
            let bc = wbadword[bi];
            let gc = wgoodword[gi];
            if bc != gc {
                break (bc, gc);
            }
            if bc == NUL {
                // Both words end here: this alternative is complete.
                minscore = minscore.min(score);
                if !pop(&mut stack, &mut stackidx, &mut bi, &mut gi, &mut score) {
                    break 'alternatives;
                }
                continue 'alternatives;
            }
            bi += 1;
            gi += 1;
        };

        if gc == NUL {
            // The good word ended: delete the rest of the bad word.
            loop {
                score += SCORE_DEL;
                if score >= minscore {
                    break;
                }
                bi += 1;
                if wbadword[bi] == NUL {
                    minscore = score;
                    break;
                }
            }
        } else if bc == NUL {
            // The bad word ended: insert the rest of the good word.
            loop {
                score += SCORE_INS;
                if score >= minscore {
                    break;
                }
                gi += 1;
                if wgoodword[gi] == NUL {
                    minscore = score;
                    break;
                }
            }
        } else {
            // Both words continue. Try a delete and an insert; each either
            // resolves right here (when so close to the limit that the rest
            // has to match exactly) or goes on the stack for later.
            for round in 0..=1 {
                let score_off = score + if round == 0 { SCORE_DEL } else { SCORE_INS };
                if score_off >= minscore {
                    continue;
                }
                if score_off + SCORE_EDIT_MIN >= minscore {
                    let mut bi2 = bi + 1 - round;
                    let mut gi2 = gi + round;
                    while wgoodword[gi2] == wbadword[bi2] {
                        if wgoodword[gi2] == NUL {
                            minscore = score_off;
                            break;
                        }
                        bi2 += 1;
                        gi2 += 1;
                    }
                } else {
                    stack[stackidx] = Alternative {
                        badi: bi + 1 - round,
                        goodi: gi + round,
                        score: score_off,
                    };
                    stackidx += 1;
                }
            }

            // A swap that makes the words match is always cheaper than the
            // substitution that would also fix it, so there is no need to
            // try both.
            if score + (SCORE_SWAP) < minscore && gc == wbadword[bi + 1] && bc == wgoodword[gi + 1]
            {
                gi += 2;
                bi += 2;
                score += SCORE_SWAP;
                continue 'alternatives;
            }

            // Substituting is the same as deleting a character from both
            // words at once.
            score += substitute_cost(slang, bc, gc);
            if score < minscore {
                gi += 1;
                bi += 1;
                continue 'alternatives;
            }
        }

        if !pop(&mut stack, &mut stackidx, &mut bi, &mut gi, &mut score) {
            break;
        }
    }

    // Past the limit the real score may be much higher; say so loudly, so
    // that a later bonus cannot pull it back under.
    if minscore > limit {
        SCORE_MAXMAX
    } else {
        minscore
    }
}

/// Take the next alternative off the stack; false when there is none left.
fn pop(
    stack: &[Alternative; 10],
    stackidx: &mut usize,
    bi: &mut usize,
    gi: &mut usize,
    score: &mut c_int,
) -> bool {
    if *stackidx == 0 {
        return false;
    }
    *stackidx -= 1;
    *gi = stack[*stackidx].goodi;
    *bi = stack[*stackidx].badi;
    *score = stack[*stackidx].score;
    true
}

/// The sound-a-like score of one suggestion against the bad word.
///
/// `badsound` is the sound-folded bad word. When the suggestion replaces a
/// different number of characters than the bad word occupies, the two
/// cannot be compared as they stand: whichever side is short gets the
/// missing stretch of the original line appended before folding.
///
/// # Safety
///
/// `stp` and `su` must be valid, and `su`'s bad word must still point into
/// the line it was taken from.
pub unsafe fn stp_sal_score(
    stp: &suggest_T,
    su: &suginfo_T,
    slang: *mut slang_T,
    badsound: &SoundBuf,
) -> c_int {
    let mut badsound2 = EMPTY_SOUND;
    let mut fword = EMPTY_SOUND;
    let mut goodsound = EMPTY_SOUND;
    let mut goodword = EMPTY_SOUND;

    let lendiff = su.su_badlen - stp.st_orglen;

    // SAFETY: the pointers come from the caller's suggestion list and bad
    // word; every buffer below is `MAXWLEN` and every helper is told so.
    unsafe {
        let pbad: &SoundBuf = if lendiff >= 0 {
            badsound
        } else {
            // Sound-fold the bad word with the extra characters that the
            // suggestion covers.
            spell_casefold(
                curwin.get(),
                su.su_badptr,
                stp.st_orglen,
                fword.as_mut_ptr(),
                MAXWLEN as c_int,
            );

            // Joining two words changes the sound a lot -- "t he" sounds
            // like "t h" where "the" sounds like "@" -- so drop the space,
            // unless the good word has one too.
            if ascii_iswhite(*su.su_badptr.offset(su.su_badlen as isize) as c_int)
                && *skiptowhite(stp.st_word) == NUL as c_char
            {
                let mut p = fword.as_mut_ptr();
                loop {
                    p = skiptowhite(p);
                    if *p == NUL as c_char {
                        break;
                    }
                    // Close the gap over the space in place.
                    strcpy(p, p.add(1));
                }
            }

            spell_soundfold(slang, fword.as_mut_ptr(), true, badsound2.as_mut_ptr());
            &badsound2
        };

        let pgood = if lendiff > 0 && stp.st_wordlen + lendiff < MAXWLEN as c_int {
            // Append the part of the bad word the suggestion does not
            // reach, so that what gets folded is the whole replacement.
            strcpy(goodword.as_mut_ptr(), stp.st_word);
            xmemcpyz(
                goodword.as_mut_ptr().offset(stp.st_wordlen as isize) as *mut _,
                su.su_badptr.offset((su.su_badlen - lendiff) as isize) as *const _,
                lendiff as size_t,
            );
            goodword.as_mut_ptr()
        } else {
            stp.st_word
        };

        spell_soundfold(slang, pgood, false, goodsound.as_mut_ptr());

        soundalike_score(&goodsound, pbad)
    }
}
