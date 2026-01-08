//! Spell suggestion algorithms for Neovim
//!
//! This module provides scoring algorithms used to generate and rank
//! spelling suggestions, including edit distance (Levenshtein) and
//! sound-alike scoring.

#![allow(clippy::option_if_let_else)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

use crate::SlangHandle;

// =============================================================================
// Score Constants
// =============================================================================

/// Score for various edit operations
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoreValues {
    pub split: c_int,
    pub split_no: c_int,
    pub icase: c_int,
    pub region: c_int,
    pub rare: c_int,
    pub swap: c_int,
    pub swap3: c_int,
    pub rep: c_int,
    pub subst: c_int,
    pub similar: c_int,
    pub subcomp: c_int,
    pub del: c_int,
    pub deldup: c_int,
    pub delcomp: c_int,
    pub ins: c_int,
    pub insdup: c_int,
    pub inscomp: c_int,
    pub nonword: c_int,
}

impl Default for ScoreValues {
    fn default() -> Self {
        SCORES
    }
}

/// Default score values matching C code
pub const SCORES: ScoreValues = ScoreValues {
    split: 149,
    split_no: 249,
    icase: 52,
    region: 200,
    rare: 180,
    swap: 75,
    swap3: 110,
    rep: 65,
    subst: 93,
    similar: 33,
    subcomp: 33,
    del: 94,
    deldup: 66,
    delcomp: 28,
    ins: 96,
    insdup: 67,
    inscomp: 30,
    nonword: 103,
};

/// Maximum score value (accept any score)
pub const SCORE_MAXMAX: c_int = 999_999;

/// Score limit for spell_edit_score_limit()
pub const SCORE_LIMITMAX: c_int = 350;

/// Minimum edit score for quick checks
pub const SCORE_EDIT_MIN: c_int = SCORES.similar;

/// Maximum word length
pub const MAXWLEN: usize = 254;

/// Maximum number of suggestions to generate
pub const MAXSUG: usize = 25;

/// Big score for compound penalty
pub const SCORE_BIG: c_int = 3 * SCORES.ins;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // Character classification from spellfile
    fn nvim_slang_has_map(slang: SlangHandle) -> bool;

    // Similar character check - uses slang's MAP data
    fn nvim_similar_chars(slang: SlangHandle, c1: c_int, c2: c_int) -> bool;

    // UTF-8 to character conversion
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_ptr2len(p: *const c_char) -> c_int;

    // Case folding for spell checking
    fn spell_tofold(c: c_int) -> c_int;
}

// =============================================================================
// Edit Distance Scoring
// =============================================================================

/// Computes the edit distance score between two words.
///
/// The score is based on the number of edits (deletes, inserts, substitutes, swaps)
/// needed to transform `badword` into `goodword`. Lower scores are better.
///
/// Uses the Du and Chang (1992) algorithm as implemented in Aspell.
///
/// # Safety
///
/// Both `badword` and `goodword` must be valid null-terminated UTF-8 strings.
/// `slang` may be null if similar character checking is not needed.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_edit_score(
    slang: SlangHandle,
    badword: *const c_char,
    goodword: *const c_char,
) -> c_int {
    if badword.is_null() || goodword.is_null() {
        return SCORE_MAXMAX;
    }

    // Convert multi-byte strings to code point arrays
    let mut wbadword = [0i32; MAXWLEN];
    let mut wgoodword = [0i32; MAXWLEN];

    let badlen = utf8_to_codepoints(badword, &mut wbadword);
    let goodlen = utf8_to_codepoints(goodword, &mut wgoodword);

    // Add terminating NUL equivalent
    let badlen = badlen + 1;
    let goodlen = goodlen + 1;

    // Check for similar characters if slang has MAP data
    let has_map = !slang.is_null() && nvim_slang_has_map(slang);

    spell_edit_score_impl(slang, &wbadword, badlen, &wgoodword, goodlen, has_map)
}

/// Internal edit distance calculation using code point arrays.
fn spell_edit_score_impl(
    slang: SlangHandle,
    wbadword: &[i32; MAXWLEN],
    badlen: usize,
    wgoodword: &[i32; MAXWLEN],
    goodlen: usize,
    has_map: bool,
) -> c_int {
    // Allocate the dynamic programming table
    // CNT(i, j) = cnt[i + j * (badlen + 1)]
    let table_size = (badlen + 1) * (goodlen + 1);
    let mut cnt = vec![0i32; table_size];

    let cnt_idx = |i: usize, j: usize| -> usize { i + j * (badlen + 1) };

    // Initialize first row and column
    cnt[cnt_idx(0, 0)] = 0;
    for j in 1..=goodlen {
        cnt[cnt_idx(0, j)] = cnt[cnt_idx(0, j - 1)] + SCORES.ins;
    }

    for i in 1..=badlen {
        cnt[cnt_idx(i, 0)] = cnt[cnt_idx(i - 1, 0)] + SCORES.del;

        for j in 1..=goodlen {
            let bc = wbadword[i - 1];
            let gc = wgoodword[j - 1];

            if bc == gc {
                // Characters match
                cnt[cnt_idx(i, j)] = cnt[cnt_idx(i - 1, j - 1)];
            } else {
                // Characters differ
                let subst_score = if unsafe { spell_tofold(bc) == spell_tofold(gc) } {
                    // Only case difference
                    SCORES.icase
                } else if has_map && unsafe { nvim_similar_chars(slang, gc, bc) } {
                    // Similar characters according to MAP
                    SCORES.similar
                } else {
                    // Full substitution
                    SCORES.subst
                };

                cnt[cnt_idx(i, j)] = subst_score + cnt[cnt_idx(i - 1, j - 1)];

                // Check for swap
                if i > 1 && j > 1 {
                    let pbc = wbadword[i - 2];
                    let pgc = wgoodword[j - 2];
                    if bc == pgc && pbc == gc {
                        let swap_score = SCORES.swap + cnt[cnt_idx(i - 2, j - 2)];
                        cnt[cnt_idx(i, j)] = cnt[cnt_idx(i, j)].min(swap_score);
                    }
                }

                // Check deletion
                let del_score = SCORES.del + cnt[cnt_idx(i - 1, j)];
                cnt[cnt_idx(i, j)] = cnt[cnt_idx(i, j)].min(del_score);

                // Check insertion
                let ins_score = SCORES.ins + cnt[cnt_idx(i, j - 1)];
                cnt[cnt_idx(i, j)] = cnt[cnt_idx(i, j)].min(ins_score);
            }
        }
    }

    cnt[cnt_idx(badlen - 1, goodlen - 1)]
}

/// Computes the edit distance score with an early termination limit.
///
/// Returns `SCORE_MAXMAX` if the score would exceed `limit`, allowing
/// faster rejection of poor matches.
///
/// # Safety
///
/// Both `badword` and `goodword` must be valid null-terminated UTF-8 strings.
/// `slang` may be null if similar character checking is not needed.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_edit_score_limit(
    slang: SlangHandle,
    badword: *const c_char,
    goodword: *const c_char,
    limit: c_int,
) -> c_int {
    if badword.is_null() || goodword.is_null() {
        return SCORE_MAXMAX;
    }

    // Convert multi-byte strings to code point arrays
    let mut wbadword = [0i32; MAXWLEN];
    let mut wgoodword = [0i32; MAXWLEN];

    let badlen = utf8_to_codepoints(badword, &mut wbadword);
    let goodlen = utf8_to_codepoints(goodword, &mut wgoodword);

    // Add terminating NUL equivalent
    wbadword[badlen] = 0;
    wgoodword[goodlen] = 0;

    let has_map = !slang.is_null() && nvim_slang_has_map(slang);

    spell_edit_score_limit_impl(slang, &wbadword, &wgoodword, limit, has_map)
}

/// Stack entry for the limited edit distance algorithm
#[derive(Clone, Copy, Default)]
struct LimitScore {
    badi: usize,
    goodi: usize,
    score: c_int,
}

/// Internal limited edit distance calculation.
///
/// Uses a stack-based approach from Aspell's leditdist.cpp.
fn spell_edit_score_limit_impl(
    slang: SlangHandle,
    wbadword: &[i32; MAXWLEN],
    wgoodword: &[i32; MAXWLEN],
    limit: c_int,
    has_map: bool,
) -> c_int {
    let mut stack = [LimitScore::default(); 10]; // Allow for over 3*2 edits
    let mut stackidx = 0usize;

    let mut bi = 0usize;
    let mut gi = 0usize;
    let mut score = 0;
    let mut minscore = limit + 1;

    loop {
        // Skip over equal characters
        loop {
            let bc = wbadword[bi];
            let gc = wgoodword[gi];

            if bc != gc {
                break;
            }
            if bc == 0 {
                // Both words end
                if score < minscore {
                    minscore = score;
                }
                // Pop next alternative
                if stackidx == 0 {
                    return if minscore > limit {
                        SCORE_MAXMAX
                    } else {
                        minscore
                    };
                }
                stackidx -= 1;
                bi = stack[stackidx].badi;
                gi = stack[stackidx].goodi;
                score = stack[stackidx].score;
                continue;
            }
            bi += 1;
            gi += 1;
        }

        let bc = wbadword[bi];
        let gc = wgoodword[gi];

        if gc == 0 {
            // Goodword ends, delete remaining badword chars
            loop {
                score += SCORES.del;
                if score >= minscore {
                    break;
                }
                bi += 1;
                if wbadword[bi] == 0 {
                    minscore = score;
                    break;
                }
            }
        } else if bc == 0 {
            // Badword ends, insert remaining goodword chars
            loop {
                score += SCORES.ins;
                if score >= minscore {
                    break;
                }
                gi += 1;
                if wgoodword[gi] == 0 {
                    minscore = score;
                    break;
                }
            }
        } else {
            // Both words continue - try different edits
            // Round 0: try deleting from badword
            // Round 1: try inserting in badword
            for round in 0..=1 {
                let score_off = score + if round == 0 { SCORES.del } else { SCORES.ins };

                if score_off < minscore {
                    if score_off + SCORE_EDIT_MIN >= minscore {
                        // Near limit - rest must match exactly
                        let mut bi2 = bi + 1 - round;
                        let mut gi2 = gi + round;

                        while wgoodword[gi2] == wbadword[bi2] {
                            if wgoodword[gi2] == 0 {
                                minscore = score_off;
                                break;
                            }
                            bi2 += 1;
                            gi2 += 1;
                        }
                    } else if stackidx < stack.len() {
                        // Push alternative onto stack
                        stack[stackidx].badi = bi + 1 - round;
                        stack[stackidx].goodi = gi + round;
                        stack[stackidx].score = score_off;
                        stackidx += 1;
                    }
                }
            }

            // Try substitution or swap
            if wbadword[bi] == wgoodword[gi + 1] && wbadword[bi + 1] == wgoodword[gi] {
                // Swap is possible
                let score_off = score + SCORES.swap;
                if score_off < minscore {
                    if score_off + SCORE_EDIT_MIN >= minscore {
                        // Near limit - check if rest matches
                        let mut bi2 = bi + 2;
                        let mut gi2 = gi + 2;
                        while wgoodword[gi2] == wbadword[bi2] {
                            if wgoodword[gi2] == 0 {
                                minscore = score_off;
                                break;
                            }
                            bi2 += 1;
                            gi2 += 1;
                        }
                    } else if stackidx < stack.len() {
                        stack[stackidx].badi = bi + 2;
                        stack[stackidx].goodi = gi + 2;
                        stack[stackidx].score = score_off;
                        stackidx += 1;
                    }
                }
            }

            // Substitution
            let bc = wbadword[bi];
            let gc = wgoodword[gi];
            let subst_score = if unsafe { spell_tofold(bc) == spell_tofold(gc) } {
                SCORES.icase
            } else if has_map && unsafe { nvim_similar_chars(slang, gc, bc) } {
                SCORES.similar
            } else {
                SCORES.subst
            };

            score += subst_score;
            bi += 1;
            gi += 1;
        }

        // Check if we should pop
        if score >= minscore {
            if stackidx == 0 {
                return if minscore > limit {
                    SCORE_MAXMAX
                } else {
                    minscore
                };
            }
            stackidx -= 1;
            bi = stack[stackidx].badi;
            gi = stack[stackidx].goodi;
            score = stack[stackidx].score;
        }
    }
}

// =============================================================================
// Sound-alike Scoring
// =============================================================================

/// Computes a score for two sound-alike (phonetically folded) words.
///
/// This permits up to two edits to keep things fast. Instead of a generic
/// loop, specific cases are checked explicitly.
///
/// # Safety
///
/// Both `goodsound` and `badsound` must be valid null-terminated strings.
#[no_mangle]
pub unsafe extern "C" fn rs_soundalike_score(
    goodstart: *const c_char,
    badstart: *const c_char,
) -> c_int {
    if goodstart.is_null() || badstart.is_null() {
        return SCORE_MAXMAX;
    }

    let goodsound = std::ffi::CStr::from_ptr(goodstart).to_bytes();
    let badsound = std::ffi::CStr::from_ptr(badstart).to_bytes();

    soundalike_score_impl(goodsound, badsound)
}

/// Helper to compare slices without taking references
fn slices_equal(a: &[u8], b: &[u8]) -> bool {
    a == b
}

/// Internal sound-alike scoring implementation.
fn soundalike_score_impl(goodstart: &[u8], badstart: &[u8]) -> c_int {
    let mut goodsound = goodstart;
    let mut badsound = badstart;
    let mut score = 0;

    // Handle leading '*' (vowel indicator)
    let good_star = !goodsound.is_empty() && goodsound[0] == b'*';
    let bad_star = !badsound.is_empty() && badsound[0] == b'*';

    if (bad_star || good_star) && goodsound.first() != badsound.first() {
        let good_empty = goodsound.is_empty();
        let bad_empty = badsound.is_empty();
        let good_one = goodsound.len() == 1;
        let bad_one = badsound.len() == 1;

        if (bad_empty && good_one) || (good_empty && bad_one) {
            return SCORES.del;
        }
        if bad_empty || good_empty {
            return SCORE_MAXMAX;
        }

        let same_second = goodsound.len() > 1 && badsound.len() > 1 && goodsound[1] == badsound[1];
        let same_third = goodsound.len() > 2 && badsound.len() > 2 && goodsound[2] == badsound[2];

        if !same_second && !same_third {
            score = 2 * SCORES.del / 3;
            if bad_star {
                badsound = &badsound[1..];
            } else {
                goodsound = &goodsound[1..];
            }
        }
    }

    let goodlen = goodsound.len() as i32;
    let badlen = badsound.len() as i32;

    // Quick length check - max 2 edits possible
    let n = goodlen - badlen;
    if !(-2..=2).contains(&n) {
        return SCORE_MAXMAX;
    }

    // pl = longer string, ps = shorter string
    let (pl, ps) = if n > 0 {
        (goodsound, badsound)
    } else {
        (badsound, goodsound)
    };

    // Skip identical prefix
    let mut pli = 0usize;
    let mut psi = 0usize;
    while pli < pl.len() && psi < ps.len() && pl[pli] == ps[psi] {
        pli += 1;
        psi += 1;
    }

    match n {
        -2 | 2 => soundalike_case_two_diff(pl, ps, pli, psi, score),
        -1 | 1 => soundalike_case_one_diff(pl, ps, pli, psi, score),
        0 => soundalike_case_equal_len(pl, ps, pli, psi, score),
        _ => SCORE_MAXMAX,
    }
}

/// Handle soundalike scoring when length difference is 2
fn soundalike_case_two_diff(
    pl: &[u8],
    ps: &[u8],
    mut pli: usize,
    mut psi: usize,
    score: c_int,
) -> c_int {
    // Must delete two characters from longer string
    pli += 1; // First delete
    while pli < pl.len() && psi < ps.len() && pl[pli] == ps[psi] {
        pli += 1;
        psi += 1;
    }
    // Check if rest matches after second delete
    if pli < pl.len() && slices_equal(&pl[pli + 1..], &ps[psi..]) {
        score + SCORES.del * 2
    } else {
        SCORE_MAXMAX
    }
}

/// Handle soundalike scoring when length difference is 1
fn soundalike_case_one_diff(pl: &[u8], ps: &[u8], pli: usize, psi: usize, score: c_int) -> c_int {
    // Case 1: single delete
    let mut pl2 = pli + 1;
    let mut ps2 = psi;
    while pl2 < pl.len() && ps2 < ps.len() && pl[pl2] == ps[ps2] {
        if pl2 >= pl.len() - 1 && ps2 >= ps.len() - 1 {
            return score + SCORES.del;
        }
        pl2 += 1;
        ps2 += 1;
    }
    if pl2 >= pl.len() && ps2 >= ps.len() {
        return score + SCORES.del;
    }

    // Case 2: delete then swap
    if pl2 + 1 < pl.len()
        && ps2 + 1 < ps.len()
        && pl[pl2] == ps[ps2 + 1]
        && pl[pl2 + 1] == ps[ps2]
        && (pl2 + 2 >= pl.len() || slices_equal(&pl[pl2 + 2..], &ps[ps2 + 2..]))
    {
        return score + SCORES.del + SCORES.swap;
    }

    // Case 3: delete then substitute
    if pl2 + 1 < pl.len() && ps2 + 1 < ps.len() && slices_equal(&pl[pl2 + 1..], &ps[ps2 + 1..]) {
        return score + SCORES.del + SCORES.subst;
    }

    // Case 4: swap then delete
    if pli + 1 < pl.len() && psi + 1 < ps.len() && pl[pli] == ps[psi + 1] && pl[pli + 1] == ps[psi]
    {
        let mut pl3 = pli + 2;
        let mut ps3 = psi + 2;
        while pl3 < pl.len() && ps3 < ps.len() && pl[pl3] == ps[ps3] {
            pl3 += 1;
            ps3 += 1;
        }
        if pl3 < pl.len() && slices_equal(&pl[pl3 + 1..], &ps[ps3..]) {
            return score + SCORES.swap + SCORES.del;
        }
    }

    // Case 5: substitute then delete
    let mut pl4 = pli + 1;
    let mut ps4 = psi + 1;
    while pl4 < pl.len() && ps4 < ps.len() && pl[pl4] == ps[ps4] {
        pl4 += 1;
        ps4 += 1;
    }
    if pl4 < pl.len() && slices_equal(&pl[pl4 + 1..], &ps[ps4..]) {
        return score + SCORES.subst + SCORES.del;
    }

    SCORE_MAXMAX
}

/// Handle soundalike scoring when lengths are equal
fn soundalike_case_equal_len(pl: &[u8], ps: &[u8], pli: usize, psi: usize, score: c_int) -> c_int {
    // Case 1: already identical
    if pli >= pl.len() {
        return score;
    }

    // Case 2: swap
    if pli + 1 < pl.len() && psi + 1 < ps.len() && pl[pli] == ps[psi + 1] && pl[pli + 1] == ps[psi]
    {
        let mut pl2 = pli + 2;
        let mut ps2 = psi + 2;
        while pl2 < pl.len() && ps2 < ps.len() && pl[pl2] == ps[ps2] {
            if pl2 >= pl.len() - 1 {
                return score + SCORES.swap;
            }
            pl2 += 1;
            ps2 += 1;
        }
        if pl2 >= pl.len() && ps2 >= ps.len() {
            return score + SCORES.swap;
        }

        // Case 3: swap and swap again
        if pl2 + 1 < pl.len()
            && ps2 + 1 < ps.len()
            && pl[pl2] == ps[ps2 + 1]
            && pl[pl2 + 1] == ps[ps2]
            && slices_equal(&pl[pl2 + 2..], &ps[ps2 + 2..])
        {
            return score + SCORES.swap + SCORES.swap;
        }

        // Case 4: swap and substitute
        if pl2 + 1 < pl.len() && ps2 + 1 < ps.len() && slices_equal(&pl[pl2 + 1..], &ps[ps2 + 1..])
        {
            return score + SCORES.swap + SCORES.subst;
        }
    }

    // Case 5: substitute
    let mut pl5 = pli + 1;
    let mut ps5 = psi + 1;
    while pl5 < pl.len() && ps5 < ps.len() && pl[pl5] == ps[ps5] {
        if pl5 >= pl.len() - 1 {
            return score + SCORES.subst;
        }
        pl5 += 1;
        ps5 += 1;
    }
    if pl5 >= pl.len() && ps5 >= ps.len() {
        return score + SCORES.subst;
    }

    // Case 6: substitute and swap
    if pl5 + 1 < pl.len()
        && ps5 + 1 < ps.len()
        && pl[pl5] == ps[ps5 + 1]
        && pl[pl5 + 1] == ps[ps5]
        && slices_equal(&pl[pl5 + 2..], &ps[ps5 + 2..])
    {
        return score + SCORES.subst + SCORES.swap;
    }

    // Case 7: substitute and substitute
    if pl5 + 1 < pl.len() && ps5 + 1 < ps.len() && slices_equal(&pl[pl5 + 1..], &ps[ps5 + 1..]) {
        return score + SCORES.subst + SCORES.subst;
    }

    // Case 8: insert then delete
    let mut pl8 = pli;
    let mut ps8 = psi + 1;
    while pl8 < pl.len() && ps8 < ps.len() && pl[pl8] == ps[ps8] {
        pl8 += 1;
        ps8 += 1;
    }
    if pl8 < pl.len() && slices_equal(&pl[pl8 + 1..], &ps[ps8..]) {
        return score + SCORES.ins + SCORES.del;
    }

    // Case 9: delete then insert
    let mut pl9 = pli + 1;
    let mut ps9 = psi;
    while pl9 < pl.len() && ps9 < ps.len() && pl[pl9] == ps[ps9] {
        pl9 += 1;
        ps9 += 1;
    }
    if ps9 < ps.len() && slices_equal(&pl[pl9..], &ps[ps9 + 1..]) {
        return score + SCORES.ins + SCORES.del;
    }

    SCORE_MAXMAX
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Converts a UTF-8 string to an array of code points.
///
/// Returns the number of code points written (excluding any terminator).
///
/// # Safety
///
/// `src` must be a valid null-terminated UTF-8 string.
unsafe fn utf8_to_codepoints(src: *const c_char, dest: &mut [i32; MAXWLEN]) -> usize {
    let mut p = src;
    let mut i = 0usize;

    while *p != 0 && i < MAXWLEN - 1 {
        let c = utf_ptr2char(p);
        dest[i] = c;
        i += 1;
        let len = utf_ptr2len(p);
        p = p.add(len as usize);
    }

    i
}

/// Rescore a suggestion by combining word and sound scores.
///
/// This is used to adjust the score after finding suggestions, based on
/// the suggested word sounding like the bad word.
///
/// Formula: (3 * word_score + sound_score) / 4
///
/// Note: FFI export is in lib.rs as `rs_rescore`
#[must_use]
pub fn rescore(word_score: c_int, sound_score: c_int) -> c_int {
    (3 * word_score + sound_score) / 4
}

/// Compute the maximum word score that can achieve a given final score.
///
/// Given the maximum acceptable rescore and a known sound score, compute
/// the maximum word score that would still be acceptable.
///
/// Formula: (4 * max_score - sound_score) / 3
///
/// Note: FFI export is in lib.rs as `rs_maxscore`
#[must_use]
pub fn maxscore(max_score: c_int, sound_score: c_int) -> c_int {
    (4 * max_score - sound_score) / 3
}

/// FFI wrapper for rescore function.
#[no_mangle]
pub extern "C" fn rs_rescore_suggestion(word_score: c_int, sound_score: c_int) -> c_int {
    rescore(word_score, sound_score)
}

/// FFI wrapper for maxscore function.
#[no_mangle]
pub extern "C" fn rs_maxscore_for_suggestion(max_score: c_int, sound_score: c_int) -> c_int {
    maxscore(max_score, sound_score)
}

/// Get the SCORE_MAXMAX constant value.
#[no_mangle]
pub extern "C" fn rs_score_maxmax() -> c_int {
    SCORE_MAXMAX
}

/// Get the SCORE_BIG constant value.
#[no_mangle]
pub extern "C" fn rs_score_big_suggest() -> c_int {
    SCORE_BIG
}

/// Get the MAXSUG constant value.
#[no_mangle]
pub extern "C" fn rs_maxsug() -> usize {
    MAXSUG
}

// =============================================================================
// Suggestion List Management
// =============================================================================

/// A single spelling suggestion
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Suggested word (UTF-8, NUL-terminated in buffer)
    pub word: [u8; MAXWLEN],
    /// Length of the word
    pub word_len: usize,
    /// Length of the original (bad) word that this replaces
    pub org_len: usize,
    /// Score - lower is better
    pub score: c_int,
    /// Alternative score for tie-breaking
    pub alt_score: c_int,
    /// Whether score is based on sound-alike comparison
    pub sal_score: bool,
    /// Whether bonus has been applied to score
    pub had_bonus: bool,
}

impl Default for Suggestion {
    fn default() -> Self {
        Self {
            word: [0; MAXWLEN],
            word_len: 0,
            org_len: 0,
            score: SCORE_MAXMAX,
            alt_score: 0,
            sal_score: false,
            had_bonus: false,
        }
    }
}

impl Suggestion {
    /// Create a new suggestion with the given word and score.
    #[must_use]
    pub fn new(word: &[u8], org_len: usize, score: c_int) -> Self {
        let mut s = Self::default();
        let copy_len = word.len().min(MAXWLEN - 1);
        s.word[..copy_len].copy_from_slice(&word[..copy_len]);
        s.word_len = copy_len;
        s.org_len = org_len;
        s.score = score;
        s
    }

    /// Get the word as a byte slice (without trailing NUL)
    #[must_use]
    pub fn word_bytes(&self) -> &[u8] {
        &self.word[..self.word_len]
    }
}

/// Additional score constants for suggestion generation
pub const SCORE_FILE: c_int = 30; // suggestion from a file
pub const SCORE_MAXINIT: c_int = 350; // Initial maximum score
pub const SCORE_COMMON1: c_int = 30; // subtracted for words seen before
pub const SCORE_COMMON2: c_int = 40; // subtracted for words often seen
pub const SCORE_COMMON3: c_int = 50; // subtracted for very common words

/// Number of suggestions to keep after cleanup
pub const SUG_CLEAN_COUNT_BASE: usize = 150;

/// Compare two suggestions for sorting (lower score = better)
#[must_use]
pub fn compare_suggestions(a: &Suggestion, b: &Suggestion) -> std::cmp::Ordering {
    // Primary sort by score
    match a.score.cmp(&b.score) {
        std::cmp::Ordering::Equal => {
            // Secondary sort by alt_score
            a.alt_score.cmp(&b.alt_score)
        }
        other => other,
    }
}

/// FFI wrapper to create a new suggestion.
///
/// # Safety
/// `word` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_suggestion_new(
    word: *const u8,
    word_len: usize,
    org_len: usize,
    score: c_int,
    sug_out: *mut Suggestion,
) {
    if word.is_null() || sug_out.is_null() {
        return;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    *sug_out = Suggestion::new(word_slice, org_len, score);
}

/// FFI wrapper to compare two suggestions for sorting.
///
/// Returns:
/// - negative if a < b (a is better)
/// - 0 if a == b
/// - positive if a > b (b is better)
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_suggestion_compare(
    a: *const Suggestion,
    b: *const Suggestion,
) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }

    match compare_suggestions(&*a, &*b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Check if two suggestions have the same word.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_suggestion_same_word(
    a: *const Suggestion,
    b: *const Suggestion,
) -> bool {
    if a.is_null() || b.is_null() {
        return false;
    }

    let sug_a = &*a;
    let sug_b = &*b;

    sug_a.word_len == sug_b.word_len && sug_a.word_bytes() == sug_b.word_bytes()
}

// =============================================================================
// REP Item Application
// =============================================================================

/// Check if a REP item can be applied at a given position in the word.
///
/// # Arguments
/// * `word` - The word to check
/// * `pos` - Position in the word to start checking
/// * `rep_from` - The "from" pattern of the REP item
/// * `rep_from_len` - Length of the "from" pattern
///
/// # Returns
/// True if the REP item's "from" pattern matches at the position
#[must_use]
pub fn rep_matches_at(word: &[u8], pos: usize, rep_from: &[u8], rep_from_len: usize) -> bool {
    if pos + rep_from_len > word.len() {
        return false;
    }

    word[pos..pos + rep_from_len] == rep_from[..rep_from_len]
}

/// FFI wrapper for rep_matches_at.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_rep_matches_at(
    word: *const u8,
    word_len: usize,
    pos: usize,
    rep_from: *const u8,
    rep_from_len: usize,
) -> bool {
    if word.is_null() || rep_from.is_null() {
        return false;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    let from_slice = std::slice::from_raw_parts(rep_from, rep_from_len);

    rep_matches_at(word_slice, pos, from_slice, rep_from_len)
}

/// Apply a REP substitution to a word.
///
/// # Arguments
/// * `word` - The word to modify
/// * `pos` - Position where the "from" pattern starts
/// * `rep_from_len` - Length of the "from" pattern being replaced
/// * `rep_to` - The "to" replacement pattern
/// * `rep_to_len` - Length of the "to" pattern
/// * `output` - Buffer to write the result
///
/// # Returns
/// Length of the result, or 0 on error
pub fn apply_rep(
    word: &[u8],
    pos: usize,
    rep_from_len: usize,
    rep_to: &[u8],
    rep_to_len: usize,
    output: &mut [u8],
) -> usize {
    // Calculate output length
    let new_len = word.len() - rep_from_len + rep_to_len;
    if new_len >= output.len() {
        return 0;
    }

    // Copy prefix
    output[..pos].copy_from_slice(&word[..pos]);

    // Copy replacement
    output[pos..pos + rep_to_len].copy_from_slice(&rep_to[..rep_to_len]);

    // Copy suffix
    let suffix_start = pos + rep_from_len;
    let suffix_len = word.len() - suffix_start;
    output[pos + rep_to_len..pos + rep_to_len + suffix_len].copy_from_slice(&word[suffix_start..]);

    // NUL-terminate
    output[new_len] = 0;

    new_len
}

/// FFI wrapper for apply_rep.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_apply_rep(
    word: *const u8,
    word_len: usize,
    pos: usize,
    rep_from_len: usize,
    rep_to: *const u8,
    rep_to_len: usize,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if word.is_null() || rep_to.is_null() || output.is_null() {
        return 0;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    let to_slice = std::slice::from_raw_parts(rep_to, rep_to_len);
    let output_slice = std::slice::from_raw_parts_mut(output, output_len);

    apply_rep(
        word_slice,
        pos,
        rep_from_len,
        to_slice,
        rep_to_len,
        output_slice,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scores_default() {
        let scores = ScoreValues::default();
        assert_eq!(scores.del, 94);
        assert_eq!(scores.ins, 96);
        assert_eq!(scores.subst, 93);
        assert_eq!(scores.swap, 75);
        assert_eq!(scores.icase, 52);
        assert_eq!(scores.similar, 33);
    }

    #[test]
    fn test_rescore() {
        // (3 * 100 + 50) / 4 = 350 / 4 = 87
        assert_eq!(rescore(100, 50), 87);
        // (3 * 200 + 100) / 4 = 700 / 4 = 175
        assert_eq!(rescore(200, 100), 175);
    }

    #[test]
    fn test_maxscore() {
        // (4 * 100 - 50) / 3 = 350 / 3 = 116
        assert_eq!(maxscore(100, 50), 116);
        // (4 * 200 - 100) / 3 = 700 / 3 = 233
        assert_eq!(maxscore(200, 100), 233);
    }

    #[test]
    fn test_soundalike_identical() {
        let result = soundalike_score_impl(b"hello", b"hello");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_soundalike_one_subst() {
        // "hallo" vs "hello" - one substitution
        let result = soundalike_score_impl(b"hallo", b"hello");
        assert_eq!(result, SCORES.subst);
    }

    #[test]
    fn test_soundalike_one_swap() {
        // "ehllo" vs "hello" - swap 'e' and 'h'
        let result = soundalike_score_impl(b"ehllo", b"hello");
        assert_eq!(result, SCORES.swap);
    }

    #[test]
    fn test_soundalike_one_delete() {
        // "helo" vs "hello" - one delete
        let result = soundalike_score_impl(b"helo", b"hello");
        assert_eq!(result, SCORES.del);
    }

    #[test]
    fn test_soundalike_two_deletes() {
        // "heo" vs "hello" - two deletes
        let result = soundalike_score_impl(b"heo", b"hello");
        assert_eq!(result, SCORES.del * 2);
    }

    #[test]
    fn test_soundalike_too_different() {
        // Length difference > 2
        let result = soundalike_score_impl(b"hi", b"hello");
        assert_eq!(result, SCORE_MAXMAX);
    }

    // =========================================================================
    // Phase 7: Additional Validation Tests
    // =========================================================================

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_score_constants() {
        // Verify all score constants are positive
        assert!(SCORES.split > 0);
        assert!(SCORES.split_no > 0);
        assert!(SCORES.icase > 0);
        assert!(SCORES.region > 0);
        assert!(SCORES.rare > 0);
        assert!(SCORES.swap > 0);
        assert!(SCORES.swap3 > 0);
        assert!(SCORES.rep > 0);
        assert!(SCORES.subst > 0);
        assert!(SCORES.similar > 0);
        assert!(SCORES.del > 0);
        assert!(SCORES.ins > 0);
        assert!(SCORES.nonword > 0);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_score_relationships() {
        // swap should be cheaper than two subst
        assert!(SCORES.swap < SCORES.subst * 2);
        // duplicate operations should be cheaper than regular
        assert!(SCORES.deldup < SCORES.del);
        assert!(SCORES.insdup < SCORES.ins);
        // similar should be cheapest subst
        assert!(SCORES.similar < SCORES.subst);
    }

    #[test]
    fn test_soundalike_empty() {
        // Empty strings - both empty means no difference
        let result = soundalike_score_impl(b"", b"");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_soundalike_one_empty_too_different() {
        // One empty string, other has 3+ chars - length difference > 2
        let result = soundalike_score_impl(b"", b"abc");
        assert_eq!(result, SCORE_MAXMAX);
    }

    #[test]
    fn test_soundalike_length_diff_within_bounds() {
        // Length difference of 2 (within bounds)
        let result = soundalike_score_impl(b"ab", b"abcd");
        // After "ab" matches, need to delete "cd" - that's 2 deletes
        assert_eq!(result, SCORES.del * 2);
    }

    #[test]
    fn test_soundalike_swap_at_end() {
        // "abdc" vs "abcd" with swapped last chars
        let result = soundalike_score_impl(b"abdc", b"abcd");
        assert_eq!(result, SCORES.swap);
    }

    #[test]
    fn test_rescore_boundary() {
        // Edge cases
        assert_eq!(rescore(0, 0), 0);
        assert_eq!(rescore(1, 0), 0); // (3*1 + 0) / 4 = 0
        assert_eq!(rescore(2, 0), 1); // (3*2 + 0) / 4 = 1
    }

    #[test]
    fn test_maxscore_boundary() {
        assert_eq!(maxscore(0, 0), 0);
    }

    #[test]
    fn test_score_big_constant() {
        assert_eq!(SCORE_BIG, 3 * SCORES.ins);
    }

    #[test]
    fn test_score_limitmax() {
        assert_eq!(SCORE_LIMITMAX, 350);
    }

    #[test]
    fn test_maxsug_constant() {
        assert_eq!(MAXSUG, 25);
    }

    // =========================================================================
    // Suggestion Management Tests
    // =========================================================================

    #[test]
    fn test_suggestion_default() {
        let sug = Suggestion::default();
        assert_eq!(sug.word_len, 0);
        assert_eq!(sug.org_len, 0);
        assert_eq!(sug.score, SCORE_MAXMAX);
        assert!(!sug.sal_score);
        assert!(!sug.had_bonus);
    }

    #[test]
    fn test_suggestion_new() {
        let sug = Suggestion::new(b"hello", 5, 100);
        assert_eq!(sug.word_len, 5);
        assert_eq!(sug.org_len, 5);
        assert_eq!(sug.score, 100);
        assert_eq!(sug.word_bytes(), b"hello");
    }

    #[test]
    fn test_compare_suggestions_by_score() {
        let sug1 = Suggestion::new(b"hello", 5, 100);
        let sug2 = Suggestion::new(b"world", 5, 200);

        assert_eq!(compare_suggestions(&sug1, &sug2), std::cmp::Ordering::Less);
        assert_eq!(
            compare_suggestions(&sug2, &sug1),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_compare_suggestions_equal_score() {
        let mut sug1 = Suggestion::new(b"hello", 5, 100);
        let mut sug2 = Suggestion::new(b"world", 5, 100);

        sug1.alt_score = 10;
        sug2.alt_score = 20;

        assert_eq!(compare_suggestions(&sug1, &sug2), std::cmp::Ordering::Less);
    }

    // =========================================================================
    // REP Application Tests
    // =========================================================================

    #[test]
    fn test_rep_matches_at() {
        assert!(rep_matches_at(b"hello", 0, b"he", 2));
        assert!(rep_matches_at(b"hello", 2, b"ll", 2));
        assert!(!rep_matches_at(b"hello", 0, b"wo", 2));
        assert!(!rep_matches_at(b"hello", 4, b"lo", 2)); // out of bounds
    }

    #[test]
    fn test_apply_rep_simple() {
        let mut output = [0u8; 20];
        let len = apply_rep(b"hello", 0, 2, b"wo", 2, &mut output);
        assert_eq!(len, 5);
        assert_eq!(&output[..5], b"wollo");
    }

    #[test]
    fn test_apply_rep_middle() {
        let mut output = [0u8; 20];
        let len = apply_rep(b"hello", 2, 2, b"pp", 2, &mut output);
        assert_eq!(len, 5);
        assert_eq!(&output[..5], b"heppo");
    }

    #[test]
    fn test_apply_rep_expansion() {
        let mut output = [0u8; 20];
        // Replace "l" with "ll" (makes word longer)
        let len = apply_rep(b"helo", 2, 1, b"ll", 2, &mut output);
        assert_eq!(len, 5);
        assert_eq!(&output[..5], b"hello");
    }

    #[test]
    fn test_apply_rep_contraction() {
        let mut output = [0u8; 20];
        // Replace "ll" with "l" (makes word shorter)
        let len = apply_rep(b"hello", 2, 2, b"l", 1, &mut output);
        assert_eq!(len, 4);
        assert_eq!(&output[..4], b"helo");
    }
}
