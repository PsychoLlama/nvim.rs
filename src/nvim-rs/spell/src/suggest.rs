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
    // UTF-8 to character conversion
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    // Case folding for spell checking
    fn spell_tofold(c: c_int) -> c_int;

    // Hashtable lookup (from nvim/hashtab.c)
    fn hash_find(ht: *const crate::HashtabRaw, key: *const c_char) -> *mut crate::HashitemRaw;

    // Magic sentinel for removed hash items
    static hash_removed: c_char;

    // Check if byte n is in string str (from nvim/spell.h)
    fn byte_in_str(str: *const u8, n: c_int) -> bool;

    // Check if compflags match a COMPOUNDRULE (from nvim/spell.c)
    fn match_compoundrule(slang: crate::SlangHandle, compflags: *const u8) -> bool;
}

// =============================================================================
// Similar Character Detection (MAP data)
// =============================================================================

/// Check if two characters belong to the same MAP group.
///
/// MAP lines in .aff files group similar characters (e.g., accented variants).
/// This function returns true if c1 and c2 are in the same group.
///
/// # Safety
/// `slang` must be a valid non-null handle with MAP data loaded.
unsafe fn hashitem_is_empty(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq((*hi).hi_key, std::ptr::addr_of!(hash_removed).cast_mut())
}

#[must_use]
#[export_name = "similar_chars"]
pub unsafe extern "C" fn similar_chars(slang: SlangHandle, c1: c_int, c2: c_int) -> bool {
    let m1 = if c1 >= 256 {
        let mut buf = [0u8; 8]; // MB_MAXCHAR + 1
        let len = utf_char2bytes(c1, buf.as_mut_ptr().cast::<c_char>()) as usize;
        buf[len] = 0;
        let hi = hash_find(slang.map_hash(), buf.as_ptr().cast::<c_char>());
        if hashitem_is_empty(hi) {
            0
        } else {
            // Value stored after the key's NUL terminator
            let key_end = (*hi).hi_key.add(libc_strlen((*hi).hi_key) + 1);
            utf_ptr2char(key_end)
        }
    } else {
        *slang.map_array().add(c1 as usize)
    };

    if m1 == 0 {
        return false;
    }

    let m2 = if c2 >= 256 {
        let mut buf = [0u8; 8];
        let len = utf_char2bytes(c2, buf.as_mut_ptr().cast::<c_char>()) as usize;
        buf[len] = 0;
        let hi = hash_find(slang.map_hash(), buf.as_ptr().cast::<c_char>());
        if hashitem_is_empty(hi) {
            0
        } else {
            let key_end = (*hi).hi_key.add(libc_strlen((*hi).hi_key) + 1);
            utf_ptr2char(key_end)
        }
    } else {
        *slang.map_array().add(c2 as usize)
    };

    m1 == m2
}

/// Get byte length of a null-terminated C string (equivalent to strlen).
/// Used in similar_chars to skip past a hash key to its value.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// Check if a slang_T has MAP data.
/// Replaces the C accessor `nvim_slang_has_map`.
///
/// # Safety
/// `slang` must be a valid non-null handle.
#[must_use]
#[export_name = "nvim_slang_has_map"]
pub unsafe extern "C" fn rs_nvim_slang_has_map(slang: SlangHandle) -> bool {
    slang.has_map()
}

/// Check if two characters are in the same MAP group.
/// Replaces the C accessor `nvim_similar_chars`.
///
/// # Safety
/// `slang` must be a valid non-null handle.
#[must_use]
#[export_name = "nvim_similar_chars"]
pub unsafe extern "C" fn rs_nvim_similar_chars(slang: SlangHandle, c1: c_int, c2: c_int) -> bool {
    similar_chars(slang, c1, c2)
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
    let has_map = !slang.is_null() && slang.has_map();

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
                } else if has_map && unsafe { similar_chars(slang, gc, bc) } {
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

    let has_map = !slang.is_null() && slang.has_map();

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
            } else if has_map && unsafe { similar_chars(slang, gc, bc) } {
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

// =============================================================================
// Trie Walk State Machine
// =============================================================================

/// State for the trie walk suggestion algorithm.
///
/// At each node in the spelling tree, these states are tried in order to
/// generate suggestions. The state machine explores the tree by trying
/// various transformations (deletions, insertions, swaps, etc.) to convert
/// the bad word into a valid dictionary word.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrieWalkState {
    /// At start of node - check for NUL bytes (goodword ends);
    /// if badword ends there is a match, otherwise try splitting word.
    #[default]
    Start = 0,
    /// Try without prefix first
    NoPrefix = 1,
    /// Undo splitting
    SplitUndo = 2,
    /// Past NUL bytes at start of the node
    EndNul = 3,
    /// Use each byte of the node
    Plain = 4,
    /// Delete a byte from the bad word
    Del = 5,
    /// Prepare for inserting bytes
    InsPrep = 6,
    /// Insert a byte in the bad word
    Ins = 7,
    /// Swap two bytes
    Swap = 8,
    /// Undo swap two characters
    Unswap = 9,
    /// Swap two characters over three
    Swap3 = 10,
    /// Undo swap two characters over three
    Unswap3 = 11,
    /// Undo rotate three characters left
    Unrot3L = 12,
    /// Undo rotate three characters right
    Unrot3R = 13,
    /// Prepare for using REP items
    RepIni = 14,
    /// Use matching REP items from the .aff file
    Rep = 15,
    /// Undo a REP item replacement
    RepUndo = 16,
    /// End of this node
    Final = 17,
}

/// Values for ts_isdiff field in TryState
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffType {
    /// No different byte (yet)
    #[default]
    None = 0,
    /// Different byte found
    Yes = 1,
    /// Inserting character
    Insert = 2,
}

/// Flags for ts_flags field in TryState
pub mod try_state_flags {
    /// Already checked that prefix is OK
    pub const TSF_PREFIXOK: u8 = 1;
    /// Tried split at this point
    pub const TSF_DIDSPLIT: u8 = 2;
    /// Did a delete, ts_delidx has index
    pub const TSF_DIDDEL: u8 = 4;
}

/// Special values for ts_prefixdepth
pub mod prefix_depth {
    /// Not using prefixes
    pub const PFD_NOPREFIX: u8 = 0xff;
    /// Walking through the prefix tree
    pub const PFD_PREFIXTREE: u8 = 0xfe;
    /// Highest value that's not special
    pub const PFD_NOTSPECIAL: u8 = 0xfd;
}

/// State at each level in the trie walk suggestion search.
///
/// This struct tracks the state of the search at each depth level of the
/// spelling trie. The search uses a stack of these states to explore
/// different transformation paths.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TryState {
    /// State at this level
    pub state: TrieWalkState,
    /// Score accumulated so far
    pub score: c_int,
    /// Index in tree array, start of node
    pub arridx: u32,
    /// Index in list of child nodes
    pub curi: i16,
    /// Index in fword[], case-folded bad word
    pub fidx: u8,
    /// ts_fidx at which bytes may be changed
    pub fidxtry: u8,
    /// Valid length of tword[]
    pub twordlen: u8,
    /// Stack depth for end of prefix or PFD_PREFIXTREE or PFD_NOPREFIX
    pub prefixdepth: u8,
    /// TSF_ flags
    pub flags: u8,
    /// Number of bytes in tword character
    pub tcharlen: u8,
    /// Current byte index in tword character
    pub tcharidx: u8,
    /// DIFF_ values
    pub isdiff: DiffType,
    /// Index in fword where badword char started
    pub fcharstart: u8,
    /// Length of word in "preword[]"
    pub prewordlen: u8,
    /// Index in "tword" after last split
    pub splitoff: u8,
    /// "ts_fidx" at word split
    pub splitfidx: u8,
    /// Number of compound words used
    pub complen: u8,
    /// Index for "compflags" where word was split
    pub compsplit: u8,
    /// su_badflags saved here
    pub save_badflags: u8,
    /// Index in fword for char that was deleted, valid when flags has TSF_DIDDEL
    pub delidx: u8,
}

impl Default for TryState {
    fn default() -> Self {
        Self {
            state: TrieWalkState::Start,
            score: 0,
            arridx: 0,
            curi: 1, // Start at 1 to skip the length byte
            fidx: 0,
            fidxtry: 0,
            twordlen: 0,
            prefixdepth: prefix_depth::PFD_NOPREFIX,
            flags: 0,
            tcharlen: 0,
            tcharidx: 0,
            isdiff: DiffType::None,
            fcharstart: 0,
            prewordlen: 0,
            splitoff: 0,
            splitfidx: 0,
            complen: 0,
            compsplit: 0,
            save_badflags: 0,
            delidx: 0,
        }
    }
}

impl TryState {
    /// Create a new TryState initialized for starting a search
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're in the prefix tree
    #[must_use]
    pub const fn in_prefix_tree(&self) -> bool {
        self.prefixdepth == prefix_depth::PFD_PREFIXTREE
    }

    /// Check if we're not using prefixes
    #[must_use]
    pub const fn no_prefix(&self) -> bool {
        self.prefixdepth == prefix_depth::PFD_NOPREFIX
    }

    /// Check if prefix is OK (flag is set)
    #[must_use]
    pub const fn prefix_ok(&self) -> bool {
        self.flags & try_state_flags::TSF_PREFIXOK != 0
    }

    /// Check if split was tried at this point
    #[must_use]
    pub const fn did_split(&self) -> bool {
        self.flags & try_state_flags::TSF_DIDSPLIT != 0
    }

    /// Check if a delete was done
    #[must_use]
    pub const fn did_del(&self) -> bool {
        self.flags & try_state_flags::TSF_DIDDEL != 0
    }
}

/// FFI wrapper to create a new TryState.
///
/// # Safety
/// `out` must be a valid pointer to a TryState.
#[no_mangle]
pub unsafe extern "C" fn rs_trystate_new(out: *mut TryState) {
    if out.is_null() {
        return;
    }
    *out = TryState::new();
}

/// FFI wrapper to get the state from a TryState.
///
/// # Safety
/// `ts` must be a valid pointer to a TryState.
#[no_mangle]
pub unsafe extern "C" fn rs_trystate_get_state(ts: *const TryState) -> c_int {
    if ts.is_null() {
        return 0;
    }
    (*ts).state as c_int
}

/// FFI wrapper to set the state in a TryState.
///
/// # Safety
/// `ts` must be a valid pointer to a TryState.
#[no_mangle]
pub unsafe extern "C" fn rs_trystate_set_state(ts: *mut TryState, state: c_int) {
    if ts.is_null() {
        return;
    }
    let state = match state {
        1 => TrieWalkState::NoPrefix,
        2 => TrieWalkState::SplitUndo,
        3 => TrieWalkState::EndNul,
        4 => TrieWalkState::Plain,
        5 => TrieWalkState::Del,
        6 => TrieWalkState::InsPrep,
        7 => TrieWalkState::Ins,
        8 => TrieWalkState::Swap,
        9 => TrieWalkState::Unswap,
        10 => TrieWalkState::Swap3,
        11 => TrieWalkState::Unswap3,
        12 => TrieWalkState::Unrot3L,
        13 => TrieWalkState::Unrot3R,
        14 => TrieWalkState::RepIni,
        15 => TrieWalkState::Rep,
        16 => TrieWalkState::RepUndo,
        17 => TrieWalkState::Final,
        // 0 and any other value defaults to Start
        _ => TrieWalkState::Start,
    };
    (*ts).state = state;
}

/// Go one level deeper in the trie walk, copying state.
///
/// This is called when we want to try a different path at the current
/// node by going deeper into the tree.
///
/// # Arguments
/// * `stack` - Pointer to the stack array
/// * `depth` - Current depth in the stack
/// * `score_add` - Score to add for this step
///
/// # Safety
/// Caller must ensure stack has at least depth+2 valid elements.
#[no_mangle]
pub unsafe extern "C" fn rs_go_deeper(stack: *mut TryState, depth: usize, score_add: c_int) {
    if stack.is_null() || depth + 1 >= MAXWLEN {
        return;
    }

    let current = &*stack.add(depth);
    let next = &mut *stack.add(depth + 1);

    // Copy relevant fields from current to next level
    next.state = TrieWalkState::Start;
    next.score = current.score + score_add;
    next.curi = 1;
    next.fidx = current.fidx;
    next.fidxtry = current.fidxtry;
    next.twordlen = current.twordlen;
    next.prefixdepth = current.prefixdepth;
    next.flags = 0;
    next.tcharlen = 0;
    next.tcharidx = 0;
    next.isdiff = current.isdiff;
    next.fcharstart = current.fcharstart;
    next.prewordlen = current.prewordlen;
    next.splitoff = current.splitoff;
    next.splitfidx = current.splitfidx;
    next.complen = current.complen;
    next.compsplit = current.compsplit;
    next.save_badflags = current.save_badflags;
    next.delidx = current.delidx;
}

// =============================================================================
// Word Transformation Helpers
// =============================================================================

/// Swap two adjacent bytes in a word.
///
/// # Arguments
/// * `word` - The word buffer to modify
/// * `pos` - Position of the first byte to swap
///
/// # Returns
/// True if the swap was performed, false if out of bounds
#[must_use]
pub fn swap_bytes(word: &mut [u8], pos: usize) -> bool {
    if pos + 1 >= word.len() {
        return false;
    }
    word.swap(pos, pos + 1);
    true
}

/// FFI wrapper for swap_bytes.
///
/// # Safety
/// `word` must be valid for `word_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_swap_bytes(word: *mut u8, word_len: usize, pos: usize) -> bool {
    if word.is_null() || pos + 1 >= word_len {
        return false;
    }
    let slice = std::slice::from_raw_parts_mut(word, word_len);
    swap_bytes(slice, pos)
}

/// Swap two bytes over three positions (abc -> cba).
///
/// # Arguments
/// * `word` - The word buffer to modify
/// * `pos` - Position of the first byte
///
/// # Returns
/// True if the swap was performed, false if out of bounds
#[must_use]
pub fn swap3_bytes(word: &mut [u8], pos: usize) -> bool {
    if pos + 2 >= word.len() {
        return false;
    }
    word.swap(pos, pos + 2);
    true
}

/// FFI wrapper for swap3_bytes.
///
/// # Safety
/// `word` must be valid for `word_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_swap3_bytes(word: *mut u8, word_len: usize, pos: usize) -> bool {
    if word.is_null() || pos + 2 >= word_len {
        return false;
    }
    let slice = std::slice::from_raw_parts_mut(word, word_len);
    swap3_bytes(slice, pos)
}

/// Rotate three bytes left (abc -> bca).
///
/// # Arguments
/// * `word` - The word buffer to modify
/// * `pos` - Position of the first byte
///
/// # Returns
/// True if the rotation was performed, false if out of bounds
#[must_use]
pub fn rotate3_left(word: &mut [u8], pos: usize) -> bool {
    if pos + 2 >= word.len() {
        return false;
    }
    let a = word[pos];
    word[pos] = word[pos + 1];
    word[pos + 1] = word[pos + 2];
    word[pos + 2] = a;
    true
}

/// FFI wrapper for rotate3_left.
///
/// # Safety
/// `word` must be valid for `word_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_rotate3_left(word: *mut u8, word_len: usize, pos: usize) -> bool {
    if word.is_null() || pos + 2 >= word_len {
        return false;
    }
    let slice = std::slice::from_raw_parts_mut(word, word_len);
    rotate3_left(slice, pos)
}

/// Rotate three bytes right (abc -> cab).
///
/// # Arguments
/// * `word` - The word buffer to modify
/// * `pos` - Position of the first byte
///
/// # Returns
/// True if the rotation was performed, false if out of bounds
#[must_use]
pub fn rotate3_right(word: &mut [u8], pos: usize) -> bool {
    if pos + 2 >= word.len() {
        return false;
    }
    let c = word[pos + 2];
    word[pos + 2] = word[pos + 1];
    word[pos + 1] = word[pos];
    word[pos] = c;
    true
}

/// FFI wrapper for rotate3_right.
///
/// # Safety
/// `word` must be valid for `word_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_rotate3_right(word: *mut u8, word_len: usize, pos: usize) -> bool {
    if word.is_null() || pos + 2 >= word_len {
        return false;
    }
    let slice = std::slice::from_raw_parts_mut(word, word_len);
    rotate3_right(slice, pos)
}

/// Delete a byte from a word, shifting remaining bytes left.
///
/// # Arguments
/// * `word` - The word buffer to modify
/// * `word_len` - Current length of the word
/// * `pos` - Position of byte to delete
///
/// # Returns
/// New length of the word, or original length if out of bounds
#[must_use]
pub fn delete_byte(word: &mut [u8], word_len: usize, pos: usize) -> usize {
    if pos >= word_len {
        return word_len;
    }
    // Shift bytes left
    word.copy_within(pos + 1..word_len, pos);
    word_len - 1
}

/// FFI wrapper for delete_byte.
///
/// # Safety
/// `word` must be valid for at least `word_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_delete_byte(word: *mut u8, word_len: usize, pos: usize) -> usize {
    if word.is_null() || pos >= word_len {
        return word_len;
    }
    let slice = std::slice::from_raw_parts_mut(word, word_len);
    delete_byte(slice, word_len, pos)
}

/// Insert a byte into a word, shifting remaining bytes right.
///
/// # Arguments
/// * `word` - The word buffer to modify (must have room for one more byte)
/// * `word_len` - Current length of the word
/// * `pos` - Position where to insert
/// * `byte` - The byte to insert
///
/// # Returns
/// New length of the word
#[must_use]
pub fn insert_byte(word: &mut [u8], word_len: usize, pos: usize, byte: u8) -> usize {
    if pos > word_len || word_len + 1 >= word.len() {
        return word_len;
    }
    // Shift bytes right
    word.copy_within(pos..word_len, pos + 1);
    word[pos] = byte;
    word_len + 1
}

/// FFI wrapper for insert_byte.
///
/// # Safety
/// `word` must be valid for at least `word_len + 1` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_byte(
    word: *mut u8,
    buffer_len: usize,
    word_len: usize,
    pos: usize,
    byte: u8,
) -> usize {
    if word.is_null() || pos > word_len || word_len + 1 >= buffer_len {
        return word_len;
    }
    let slice = std::slice::from_raw_parts_mut(word, buffer_len);
    insert_byte(slice, word_len, pos, byte)
}

/// Substitute a byte in a word.
///
/// # Arguments
/// * `word` - The word buffer to modify
/// * `pos` - Position of byte to substitute
/// * `byte` - The new byte value
///
/// # Returns
/// The old byte value, or 0 if out of bounds
#[must_use]
pub fn substitute_byte(word: &mut [u8], pos: usize, byte: u8) -> u8 {
    if pos >= word.len() {
        return 0;
    }
    let old = word[pos];
    word[pos] = byte;
    old
}

/// FFI wrapper for substitute_byte.
///
/// # Safety
/// `word` must be valid for `word_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_substitute_byte(
    word: *mut u8,
    word_len: usize,
    pos: usize,
    byte: u8,
) -> u8 {
    if word.is_null() || pos >= word_len {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(word, word_len);
    substitute_byte(slice, pos, byte)
}

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
// Trie Tree Traversal Functions
// =============================================================================

/// Find a sound-folded word in the soundfold tree.
///
/// Returns the word number for the given soundfolded word, or -1 if not found.
/// Mirrors C `soundfold_find()`.
///
/// # Safety
/// `slang` must be valid and have a loaded soundfold tree (sl_sbyts/sl_sidxs).
/// `word` must be a valid null-terminated byte string.
#[must_use]
#[export_name = "soundfold_find"]
pub unsafe extern "C" fn rs_soundfold_find(
    slang: crate::SlangHandle,
    word: *const c_char,
) -> c_int {
    use crate::IdxT;

    let byts = slang.sbyts();
    let idxs = slang.sidxs();

    if byts.is_null() || idxs.is_null() || word.is_null() {
        return -1;
    }

    let word_bytes = word.cast::<u8>();
    let mut arridx: IdxT = 0;
    let mut wlen: usize = 0;
    let mut wordnr: c_int = 0;

    loop {
        // First byte is the count of possible bytes at this node.
        let len = c_int::from(*byts.add(arridx as usize));
        arridx += 1;

        let c = *word_bytes.add(wlen);

        // If the first possible byte is NUL, the word can end here.
        if *byts.add(arridx as usize) == 0 {
            if c == 0 {
                // Word ends here: found it.
                break;
            }
            // Skip over NUL entries (there can be several).
            let mut remaining = len;
            while remaining > 0 && *byts.add(arridx as usize) == 0 {
                arridx += 1;
                remaining -= 1;
            }
            if remaining == 0 {
                return -1; // no children, word should have ended here
            }
            wordnr += 1;
        }

        // If the word ended but the tree hasn't: not found.
        if c == 0 {
            return -1;
        }

        // Binary search among accepted bytes (Tab treated as Space).
        let search_byte = if c == b'\t' { b' ' } else { c };

        let mut lo = arridx;
        let hi = arridx + len - 1;
        // Linear scan: find first byte >= search_byte
        while lo < hi && *byts.add(lo as usize) < search_byte {
            // Count the words in this child's subtree.
            wordnr += *idxs.add(*idxs.add(lo as usize) as usize);
            lo += 1;
        }
        if *byts.add(lo as usize) != search_byte {
            return -1; // byte not found
        }

        // Continue to the child node.
        arridx = *idxs.add(lo as usize);
        wlen += 1;

        // One space in the good word may stand for several in the bad word.
        if c == b' ' || c == b'\t' {
            while *word_bytes.add(wlen) == b' ' || *word_bytes.add(wlen) == b'\t' {
                wlen += 1;
            }
        }
    }

    wordnr
}

/// Find the keep-case version of a case-folded word in the keep-case trie.
///
/// Writes the result into `kword` (null-terminated). On failure, writes an
/// empty string. Mirrors C `find_keepcap_word()`.
///
/// # Safety
/// `slang` must be valid with a loaded keep-case tree.
/// `fword` and `kword` must be valid buffers (kword at least MAXWLEN bytes).
#[allow(clippy::many_single_char_names)]
#[export_name = "find_keepcap_word"]
pub unsafe extern "C" fn rs_find_keepcap_word(
    slang: crate::SlangHandle,
    fword: *mut c_char,
    kword: *mut c_char,
) {
    use crate::IdxT;

    let byts = slang.kbyts();
    let idxs = slang.kidxs();

    if byts.is_null() {
        *kword = 0;
        return;
    }

    // Make an all-caps version of fword.
    let mut uword = [0u8; MAXWLEN];
    crate::rs_allcap_copy(fword, uword.as_mut_ptr().cast::<c_char>());

    // State arrays for DFS traversal.
    let mut arridx = [0 as IdxT; MAXWLEN];
    let mut round = [0i32; MAXWLEN];
    let mut fwordidx = [0usize; MAXWLEN];
    let mut uwordidx = [0usize; MAXWLEN];
    let mut kwordlen = [0usize; MAXWLEN];

    let mut depth: isize = 0;
    arridx[0] = 0;
    round[0] = 0;
    fwordidx[0] = 0;
    uwordidx[0] = 0;
    kwordlen[0] = 0;

    while depth >= 0 {
        let d = depth as usize;
        let fc = *fword.add(fwordidx[d]) as u8;

        if fc == 0 {
            // At end of fword: check if tree allows word end here.
            if *byts.add(arridx[d] as usize + 1) == 0 {
                *kword.add(kwordlen[d]) = 0;
                return;
            }
            depth -= 1;
        } else {
            round[d] += 1;
            if round[d] > 2 {
                // Tried both fold-case and upper-case: backtrack.
                depth -= 1;
            } else {
                // round 1 = fold-case, round 2 = upper-case
                let flen = utf_ptr2len(fword.add(fwordidx[d]).cast::<c_char>()) as usize;
                let ulen = utf_ptr2len(uword.as_ptr().add(uwordidx[d]).cast::<c_char>()) as usize;
                let (p, l): (*const c_char, usize) = if round[d] == 1 {
                    (fword.add(fwordidx[d]).cast::<c_char>(), flen)
                } else {
                    (uword.as_ptr().add(uwordidx[d]).cast::<c_char>(), ulen)
                };

                // Try to match the bytes of 'p' (length 'l') in the trie.
                let mut tryidx = arridx[d];
                let mut matched = true;
                let mut remaining = l;
                let mut pb: *const c_char = p;
                while remaining > 0 {
                    let node_len = *byts.add(tryidx as usize) as usize;
                    tryidx += 1;
                    let c = *pb.cast::<u8>();
                    pb = pb.add(1);

                    // Binary search for c.
                    let lo = tryidx;
                    let hi = tryidx + node_len as IdxT - 1;
                    let mut lo_idx = lo;
                    let mut hi_idx = hi;
                    while lo_idx < hi_idx {
                        let mid = i32::midpoint(lo_idx, hi_idx);
                        match (*byts.add(mid as usize)).cmp(&c) {
                            std::cmp::Ordering::Greater => hi_idx = mid - 1,
                            std::cmp::Ordering::Less => lo_idx = mid + 1,
                            std::cmp::Ordering::Equal => {
                                lo_idx = mid;
                                hi_idx = mid;
                                break;
                            }
                        }
                    }

                    if hi_idx < lo_idx || *byts.add(lo_idx as usize) != c {
                        matched = false;
                        break;
                    }

                    tryidx = *idxs.add(lo_idx as usize);
                    remaining -= 1;
                }

                if matched {
                    // Copy the matched bytes to kword and go deeper.
                    let copy_len = if round[d] == 1 { flen } else { ulen };
                    let src: *const u8 = p.cast::<u8>();
                    std::ptr::copy_nonoverlapping(
                        src,
                        kword.add(kwordlen[d]).cast::<u8>(),
                        copy_len,
                    );

                    let nd = d + 1;
                    kwordlen[nd] = kwordlen[d] + copy_len;
                    fwordidx[nd] = fwordidx[d] + flen;
                    uwordidx[nd] = uwordidx[d] + ulen;
                    arridx[nd] = tryidx;
                    round[nd] = 0;
                    depth += 1;
                }
            }
        }
    }

    // Not found.
    *kword = 0;
}

// =============================================================================
// Phase 5: go_deeper
// =============================================================================

/// Advance one level in the trie walk stack.
///
/// Copies the current frame to depth+1, then sets state=START, curi=1, flags=0,
/// and adjusts score by score_add.  Mirrors C `go_deeper()`.
///
/// # Safety
/// `stack` must point to at least `depth + 2` valid TryState entries.
#[export_name = "go_deeper"]
pub unsafe extern "C" fn rs_go_deeper_export(stack: *mut TryState, depth: c_int, score_add: c_int) {
    let depth = depth as usize;
    // Full struct copy (mirrors `stack[depth+1] = stack[depth]` in C)
    std::ptr::copy_nonoverlapping(stack.add(depth), stack.add(depth + 1), 1);
    let next = &mut *stack.add(depth + 1);
    next.state = TrieWalkState::Start;
    next.score += score_add;
    next.curi = 1;
    next.flags = 0;
}

// =============================================================================
// Phase 4: can_be_compound, score_wordcount_adj, badword_captype
// =============================================================================

/// Check if compound flags collected so far could form a valid compound word.
///
/// Returns true if adding `flag` to the compound flags is allowed.
///
/// # Safety
/// `sp` must be a valid non-null pointer to a trystate_T (C ABI compatible with TryState).
/// `slang` must be a valid non-null SlangHandle.
/// `compflags` must point to a buffer of at least MAXWLEN bytes.
#[must_use]
#[export_name = "can_be_compound"]
pub unsafe extern "C" fn rs_can_be_compound(
    sp: *const TryState,
    slang: crate::SlangHandle,
    compflags: *mut u8,
    flag: c_int,
) -> bool {
    let complen = (*sp).complen as usize;
    let compsplit = (*sp).compsplit as usize;

    // If flag doesn't appear in the relevant flags string, it can't compound.
    let flags_ptr = if complen == compsplit {
        slang.compstartflags()
    } else {
        slang.compallflags()
    };
    if !byte_in_str(flags_ptr, flag) {
        return false;
    }

    // If there are no wildcards, check that collected flags match a COMPOUNDRULE.
    let comprules = slang.comprules();
    if !comprules.is_null() && complen > compsplit {
        *compflags.add(complen) = flag as u8;
        *compflags.add(complen + 1) = 0;
        let result = match_compoundrule(slang, compflags.add(compsplit));
        *compflags.add(complen) = 0;
        return result;
    }

    true
}

/// Adjust the score of common words.
///
/// Subtracts a bonus from `score` when `word` appears frequently in `sl_wordcount`.
///
/// # Safety
/// `slang` must be a valid non-null SlangHandle.
/// `word` must be a valid null-terminated string.
#[must_use]
#[export_name = "score_wordcount_adj"]
pub unsafe extern "C" fn rs_score_wordcount_adj(
    slang: crate::SlangHandle,
    score: c_int,
    word: *const c_char,
    split: bool,
) -> c_int {
    let wordcount_ht = slang.wordcount();
    let hi = hash_find(wordcount_ht, word);
    if hashitem_is_empty(hi) {
        return score;
    }

    // HI2WC: wc_count is 2 bytes before wc_word (the hash key)
    // wordcount_T = { wc_count: u16, wc_word: [] }
    // WC_KEY_OFF = offsetof(wordcount_T, wc_word) = 2
    // Read as two bytes to avoid alignment issues (little-endian u16)
    let base = (*hi).hi_key.cast::<u8>().sub(2);
    let wc_count = u16::from_le_bytes([*base, *base.add(1)]);

    let bonus = if c_int::from(wc_count) < crate::SCORE_THRES2 {
        crate::SCORE_COMMON1
    } else if c_int::from(wc_count) < crate::SCORE_THRES3 {
        crate::SCORE_COMMON2
    } else {
        crate::SCORE_COMMON3
    };

    let newscore = if split {
        score - bonus / 2
    } else {
        score - bonus
    };
    if newscore < 0 {
        0
    } else {
        newscore
    }
}

/// Like captype() but for KEEPCAP words also adds ONECAP/ALLCAP/MIXCAP flags.
///
/// # Safety
/// `word` and `end` must be valid pointers with `word <= end`.
#[must_use]
#[export_name = "badword_captype"]
#[allow(clippy::cast_sign_loss)] // char counts are always non-negative
pub unsafe extern "C" fn rs_badword_captype(word: *const c_char, end: *const c_char) -> c_int {
    let flags = crate::rs_captype(word, end);

    if flags & crate::WF_KEEPCAP_FLAG == 0 {
        return flags;
    }

    // Count upper and lower case letters.
    let mut upper_count: c_int = 0;
    let mut lower_count: c_int = 0;
    let mut first_is_upper = false;
    let mut p = word;
    while p < end {
        let c = utf_ptr2char(p);
        if crate::spell_isupper(c) {
            upper_count += 1;
            if std::ptr::eq(p, word) {
                first_is_upper = true;
            }
        } else {
            lower_count += 1;
        }
        // Advance by char width (MB_PTR_ADV)
        let l = utf_ptr2len(p).max(1) as usize;
        p = p.add(l);
    }

    let mut result = flags;
    if upper_count > lower_count && upper_count > 2 {
        result |= crate::WF_ALLCAP_FLAG;
    } else if first_is_upper {
        result |= crate::WF_ONECAP_FLAG;
    }
    // WF_MIXCAP = 0x20 (mix of upper and lower case, defined in spellsuggest.c)
    if upper_count >= 2 && lower_count >= 2 {
        result |= 0x20;
    }

    result
}

// =============================================================================
// Phase 149: Suggestion Generation - additional FFI exports
// =============================================================================

/// Get the default DEL score.
#[no_mangle]
pub extern "C" fn rs_score_default_del() -> c_int {
    SCORES.del
}

/// Get the default INS score.
#[no_mangle]
pub extern "C" fn rs_score_default_ins() -> c_int {
    SCORES.ins
}

/// Get the default SUBST score.
#[no_mangle]
pub extern "C" fn rs_score_default_subst() -> c_int {
    SCORES.subst
}

/// Get the default SWAP score.
#[no_mangle]
pub extern "C" fn rs_score_default_swap() -> c_int {
    SCORES.swap
}

/// Get the default ICASE score.
#[no_mangle]
pub extern "C" fn rs_score_default_icase() -> c_int {
    SCORES.icase
}

/// Get the default SIMILAR score.
#[no_mangle]
pub extern "C" fn rs_score_default_similar() -> c_int {
    SCORES.similar
}

/// State values for suggestion trie walk.
pub mod suggest_state {
    use std::ffi::c_int;

    /// Start state.
    pub const STATE_START: c_int = 0;
    /// Initial state (processing first char).
    pub const STATE_NOPREFIX: c_int = 1;
    /// Split word state.
    pub const STATE_SPLITUNDO: c_int = 2;
    /// End of word state.
    pub const STATE_ENDNUL: c_int = 3;
    /// Plain match state.
    pub const STATE_PLAIN: c_int = 4;
    /// Delete state.
    pub const STATE_DEL: c_int = 5;
    /// Insert state.
    pub const STATE_INS: c_int = 6;
    /// Swap state.
    pub const STATE_SWAP: c_int = 7;
    /// Unswap state.
    pub const STATE_UNSWAP: c_int = 8;
    /// Unrotate 3L state.
    pub const STATE_UNROT3L: c_int = 9;
    /// Unrotate 3R state.
    pub const STATE_UNROT3R: c_int = 10;
    /// REP initial state.
    pub const STATE_REP_INI: c_int = 11;
    /// REP undo state.
    pub const STATE_REP_UNDO: c_int = 12;
    /// REP state.
    pub const STATE_REP: c_int = 13;
    /// Final state.
    pub const STATE_FINAL: c_int = 14;
}

/// Get suggestion state START.
#[no_mangle]
pub extern "C" fn rs_suggest_state_start() -> c_int {
    suggest_state::STATE_START
}

/// Get suggestion state NOPREFIX.
#[no_mangle]
pub extern "C" fn rs_suggest_state_noprefix() -> c_int {
    suggest_state::STATE_NOPREFIX
}

/// Get suggestion state SPLITUNDO.
#[no_mangle]
pub extern "C" fn rs_suggest_state_splitundo() -> c_int {
    suggest_state::STATE_SPLITUNDO
}

/// Get suggestion state ENDNUL.
#[no_mangle]
pub extern "C" fn rs_suggest_state_endnul() -> c_int {
    suggest_state::STATE_ENDNUL
}

/// Get suggestion state PLAIN.
#[no_mangle]
pub extern "C" fn rs_suggest_state_plain() -> c_int {
    suggest_state::STATE_PLAIN
}

/// Get suggestion state DEL.
#[no_mangle]
pub extern "C" fn rs_suggest_state_del() -> c_int {
    suggest_state::STATE_DEL
}

/// Get suggestion state INS.
#[no_mangle]
pub extern "C" fn rs_suggest_state_ins() -> c_int {
    suggest_state::STATE_INS
}

/// Get suggestion state SWAP.
#[no_mangle]
pub extern "C" fn rs_suggest_state_swap() -> c_int {
    suggest_state::STATE_SWAP
}

/// Get suggestion state FINAL.
#[no_mangle]
pub extern "C" fn rs_suggest_state_final() -> c_int {
    suggest_state::STATE_FINAL
}

/// Suggestion result structure.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SuggestionResult {
    /// Score for this suggestion (lower is better).
    pub score: c_int,
    /// Length of the suggestion word.
    pub word_len: c_int,
    /// Whether this is a split suggestion.
    pub is_split: bool,
    /// Whether the suggestion is from a sound-alike match.
    pub is_soundalike: bool,
}

impl SuggestionResult {
    /// Create a new suggestion result.
    #[must_use]
    pub const fn new(score: c_int, word_len: c_int) -> Self {
        Self {
            score,
            word_len,
            is_split: false,
            is_soundalike: false,
        }
    }

    /// Create a split suggestion result.
    #[must_use]
    pub const fn split(score: c_int, word_len: c_int) -> Self {
        Self {
            score,
            word_len,
            is_split: true,
            is_soundalike: false,
        }
    }

    /// Create a soundalike suggestion result.
    #[must_use]
    pub const fn soundalike(score: c_int, word_len: c_int) -> Self {
        Self {
            score,
            word_len,
            is_split: false,
            is_soundalike: true,
        }
    }
}

/// Create a new suggestion result.
#[no_mangle]
pub extern "C" fn rs_suggestion_result_new(score: c_int, word_len: c_int) -> SuggestionResult {
    SuggestionResult::new(score, word_len)
}

/// Create a split suggestion result.
#[no_mangle]
pub extern "C" fn rs_suggestion_result_split(score: c_int, word_len: c_int) -> SuggestionResult {
    SuggestionResult::split(score, word_len)
}

/// Create a soundalike suggestion result.
#[no_mangle]
pub extern "C" fn rs_suggestion_result_soundalike(
    score: c_int,
    word_len: c_int,
) -> SuggestionResult {
    SuggestionResult::soundalike(score, word_len)
}

/// Check if suggestion result is a split.
///
/// # Safety
/// `result` must be a valid pointer to a SuggestionResult.
#[no_mangle]
pub unsafe extern "C" fn rs_suggestion_result_is_split(result: *const SuggestionResult) -> bool {
    if result.is_null() {
        return false;
    }
    (*result).is_split
}

/// Check if suggestion result is soundalike.
///
/// # Safety
/// `result` must be a valid pointer to a SuggestionResult.
#[no_mangle]
pub unsafe extern "C" fn rs_suggestion_result_is_soundalike(
    result: *const SuggestionResult,
) -> bool {
    if result.is_null() {
        return false;
    }
    (*result).is_soundalike
}

/// Get score from suggestion result.
///
/// # Safety
/// `result` must be a valid pointer to a SuggestionResult.
#[no_mangle]
pub unsafe extern "C" fn rs_suggestion_result_get_score(result: *const SuggestionResult) -> c_int {
    if result.is_null() {
        return SCORE_MAXMAX;
    }
    (*result).score
}

/// Maximum number of suggestions to generate.
pub const MAX_SUGGESTIONS: usize = 25;

/// Get max suggestions constant.
#[no_mangle]
pub extern "C" fn rs_max_suggestions() -> usize {
    MAX_SUGGESTIONS
}

/// Minimum score improvement to keep a suggestion.
pub const MIN_SCORE_IMPROVEMENT: c_int = 10;

/// Get minimum score improvement constant.
#[no_mangle]
pub extern "C" fn rs_min_score_improvement() -> c_int {
    MIN_SCORE_IMPROVEMENT
}

/// Check if a score is good enough for a suggestion.
#[no_mangle]
pub extern "C" fn rs_score_is_good_suggestion(score: c_int, best_score: c_int) -> bool {
    // A suggestion is good if its score is within 3x the best score
    score <= best_score * 3
}

/// Calculate combined score from word and soundalike scores.
#[no_mangle]
pub extern "C" fn rs_suggest_combine_scores(word_score: c_int, sound_score: c_int) -> c_int {
    // Weight the word score more heavily
    (word_score * 3 + sound_score) / 4
}

// =============================================================================
// Phase 322: Spell Suggest Option Parsing
// =============================================================================

/// Spellsuggest flags
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpsFlags {
    /// Which algorithm to use (SPS_BEST, SPS_FAST, SPS_DOUBLE)
    pub flags: c_int,
    /// Maximum number of suggestions
    pub limit: c_int,
}

/// Use best suggestions (default)
pub const SPS_BEST: c_int = 1;
/// Use fast suggestions (less accurate but quicker)
pub const SPS_FAST: c_int = 2;
/// Double-check suggestions
pub const SPS_DOUBLE: c_int = 4;

/// Default suggestion limit
pub const SPS_LIMIT_DEFAULT: c_int = 9999;

/// FFI export for SPS_BEST constant
#[no_mangle]
pub extern "C" fn rs_sps_best() -> c_int {
    SPS_BEST
}

/// FFI export for SPS_FAST constant
#[no_mangle]
pub extern "C" fn rs_sps_fast() -> c_int {
    SPS_FAST
}

/// FFI export for SPS_DOUBLE constant
#[no_mangle]
pub extern "C" fn rs_sps_double() -> c_int {
    SPS_DOUBLE
}

/// FFI export for default limit
#[no_mangle]
pub extern "C" fn rs_sps_limit_default() -> c_int {
    SPS_LIMIT_DEFAULT
}

/// Parse a single spellsuggest option value.
///
/// Returns the SPS_* flag value, or -1 if invalid.
/// For numeric values, returns 0 and the parsed number.
///
/// # Arguments
/// * `value` - The option value to parse (null-terminated)
/// * `limit_out` - Output for numeric limit (set if value is a number)
///
/// Returns the SPS_* flag value, or:
/// - 0 for numeric values (limit stored in limit_out)
/// - 0 for expr:/file:/timeout: prefixed values (valid but no flag)
/// - -1 for invalid values
pub fn parse_sps_value(value: &[u8], limit_out: &mut c_int) -> c_int {
    if value.is_empty() {
        return -1;
    }

    // Check if it starts with a digit
    if value[0].is_ascii_digit() {
        // Parse numeric limit
        let mut num = 0i32;
        let mut idx = 0;
        while idx < value.len() && value[idx].is_ascii_digit() {
            num = num
                .saturating_mul(10)
                .saturating_add(i32::from(value[idx] - b'0'));
            idx += 1;
        }
        // Check that rest is empty (or NUL)
        if idx < value.len() && value[idx] != 0 && !value[idx].is_ascii_digit() {
            return -1;
        }
        *limit_out = num;
        return 0;
    }

    // Find actual length (stop at NUL)
    let len = value.iter().position(|&c| c == 0).unwrap_or(value.len());
    let val = &value[..len];

    // Check for known keywords
    if val == b"best" {
        return SPS_BEST;
    }
    if val == b"fast" {
        return SPS_FAST;
    }
    if val == b"double" {
        return SPS_DOUBLE;
    }

    // Check for valid prefixes
    if val.len() >= 5 {
        if val.starts_with(b"expr:") {
            return 0; // Valid but no flag
        }
        if val.starts_with(b"file:") {
            return 0; // Valid but no flag
        }
    }

    if val.len() >= 8 && val.starts_with(b"timeout:") {
        // Check that timeout value is valid (digit or - followed by digit)
        let rest = &val[8..];
        if !rest.is_empty() {
            if rest[0].is_ascii_digit() {
                return 0; // Valid
            }
            if rest[0] == b'-' && rest.len() > 1 && rest[1].is_ascii_digit() {
                return 0; // Valid negative timeout
            }
        }
        return -1; // Invalid timeout format
    }

    -1 // Unknown value
}

/// Parse the 'spellsuggest' option value.
///
/// # Arguments
/// * `option` - The full option string (null-terminated, comma-separated)
///
/// # Returns
/// `SpsFlags` with the parsed flags and limit.
///
/// # Errors
/// Returns `Err(())` if the option contains invalid values or conflicting flags.
#[allow(clippy::result_unit_err)]
pub fn parse_spellsuggest(option: &[u8]) -> Result<SpsFlags, ()> {
    let mut result = SpsFlags {
        flags: 0,
        limit: SPS_LIMIT_DEFAULT,
    };

    if option.is_empty() || option[0] == 0 {
        result.flags = SPS_BEST;
        return Ok(result);
    }

    let mut idx = 0;
    while idx < option.len() && option[idx] != 0 {
        // Skip leading commas
        while idx < option.len() && option[idx] == b',' {
            idx += 1;
        }

        if idx >= option.len() || option[idx] == 0 {
            break;
        }

        // Find end of this part (comma or NUL)
        let start = idx;
        while idx < option.len() && option[idx] != b',' && option[idx] != 0 {
            idx += 1;
        }

        let part = &option[start..idx];
        let mut part_limit = 0;
        let flag = parse_sps_value(part, &mut part_limit);

        if flag == -1 {
            // Invalid value
            return Err(());
        }

        if flag == 0 && part[0].is_ascii_digit() {
            // Numeric limit
            result.limit = part_limit;
        } else if flag != 0 {
            // Check for conflicting flags
            if result.flags != 0 {
                return Err(());
            }
            result.flags = flag;
        }
        // flag == 0 and not numeric means expr:/file:/timeout: - no action needed
    }

    if result.flags == 0 {
        result.flags = SPS_BEST;
    }

    Ok(result)
}

/// FFI wrapper for parse_sps_value
///
/// # Safety
/// `value` must be a valid pointer to a null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_parse_sps_value(
    value: *const u8,
    value_len: usize,
    limit_out: *mut c_int,
) -> c_int {
    if value.is_null() || limit_out.is_null() {
        return -1;
    }

    let value_slice = std::slice::from_raw_parts(value, value_len);
    let mut limit = 0;
    let result = parse_sps_value(value_slice, &mut limit);
    *limit_out = limit;
    result
}

/// FFI wrapper for parse_spellsuggest
///
/// # Safety
/// `option` must be a valid pointer, `result_out` must be valid
#[no_mangle]
pub unsafe extern "C" fn rs_parse_spellsuggest(
    option: *const u8,
    option_len: usize,
    result_out: *mut SpsFlags,
) -> c_int {
    if option.is_null() || result_out.is_null() {
        *result_out = SpsFlags {
            flags: SPS_BEST,
            limit: SPS_LIMIT_DEFAULT,
        };
        return -1; // FAIL
    }

    let option_slice = std::slice::from_raw_parts(option, option_len);
    if let Ok(flags) = parse_spellsuggest(option_slice) {
        *result_out = flags;
        0 // OK
    } else {
        *result_out = SpsFlags {
            flags: SPS_BEST,
            limit: SPS_LIMIT_DEFAULT,
        };
        -1 // FAIL
    }
}

/// Check if spellsuggest option is valid (for option validation)
///
/// # Safety
/// `option` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_spell_check_sps(option: *const u8, option_len: usize) -> c_int {
    if option.is_null() {
        return -1; // FAIL
    }

    let option_slice = std::slice::from_raw_parts(option, option_len);
    if parse_spellsuggest(option_slice).is_ok() {
        0 // OK
    } else {
        -1 // FAIL
    }
}

extern "C" {
    fn nvim_spellsug_set_sps_flags(f: c_int);
    fn nvim_spellsug_set_sps_limit(l: c_int);
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    #[link_name = "curwin"]
    static stp_sal_curwin: *mut std::ffi::c_void;
    /// spell_check: checks if `ptr` is misspelled; sets `*attrp` if so.
    fn spell_check(
        wp: *mut std::ffi::c_void,
        ptr: *mut c_char,
        attrp: *mut c_int,
        capcol: *mut c_int,
        docount: bool,
    ) -> usize;
    fn nvim_hlf_count() -> c_int;
}

/// Full replacement for C spell_check_sps():
/// parses p_sps, sets sps_flags and sps_limit globals, returns OK/FAIL.
///
/// # Safety
/// `p_sps_val` must be a valid null-terminated C string (may be empty).
#[no_mangle]
pub unsafe extern "C" fn rs_spell_check_sps_full(p_sps_val: *const c_char) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = -1;

    if p_sps_val.is_null() {
        nvim_spellsug_set_sps_flags(SPS_BEST);
        nvim_spellsug_set_sps_limit(SPS_LIMIT_DEFAULT);
        return FAIL;
    }

    let len = {
        let mut p = p_sps_val;
        while *p != 0 {
            p = p.add(1);
        }
        p.offset_from(p_sps_val) as usize
    };
    let option_slice = std::slice::from_raw_parts(p_sps_val.cast::<u8>(), len);

    if let Ok(flags) = parse_spellsuggest(option_slice) {
        nvim_spellsug_set_sps_flags(flags.flags);
        nvim_spellsug_set_sps_limit(flags.limit);
        OK
    } else {
        nvim_spellsug_set_sps_flags(SPS_BEST);
        nvim_spellsug_set_sps_limit(SPS_LIMIT_DEFAULT);
        FAIL
    }
}

// =============================================================================
// Phase 2: suggest_T mirror and cleanup_suggestions
// =============================================================================

/// Rust mirror of the C `suggest_T` struct.
///
/// SAFETY: Layout must exactly match the C struct. The C definition is:
///   char *st_word;      // 8 bytes (pointer)
///   int st_wordlen;     // 4 bytes
///   int st_orglen;      // 4 bytes
///   int st_score;       // 4 bytes
///   int st_altscore;    // 4 bytes
///   bool st_salscore;   // 1 byte
///   bool st_had_bonus;  // 1 byte
///   // 6 bytes padding (to align pointer)
///   slang_T *st_slang;  // 8 bytes (pointer)
/// Total: 40 bytes
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CSuggestT {
    /// Suggested word, allocated C string
    pub st_word: *mut c_char,
    /// strlen(st_word)
    pub st_wordlen: c_int,
    /// length of replaced text
    pub st_orglen: c_int,
    /// lower is better
    pub st_score: c_int,
    /// used when st_score compares equal
    pub st_altscore: c_int,
    /// st_score is for soundalike
    pub st_salscore: bool,
    /// bonus already included in score
    pub st_had_bonus: bool,
    _pad: [u8; 6],
    /// language used for sound folding (opaque pointer)
    pub st_slang: *mut std::ffi::c_void,
}

extern "C" {
    // Memory free (from nvim/memory.h)
    fn xfree(ptr: *mut std::ffi::c_void);
}

/// Compare two suggestions for sorting.
/// Order: st_score asc, st_altscore asc, st_word case-insensitive asc.
unsafe fn csuggst_cmp(a: &CSuggestT, b: &CSuggestT) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match a.st_score.cmp(&b.st_score) {
        Ordering::Equal => {}
        other => return other,
    }
    match a.st_altscore.cmp(&b.st_altscore) {
        Ordering::Equal => {}
        other => return other,
    }
    // case-insensitive strcmp using C locale (matches STRICMP)
    let r = libc_strcasecmp(a.st_word, b.st_word);
    r.cmp(&0)
}

/// ASCII case-insensitive strcmp (matches STRICMP / strcasecmp in C locale).
unsafe fn libc_strcasecmp(a: *const c_char, b: *const c_char) -> c_int {
    let mut pa = a;
    let mut pb = b;
    loop {
        let ca = c_int::from((*pa as u8).to_ascii_lowercase());
        let cb = c_int::from((*pb as u8).to_ascii_lowercase());
        if ca != cb {
            return ca - cb;
        }
        if ca == 0 {
            return 0;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }
}

/// Rust replacement for C `cleanup_suggestions`.
///
/// Sorts the suggestion array by score then truncates to `keep` items,
/// freeing discarded entries. Returns the max score.
///
/// # Safety
/// `data` must be a valid pointer to at least `*gap_len` `CSuggestT` items.
/// `gap_len` must be a valid pointer to the current length.
/// All `st_word` pointers in the array must be C-allocated (freed via xfree).
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_suggestions(
    data: *mut CSuggestT,
    gap_len: *mut c_int,
    maxscore: c_int,
    keep: c_int,
) -> c_int {
    let len = *gap_len as usize;
    if len == 0 {
        return maxscore;
    }

    // Sort in place using Rust stable sort.
    let slice = std::slice::from_raw_parts_mut(data, len);
    slice.sort_by(|a, b| csuggst_cmp(a, b));

    let keep_usize = keep as usize;
    if len > keep_usize {
        // Free the words we're discarding.
        for item in &slice[keep_usize..len] {
            xfree(item.st_word.cast::<std::ffi::c_void>());
        }
        *gap_len = keep;
        if keep >= 1 {
            return slice[keep_usize - 1].st_score;
        }
    }
    maxscore
}

// =============================================================================
// Phase 3: stp_sal_score
// =============================================================================

/// Advance pointer past non-whitespace bytes (equivalent to C skiptowhite).
unsafe fn skip_to_white(mut p: *const c_char) -> *const c_char {
    while *p != 0 && rs_ascii_iswhite(c_int::from(*p as u8)) == 0 {
        p = p.add(1);
    }
    p
}

/// Rust replacement for C `stp_sal_score`.
///
/// Computes sound-alike score for a suggestion by soundfolding the good word
/// and comparing to the soundfolded bad word.
///
/// # Safety
/// All pointers must be valid. `stp` must point to a valid `CSuggestT`.
/// `su_badptr` and `badsound` must be valid null-terminated C strings.
/// `slang` must be a valid non-null handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stp_sal_score(
    stp: *mut CSuggestT,
    su_badptr: *const c_char,
    su_badlen: c_int,
    slang: SlangHandle,
    badsound: *const c_char,
) -> c_int {
    let mut badsound2 = [0u8; MAXWLEN];
    let mut fword = [0u8; MAXWLEN];
    let mut goodsound = [0u8; MAXWLEN];
    let mut goodword = [0u8; MAXWLEN];

    let lendiff = su_badlen - (*stp).st_orglen;

    let pbad: *const c_char = if lendiff >= 0 {
        badsound
    } else {
        // Soundfold the bad word with more characters following.
        crate::check::rs_spell_casefold_c_compat(
            stp_sal_curwin.cast(),
            su_badptr,
            (*stp).st_orglen,
            fword.as_mut_ptr().cast::<c_char>(),
            MAXWLEN as c_int,
        );

        // When joining two words the sound often changes a lot. Avoid that
        // by removing the space -- but only if the good word has no space.
        let badptr_after_badlen = su_badptr.add(su_badlen as usize);
        if rs_ascii_iswhite(c_int::from(*badptr_after_badlen as u8)) != 0
            && *skip_to_white((*stp).st_word.cast::<c_char>()) == 0
        {
            // Remove spaces from fword in place.
            let mut p = fword.as_mut_ptr();
            loop {
                p = skip_to_white(p.cast::<c_char>()).cast_mut().cast::<u8>();
                if *p == 0 {
                    break;
                }
                // STRMOVE(p, p+1): shift everything left by one
                let remaining = {
                    let mut q = p.add(1);
                    while *q != 0 {
                        q = q.add(1);
                    }
                    q.offset_from(p) as usize // includes the NUL at new position
                };
                std::ptr::copy(p.add(1), p, remaining + 1);
            }
        }

        crate::rs_spell_soundfold(
            slang.0,
            fword.as_mut_ptr().cast::<c_char>(),
            true,
            badsound2.as_mut_ptr().cast::<c_char>(),
        );
        badsound2.as_ptr().cast::<c_char>()
    };

    if lendiff > 0 && (*stp).st_wordlen + lendiff < MAXWLEN as c_int {
        // Add part of the bad word to the good word so we soundfold what
        // replaces the bad word.
        let wlen = (*stp).st_wordlen as usize;
        let src = std::slice::from_raw_parts((*stp).st_word.cast::<u8>(), wlen);
        goodword[..wlen].copy_from_slice(src);
        let ld = lendiff as usize;
        let bad_extra =
            std::slice::from_raw_parts(su_badptr.add(su_badlen as usize - ld).cast::<u8>(), ld);
        goodword[wlen..wlen + ld].copy_from_slice(bad_extra);
        goodword[wlen + ld] = 0;

        crate::rs_spell_soundfold(
            slang.0,
            goodword.as_mut_ptr().cast::<c_char>(),
            false,
            goodsound.as_mut_ptr().cast::<c_char>(),
        );
    } else {
        crate::rs_spell_soundfold(
            slang.0,
            (*stp).st_word.cast::<c_char>(),
            false,
            goodsound.as_mut_ptr().cast::<c_char>(),
        );
    }

    // Find the NUL length for goodsound and pbad, then compute score.
    let good_end = {
        let mut q = goodsound.as_ptr();
        while *q != 0 {
            q = q.add(1);
        }
        q.offset_from(goodsound.as_ptr()) as usize
    };
    let bad_end = {
        let mut q = pbad.cast::<u8>();
        while *q != 0 {
            q = q.add(1);
        }
        q.offset_from(pbad.cast::<u8>()) as usize
    };

    soundalike_score_impl(
        &goodsound[..good_end],
        std::slice::from_raw_parts(pbad.cast::<u8>(), bad_end),
    )
}

// =============================================================================
// Phase 4: check_suggestions
// =============================================================================

/// Rust replacement for C `check_suggestions`.
///
/// Removes suggestions from the array that spell_check considers misspelled
/// (catches "the the" -> "thee the" cases by appending trailing bad text).
///
/// # Safety
/// `data` must be a valid pointer to `*gap_len` `CSuggestT` items.
/// `gap_len` must be a valid pointer to the current length.
/// `su_badptr` must be a valid null-terminated C string.
/// All `st_word` pointers must be C-heap-allocated (freed via xfree).
#[no_mangle]
pub unsafe extern "C" fn rs_check_suggestions(
    data: *mut CSuggestT,
    gap_len: *mut c_int,
    su_badptr: *const c_char,
) {
    let hlf_count = nvim_hlf_count();
    let len = *gap_len as usize;
    if len == 0 {
        return;
    }

    let mut cur_len = len;
    let mut i = cur_len;
    loop {
        if i == 0 {
            break;
        }
        i -= 1;

        let stp = &*data.add(i);
        // Build longword = stp->st_word + (su_badptr + stp->st_orglen)
        let mut longword = [0u8; MAXWLEN + 1];
        let wlen = stp.st_wordlen as usize;
        // Copy st_word bytes
        let word_bytes = std::slice::from_raw_parts(stp.st_word.cast::<u8>(), wlen);
        let copy_len = wlen.min(MAXWLEN);
        longword[..copy_len].copy_from_slice(&word_bytes[..copy_len]);
        // Append su_badptr + st_orglen
        let tail_ptr = su_badptr.add(stp.st_orglen as usize).cast::<u8>();
        let remaining = MAXWLEN + 1 - copy_len;
        let mut j = 0usize;
        while j < remaining {
            let b = *tail_ptr.add(j);
            longword[copy_len + j] = b;
            if b == 0 {
                break;
            }
            j += 1;
        }
        if copy_len + j <= MAXWLEN {
            longword[copy_len + j] = 0;
        }

        let mut attr = hlf_count;
        spell_check(
            stp_sal_curwin,
            longword.as_mut_ptr().cast::<c_char>(),
            &raw mut attr,
            std::ptr::null_mut(),
            false,
        );

        if attr != hlf_count {
            // Remove this entry.
            xfree((*data.add(i)).st_word.cast::<std::ffi::c_void>());
            cur_len -= 1;
            if i < cur_len {
                std::ptr::copy(data.add(i + 1), data.add(i), cur_len - i);
            }
        }
    }
    *gap_len = cur_len as c_int;
}

// =============================================================================
// Phase 5: score_combine
// =============================================================================

extern "C" {
    fn nvim_suginfo_get_ga(su: *mut std::ffi::c_void) -> *mut crate::GArrayRaw;
    fn nvim_suginfo_get_sga(su: *mut std::ffi::c_void) -> *mut crate::GArrayRaw;
    fn nvim_suginfo_get_fbadword(su: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_suginfo_get_badword(su: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_suginfo_get_badptr(su: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_suginfo_get_badlen(su: *mut std::ffi::c_void) -> c_int;
    fn nvim_suginfo_get_maxscore(su: *mut std::ffi::c_void) -> c_int;
    fn nvim_suginfo_get_maxcount(su: *mut std::ffi::c_void) -> c_int;
    fn nvim_suginfo_set_ga(su: *mut std::ffi::c_void, ga: crate::GArrayRaw);
    fn nvim_win_get_b_langp(wp: *const std::ffi::c_void) -> *const crate::GArrayRaw;
    #[link_name = "ga_init"]
    fn ga_init_suggest(gap: *mut crate::GArrayRaw, itemsize: c_int, growsize: c_int);
    #[link_name = "ga_grow"]
    fn ga_grow_suggest(gap: *mut crate::GArrayRaw, n: c_int);
    #[link_name = "ga_clear"]
    fn ga_clear_suggest(gap: *mut crate::GArrayRaw);
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

/// Rust replacement for C `score_combine` (the suggestion list combiner).
///
/// Combines su->su_ga and su->su_sga suggestion lists by applying
/// sound-alike rescoring and merging with deduplication.
///
/// # Safety
/// `su` must be a valid pointer to a `suginfo_T` struct.
#[no_mangle]
pub unsafe extern "C" fn rs_score_combine_lists(su: *mut std::ffi::c_void) {
    let mut badsound = [0u8; MAXWLEN];
    let mut slang: SlangHandle = SlangHandle::null();

    let langp_ga = nvim_win_get_b_langp(stp_sal_curwin);
    let langp_len = (*langp_ga).ga_len;

    // Step 1: Add alternate score to su_ga using sound-alike scoring.
    for lpi in 0..langp_len {
        let lp = crate::langp_entry(langp_ga, lpi);
        let lp_slang = SlangHandle((*lp).lp_slang);
        if (*(*lp).lp_slang).sl_sal.ga_len > 0 {
            slang = lp_slang;
            crate::rs_spell_soundfold(
                slang.0,
                nvim_suginfo_get_fbadword(su).cast_mut(),
                true,
                badsound.as_mut_ptr().cast::<c_char>(),
            );

            let su_badptr = nvim_suginfo_get_badptr(su);
            let su_badlen = nvim_suginfo_get_badlen(su);
            let ga = nvim_suginfo_get_ga(su);
            let ga_len = (*ga).ga_len as usize;
            let ga_data = (*ga).ga_data.cast::<CSuggestT>();
            for i in 0..ga_len {
                let stp = ga_data.add(i);
                (*stp).st_altscore =
                    rs_stp_sal_score(stp, su_badptr, su_badlen, slang, badsound.as_ptr().cast());
                if (*stp).st_altscore == SCORE_MAXMAX {
                    (*stp).st_score = ((*stp).st_score * 3 + SCORE_BIG) / 4;
                } else {
                    (*stp).st_score = ((*stp).st_score * 3 + (*stp).st_altscore) / 4;
                }
                (*stp).st_salscore = false;
            }
            break;
        }
    }

    if slang.is_null() {
        // Using "double" without sound folding.
        let ga = nvim_suginfo_get_ga(su);
        let maxscore = nvim_suginfo_get_maxscore(su);
        let maxcount = nvim_suginfo_get_maxcount(su);
        rs_cleanup_suggestions(
            (*ga).ga_data.cast::<CSuggestT>(),
            &raw mut (*ga).ga_len,
            maxscore,
            maxcount,
        );
        return;
    }

    // Step 2: Add alternate score to su_sga using edit distance scoring.
    let badword = nvim_suginfo_get_badword(su);
    {
        let sga = nvim_suginfo_get_sga(su);
        let sga_len = (*sga).ga_len as usize;
        let sga_data = (*sga).ga_data.cast::<CSuggestT>();
        for i in 0..sga_len {
            let stp = sga_data.add(i);
            (*stp).st_altscore = rs_spell_edit_score(slang, badword, (*stp).st_word);
            if (*stp).st_score == SCORE_MAXMAX {
                (*stp).st_score = (SCORE_BIG * 7 + (*stp).st_altscore) / 8;
            } else {
                (*stp).st_score = ((*stp).st_score * 7 + (*stp).st_altscore) / 8;
            }
            (*stp).st_salscore = true;
        }
    }

    // Step 3: Clean up both lists.
    let maxscore = nvim_suginfo_get_maxscore(su);
    let maxcount = nvim_suginfo_get_maxcount(su);
    let su_badptr = nvim_suginfo_get_badptr(su);

    {
        let ga = nvim_suginfo_get_ga(su);
        rs_check_suggestions(
            (*ga).ga_data.cast::<CSuggestT>(),
            &raw mut (*ga).ga_len,
            su_badptr,
        );
        rs_cleanup_suggestions(
            (*ga).ga_data.cast::<CSuggestT>(),
            &raw mut (*ga).ga_len,
            maxscore,
            maxcount,
        );
    }

    {
        let sga = nvim_suginfo_get_sga(su);
        rs_check_suggestions(
            (*sga).ga_data.cast::<CSuggestT>(),
            &raw mut (*sga).ga_len,
            su_badptr,
        );
        rs_cleanup_suggestions(
            (*sga).ga_data.cast::<CSuggestT>(),
            &raw mut (*sga).ga_len,
            maxscore,
            maxcount,
        );
    }

    // Step 4: Merge both lists into a new garray, deduplicating.
    let ga = nvim_suginfo_get_ga(su);
    let sga = nvim_suginfo_get_sga(su);
    let total = (*ga).ga_len + (*sga).ga_len;

    let mut merged = crate::GArrayRaw {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init_suggest(
        &raw mut merged,
        std::mem::size_of::<CSuggestT>() as c_int,
        1,
    );
    ga_grow_suggest(&raw mut merged, total);

    let merged_data = merged.ga_data.cast::<CSuggestT>();
    let ga_data = (*ga).ga_data.cast::<CSuggestT>();
    let sga_data = (*sga).ga_data.cast::<CSuggestT>();
    let ga_len = (*ga).ga_len as usize;
    let sga_len = (*sga).ga_len as usize;
    let outer_len = ga_len.max(sga_len);

    for i in 0..outer_len {
        for round in 0..2usize {
            let (src_data, src_len) = if round == 0 {
                (ga_data, ga_len)
            } else {
                (sga_data, sga_len)
            };
            if i >= src_len {
                continue;
            }
            let word = (*src_data.add(i)).st_word;
            // Check if this word is already in the merged list.
            let mut already_there = false;
            for j in 0..merged.ga_len as usize {
                if strcmp((*merged_data.add(j)).st_word, word) == 0 {
                    already_there = true;
                    break;
                }
            }
            if already_there {
                xfree(word.cast::<std::ffi::c_void>());
            } else {
                *merged_data.add(merged.ga_len as usize) = *src_data.add(i);
                merged.ga_len += 1;
            }
        }
    }

    ga_clear_suggest(ga);
    ga_clear_suggest(sga);

    // Step 5: Truncate to maxcount.
    if merged.ga_len > maxcount {
        for i in maxcount as usize..merged.ga_len as usize {
            xfree((*merged_data.add(i)).st_word.cast::<std::ffi::c_void>());
        }
        merged.ga_len = maxcount;
    }

    nvim_suginfo_set_ga(su, merged);
}

// =============================================================================
// Phase 2: suggest_trie_walk
// =============================================================================

// Word flags used inside suggest_trie_walk
const WF_NOSUGGEST: c_int = 0x0800;
const WF_BANNED: c_int = 0x10;
const WF_NEEDCOMP: c_int = 0x1000;
const WF_REGION: c_int = 0x01;
const WF_RARE: c_int = 0x08;
const WF_KEEPCAP: c_int = 0x80;
const WF_ALLCAP: c_int = 0x04;
const WF_ONECAP: c_int = 0x02;
const WF_MIXCAP: c_int = 0x20;

extern "C" {
    fn nvim_spellsug_get_timeout() -> c_int;
    fn profile_setlimit(msec: i64) -> u64;
    fn profile_passed_limit(tm: u64) -> bool;
    fn os_breakcheck();
    fn utf_iscomposing_legacy(c: c_int) -> bool;
    fn mb_charlen(s: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    // nvim_add_sound_suggest: removed; rs_add_sound_suggest called directly
    #[link_name = "skiptowhite"]
    fn trie_skiptowhite(p: *const c_char) -> *mut c_char;
    #[link_name = "skipwhite"]
    fn trie_skipwhite(p: *const c_char) -> *mut c_char;
    // hash_find: already in suggest.rs extern (suggest module has its own block)
    #[link_name = "hash_find"]
    fn trie_hash_find(ht: *const crate::HashtabRaw, key: *const c_char) -> *mut crate::HashitemRaw;
}

/// C fromto_T struct (two char* pointers).
#[repr(C)]
struct FromtoC {
    ft_from: *mut c_char,
    ft_to: *mut c_char,
}

/// True if a hashitem is empty for the trie walk.
#[inline]
unsafe fn trie_hi_empty(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq(
            (*hi).hi_key,
            std::ptr::addr_of!(suggest_hash_removed).cast_mut(),
        )
}

/// WAS_BANNED: check if `word` is in su->su_banned.
#[inline]
unsafe fn was_banned(su: *mut std::ffi::c_void, word: *const c_char) -> bool {
    let ht = nvim_suginfo_get_banned(su);
    let hi = trie_hash_find(ht, word);
    !trie_hi_empty(hi)
}

/// TRY_DEEPER: check depth and score limits.
#[inline]
fn try_deeper(depth: usize, stack: &[TryState], add: c_int, su_maxscore: c_int) -> bool {
    depth < MAXWLEN - 1 && stack[depth].score + add < su_maxscore
}

/// STRMOVE(dst, src): memmove to shift a C string left (overlapping).
/// dst and src may overlap; copies until NUL.
#[inline]
unsafe fn strmove(dst: *mut c_char, src: *const c_char) {
    let mut len = 0usize;
    while *src.add(len) != 0 {
        len += 1;
    }
    // +1 to include the NUL terminator
    std::ptr::copy(src, dst, len + 1);
}

/// strcat: append src to end of dst (like C strcat).
#[inline]
unsafe fn c_strcat(dst: *mut c_char, src: *const c_char) {
    let mut end = dst;
    while *end != 0 {
        end = end.add(1);
    }
    let mut s = src;
    while *s != 0 {
        *end = *s;
        end = end.add(1);
        s = s.add(1);
    }
    *end = 0;
}

/// strlen for raw C char pointer.
#[inline]
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// ASCII iswhite check.
#[inline]
unsafe fn c_ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// MB_BYTE2LEN: Length of multibyte character from first byte.
/// For UTF-8: 1-byte (0x00-0x7F), 2-byte (0xC0-0xDF), etc.
#[inline]
fn mb_byte2len(c: u8) -> usize {
    if c < 0x80 {
        1
    } else if c < 0xC0 {
        // continuation byte - shouldn't happen for first byte
        1
    } else if c < 0xE0 {
        2
    } else if c < 0xF0 {
        3
    } else {
        4
    }
}

/// STRCPY (unsafe raw C string copy).
#[inline]
unsafe fn c_strcpy(dst: *mut c_char, src: *const c_char) {
    let len = c_strlen(src);
    std::ptr::copy_nonoverlapping(src, dst, len + 1);
}

/// strncmp: compare n bytes, return true if equal.
#[inline]
unsafe fn c_strncmp(a: *const c_char, b: *const c_char, n: usize) -> bool {
    std::slice::from_raw_parts(a.cast::<u8>(), n) == std::slice::from_raw_parts(b.cast::<u8>(), n)
}

/// go_deeper: copy stack[depth] to stack[depth+1], init new frame.
#[inline]
unsafe fn go_deeper_inline(stack: *mut TryState, depth: usize, score_add: c_int) {
    std::ptr::copy_nonoverlapping(stack.add(depth), stack.add(depth + 1), 1);
    let next = &mut *stack.add(depth + 1);
    next.state = TrieWalkState::Start;
    next.score += score_add;
    next.curi = 1;
    next.flags = 0;
}

/// Rust implementation of C `suggest_trie_walk`.
///
/// The main trie walk state machine for generating suggestions by trying
/// insert/delete/swap/replace operations on the bad word.
///
/// # Safety
/// All pointers must be valid. `su` is an opaque suginfo_T.
/// `lp` is a valid LangpT. `fword` is a mutable MAXWLEN-sized buffer.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::used_underscore_binding)]
pub unsafe extern "C" fn rs_suggest_trie_walk(
    su: *mut std::ffi::c_void,
    lp: *const crate::LangpT,
    fword: *mut c_char,
    soundfold: bool,
) {
    let mut tword = [0u8; MAXWLEN];
    let mut stack: Vec<TryState> = vec![TryState::default(); MAXWLEN];
    let mut preword = [0u8; MAXWLEN * 3];
    let mut compflags = [0u8; MAXWLEN];

    let slang_ptr = (*lp).lp_slang;
    let slang = SlangHandle(slang_ptr);

    let (fbyts_ptr, fidxs_ptr, pbyts_ptr, pidxs_ptr): (*mut u8, *mut c_int, *mut u8, *mut c_int) =
        if soundfold {
            (
                (*slang_ptr).sl_sbyts,
                (*slang_ptr).sl_sidxs,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } else {
            (
                (*slang_ptr).sl_fbyts,
                (*slang_ptr).sl_fidxs,
                (*slang_ptr).sl_pbyts,
                (*slang_ptr).sl_pidxs,
            )
        };

    // byts/idxs: current active tree (can switch between prefix tree and fbyts)
    let mut byts: *mut u8 = if soundfold || pbyts_ptr.is_null() {
        fbyts_ptr
    } else {
        pbyts_ptr
    };
    let mut idxs: *mut c_int = if soundfold || pidxs_ptr.is_null() {
        fidxs_ptr
    } else {
        pidxs_ptr
    };

    let mut depth: isize = 0;
    {
        let sp = &mut stack[0];
        sp.curi = 1;
        sp.score = 0;
        sp.arridx = 0;
        sp.fidx = 0;
        sp.fidxtry = 0;
        sp.twordlen = 0;
        sp.flags = 0;
        sp.tcharlen = 0;
        sp.tcharidx = 0;
        sp.isdiff = DiffType::None;
        sp.fcharstart = 0;
        sp.prewordlen = 0;
        sp.splitoff = 0;
        sp.splitfidx = 0;
        sp.complen = 0;
        sp.compsplit = 0;
        sp.save_badflags = 0;
        sp.delidx = 0;
        sp.prefixdepth = if soundfold || pbyts_ptr.is_null() {
            prefix_depth::PFD_NOPREFIX
        } else {
            prefix_depth::PFD_PREFIXTREE
        };
        sp.state = if soundfold || pbyts_ptr.is_null() {
            TrieWalkState::Start
        } else {
            TrieWalkState::NoPrefix
        };
    }

    // Time limit setup
    let timeout = nvim_spellsug_get_timeout();
    let time_limit: u64 = if timeout > 0 {
        profile_setlimit(i64::from(timeout))
    } else {
        0
    };

    let mut repextra: c_int = 0;
    let mut breakcheckcount: c_int = 1000;

    // Cache frequently-used suginfo fields at start of loop to reduce overhead.
    // These are re-fetched from accessors when they might change.
    let su_badptr = nvim_suginfo_get_badptr(su);
    let su_badlen = nvim_suginfo_get_badlen(su);

    while depth >= 0 && !crate::got_int {
        let d = depth as usize;
        // We take a raw pointer to stack[d] so we can mutate it while also
        // calling helper functions. All helpers that modify sp go through this pointer.
        let sp_raw = stack.as_mut_ptr().add(d);

        macro_rules! sp {
            () => {
                &*sp_raw
            };
        }
        macro_rules! sp_mut {
            () => {
                &mut *sp_raw
            };
        }

        let su_maxscore = nvim_suginfo_get_maxscore(su);

        match (*sp_raw).state {
            TrieWalkState::Start | TrieWalkState::NoPrefix => {
                let arridx = sp!().arridx as usize;
                let len = *byts.add(arridx) as usize;
                let arridx_cur = arridx + sp!().curi as usize;

                if sp!().prefixdepth == prefix_depth::PFD_PREFIXTREE {
                    // Skip over NUL bytes.
                    let mut n = 0usize;
                    while n < len && *byts.add(arridx_cur + n) == 0 {
                        n += 1;
                    }
                    sp_mut!().curi = (sp!().curi + n as i16).max(sp!().curi);
                    // Actually: sp->ts_curi += n (we already computed sp!().curi above)
                    // but the original code does sp->ts_curi = sp->ts_curi + n; let me redo
                    {
                        let old_curi = (*sp_raw).curi;
                        (*sp_raw).curi = old_curi + n as i16;
                    }

                    let n_state = (*sp_raw).state as c_int;
                    (*sp_raw).state = TrieWalkState::EndNul;
                    (*sp_raw).save_badflags = nvim_suginfo_get_badflags(su) as u8;

                    if depth < (MAXWLEN - 1) as isize
                        && (*byts.add(arridx_cur + n) == 0
                            || n_state == TrieWalkState::NoPrefix as c_int)
                    {
                        let fidx = (*sp_raw).fidx as usize;
                        let n_nofold = crate::rs_nofold_len(fword, fidx as c_int, su_badptr);
                        let flags_pref =
                            rs_badword_captype(su_badptr, su_badptr.add(n_nofold as usize));
                        let new_badflags = rs_badword_captype(
                            su_badptr.add(n_nofold as usize),
                            su_badptr.add(su_badlen as usize),
                        );
                        nvim_suginfo_set_badflags(su, new_badflags);
                        go_deeper_inline(stack.as_mut_ptr(), d, 0);
                        depth += 1;
                        let d2 = depth as usize;
                        let sp2 = &mut stack[d2];
                        sp2.prefixdepth = d as u8;
                        byts = fbyts_ptr;
                        idxs = fidxs_ptr;
                        sp2.arridx = 0;

                        // Move prefix to preword with right case.
                        tword[sp2.twordlen as usize] = 0;
                        crate::rs_make_case_word(
                            tword.as_ptr().add(sp2.splitoff as usize).cast::<c_char>(),
                            preword
                                .as_mut_ptr()
                                .add(sp2.prewordlen as usize)
                                .cast::<c_char>(),
                            flags_pref,
                        );
                        let plen = c_strlen(preword.as_ptr().cast::<c_char>());
                        sp2.prewordlen = plen as u8;
                        sp2.splitoff = sp2.twordlen;
                    }
                    // 'break' in C switch → continue outer loop
                    continue;
                }

                if (*sp_raw).curi as usize > len || *byts.add(arridx_cur) != 0 {
                    (*sp_raw).state = TrieWalkState::EndNul;
                    (*sp_raw).save_badflags = nvim_suginfo_get_badflags(su) as u8;
                    continue;
                }

                // End of word in tree.
                (*sp_raw).curi += 1;

                let flags = *idxs.add(arridx_cur);

                if flags & WF_NOSUGGEST != 0 {
                    continue;
                }

                let fidx = (*sp_raw).fidx as usize;
                let fword_ends = *fword.add(fidx) == 0
                    || if soundfold {
                        c_ascii_iswhite(*fword.add(fidx) as u8)
                    } else {
                        !crate::rs_spell_iswordp(fword.add(fidx), stp_sal_curwin)
                    };

                tword[(*sp_raw).twordlen as usize] = 0;

                // Check prefix validity.
                if (*sp_raw).prefixdepth <= prefix_depth::PFD_NOTSPECIAL
                    && ((*sp_raw).flags & try_state_flags::TSF_PREFIXOK) == 0
                    && !pbyts_ptr.is_null()
                {
                    let n_idx = stack[(*sp_raw).prefixdepth as usize].arridx as usize;
                    let plen = *pbyts_ptr.add(n_idx) as usize;
                    let mut c = 0usize;
                    while c < plen && *pbyts_ptr.add(n_idx + 1 + c) == 0 {
                        c += 1;
                    }
                    if c > 0 {
                        let vc = crate::check::rs_valid_word_prefix(
                            c as c_int,
                            (n_idx + 1) as c_int,
                            flags,
                            tword
                                .as_mut_ptr()
                                .add((*sp_raw).splitoff as usize)
                                .cast::<c_char>(),
                            slang.0,
                            false,
                        );
                        if vc == 0 {
                            continue;
                        }
                        (*sp_raw).flags |= try_state_flags::TSF_PREFIXOK;
                    }
                }

                // Check NEEDCOMPOUND.
                let goodword_ends_base = !((*sp_raw).complen == (*sp_raw).compsplit
                    && fword_ends
                    && (flags & WF_NEEDCOMP) != 0);

                let mut goodword_ends = goodword_ends_base;
                let mut p_compound: *const c_char = std::ptr::null();
                let mut compound_ok = true;

                if (*sp_raw).complen > (*sp_raw).compsplit {
                    if (*slang_ptr).sl_nobreak {
                        // NOBREAK: check if the previous word was correct and add it.
                        let split_fidx = (*sp_raw).splitfidx as usize;
                        let split_off = (*sp_raw).splitoff as usize;
                        let flen_diff = fidx - split_fidx;
                        let tlen_diff = (*sp_raw).twordlen as usize - split_off;
                        if flen_diff == tlen_diff
                            && c_strncmp(
                                fword.add(split_fidx),
                                tword.as_ptr().add(split_off).cast::<c_char>(),
                                flen_diff,
                            )
                        {
                            preword[(*sp_raw).prewordlen as usize] = 0;
                            let newscore = rs_score_wordcount_adj(
                                slang,
                                (*sp_raw).score,
                                preword
                                    .as_ptr()
                                    .add((*sp_raw).prewordlen as usize)
                                    .cast::<c_char>(),
                                (*sp_raw).prewordlen > 0,
                            );
                            if newscore <= su_maxscore {
                                rs_add_suggestion(
                                    su,
                                    nvim_suginfo_get_ga(su),
                                    preword.as_ptr().cast::<c_char>(),
                                    c_int::from((*sp_raw).splitfidx) - repextra,
                                    newscore,
                                    0,
                                    false,
                                    (*lp).lp_sallang.cast(),
                                    false,
                                );
                            }
                        }
                        continue;
                    }
                    // Compound word check.
                    if (flags >> 24 == 0)
                        || ((*sp_raw).twordlen as usize - (*sp_raw).splitoff as usize)
                            < (*slang_ptr).sl_compminlen as usize
                    {
                        continue;
                    }
                    if (*slang_ptr).sl_compminlen > 0
                        && mb_charlen(
                            tword
                                .as_ptr()
                                .add((*sp_raw).splitoff as usize)
                                .cast::<c_char>(),
                        ) < (*slang_ptr).sl_compminlen
                    {
                        continue;
                    }

                    let complen_idx = (*sp_raw).complen as usize;
                    compflags[complen_idx] = (flags as u32 >> 24) as u8;
                    compflags[complen_idx + 1] = 0;

                    let prelen = (*sp_raw).prewordlen as usize;
                    let twlen = (*sp_raw).twordlen as usize;
                    let spoff = (*sp_raw).splitoff as usize;
                    let copy_n = twlen - spoff;
                    // xmemcpyz: copy with NUL terminator
                    std::ptr::copy_nonoverlapping(
                        tword.as_ptr().add(spoff),
                        preword.as_mut_ptr().add(prelen),
                        copy_n,
                    );
                    preword[prelen + copy_n] = 0;

                    // Check CHECKCOMPOUNDPATTERN rules.
                    if crate::check::rs_match_checkcompoundpattern(
                        preword.as_ptr().cast::<c_char>(),
                        prelen as c_int,
                        std::ptr::addr_of!((*slang_ptr).sl_comppat),
                    ) {
                        compound_ok = false;
                    }

                    if compound_ok {
                        // Find last word in preword.
                        let mut pp = preword.as_ptr().cast::<c_char>();
                        loop {
                            let next = trie_skiptowhite(pp);
                            if *next == 0 {
                                break;
                            }
                            pp = trie_skipwhite(next);
                        }
                        p_compound = pp;
                        if fword_ends
                            && !crate::check::rs_can_compound_c_compat(
                                slang_ptr,
                                p_compound,
                                compflags.as_ptr().add((*sp_raw).compsplit as usize),
                            )
                        {
                            compound_ok = false;
                        }
                    }

                    // Get pointer to last char of previous word.
                    if compound_ok {
                        let mut pp = preword
                            .as_ptr()
                            .add((*sp_raw).prewordlen as usize)
                            .cast::<c_char>();
                        pp = mb_ptr_back(preword.as_ptr().cast::<c_char>(), pp);
                        p_compound = pp;
                    }
                }

                // Form the word with proper case in preword.
                let prelen = (*sp_raw).prewordlen as usize;
                let sp_off = (*sp_raw).splitoff as usize;
                if soundfold {
                    c_strcpy(
                        preword.as_mut_ptr().add(prelen).cast::<c_char>(),
                        tword.as_ptr().add(sp_off).cast::<c_char>(),
                    );
                } else if flags & WF_KEEPCAP != 0 {
                    rs_find_keepcap_word(
                        slang,
                        tword.as_mut_ptr().add(sp_off).cast::<c_char>(),
                        preword.as_mut_ptr().add(prelen).cast::<c_char>(),
                    );
                } else {
                    let mut c = nvim_suginfo_get_badflags(su);
                    if (c & WF_ALLCAP) != 0 && su_badlen == crate::utfc_ptr2len(su_badptr) {
                        c = WF_ONECAP;
                    }
                    c |= flags;
                    // When appending compound after word char, no onecap.
                    if !p_compound.is_null()
                        && crate::rs_spell_iswordp_nmw(p_compound, stp_sal_curwin)
                    {
                        c &= !WF_ONECAP;
                    }
                    crate::rs_make_case_word(
                        tword.as_ptr().add(sp_off).cast::<c_char>(),
                        preword.as_mut_ptr().add(prelen).cast::<c_char>(),
                        c,
                    );
                }

                if !soundfold {
                    // Don't use banned word.
                    if flags & WF_BANNED != 0 {
                        rs_add_banned(su, preword.as_mut_ptr().add(prelen).cast::<c_char>());
                        continue;
                    }
                    let prelen2 = prelen;
                    if ((*sp_raw).complen == (*sp_raw).compsplit
                        && was_banned(su, preword.as_ptr().add(prelen2).cast::<c_char>()))
                        || was_banned(su, preword.as_ptr().cast::<c_char>())
                    {
                        if (*slang_ptr).sl_compprog.is_null() {
                            continue;
                        }
                        goodword_ends = false;
                    }
                }

                let mut newscore: c_int = 0;
                if !soundfold {
                    if (flags & WF_REGION) != 0
                        && ((flags as u32 >> 16) as c_int & (*lp).lp_region) == 0
                    {
                        newscore += SCORES.region;
                    }
                    if (flags & WF_RARE) != 0 {
                        newscore += SCORES.rare;
                    }
                    if !crate::spell_valid_case_impl(
                        nvim_suginfo_get_badflags(su),
                        crate::rs_captype(
                            preword.as_ptr().add(prelen).cast::<c_char>(),
                            std::ptr::null(),
                        ),
                    ) {
                        newscore += SCORES.icase;
                    }
                }

                // Add suggestion if words match.
                if fword_ends && goodword_ends && (*sp_raw).fidx >= (*sp_raw).fidxtry && compound_ok
                {
                    if soundfold {
                        rs_add_sound_suggest(
                            su,
                            preword.as_mut_ptr().cast::<c_char>(),
                            (*sp_raw).score,
                            lp,
                        );
                    } else if (*sp_raw).fidx > 0 {
                        // Penalty for non-word char to word char transition.
                        let fidx_cur = (*sp_raw).fidx as usize;
                        let mut pp = fword.add(fidx_cur);
                        pp = mb_ptr_back(fword, pp).cast_mut();
                        if !crate::rs_spell_iswordp(pp, stp_sal_curwin) && preword[0] != 0 {
                            let preword_end_ptr = preword
                                .as_ptr()
                                .add(c_strlen(preword.as_ptr().cast::<c_char>()));
                            let mut pp2 = preword_end_ptr as *const c_char;
                            pp2 = mb_ptr_back(preword.as_ptr().cast::<c_char>(), pp2);
                            if crate::rs_spell_iswordp(pp2, stp_sal_curwin) {
                                newscore += SCORES.nonword;
                            }
                        }

                        let score = rs_score_wordcount_adj(
                            slang,
                            (*sp_raw).score + newscore,
                            preword.as_ptr().add(prelen).cast::<c_char>(),
                            prelen > 0,
                        );

                        if score <= su_maxscore {
                            rs_add_suggestion(
                                su,
                                nvim_suginfo_get_ga(su),
                                preword.as_ptr().cast::<c_char>(),
                                c_int::from((*sp_raw).fidx) - repextra,
                                score,
                                0,
                                false,
                                (*lp).lp_sallang.cast(),
                                false,
                            );

                            if nvim_suginfo_get_badflags(su) & WF_MIXCAP != 0 {
                                let c = crate::rs_captype(
                                    preword.as_ptr().cast::<c_char>(),
                                    std::ptr::null(),
                                );
                                if c == 0 || c == WF_ALLCAP {
                                    crate::rs_make_case_word(
                                        tword.as_ptr().add(sp_off).cast::<c_char>(),
                                        preword.as_mut_ptr().add(prelen).cast::<c_char>(),
                                        if c == 0 { WF_ALLCAP } else { 0 },
                                    );
                                    rs_add_suggestion(
                                        su,
                                        nvim_suginfo_get_ga(su),
                                        preword.as_ptr().cast::<c_char>(),
                                        c_int::from((*sp_raw).fidx) - repextra,
                                        score + SCORES.icase,
                                        0,
                                        false,
                                        (*lp).lp_sallang.cast(),
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }

                // Try word split and/or compounding.
                if ((*sp_raw).fidx >= (*sp_raw).fidxtry || fword_ends) && (*sp_raw).tcharlen == 0 {
                    let try_split =
                        c_int::from((*sp_raw).fidx) - repextra < su_badlen && !soundfold;

                    let mut try_compound = false;
                    if !soundfold
                        && !(*slang_ptr).sl_nocompoundsugs
                        && !(*slang_ptr).sl_compprog.is_null()
                        && (flags >> 24 != 0)
                        && ((*sp_raw).twordlen as usize - (*sp_raw).splitoff as usize)
                            >= (*slang_ptr).sl_compminlen as usize
                        && ((*slang_ptr).sl_compminlen == 0
                            || mb_charlen(
                                tword
                                    .as_ptr()
                                    .add((*sp_raw).splitoff as usize)
                                    .cast::<c_char>(),
                            ) >= (*slang_ptr).sl_compminlen)
                        && ((*slang_ptr).sl_compsylmax < MAXWLEN as c_int
                            || (c_int::from((*sp_raw).complen) + 1
                                - c_int::from((*sp_raw).compsplit))
                                < (*slang_ptr).sl_compmax)
                        && rs_can_be_compound(sp_raw, slang, compflags.as_mut_ptr(), flags >> 24)
                    {
                        try_compound = true;
                        compflags[(*sp_raw).complen as usize] = (flags as u32 >> 24) as u8;
                        compflags[(*sp_raw).complen as usize + 1] = 0;
                    }

                    if (*slang_ptr).sl_nobreak && !(*slang_ptr).sl_nocompoundsugs {
                        try_compound = true;
                    } else if !fword_ends
                        && try_compound
                        && ((*sp_raw).flags & try_state_flags::TSF_DIDSPLIT) == 0
                    {
                        try_compound = false;
                        (*sp_raw).flags |= try_state_flags::TSF_DIDSPLIT;
                        (*sp_raw).curi -= 1;
                        compflags[(*sp_raw).complen as usize] = 0;
                    } else {
                        (*sp_raw).flags &= !try_state_flags::TSF_DIDSPLIT;
                    }

                    if try_split || try_compound {
                        let mut newscore2 = newscore;
                        if !try_compound && (!fword_ends || !goodword_ends) {
                            if (*sp_raw).complen == (*sp_raw).compsplit
                                && (flags & WF_NEEDCOMP) != 0
                            {
                                continue;
                            }
                            let mut pp = preword.as_ptr().cast::<c_char>();
                            loop {
                                let next = trie_skiptowhite(pp);
                                if *next == 0 {
                                    break;
                                }
                                pp = trie_skipwhite(next);
                            }
                            if (*sp_raw).complen > (*sp_raw).compsplit
                                && !crate::check::rs_can_compound_c_compat(
                                    slang_ptr,
                                    pp,
                                    compflags.as_ptr().add((*sp_raw).compsplit as usize),
                                )
                            {
                                continue;
                            }

                            if (*slang_ptr).sl_nosplitsugs {
                                newscore2 += SCORES.split_no;
                            } else {
                                newscore2 += SCORES.split;
                            }

                            newscore2 = rs_score_wordcount_adj(
                                slang,
                                newscore2,
                                preword
                                    .as_ptr()
                                    .add((*sp_raw).prewordlen as usize)
                                    .cast::<c_char>(),
                                true,
                            );
                        }

                        if try_deeper(d, &stack, newscore2, su_maxscore) {
                            go_deeper_inline(stack.as_mut_ptr(), d, newscore2);
                            stack[d].save_badflags = nvim_suginfo_get_badflags(su) as u8;
                            stack[d].state = TrieWalkState::SplitUndo;

                            depth += 1;
                            let d2 = depth as usize;

                            if !try_compound && !fword_ends {
                                c_strcat(preword.as_mut_ptr().cast::<c_char>(), c" ".as_ptr());
                            }
                            let plen2 = c_strlen(preword.as_ptr().cast::<c_char>());
                            stack[d2].prewordlen = plen2 as u8;
                            stack[d2].splitoff = stack[d2].twordlen;
                            stack[d2].splitfidx = stack[d2].fidx;

                            // Skip non-word char at split.
                            if ((!try_compound
                                && !crate::rs_spell_iswordp_nmw(
                                    fword.add(stack[d2].fidx as usize),
                                    stp_sal_curwin,
                                ))
                                || fword_ends)
                                && *fword.add(stack[d2].fidx as usize) != 0
                                && goodword_ends
                            {
                                let l = crate::utfc_ptr2len(fword.add(stack[d2].fidx as usize))
                                    as usize;
                                if fword_ends {
                                    let fidx2 = stack[d2].fidx as usize;
                                    std::ptr::copy_nonoverlapping(
                                        fword.add(fidx2).cast::<u8>(),
                                        preword.as_mut_ptr().add(plen2),
                                        l,
                                    );
                                    let new_plen = plen2 + l;
                                    stack[d2].prewordlen = new_plen as u8;
                                    preword[new_plen] = 0;
                                } else {
                                    stack[d2].score -= SCORES.split - SCORES.subst;
                                }
                                stack[d2].fidx = (stack[d2].fidx as usize + l) as u8;
                            }

                            if try_compound {
                                stack[d2].complen += 1;
                            } else {
                                stack[d2].compsplit = stack[d2].complen;
                            }
                            stack[d2].prefixdepth = prefix_depth::PFD_NOPREFIX;

                            // Set su_badflags.
                            let n2 =
                                crate::rs_nofold_len(fword, c_int::from(stack[d2].fidx), su_badptr);
                            let new_bflags = rs_badword_captype(
                                su_badptr.add(n2 as usize),
                                su_badptr.add(su_badlen as usize),
                            );
                            nvim_suginfo_set_badflags(su, new_bflags);

                            stack[d2].arridx = 0;

                            if !pbyts_ptr.is_null() {
                                byts = pbyts_ptr;
                                idxs = pidxs_ptr;
                                stack[d2].prefixdepth = prefix_depth::PFD_PREFIXTREE;
                                stack[d2].state = TrieWalkState::NoPrefix;
                            }
                        }
                    }
                }
            } // end STATE_START / STATE_NOPREFIX

            TrieWalkState::SplitUndo => {
                nvim_suginfo_set_badflags(su, c_int::from((*sp_raw).save_badflags));
                (*sp_raw).state = TrieWalkState::Start;
                byts = fbyts_ptr;
                idxs = fidxs_ptr;
            }

            TrieWalkState::EndNul => {
                nvim_suginfo_set_badflags(su, c_int::from((*sp_raw).save_badflags));
                let fidx = (*sp_raw).fidx as usize;
                if *fword.add(fidx) == 0 && (*sp_raw).tcharlen == 0 {
                    (*sp_raw).state = TrieWalkState::Del;
                    continue;
                }
                (*sp_raw).state = TrieWalkState::Plain;
                // FALLTHROUGH to STATE_PLAIN
                let arridx = (*sp_raw).arridx as usize;
                if (*sp_raw).curi as usize > *byts.add(arridx) as usize {
                    (*sp_raw).state = if (*sp_raw).fidx >= (*sp_raw).fidxtry {
                        TrieWalkState::Del
                    } else {
                        TrieWalkState::Final
                    };
                } else {
                    let arridx_cur = arridx + (*sp_raw).curi as usize;
                    (*sp_raw).curi += 1;
                    let c = *byts.add(arridx_cur);

                    let newscore = if c == *fword.add((*sp_raw).fidx as usize) as u8
                        || ((*sp_raw).tcharlen > 0 && (*sp_raw).isdiff != DiffType::None)
                    {
                        0
                    } else {
                        SCORES.subst
                    };

                    if (newscore == 0
                        || ((*sp_raw).fidx >= (*sp_raw).fidxtry
                            && (((*sp_raw).flags & try_state_flags::TSF_DIDDEL) == 0
                                || c != *fword.add((*sp_raw).delidx as usize) as u8)))
                        && try_deeper(d, &stack, newscore, su_maxscore)
                    {
                        go_deeper_inline(stack.as_mut_ptr(), d, newscore);
                        depth += 1;
                        let d2 = depth as usize;
                        let sp2 = &mut stack[d2];
                        if *fword.add(sp2.fidx as usize) != 0 {
                            sp2.fidx += 1;
                        }
                        tword[sp2.twordlen as usize] = c as char as u8;
                        sp2.twordlen += 1;
                        sp2.arridx = *idxs.add(arridx_cur) as u32;
                        if newscore == SCORES.subst {
                            sp2.isdiff = DiffType::Yes;
                        }
                        if sp2.tcharlen == 0 {
                            sp2.tcharidx = 0;
                            sp2.tcharlen = mb_byte2len(c) as u8;
                            sp2.fcharstart = sp2.fidx - 1;
                            sp2.isdiff = if newscore != 0 {
                                DiffType::Yes
                            } else {
                                DiffType::None
                            };
                        } else if sp2.isdiff == DiffType::Insert && sp2.fidx > 0 {
                            sp2.fidx -= 1;
                        }
                        sp2.tcharidx += 1;
                        if sp2.tcharidx == sp2.tcharlen {
                            if sp2.isdiff == DiffType::Yes {
                                sp2.fidx = sp2.fcharstart
                                    + crate::utfc_ptr2len(fword.add(sp2.fcharstart as usize)) as u8;
                                let tc = utf_ptr2char(
                                    tword
                                        .as_ptr()
                                        .add(sp2.twordlen as usize - sp2.tcharlen as usize)
                                        .cast::<c_char>(),
                                );
                                let fc = utf_ptr2char(fword.add(sp2.fcharstart as usize));
                                if utf_iscomposing_legacy(tc) && utf_iscomposing_legacy(fc) {
                                    sp2.score -= SCORES.subst - SCORES.subcomp;
                                } else if !soundfold
                                    && (*slang_ptr).sl_has_map
                                    && similar_chars(
                                        slang,
                                        utf_ptr2char(
                                            tword
                                                .as_ptr()
                                                .add(sp2.twordlen as usize - sp2.tcharlen as usize)
                                                .cast::<c_char>(),
                                        ),
                                        utf_ptr2char(fword.add(sp2.fcharstart as usize)),
                                    )
                                {
                                    sp2.score -= SCORES.subst - SCORES.similar;
                                }
                            } else if sp2.isdiff == DiffType::Insert && sp2.twordlen > sp2.tcharlen
                            {
                                let pp = tword
                                    .as_ptr()
                                    .add(sp2.twordlen as usize - sp2.tcharlen as usize);
                                let ci = utf_ptr2char(pp.cast::<c_char>());
                                if utf_iscomposing_legacy(ci) {
                                    sp2.score -= SCORES.ins - SCORES.inscomp;
                                } else {
                                    let pp2 = mb_ptr_back(
                                        tword.as_ptr().cast::<c_char>(),
                                        pp.cast::<c_char>(),
                                    );
                                    if ci == utf_ptr2char(pp2) {
                                        sp2.score -= SCORES.ins - SCORES.insdup;
                                    }
                                }
                            }
                            sp2.tcharlen = 0;
                        }
                    }
                }
            }

            TrieWalkState::Plain => {
                let arridx = (*sp_raw).arridx as usize;
                if (*sp_raw).curi as usize > *byts.add(arridx) as usize {
                    (*sp_raw).state = if (*sp_raw).fidx >= (*sp_raw).fidxtry {
                        TrieWalkState::Del
                    } else {
                        TrieWalkState::Final
                    };
                } else {
                    let arridx_cur = arridx + (*sp_raw).curi as usize;
                    (*sp_raw).curi += 1;
                    let c = *byts.add(arridx_cur);

                    let newscore = if c == *fword.add((*sp_raw).fidx as usize) as u8
                        || ((*sp_raw).tcharlen > 0 && (*sp_raw).isdiff != DiffType::None)
                    {
                        0
                    } else {
                        SCORES.subst
                    };

                    if (newscore == 0
                        || ((*sp_raw).fidx >= (*sp_raw).fidxtry
                            && (((*sp_raw).flags & try_state_flags::TSF_DIDDEL) == 0
                                || c != *fword.add((*sp_raw).delidx as usize) as u8)))
                        && try_deeper(d, &stack, newscore, su_maxscore)
                    {
                        go_deeper_inline(stack.as_mut_ptr(), d, newscore);
                        depth += 1;
                        let d2 = depth as usize;
                        let sp2 = &mut stack[d2];
                        if *fword.add(sp2.fidx as usize) != 0 {
                            sp2.fidx += 1;
                        }
                        tword[sp2.twordlen as usize] = c as char as u8;
                        sp2.twordlen += 1;
                        sp2.arridx = *idxs.add(arridx_cur) as u32;
                        if newscore == SCORES.subst {
                            sp2.isdiff = DiffType::Yes;
                        }
                        if sp2.tcharlen == 0 {
                            sp2.tcharidx = 0;
                            sp2.tcharlen = mb_byte2len(c) as u8;
                            sp2.fcharstart = sp2.fidx - 1;
                            sp2.isdiff = if newscore != 0 {
                                DiffType::Yes
                            } else {
                                DiffType::None
                            };
                        } else if sp2.isdiff == DiffType::Insert && sp2.fidx > 0 {
                            sp2.fidx -= 1;
                        }
                        sp2.tcharidx += 1;
                        if sp2.tcharidx == sp2.tcharlen {
                            if sp2.isdiff == DiffType::Yes {
                                sp2.fidx = sp2.fcharstart
                                    + crate::utfc_ptr2len(fword.add(sp2.fcharstart as usize)) as u8;
                                let tc = utf_ptr2char(
                                    tword
                                        .as_ptr()
                                        .add(sp2.twordlen as usize - sp2.tcharlen as usize)
                                        .cast::<c_char>(),
                                );
                                let fc = utf_ptr2char(fword.add(sp2.fcharstart as usize));
                                if utf_iscomposing_legacy(tc) && utf_iscomposing_legacy(fc) {
                                    sp2.score -= SCORES.subst - SCORES.subcomp;
                                } else if !soundfold
                                    && (*slang_ptr).sl_has_map
                                    && similar_chars(
                                        slang,
                                        utf_ptr2char(
                                            tword
                                                .as_ptr()
                                                .add(sp2.twordlen as usize - sp2.tcharlen as usize)
                                                .cast::<c_char>(),
                                        ),
                                        utf_ptr2char(fword.add(sp2.fcharstart as usize)),
                                    )
                                {
                                    sp2.score -= SCORES.subst - SCORES.similar;
                                }
                            } else if sp2.isdiff == DiffType::Insert && sp2.twordlen > sp2.tcharlen
                            {
                                let pp = tword
                                    .as_ptr()
                                    .add(sp2.twordlen as usize - sp2.tcharlen as usize);
                                let ci = utf_ptr2char(pp.cast::<c_char>());
                                if utf_iscomposing_legacy(ci) {
                                    sp2.score -= SCORES.ins - SCORES.inscomp;
                                } else {
                                    let pp2 = mb_ptr_back(
                                        tword.as_ptr().cast::<c_char>(),
                                        pp.cast::<c_char>(),
                                    );
                                    if ci == utf_ptr2char(pp2) {
                                        sp2.score -= SCORES.ins - SCORES.insdup;
                                    }
                                }
                            }
                            sp2.tcharlen = 0;
                        }
                    }
                }
            }

            TrieWalkState::Del => {
                if (*sp_raw).tcharlen > 0 {
                    (*sp_raw).state = TrieWalkState::Final;
                    continue;
                }
                (*sp_raw).state = TrieWalkState::InsPrep;
                (*sp_raw).curi = 1;

                let fidx = (*sp_raw).fidx as usize;
                let newscore = if soundfold && fidx == 0 && *fword.add(fidx) == b'*' as c_char {
                    2 * SCORES.del / 3
                } else {
                    SCORES.del
                };

                if *fword.add(fidx) != 0 && try_deeper(d, &stack, newscore, su_maxscore) {
                    go_deeper_inline(stack.as_mut_ptr(), d, newscore);
                    depth += 1;
                    let d2 = depth as usize;
                    stack[d2].flags |= try_state_flags::TSF_DIDDEL;
                    stack[d2].delidx = (*sp_raw).fidx;

                    let c = utf_ptr2char(fword.add(fidx));
                    let adv = crate::utfc_ptr2len(fword.add(fidx)) as u8;
                    stack[d2].fidx = stack[d2].fidx.wrapping_add(adv);
                    let new_fidx = stack[d2].fidx as usize;
                    if utf_iscomposing_legacy(c) {
                        stack[d2].score -= SCORES.del - SCORES.delcomp;
                    } else if c == utf_ptr2char(fword.add(new_fidx)) {
                        stack[d2].score -= SCORES.del - SCORES.deldup;
                    }
                }
            }

            TrieWalkState::InsPrep => {
                if (*sp_raw).flags & try_state_flags::TSF_DIDDEL != 0 {
                    (*sp_raw).state = TrieWalkState::Swap;
                    continue;
                }
                let n = (*sp_raw).arridx as usize;
                loop {
                    if (*sp_raw).curi as usize > *byts.add(n) as usize {
                        (*sp_raw).state = TrieWalkState::Swap;
                        break;
                    }
                    if *byts.add(n + (*sp_raw).curi as usize) != 0 {
                        (*sp_raw).state = TrieWalkState::Ins;
                        break;
                    }
                    (*sp_raw).curi += 1;
                }
            }

            TrieWalkState::Ins => {
                let n = (*sp_raw).arridx as usize;
                if (*sp_raw).curi as usize > *byts.add(n) as usize {
                    (*sp_raw).state = TrieWalkState::Swap;
                    continue;
                }
                let arridx_cur = n + (*sp_raw).curi as usize;
                (*sp_raw).curi += 1;

                // Bounds check
                if byts == fbyts_ptr && arridx_cur >= (*slang_ptr).sl_fbyts_len as usize {
                    crate::got_int = true;
                    continue;
                }

                let c = *byts.add(arridx_cur);
                let newscore = if soundfold && (*sp_raw).twordlen == 0 && c == b'*' {
                    2 * SCORES.ins / 3
                } else {
                    SCORES.ins
                };

                if c != *fword.add((*sp_raw).fidx as usize) as u8
                    && try_deeper(d, &stack, newscore, su_maxscore)
                {
                    go_deeper_inline(stack.as_mut_ptr(), d, newscore);
                    depth += 1;
                    let d2 = depth as usize;
                    let sp2 = &mut stack[d2];
                    tword[sp2.twordlen as usize] = c;
                    sp2.twordlen += 1;
                    sp2.arridx = *idxs.add(arridx_cur) as u32;
                    let fl = mb_byte2len(c);
                    if fl > 1 {
                        sp2.tcharlen = fl as u8;
                        sp2.tcharidx = 1;
                        sp2.isdiff = DiffType::Insert;
                    }
                    if fl == 1 && sp2.twordlen >= 2 && tword[sp2.twordlen as usize - 2] == c {
                        sp2.score -= SCORES.ins - SCORES.insdup;
                    }
                }
            }

            TrieWalkState::Swap => {
                let fidx = (*sp_raw).fidx as usize;
                let pp = fword.add(fidx);
                let c = *pp as u8;
                if c == 0 {
                    (*sp_raw).state = TrieWalkState::Final;
                    continue;
                }
                if !soundfold && !crate::rs_spell_iswordp(pp, stp_sal_curwin) {
                    (*sp_raw).state = TrieWalkState::RepIni;
                    continue;
                }
                let n = utf_ptr2len(pp) as usize;
                let c = utf_ptr2char(pp);
                let c2 = if *pp.add(n) == 0 {
                    0
                } else if !soundfold && !crate::rs_spell_iswordp(pp.add(n), stp_sal_curwin) {
                    c // don't swap non-word char
                } else {
                    utf_ptr2char(pp.add(n))
                };

                if c2 == 0 {
                    (*sp_raw).state = TrieWalkState::RepIni;
                    continue;
                }
                if c == c2 {
                    (*sp_raw).state = TrieWalkState::Swap3;
                    // FALLTHROUGH to Swap3 -- handled by next loop iteration
                    continue;
                }
                if try_deeper(d, &stack, SCORES.swap, su_maxscore) {
                    go_deeper_inline(stack.as_mut_ptr(), d, SCORES.swap);
                    (*sp_raw).state = TrieWalkState::Unswap;
                    depth += 1;
                    let fl = utf_char2len(c2) as usize;
                    // Swap: move c2 to start, put c after
                    std::ptr::copy(pp.add(n), pp, fl);
                    utf_char2bytes(c, pp.add(fl));
                    stack[depth as usize].fidxtry = (fidx + n + fl) as u8;
                } else {
                    (*sp_raw).state = TrieWalkState::RepIni;
                }
            }

            TrieWalkState::Unswap => {
                let fidx = (*sp_raw).fidx as usize;
                let pp = fword.add(fidx);
                let n = crate::utfc_ptr2len(pp) as usize;
                let c = utf_ptr2char(pp.add(n));
                let pp_tail = pp.add(n);
                let tail_len = crate::utfc_ptr2len(pp_tail) as usize;
                std::ptr::copy(pp, pp.add(tail_len), n);
                utf_char2bytes(c, pp);
                // FALLTHROUGH to Swap3
                (*sp_raw).state = TrieWalkState::Swap3;
                // Swap3 logic inline:
                let fidx2 = (*sp_raw).fidx as usize;
                let pp2 = fword.add(fidx2);
                let n2 = utf_ptr2len(pp2) as usize;
                let c2b = utf_ptr2char(pp2);
                let fl2 = utf_ptr2len(pp2.add(n2)) as usize;
                let c2c = utf_ptr2char(pp2.add(n2));
                let c3b =
                    if !soundfold && !crate::rs_spell_iswordp(pp2.add(n2 + fl2), stp_sal_curwin) {
                        c2b
                    } else {
                        utf_ptr2char(pp2.add(n2 + fl2))
                    };
                if c2b == c3b || c3b == 0 {
                    (*sp_raw).state = TrieWalkState::RepIni;
                    continue;
                }
                if try_deeper(d, &stack, SCORES.swap3, su_maxscore) {
                    go_deeper_inline(stack.as_mut_ptr(), d, SCORES.swap3);
                    (*sp_raw).state = TrieWalkState::Unswap3;
                    depth += 1;
                    let tl2 = utf_char2len(c3b) as usize;
                    std::ptr::copy(pp2.add(n2 + fl2), pp2, tl2);
                    utf_char2bytes(c2c, pp2.add(tl2));
                    utf_char2bytes(c2b, pp2.add(fl2 + tl2));
                    stack[depth as usize].fidxtry = (fidx2 + n2 + fl2 + tl2) as u8;
                } else {
                    (*sp_raw).state = TrieWalkState::RepIni;
                }
            }

            TrieWalkState::Swap3 => {
                let fidx = (*sp_raw).fidx as usize;
                let pp = fword.add(fidx);
                let n = utf_ptr2len(pp) as usize;
                let c = utf_ptr2char(pp);
                let fl = utf_ptr2len(pp.add(n)) as usize;
                let c2 = utf_ptr2char(pp.add(n));
                let c3 = if !soundfold && !crate::rs_spell_iswordp(pp.add(n + fl), stp_sal_curwin) {
                    c
                } else {
                    utf_ptr2char(pp.add(n + fl))
                };
                if c == c3 || c3 == 0 {
                    (*sp_raw).state = TrieWalkState::RepIni;
                    continue;
                }
                if try_deeper(d, &stack, SCORES.swap3, su_maxscore) {
                    go_deeper_inline(stack.as_mut_ptr(), d, SCORES.swap3);
                    (*sp_raw).state = TrieWalkState::Unswap3;
                    depth += 1;
                    let tl = utf_char2len(c3) as usize;
                    std::ptr::copy(pp.add(n + fl), pp, tl);
                    utf_char2bytes(c2, pp.add(tl));
                    utf_char2bytes(c, pp.add(fl + tl));
                    stack[depth as usize].fidxtry = (fidx + n + fl + tl) as u8;
                } else {
                    (*sp_raw).state = TrieWalkState::RepIni;
                }
            }

            TrieWalkState::Unswap3 => {
                // Undo STATE_SWAP3: "321" -> "123"
                let fidx = (*sp_raw).fidx as usize;
                let pp = fword.add(fidx);
                let n = crate::utfc_ptr2len(pp) as usize;
                let c2 = utf_ptr2char(pp.add(n));
                let fl = crate::utfc_ptr2len(pp.add(n)) as usize;
                let c = utf_ptr2char(pp.add(n + fl));
                let tl = crate::utfc_ptr2len(pp.add(n + fl)) as usize;
                std::ptr::copy(pp, pp.add(fl + tl), n);
                utf_char2bytes(c, pp);
                utf_char2bytes(c2, pp.add(tl));
                let pp_mid = pp.add(tl);

                if !soundfold && !crate::rs_spell_iswordp(pp_mid, stp_sal_curwin) {
                    (*sp_raw).state = TrieWalkState::RepIni;
                    continue;
                }

                // Rotate three characters left: "123" -> "231"
                if try_deeper(d, &stack, SCORES.swap3, su_maxscore) {
                    go_deeper_inline(stack.as_mut_ptr(), d, SCORES.swap3);
                    (*sp_raw).state = TrieWalkState::Unrot3L;
                    depth += 1;
                    let pp3 = fword.add(fidx);
                    let n3 = utf_ptr2len(pp3) as usize;
                    let c3 = utf_ptr2char(pp3);
                    let fl3 = utf_ptr2len(pp3.add(n3)) as usize;
                    let fl3b = utf_ptr2len(pp3.add(n3 + fl3)) as usize;
                    let total_fl = fl3 + fl3b;
                    std::ptr::copy(pp3.add(n3), pp3, total_fl);
                    utf_char2bytes(c3, pp3.add(total_fl));
                    stack[depth as usize].fidxtry = (fidx + n3 + total_fl) as u8;
                } else {
                    (*sp_raw).state = TrieWalkState::RepIni;
                }
            }

            TrieWalkState::Unrot3L => {
                // Undo ROT3L: "231" -> "123"
                let fidx = (*sp_raw).fidx as usize;
                let pp = fword.add(fidx);
                let n1 = crate::utfc_ptr2len(pp) as usize;
                let n2 = crate::utfc_ptr2len(pp.add(n1)) as usize;
                let n = n1 + n2;
                let c = utf_ptr2char(pp.add(n));
                let tl = crate::utfc_ptr2len(pp.add(n)) as usize;
                std::ptr::copy(pp, pp.add(tl), n);
                utf_char2bytes(c, pp);

                // Rotate three bytes right: "123" -> "312"
                if try_deeper(d, &stack, SCORES.swap3, su_maxscore) {
                    go_deeper_inline(stack.as_mut_ptr(), d, SCORES.swap3);
                    (*sp_raw).state = TrieWalkState::Unrot3R;
                    depth += 1;
                    let pp2 = fword.add(fidx);
                    let na = utf_ptr2len(pp2) as usize;
                    let nb = utf_ptr2len(pp2.add(na)) as usize;
                    let nc = na + nb;
                    let c2 = utf_ptr2char(pp2.add(nc));
                    let tl2 = utf_ptr2len(pp2.add(nc)) as usize;
                    std::ptr::copy(pp2.add(tl2), pp2, nc);
                    utf_char2bytes(c2, pp2.add(nc));
                    stack[depth as usize].fidxtry = (fidx + nc + tl2) as u8;
                } else {
                    (*sp_raw).state = TrieWalkState::RepIni;
                }
            }

            TrieWalkState::Unrot3R => {
                // Undo ROT3R: "312" -> "123"
                let fidx = (*sp_raw).fidx as usize;
                let pp = fword.add(fidx);
                let c = utf_ptr2char(pp);
                let tl = crate::utfc_ptr2len(pp) as usize;
                let n1 = crate::utfc_ptr2len(pp.add(tl)) as usize;
                let n2 = crate::utfc_ptr2len(pp.add(tl + n1)) as usize;
                let n = n1 + n2;
                std::ptr::copy(pp.add(tl), pp, n);
                utf_char2bytes(c, pp.add(n));
                // FALLTHROUGH to RepIni
                (*sp_raw).state = TrieWalkState::RepIni;
                // RepIni logic inline:
                if ((*lp).lp_replang.is_null() && !soundfold)
                    || (*sp_raw).score + SCORES.rep >= su_maxscore
                    || (*sp_raw).fidx < (*sp_raw).fidxtry
                {
                    (*sp_raw).state = TrieWalkState::Final;
                    continue;
                }
                let fchar = *fword.add((*sp_raw).fidx as usize) as u8;
                (*sp_raw).curi = if soundfold {
                    (*slang_ptr).sl_repsal_first[fchar as usize]
                } else {
                    (*(*lp).lp_replang).sl_rep_first[fchar as usize]
                };
                if (*sp_raw).curi < 0 {
                    (*sp_raw).state = TrieWalkState::Final;
                    continue;
                }
                (*sp_raw).state = TrieWalkState::Rep;
                // FALLTHROUGH to Rep -- will be handled next iteration
            }

            TrieWalkState::RepIni => {
                if ((*lp).lp_replang.is_null() && !soundfold)
                    || (*sp_raw).score + SCORES.rep >= su_maxscore
                    || (*sp_raw).fidx < (*sp_raw).fidxtry
                {
                    (*sp_raw).state = TrieWalkState::Final;
                    continue;
                }
                let fchar = *fword.add((*sp_raw).fidx as usize) as u8;
                (*sp_raw).curi = if soundfold {
                    (*slang_ptr).sl_repsal_first[fchar as usize]
                } else {
                    (*(*lp).lp_replang).sl_rep_first[fchar as usize]
                };
                if (*sp_raw).curi < 0 {
                    (*sp_raw).state = TrieWalkState::Final;
                    continue;
                }
                (*sp_raw).state = TrieWalkState::Rep;
                // FALLTHROUGH
                let gap_ptr = if soundfold {
                    std::ptr::addr_of!((*slang_ptr).sl_repsal)
                } else {
                    std::ptr::addr_of!((*(*lp).lp_replang).sl_rep)
                };
                let gap_len = (*gap_ptr).ga_len as i16;
                let p_fword = fword.add((*sp_raw).fidx as usize);
                let ftp_base = (*gap_ptr).ga_data.cast::<FromtoC>();
                while (*sp_raw).curi < gap_len {
                    let ftp = ftp_base.add((*sp_raw).curi as usize);
                    (*sp_raw).curi += 1;
                    if *(*ftp).ft_from != *p_fword {
                        (*sp_raw).curi = gap_len;
                        break;
                    }
                    let from_len = c_strlen((*ftp).ft_from);
                    if c_strncmp((*ftp).ft_from, p_fword, from_len)
                        && try_deeper(d, &stack, SCORES.rep, su_maxscore)
                    {
                        go_deeper_inline(stack.as_mut_ptr(), d, SCORES.rep);
                        (*sp_raw).state = TrieWalkState::RepUndo;
                        depth += 1;
                        let fl = from_len;
                        let to_len = c_strlen((*ftp).ft_to);
                        let tl = to_len;
                        if fl != tl {
                            strmove(p_fword.add(tl), p_fword.add(fl));
                            repextra += tl as c_int - fl as c_int;
                        }
                        std::ptr::copy_nonoverlapping((*ftp).ft_to, p_fword, tl);
                        stack[depth as usize].fidxtry = ((*sp_raw).fidx as usize + tl) as u8;
                        stack[depth as usize].tcharlen = 0;
                        break;
                    }
                }
                if (*sp_raw).curi >= gap_len && (*sp_raw).state == TrieWalkState::Rep {
                    (*sp_raw).state = TrieWalkState::Final;
                }
            }

            TrieWalkState::Rep => {
                let gap_ptr = if soundfold {
                    std::ptr::addr_of!((*slang_ptr).sl_repsal)
                } else {
                    std::ptr::addr_of!((*(*lp).lp_replang).sl_rep)
                };
                let gap_len = (*gap_ptr).ga_len as i16;
                let p_fword = fword.add((*sp_raw).fidx as usize);
                let ftp_base = (*gap_ptr).ga_data.cast::<FromtoC>();
                while (*sp_raw).curi < gap_len {
                    let ftp = ftp_base.add((*sp_raw).curi as usize);
                    (*sp_raw).curi += 1;
                    if *(*ftp).ft_from != *p_fword {
                        (*sp_raw).curi = gap_len;
                        break;
                    }
                    let from_len = c_strlen((*ftp).ft_from);
                    if c_strncmp((*ftp).ft_from, p_fword, from_len)
                        && try_deeper(d, &stack, SCORES.rep, su_maxscore)
                    {
                        go_deeper_inline(stack.as_mut_ptr(), d, SCORES.rep);
                        (*sp_raw).state = TrieWalkState::RepUndo;
                        depth += 1;
                        let fl = from_len;
                        let to_len = c_strlen((*ftp).ft_to);
                        let tl = to_len;
                        if fl != tl {
                            strmove(p_fword.add(tl), p_fword.add(fl));
                            repextra += tl as c_int - fl as c_int;
                        }
                        std::ptr::copy_nonoverlapping((*ftp).ft_to, p_fword, tl);
                        stack[depth as usize].fidxtry = ((*sp_raw).fidx as usize + tl) as u8;
                        stack[depth as usize].tcharlen = 0;
                        break;
                    }
                }
                if (*sp_raw).curi >= gap_len && (*sp_raw).state == TrieWalkState::Rep {
                    (*sp_raw).state = TrieWalkState::Final;
                }
            }

            TrieWalkState::RepUndo => {
                let gap_ptr = if soundfold {
                    std::ptr::addr_of!((*slang_ptr).sl_repsal)
                } else {
                    std::ptr::addr_of!((*(*lp).lp_replang).sl_rep)
                };
                let ftp = (*gap_ptr)
                    .ga_data
                    .cast::<FromtoC>()
                    .add((*sp_raw).curi as usize - 1);
                let fl = c_strlen((*ftp).ft_from);
                let tl = c_strlen((*ftp).ft_to);
                let p_fword = fword.add((*sp_raw).fidx as usize);
                if fl != tl {
                    strmove(p_fword.add(fl), p_fword.add(tl));
                    repextra -= tl as c_int - fl as c_int;
                }
                std::ptr::copy_nonoverlapping((*ftp).ft_from, p_fword, fl);
                (*sp_raw).state = TrieWalkState::Rep;
            }

            TrieWalkState::Final => {
                // Did all states at this level, go up one level.
                depth -= 1;

                if depth >= 0 && stack[depth as usize].prefixdepth == prefix_depth::PFD_PREFIXTREE {
                    byts = pbyts_ptr;
                    idxs = pidxs_ptr;
                }

                breakcheckcount -= 1;
                if breakcheckcount == 0 {
                    os_breakcheck();
                    breakcheckcount = 1000;
                    if timeout > 0 && profile_passed_limit(time_limit) {
                        crate::got_int = true;
                    }
                }
            }
        } // end match
    } // end while
}

// Wrappers needed by rs_suggest_trie_walk to call Rust-exported functions
// without going through the C ABI (direct calls within the same crate).

// =============================================================================
// Phase 1: add_suggestion, add_banned, rescore_one, rescore_suggestions
// =============================================================================

extern "C" {
    fn nvim_suginfo_get_sfmaxscore(su: *mut std::ffi::c_void) -> c_int;
    fn nvim_suginfo_set_sfmaxscore(su: *mut std::ffi::c_void, v: c_int);
    fn nvim_suginfo_set_maxscore(su: *mut std::ffi::c_void, v: c_int);
    fn nvim_suginfo_get_banned(su: *mut std::ffi::c_void) -> *mut crate::HashtabRaw;
    fn nvim_suginfo_get_sal_badword(su: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_suginfo_get_sallang(su: *mut std::ffi::c_void) -> *mut crate::SlangRaw;
    #[allow(dead_code)]
    fn nvim_suginfo_get_badflags(su: *mut std::ffi::c_void) -> c_int;
    #[allow(dead_code)]
    fn nvim_suginfo_set_badflags(su: *mut std::ffi::c_void, v: c_int);
    #[link_name = "hash_hash"]
    fn suggest_hash_hash(key: *const c_char) -> usize;
    #[link_name = "hash_lookup"]
    fn suggest_hash_lookup(
        ht: *const crate::HashtabRaw,
        key: *const c_char,
        key_len: usize,
        hash: usize,
    ) -> *mut crate::HashitemRaw;
    #[link_name = "hash_add_item"]
    fn suggest_hash_add_item(
        ht: *mut crate::HashtabRaw,
        hi: *mut crate::HashitemRaw,
        key: *mut c_char,
        hash: usize,
    );
    #[link_name = "xmalloc"]
    fn suggest_xmalloc(size: usize) -> *mut std::ffi::c_void;
    #[link_name = "hash_removed"]
    static suggest_hash_removed: c_char;
}

/// Rust implementation of C `add_suggestion`.
///
/// Adds a suggestion to gap (either su_ga or su_sga), deduplicating and
/// pruning overflow via rs_cleanup_suggestions.
///
/// # Safety
/// All pointers must be valid. `su` must be a valid suginfo_T*.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::used_underscore_binding)]
pub unsafe extern "C" fn rs_add_suggestion(
    su: *mut std::ffi::c_void,
    gap: *mut crate::GArrayRaw,
    goodword: *const c_char,
    badlenarg: c_int,
    score: c_int,
    altscore: c_int,
    had_bonus: bool,
    slang: *mut std::ffi::c_void,
    maxsf: bool,
) {
    // Minimize badlen: walk back while trailing chars match.
    let goodword_len = libc_strlen(goodword);
    let mut pgood = goodword.add(goodword_len); // points past NUL
    let su_badptr = nvim_suginfo_get_badptr(su);
    let mut pbad = su_badptr.add(badlenarg as usize);

    let mut goodlen: c_int;
    let mut badlen: c_int;
    loop {
        goodlen = pbad_diff(pgood, goodword);
        badlen = pbad_diff(pbad, su_badptr);
        if goodlen <= 0 || badlen <= 0 {
            break;
        }
        // MB_PTR_BACK: walk one character back
        pgood = mb_ptr_back(goodword, pgood);
        pbad = mb_ptr_back(su_badptr, pbad);
        if utf_ptr2char(pgood) != utf_ptr2char(pbad) {
            break;
        }
    }

    if badlen == 0 && goodlen == 0 {
        return;
    }

    // Check for duplicate suggestion.
    let mut found_idx: c_int = -1;
    let gap_len = (*gap).ga_len as usize;
    if gap_len > 0 {
        let stp_arr = (*gap).ga_data.cast::<CSuggestT>();
        for i in 0..gap_len {
            let stp = stp_arr.add(i);
            if (*stp).st_wordlen == goodlen
                && (*stp).st_orglen == badlen
                && strncmp_n((*stp).st_word, goodword, goodlen as usize)
            {
                // Found a duplicate.
                if (*stp).st_slang.is_null() {
                    (*stp).st_slang = slang;
                }

                let new_score = score;
                let new_altscore = altscore;
                let new_had_bonus = had_bonus;

                if (*stp).st_had_bonus != had_bonus {
                    if had_bonus {
                        // existing entry needs rescore
                        rs_rescore_one(su, stp);
                    } else {
                        // new entry needs rescore: use a temporary CSuggestT
                        let mut tmp = CSuggestT {
                            st_word: (*stp).st_word,
                            st_wordlen: (*stp).st_wordlen,
                            st_orglen: badlen,
                            st_score: new_score,
                            st_altscore: new_altscore,
                            st_had_bonus: new_had_bonus,
                            st_salscore: false,
                            _pad: [0u8; 6],
                            st_slang: (*stp).st_slang,
                        };
                        rs_rescore_one(su, &raw mut tmp);
                        // update new_score/new_altscore from rescored tmp
                        let rescored_score = tmp.st_score;
                        let rescored_altscore = tmp.st_altscore;
                        let rescored_had_bonus = tmp.st_had_bonus;
                        if (*stp).st_score > rescored_score {
                            (*stp).st_score = rescored_score;
                            (*stp).st_altscore = rescored_altscore;
                            (*stp).st_had_bonus = rescored_had_bonus;
                        }
                        found_idx = i as c_int;
                        break;
                    }
                }

                if (*stp).st_score > new_score {
                    (*stp).st_score = new_score;
                    (*stp).st_altscore = new_altscore;
                    (*stp).st_had_bonus = new_had_bonus;
                }
                found_idx = i as c_int;
                break;
            }
        }
    }

    if found_idx < 0 {
        // Grow array by 1 and append.
        ga_grow_suggest(gap, 1);
        let stp = (*gap)
            .ga_data
            .cast::<CSuggestT>()
            .add((*gap).ga_len as usize);
        (*stp).st_word = xmemdupz_rs(goodword, goodlen as usize);
        (*stp).st_wordlen = goodlen;
        (*stp).st_score = score;
        (*stp).st_altscore = altscore;
        (*stp).st_had_bonus = had_bonus;
        (*stp).st_orglen = badlen;
        (*stp).st_salscore = false;
        (*stp)._pad = [0u8; 6];
        (*stp).st_slang = slang;
        (*gap).ga_len += 1;

        // If too many suggestions, cleanup.
        let gap_len_now = (*gap).ga_len;
        let maxcount = nvim_suginfo_get_maxcount(su);
        let sug_max_count = if maxcount < 130 {
            150 + 50
        } else {
            maxcount + 20 + 50
        };
        if gap_len_now > sug_max_count {
            let sug_clean = if maxcount < 130 { 150 } else { maxcount + 20 };
            if maxsf {
                let cur = nvim_suginfo_get_sfmaxscore(su);
                let new_max = rs_cleanup_suggestions(
                    (*gap).ga_data.cast::<CSuggestT>(),
                    &raw mut (*gap).ga_len,
                    cur,
                    sug_clean,
                );
                nvim_suginfo_set_sfmaxscore(su, new_max);
            } else {
                let cur = nvim_suginfo_get_maxscore(su);
                let new_max = rs_cleanup_suggestions(
                    (*gap).ga_data.cast::<CSuggestT>(),
                    &raw mut (*gap).ga_len,
                    cur,
                    sug_clean,
                );
                nvim_suginfo_set_maxscore(su, new_max);
            }
        }
    }
}

/// Compute pointer difference in bytes (equivalent to `pgood - goodword`).
#[inline]
unsafe fn pbad_diff(p: *const c_char, base: *const c_char) -> c_int {
    p.offset_from(base) as c_int
}

/// MB_PTR_BACK: move pointer one multibyte character back.
/// Scans forward from `base` to find the previous char start before `p`.
#[inline]
unsafe fn mb_ptr_back(base: *const c_char, p: *const c_char) -> *const c_char {
    if p <= base {
        return base;
    }
    // Walk back: find the start of the previous UTF-8 character.
    // A continuation byte has the top bit set and bit 6 clear (0x80..0xBF).
    let mut q = p.sub(1);
    while q > base && ((*q as u8) & 0xC0) == 0x80 {
        q = q.sub(1);
    }
    q
}

/// strncmp that takes a length and returns bool.
#[inline]
unsafe fn strncmp_n(a: *const c_char, b: *const c_char, n: usize) -> bool {
    std::slice::from_raw_parts(a.cast::<u8>(), n) == std::slice::from_raw_parts(b.cast::<u8>(), n)
}

/// Allocate a NUL-terminated copy of `data[..len]`.
#[inline]
unsafe fn xmemdupz_rs(data: *const c_char, len: usize) -> *mut c_char {
    let buf = suggest_xmalloc(len + 1).cast::<c_char>();
    std::ptr::copy_nonoverlapping(data, buf, len);
    *buf.add(len) = 0;
    buf
}

/// Returns true if a hashitem is empty (unused or removed).
#[inline]
unsafe fn suggest_hashitem_empty(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq(
            (*hi).hi_key,
            std::ptr::addr_of!(suggest_hash_removed).cast_mut(),
        )
}

/// Rust implementation of C `add_banned`.
///
/// Adds `word` to su->su_banned hashtable.
///
/// # Safety
/// `su` and `word` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_add_banned(su: *mut std::ffi::c_void, word: *mut c_char) {
    let ht = nvim_suginfo_get_banned(su);
    let word_len = libc_strlen(word);
    let hash = suggest_hash_hash(word);
    let hi = suggest_hash_lookup(ht, word, word_len, hash);
    if !suggest_hashitem_empty(hi) {
        return;
    }
    let s = xmemdupz_rs(word, word_len);
    suggest_hash_add_item(ht, hi, s, hash);
}

/// Rust implementation of C `rescore_one`.
///
/// Recomputes the score for one suggestion using sound-alike scoring.
///
/// # Safety
/// `su` and `stp` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_rescore_one(su: *mut std::ffi::c_void, stp: *mut CSuggestT) {
    let slang_ptr = (*stp).st_slang.cast::<crate::SlangRaw>();
    if slang_ptr.is_null() {
        return;
    }
    // Check !GA_EMPTY(&slang->sl_sal)
    let sl_sal_len = (*slang_ptr).sl_sal.ga_len;
    if sl_sal_len == 0 {
        return;
    }
    if (*stp).st_had_bonus {
        return;
    }

    let slang = SlangHandle(slang_ptr);
    let su_sallang = nvim_suginfo_get_sallang(su);

    let mut sal_badword = [0u8; MAXWLEN];
    let p: *const c_char = if std::ptr::eq(slang_ptr, su_sallang) {
        nvim_suginfo_get_sal_badword(su)
    } else {
        let fbadword = nvim_suginfo_get_fbadword(su);
        crate::rs_spell_soundfold(
            slang_ptr,
            fbadword.cast_mut(),
            true,
            sal_badword.as_mut_ptr().cast::<c_char>(),
        );
        sal_badword.as_ptr().cast::<c_char>()
    };

    let su_badptr = nvim_suginfo_get_badptr(su);
    let su_badlen = nvim_suginfo_get_badlen(su);
    let alt = rs_stp_sal_score(stp, su_badptr, su_badlen, slang, p);
    (*stp).st_altscore = if alt == SCORE_MAXMAX { SCORE_BIG } else { alt };
    // RESCORE(word_score, sound_score) = (3 * word + sound) / 4
    (*stp).st_score = (3 * (*stp).st_score + (*stp).st_altscore) / 4;
    (*stp).st_had_bonus = true;
}

/// Rust implementation of C `rescore_suggestions`.
///
/// Recomputes scores for all suggestions in su->su_ga.
///
/// # Safety
/// `su` must be a valid pointer to a suginfo_T.
#[no_mangle]
pub unsafe extern "C" fn rs_rescore_suggestions(su: *mut std::ffi::c_void) {
    let su_sallang = nvim_suginfo_get_sallang(su);
    if su_sallang.is_null() {
        return;
    }
    let ga = nvim_suginfo_get_ga(su);
    let ga_len = (*ga).ga_len as usize;
    let ga_data = (*ga).ga_data.cast::<CSuggestT>();
    for i in 0..ga_len {
        rs_rescore_one(su, ga_data.add(i));
    }
}

// =============================================================================
// Phase 3: add_sound_suggest, score_comp_sal
// =============================================================================

extern "C" {
    fn nvim_spellsug_get_sps_flags() -> c_int;
    fn internal_error(msg: *const c_char);
    #[link_name = "ml_get_buf"]
    fn sound_ml_get_buf(buf: *mut std::ffi::c_void, lnum: i32) -> *mut c_char;
    #[link_name = "xstrdup"]
    fn sound_xstrdup(s: *const c_char) -> *mut c_char;
}

/// Offset of `sft_word` within `sftword_T`: 2 bytes (i16 sft_score).
const SFTWORD_WORD_OFFSET: usize = 2;

/// Given a pointer to `hi_key` (which points into `sft_word`), recover
/// the pointer to the containing `sftword_T`.
/// Safety: xmalloc returns at least 8-byte aligned memory, so the i16 field
/// at offset 0 is always properly aligned.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
unsafe fn hi_key_to_sft(hi_key: *mut c_char) -> *mut SftWord {
    hi_key
        .cast::<u8>()
        .sub(SFTWORD_WORD_OFFSET)
        .cast::<SftWord>()
}

/// Rust equivalent of C `sftword_T`: layout-compatible struct for
/// soundfold dedup hash entries. The `sft_word` field is a flexible
/// array; we allocate extra bytes beyond `sft_score`.
#[repr(C)]
struct SftWord {
    sft_score: i16,
    // sft_word[] follows here in allocated memory
}

/// Check if a `hashitem_T` is empty (NULL key or removed).
#[inline]
unsafe fn sound_hi_empty(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq(
            (*hi).hi_key,
            std::ptr::addr_of!(suggest_hash_removed).cast_mut(),
        )
}

/// Rust implementation of C `add_sound_suggest`.
///
/// Finds original words for a soundfolded match and adds them as suggestions.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn rs_add_sound_suggest(
    su: *mut std::ffi::c_void,
    goodword: *mut c_char,
    score: c_int,
    lp: *const crate::LangpT,
) {
    let slang_ptr = (*lp).lp_slang;
    let slang = SlangHandle(slang_ptr);

    // It's very well possible that the same soundfold word is found several
    // times with different scores.  Only do words with a better score.
    let hash = suggest_hash_hash(goodword);
    let goodword_len = c_strlen(goodword);
    let hi = suggest_hash_lookup(
        std::ptr::addr_of!((*slang_ptr).sl_sounddone),
        goodword,
        goodword_len,
        hash,
    );

    let sft: *mut SftWord;
    if sound_hi_empty(hi) {
        // New entry: allocate sftword_T + word bytes.
        let alloc_size = SFTWORD_WORD_OFFSET + goodword_len + 1;
        let mem = suggest_xmalloc(alloc_size).cast::<u8>();
        sft = mem.cast::<SftWord>();
        (*sft).sft_score = score as i16;
        // Copy word into sft_word[]
        std::ptr::copy_nonoverlapping(
            goodword.cast::<u8>(),
            mem.add(SFTWORD_WORD_OFFSET),
            goodword_len + 1,
        );
        let word_ptr = mem.add(SFTWORD_WORD_OFFSET).cast::<c_char>();
        suggest_hash_add_item(
            std::ptr::addr_of_mut!((*slang_ptr).sl_sounddone),
            hi,
            word_ptr,
            hash,
        );
    } else {
        sft = hi_key_to_sft((*hi).hi_key);
        if score >= c_int::from((*sft).sft_score) {
            return;
        }
        (*sft).sft_score = score as i16;
    }
    let _ = sft; // used above

    // Find the word nr in the soundfold tree.
    let sfwordnr = rs_soundfold_find(SlangHandle(slang_ptr), goodword);
    if sfwordnr < 0 {
        internal_error(c"add_sound_suggest()".as_ptr());
        return;
    }

    // Go over the list of good words that produce this soundfold word.
    let nrline = sound_ml_get_buf((*slang_ptr).sl_sugbuf, sfwordnr + 1);
    let mut nrline_ptr = nrline as *const u8;
    let mut orgnr: c_int = 0;

    while *nrline_ptr != 0 {
        // Read word number offset (variable-length encoding).
        orgnr += crate::rs_bytes2offset(std::ptr::addr_of_mut!(nrline_ptr));

        let byts = (*slang_ptr).sl_fbyts;
        let idxs = (*slang_ptr).sl_fidxs;

        // Look up word "orgnr" in the trie.
        let mut n: c_int = 0;
        let mut wordcount: c_int = 0;
        let mut wlen = 0usize;
        let mut i: c_int = 1;
        let mut theword = [0u8; MAXWLEN];
        let mut found_badword = false;

        'trie: while wlen < MAXWLEN - 3 {
            i = 1;
            if wordcount == orgnr && *byts.add(n as usize + 1) == 0 {
                break; // found end of word
            }
            if *byts.add(n as usize + 1) == 0 {
                wordcount += 1;
            }

            // Skip over NUL bytes.
            while *byts.add(n as usize + i as usize) == 0 {
                if i > c_int::from(*byts.add(n as usize)) {
                    // Safety check: write "BAD" to theword.
                    let blen = wlen.min(MAXWLEN - 4);
                    theword[blen] = b'B';
                    theword[blen + 1] = b'A';
                    theword[blen + 2] = b'D';
                    wlen = blen + 3;
                    found_badword = true;
                    break 'trie;
                }
                i += 1;
            }

            // Find the sibling with the word.
            while i < c_int::from(*byts.add(n as usize)) {
                let wc = *idxs.add(*idxs.add(n as usize + i as usize) as usize); // word count under this byte
                if wordcount + wc > orgnr {
                    break;
                }
                wordcount += wc;
                i += 1;
            }

            theword[wlen] = *byts.add(n as usize + i as usize);
            n = *idxs.add(n as usize + i as usize);
            wlen += 1;
        }

        if !found_badword {
            theword[wlen] = 0;
        }

        let theword_ptr = theword.as_ptr().cast::<c_char>();
        let su_badflags = nvim_suginfo_get_badflags(su);
        let su_badword = nvim_suginfo_get_badword(su);
        let su_badlen = nvim_suginfo_get_badlen(su);
        let su_maxscore = nvim_suginfo_get_maxscore(su);
        let su_sfmaxscore = nvim_suginfo_get_sfmaxscore(su);
        let sps_flags = nvim_spellsug_get_sps_flags();

        // Go over the possible flags and regions.
        while i <= c_int::from(*byts.add(n as usize)) && *byts.add(n as usize + i as usize) == 0 {
            let mut cword = [0u8; MAXWLEN];
            let flags = *idxs.add(n as usize + i as usize);
            i += 1;

            // Skip words with the NOSUGGEST flag.
            if flags & WF_NOSUGGEST != 0 {
                continue;
            }

            let p: *const c_char;
            if flags & WF_KEEPCAP != 0 {
                // Must find the word in the keep-case tree.
                rs_find_keepcap_word(
                    slang,
                    theword_ptr.cast_mut(),
                    cword.as_mut_ptr().cast::<c_char>(),
                );
                p = cword.as_ptr().cast::<c_char>();
            } else {
                let effective_flags = flags | su_badflags;
                if (effective_flags & WF_CAPMASK) != 0 {
                    crate::rs_make_case_word(
                        theword_ptr,
                        cword.as_mut_ptr().cast::<c_char>(),
                        effective_flags,
                    );
                    p = cword.as_ptr().cast::<c_char>();
                } else {
                    p = theword_ptr;
                }
            }

            // Add the suggestion.
            if sps_flags & SPS_DOUBLE != 0 {
                if score <= su_maxscore {
                    rs_add_suggestion(
                        su,
                        nvim_suginfo_get_sga(su),
                        p,
                        su_badlen,
                        score,
                        0,
                        false,
                        (*lp).lp_sallang.cast(),
                        false,
                    );
                }
            } else {
                // Penalty for words in another region.
                let mut goodscore: c_int = if (flags & WF_REGION != 0)
                    && (((flags as u32) >> 16) & ((*lp).lp_region as u32)) == 0
                {
                    crate::SCORE_REGION
                } else {
                    0
                };

                // Small penalty for changing case of first letter.
                let gc = crate::utf_ptr2char(p);
                if crate::spell_isupper(gc) {
                    let bc = crate::utf_ptr2char(su_badword);
                    if !crate::spell_isupper(bc) && spell_tofold(bc) != spell_tofold(gc) {
                        goodscore += SCORES.icase / 2;
                    }
                }

                // Compute edit score with limit.
                let limit = maxscore_for_rs(su_sfmaxscore - goodscore, score);
                if limit > SCORE_LIMITMAX {
                    goodscore += rs_spell_edit_score(slang, su_badword, p);
                } else {
                    goodscore += rs_spell_edit_score_limit(slang, su_badword, p, limit);
                }

                if goodscore < SCORE_MAXMAX {
                    goodscore = rs_score_wordcount_adj(slang, goodscore, p, false);
                    goodscore = rescore_rs(goodscore, score);
                    if goodscore <= su_sfmaxscore {
                        rs_add_suggestion(
                            su,
                            nvim_suginfo_get_ga(su),
                            p,
                            su_badlen,
                            goodscore,
                            score,
                            true,
                            (*lp).lp_sallang.cast(),
                            true,
                        );
                    }
                }
            }
        }
    }
}

/// MAXSCORE(word_score, sound_score): max word score given max total score.
/// MAXSCORE = su_sfmaxscore - goodscore, where sound score's weight is 1/4.
/// C: #define MAXSCORE(wscore, sscore)  ((wscore) * 3 / 4 + (sscore) / 4)
/// But in add_sound_suggest the formula is: limit = MAXSCORE(su_sfmaxscore - goodscore, score)
/// which is: (sfmaxscore - goodscore) * 3 / 4 + score / 4
#[inline]
fn maxscore_for_rs(sfmax_minus_good: c_int, sound_score: c_int) -> c_int {
    sfmax_minus_good * 3 / 4 + sound_score / 4
}

/// RESCORE(word_score, sound_score) = (3 * word_score + sound_score) / 4
#[inline]
fn rescore_rs(word_score: c_int, sound_score: c_int) -> c_int {
    (3 * word_score + sound_score) / 4
}

/// WF_CAPMASK: any capitalization flags that require make_case_word.
const WF_CAPMASK: c_int = WF_ONECAP | WF_ALLCAP | WF_KEEPCAP | WF_MIXCAP;

/// Rust implementation of C `score_comp_sal`.
///
/// Computes sound-alike scores for suggestions in su_ga and adds to su_sga.
///
/// # Safety
/// `su` must be a valid pointer to a suginfo_T.
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_score_comp_sal(su: *mut std::ffi::c_void) {
    let mut badsound = [0u8; MAXWLEN];

    let su_ga = nvim_suginfo_get_ga(su);
    let su_sga = nvim_suginfo_get_sga(su);
    let su_fbadword = nvim_suginfo_get_fbadword(su);
    let su_badlen = nvim_suginfo_get_badlen(su);

    ga_grow_suggest(su_sga, (*su_ga).ga_len);

    let langp_ga = nvim_win_get_b_langp(stp_sal_curwin);
    let langp_len = (*langp_ga).ga_len;

    for lpi in 0..langp_len as usize {
        let lp = (*langp_ga).ga_data.cast::<crate::LangpT>().add(lpi);
        let slang_ptr = (*lp).lp_slang;
        if (*slang_ptr).sl_sal.ga_len == 0 {
            continue;
        }

        // Soundfold the bad word.
        crate::rs_spell_soundfold(
            slang_ptr,
            su_fbadword.cast_mut(),
            true,
            badsound.as_mut_ptr().cast::<c_char>(),
        );

        let ga_len = (*su_ga).ga_len as usize;
        let ga_data = (*su_ga).ga_data.cast::<CSuggestT>();

        for i in 0..ga_len {
            let stp = ga_data.add(i);
            let score = rs_stp_sal_score(
                stp,
                su_fbadword,
                su_badlen,
                SlangHandle(slang_ptr),
                badsound.as_ptr().cast(),
            );
            if score < SCORE_MAXMAX {
                // Add to su_sga.
                let sstp = (*su_sga)
                    .ga_data
                    .cast::<CSuggestT>()
                    .add((*su_sga).ga_len as usize);
                (*sstp).st_word = sound_xstrdup((*stp).st_word);
                (*sstp).st_wordlen = (*stp).st_wordlen;
                (*sstp).st_score = score;
                (*sstp).st_altscore = 0;
                (*sstp).st_orglen = (*stp).st_orglen;
                (*su_sga).ga_len += 1;
            }
        }
        break; // Only first language with sal
    }
}

// =============================================================================
// Phase 4: soundalike helpers and suggest_try_change/special
// =============================================================================

extern "C" {
    #[link_name = "hash_init"]
    fn sound_hash_init(ht: *mut crate::HashtabRaw);
    #[link_name = "hash_clear"]
    fn sound_hash_clear(ht: *mut crate::HashtabRaw);
    #[link_name = "xfree"]
    fn sound_xfree(ptr: *mut std::ffi::c_void);
}

/// Rust implementation of C `suggest_try_special`.
///
/// Handles the "the the" repeated word case.
///
/// # Safety
/// `su` must be a valid suginfo_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_suggest_try_special(su: *mut std::ffi::c_void) {
    let fbadword = nvim_suginfo_get_fbadword(su).cast_mut();
    // Find the end of the first word.
    let p = trie_skiptowhite(fbadword);
    let len = p.offset_from(fbadword) as usize;
    let p2 = trie_skipwhite(p);
    // Check if the two halves are equal.
    if c_strlen(p2) == len && c_strncmp(fbadword, p2, len) {
        let mut word = [0u8; MAXWLEN];
        // Temporarily NUL-terminate after first word.
        let saved = *fbadword.add(len);
        *fbadword.add(len) = 0;
        let badflags = nvim_suginfo_get_badflags(su);
        crate::rs_make_case_word(fbadword, word.as_mut_ptr().cast::<c_char>(), badflags);
        *fbadword.add(len) = saved;

        let su_ga = nvim_suginfo_get_ga(su);
        let su_badlen = nvim_suginfo_get_badlen(su);
        let su_sallang = nvim_suginfo_get_sallang(su);
        // RESCORE(SCORE_REP, 0) = (3*SCORE_REP + 0) / 4
        let score = (3 * crate::SCORE_REP) / 4;
        rs_add_suggestion(
            su,
            su_ga,
            word.as_ptr().cast::<c_char>(),
            su_badlen,
            score,
            0,
            true,
            su_sallang.cast::<std::ffi::c_void>(),
            false,
        );
    }
}

/// Rust implementation of C `suggest_try_change`.
///
/// Iterates languages and calls suggest_trie_walk for each.
///
/// # Safety
/// `su` must be a valid suginfo_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_suggest_try_change(su: *mut std::ffi::c_void) {
    let mut fword = [0u8; MAXWLEN];

    // Copy the case-folded bad word.
    let fbadword = nvim_suginfo_get_fbadword(su);
    c_strcpy(fword.as_mut_ptr().cast::<c_char>(), fbadword);

    let n = c_strlen(fword.as_ptr().cast::<c_char>());
    let badptr = nvim_suginfo_get_badptr(su);
    let badlen = nvim_suginfo_get_badlen(su);
    let p = badptr.add(badlen as usize);
    let plen = c_strlen(p);
    // Append casefold of text after bad word.
    crate::check::rs_spell_casefold_c_compat(
        stp_sal_curwin,
        p,
        plen as c_int,
        fword.as_mut_ptr().add(n).cast::<c_char>(),
        (MAXWLEN - n) as c_int,
    );

    // Make sure result is no longer than original bad word area.
    let orig_len = c_strlen(badptr);
    if orig_len < MAXWLEN {
        fword[orig_len] = 0;
    }

    let langp_ga = nvim_win_get_b_langp(stp_sal_curwin);
    let langp_len = (*langp_ga).ga_len;
    for lpi in 0..langp_len as usize {
        let lp = (*langp_ga).ga_data.cast::<crate::LangpT>().add(lpi);
        if (*(*lp).lp_slang).sl_fbyts.is_null() {
            continue;
        }
        rs_suggest_trie_walk(su, lp, fword.as_mut_ptr().cast::<c_char>(), false);
    }
}

/// Rust implementation of C `suggest_try_soundalike_prep`.
///
/// Initializes sl_sounddone hash tables for languages with sound folding.
///
/// # Safety
/// No pointers required; uses global curwin.
#[no_mangle]
pub unsafe extern "C" fn rs_suggest_try_soundalike_prep() {
    let langp_ga = nvim_win_get_b_langp(stp_sal_curwin);
    let langp_len = (*langp_ga).ga_len;
    for lpi in 0..langp_len as usize {
        let lp = (*langp_ga).ga_data.cast::<crate::LangpT>().add(lpi);
        let slang_ptr = (*lp).lp_slang;
        if (*slang_ptr).sl_sal.ga_len > 0 && !(*slang_ptr).sl_sbyts.is_null() {
            sound_hash_init(std::ptr::addr_of_mut!((*slang_ptr).sl_sounddone));
        }
    }
}

/// Rust implementation of C `suggest_try_soundalike`.
///
/// Soundfolds the bad word and calls suggest_trie_walk with soundfold=true.
///
/// # Safety
/// `su` must be a valid suginfo_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_suggest_try_soundalike(su: *mut std::ffi::c_void) {
    let mut salword = [0u8; MAXWLEN];
    let fbadword = nvim_suginfo_get_fbadword(su);

    let langp_ga = nvim_win_get_b_langp(stp_sal_curwin);
    let langp_len = (*langp_ga).ga_len;
    for lpi in 0..langp_len as usize {
        let lp = (*langp_ga).ga_data.cast::<crate::LangpT>().add(lpi);
        let slang_ptr = (*lp).lp_slang;
        if (*slang_ptr).sl_sal.ga_len > 0 && !(*slang_ptr).sl_sbyts.is_null() {
            // Soundfold the bad word.
            crate::rs_spell_soundfold(
                slang_ptr,
                fbadword.cast_mut(),
                true,
                salword.as_mut_ptr().cast::<c_char>(),
            );
            rs_suggest_trie_walk(su, lp, salword.as_mut_ptr().cast::<c_char>(), true);
        }
    }
}

/// Rust implementation of C `suggest_try_soundalike_finish`.
///
/// Frees sl_sounddone entries and clears the hash tables.
///
/// # Safety
/// No pointers required; uses global curwin.
#[no_mangle]
pub unsafe extern "C" fn rs_suggest_try_soundalike_finish() {
    let langp_ga = nvim_win_get_b_langp(stp_sal_curwin);
    let langp_len = (*langp_ga).ga_len;
    for lpi in 0..langp_len as usize {
        let lp = (*langp_ga).ga_data.cast::<crate::LangpT>().add(lpi);
        let slang_ptr = (*lp).lp_slang;
        if (*slang_ptr).sl_sal.ga_len > 0 && !(*slang_ptr).sl_sbyts.is_null() {
            // Free all sftword_T entries.
            let ht = std::ptr::addr_of_mut!((*slang_ptr).sl_sounddone);
            let mut todo = (*ht).ht_used as c_int;
            let mut hi = (*ht).ht_array;
            while todo > 0 {
                if !sound_hi_empty(hi) {
                    sound_xfree(hi_key_to_sft((*hi).hi_key).cast::<std::ffi::c_void>());
                    todo -= 1;
                }
                hi = hi.add(1);
            }
            sound_hash_clear(ht);
            sound_hash_init(ht);
        }
    }
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

    // =========================================================================
    // Trie Walk State Machine Tests
    // =========================================================================

    #[test]
    fn test_trie_walk_state_values() {
        assert_eq!(TrieWalkState::Start as c_int, 0);
        assert_eq!(TrieWalkState::NoPrefix as c_int, 1);
        assert_eq!(TrieWalkState::SplitUndo as c_int, 2);
        assert_eq!(TrieWalkState::EndNul as c_int, 3);
        assert_eq!(TrieWalkState::Plain as c_int, 4);
        assert_eq!(TrieWalkState::Del as c_int, 5);
        assert_eq!(TrieWalkState::InsPrep as c_int, 6);
        assert_eq!(TrieWalkState::Ins as c_int, 7);
        assert_eq!(TrieWalkState::Swap as c_int, 8);
        assert_eq!(TrieWalkState::Unswap as c_int, 9);
        assert_eq!(TrieWalkState::Swap3 as c_int, 10);
        assert_eq!(TrieWalkState::Unswap3 as c_int, 11);
        assert_eq!(TrieWalkState::Unrot3L as c_int, 12);
        assert_eq!(TrieWalkState::Unrot3R as c_int, 13);
        assert_eq!(TrieWalkState::RepIni as c_int, 14);
        assert_eq!(TrieWalkState::Rep as c_int, 15);
        assert_eq!(TrieWalkState::RepUndo as c_int, 16);
        assert_eq!(TrieWalkState::Final as c_int, 17);
    }

    #[test]
    fn test_diff_type_values() {
        assert_eq!(DiffType::None as u8, 0);
        assert_eq!(DiffType::Yes as u8, 1);
        assert_eq!(DiffType::Insert as u8, 2);
    }

    #[test]
    fn test_try_state_flags() {
        assert_eq!(try_state_flags::TSF_PREFIXOK, 1);
        assert_eq!(try_state_flags::TSF_DIDSPLIT, 2);
        assert_eq!(try_state_flags::TSF_DIDDEL, 4);
    }

    #[test]
    fn test_prefix_depth_values() {
        assert_eq!(prefix_depth::PFD_NOPREFIX, 0xff);
        assert_eq!(prefix_depth::PFD_PREFIXTREE, 0xfe);
        assert_eq!(prefix_depth::PFD_NOTSPECIAL, 0xfd);
    }

    #[test]
    fn test_try_state_default() {
        let ts = TryState::default();
        assert_eq!(ts.state, TrieWalkState::Start);
        assert_eq!(ts.score, 0);
        assert_eq!(ts.arridx, 0);
        assert_eq!(ts.curi, 1);
        assert_eq!(ts.fidx, 0);
        assert_eq!(ts.prefixdepth, prefix_depth::PFD_NOPREFIX);
        assert_eq!(ts.flags, 0);
        assert_eq!(ts.isdiff, DiffType::None);
    }

    #[test]
    fn test_try_state_helper_methods() {
        let mut ts = TryState::default();

        // Test no_prefix
        assert!(ts.no_prefix());

        // Test in_prefix_tree
        ts.prefixdepth = prefix_depth::PFD_PREFIXTREE;
        assert!(ts.in_prefix_tree());
        assert!(!ts.no_prefix());

        // Test flag methods
        ts.flags = 0;
        assert!(!ts.prefix_ok());
        assert!(!ts.did_split());
        assert!(!ts.did_del());

        ts.flags = try_state_flags::TSF_PREFIXOK;
        assert!(ts.prefix_ok());

        ts.flags = try_state_flags::TSF_DIDSPLIT;
        assert!(ts.did_split());

        ts.flags = try_state_flags::TSF_DIDDEL;
        assert!(ts.did_del());
    }

    // =========================================================================
    // Word Transformation Tests
    // =========================================================================

    #[test]
    fn test_swap_bytes() {
        let mut word = *b"hello";
        assert!(swap_bytes(&mut word, 0));
        assert_eq!(&word, b"ehllo");

        let mut word = *b"abcde";
        assert!(swap_bytes(&mut word, 2));
        assert_eq!(&word, b"abdce"); // c<->d swapped

        // Out of bounds
        let mut word = *b"hello";
        assert!(!swap_bytes(&mut word, 4));
        assert_eq!(&word, b"hello");
    }

    #[test]
    fn test_swap3_bytes() {
        let mut word = *b"abcde";
        assert!(swap3_bytes(&mut word, 0));
        assert_eq!(&word, b"cbade"); // a<->c swapped

        let mut word = *b"abcde";
        assert!(swap3_bytes(&mut word, 1));
        assert_eq!(&word, b"adcbe"); // b<->d swapped

        // Out of bounds
        let mut word = *b"abc";
        assert!(!swap3_bytes(&mut word, 1));
        assert_eq!(&word, b"abc");
    }

    #[test]
    fn test_rotate3_left() {
        let mut word = *b"abcde";
        assert!(rotate3_left(&mut word, 0));
        assert_eq!(&word, b"bcade"); // abc -> bca

        let mut word = *b"abcde";
        assert!(rotate3_left(&mut word, 1));
        assert_eq!(&word, b"acdbe"); // bcd -> cdb

        // Out of bounds
        let mut word = *b"abc";
        assert!(!rotate3_left(&mut word, 1));
    }

    #[test]
    fn test_rotate3_right() {
        let mut word = *b"abcde";
        assert!(rotate3_right(&mut word, 0));
        assert_eq!(&word, b"cabde"); // abc -> cab

        let mut word = *b"abcde";
        assert!(rotate3_right(&mut word, 1));
        assert_eq!(&word, b"adbce"); // bcd -> dbc

        // Out of bounds
        let mut word = *b"abc";
        assert!(!rotate3_right(&mut word, 1));
    }

    #[test]
    fn test_delete_byte() {
        let mut word = [b'h', b'e', b'l', b'l', b'o', 0, 0];
        let new_len = delete_byte(&mut word, 5, 0);
        assert_eq!(new_len, 4);
        assert_eq!(&word[..4], b"ello");

        let mut word = [b'h', b'e', b'l', b'l', b'o', 0, 0];
        let new_len = delete_byte(&mut word, 5, 2);
        assert_eq!(new_len, 4);
        assert_eq!(&word[..4], b"helo");

        // Out of bounds
        let mut word = [b'h', b'e', b'l', b'l', b'o', 0, 0];
        let new_len = delete_byte(&mut word, 5, 5);
        assert_eq!(new_len, 5);
    }

    #[test]
    fn test_insert_byte() {
        let mut word = [b'h', b'e', b'l', b'o', 0, 0, 0];
        let new_len = insert_byte(&mut word, 4, 2, b'l');
        assert_eq!(new_len, 5);
        assert_eq!(&word[..5], b"hello");

        let mut word = [b'e', b'l', b'l', b'o', 0, 0, 0];
        let new_len = insert_byte(&mut word, 4, 0, b'h');
        assert_eq!(new_len, 5);
        assert_eq!(&word[..5], b"hello");
    }

    #[test]
    fn test_substitute_byte() {
        let mut word = *b"hello";
        let old = substitute_byte(&mut word, 0, b'j');
        assert_eq!(old, b'h');
        assert_eq!(&word, b"jello");

        let mut word = *b"hello";
        let old = substitute_byte(&mut word, 4, b'a');
        assert_eq!(old, b'o');
        assert_eq!(&word, b"hella");

        // Out of bounds
        let mut word = *b"hello";
        let old = substitute_byte(&mut word, 5, b'x');
        assert_eq!(old, 0);
    }

    // =========================================================================
    // Phase 322 Tests: Spellsuggest Option Parsing
    // =========================================================================

    #[test]
    fn test_sps_constants() {
        assert_eq!(SPS_BEST, 1);
        assert_eq!(SPS_FAST, 2);
        assert_eq!(SPS_DOUBLE, 4);
        assert_eq!(SPS_LIMIT_DEFAULT, 9999);
    }

    #[test]
    fn test_parse_sps_value_keywords() {
        let mut limit = 0;
        assert_eq!(parse_sps_value(b"best", &mut limit), SPS_BEST);
        assert_eq!(parse_sps_value(b"fast", &mut limit), SPS_FAST);
        assert_eq!(parse_sps_value(b"double", &mut limit), SPS_DOUBLE);
    }

    #[test]
    fn test_parse_sps_value_numeric() {
        let mut limit = 0;
        assert_eq!(parse_sps_value(b"10", &mut limit), 0);
        assert_eq!(limit, 10);

        assert_eq!(parse_sps_value(b"25", &mut limit), 0);
        assert_eq!(limit, 25);

        assert_eq!(parse_sps_value(b"100", &mut limit), 0);
        assert_eq!(limit, 100);
    }

    #[test]
    fn test_parse_sps_value_prefixes() {
        let mut limit = 0;
        assert_eq!(parse_sps_value(b"expr:something", &mut limit), 0);
        assert_eq!(parse_sps_value(b"file:/path/to/file", &mut limit), 0);
        assert_eq!(parse_sps_value(b"timeout:100", &mut limit), 0);
        assert_eq!(parse_sps_value(b"timeout:-1", &mut limit), 0);
    }

    #[test]
    fn test_parse_sps_value_invalid() {
        let mut limit = 0;
        assert_eq!(parse_sps_value(b"unknown", &mut limit), -1);
        assert_eq!(parse_sps_value(b"10abc", &mut limit), -1);
        assert_eq!(parse_sps_value(b"timeout:", &mut limit), -1);
        assert_eq!(parse_sps_value(b"timeout:abc", &mut limit), -1);
        assert_eq!(parse_sps_value(b"", &mut limit), -1);
    }

    #[test]
    fn test_parse_spellsuggest_simple() {
        let result = parse_spellsuggest(b"best").unwrap();
        assert_eq!(result.flags, SPS_BEST);
        assert_eq!(result.limit, SPS_LIMIT_DEFAULT);

        let result = parse_spellsuggest(b"fast").unwrap();
        assert_eq!(result.flags, SPS_FAST);

        let result = parse_spellsuggest(b"double").unwrap();
        assert_eq!(result.flags, SPS_DOUBLE);
    }

    #[test]
    fn test_parse_spellsuggest_with_limit() {
        let result = parse_spellsuggest(b"best,10").unwrap();
        assert_eq!(result.flags, SPS_BEST);
        assert_eq!(result.limit, 10);

        let result = parse_spellsuggest(b"20,fast").unwrap();
        assert_eq!(result.flags, SPS_FAST);
        assert_eq!(result.limit, 20);
    }

    #[test]
    fn test_parse_spellsuggest_default() {
        // Empty string defaults to SPS_BEST
        let result = parse_spellsuggest(b"").unwrap();
        assert_eq!(result.flags, SPS_BEST);
        assert_eq!(result.limit, SPS_LIMIT_DEFAULT);
    }

    #[test]
    fn test_parse_spellsuggest_invalid() {
        // Unknown value
        assert!(parse_spellsuggest(b"unknown").is_err());

        // Duplicate flags
        assert!(parse_spellsuggest(b"best,fast").is_err());
    }

    #[test]
    fn test_parse_spellsuggest_with_expr() {
        // expr: is valid but doesn't set flags
        let result = parse_spellsuggest(b"expr:myfunction").unwrap();
        assert_eq!(result.flags, SPS_BEST); // Defaults to BEST
    }

    #[test]
    fn test_sps_flags_default() {
        let flags = SpsFlags::default();
        assert_eq!(flags.flags, 0);
        assert_eq!(flags.limit, 0);
    }
}
