//! Keeping the list of candidate suggestions.
//!
//! Everything that finds a candidate word funnels into [`add_suggestion`],
//! which is the only place a `suggest_T` is created. It also decides what
//! "the same suggestion" means: two candidates are the same when they
//! replace the same stretch of the bad word with the same text, and the
//! lower of the two scores wins.
//!
//! The list is deliberately allowed to grow past what will be shown --
//! sorting and truncating on every addition would cost more than it saves
//! -- so [`cleanup_suggestions`] runs when it gets far enough over, sorts
//! on score and drops the tail. Its return value is the new score ceiling:
//! once the list is full, a candidate scoring worse than the last one kept
//! cannot make it and the search can prune on that.
//!
//! [`check_suggestions`] is the late filter. A suggestion can itself be
//! misspelled -- "the the" split into two words is the usual case -- and
//! that is only visible by spell-checking the replacement in place, which
//! is too expensive to do per candidate.
//!
//! [`score_comp_sal`] and [`score_combine`] implement `'spellsuggest'`'s
//! "double" mode, where the edit-distance list and the sound-a-like list
//! are scored separately and then interleaved.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_grow, ga_init};
use crate::src::nvim::hashtab::{hash_add_item, hash_hash, hash_lookup, hash_removed};
use crate::src::nvim::main::curwin;
use crate::src::nvim::mbyte::{utf_head_off, utf_ptr2char};
use crate::src::nvim::memory::{xfree, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::os::libc::{qsort, strcasecmp, strcmp, strlen, strncmp};
use crate::src::nvim::spell::{spell_check, spell_soundfold};
use crate::src::nvim::spellsuggest::score::{EMPTY_SOUND, spell_edit_score, stp_sal_score};
use crate::src::nvim::spellsuggest::{
    HLF_COUNT, MAXWLEN, SCORE_INS, SCORE_MAXMAX, suggest_T, suginfo_T, window_langs,
};
use crate::src::nvim::types::{__compar_fn_t, garray_T, hlf_T, size_t, slang_T};
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr, slice};

/// A sound-a-like score that could not be computed stands in as "three
/// insertions apart", which is far but not infinitely so.
const SCORE_BIG: c_int = SCORE_INS as c_int * 3;

/// Blend a word's edit-distance score with its sound-a-like score. The
/// edit distance dominates; the sound only nudges.
fn rescore(word_score: c_int, sound_score: c_int) -> c_int {
    (3 * word_score + sound_score) / 4
}

/// How many suggestions to keep when the list is cleaned up. Always
/// comfortably more than will be displayed, because a later pass can
/// rescore them and change the order.
fn clean_count(su: &suginfo_T) -> c_int {
    if su.su_maxcount < 130 {
        150
    } else {
        su.su_maxcount + 20
    }
}

/// The size the list is allowed to reach before it gets cleaned up.
fn max_count(su: &suginfo_T) -> c_int {
    clean_count(su) + 50
}

/// The suggestions a garray holds.
///
/// # Safety
///
/// `gap` must be a garray of `suggest_T`.
pub unsafe fn suggestions<'a>(gap: *mut garray_T) -> &'a mut [suggest_T] {
    // SAFETY: the caller guarantees the element type; an empty garray has a
    // null data pointer, which `from_raw_parts_mut` rejects even at length
    // zero.
    unsafe {
        if (*gap).ga_data.is_null() || (*gap).ga_len <= 0 {
            &mut []
        } else {
            slice::from_raw_parts_mut((*gap).ga_data as *mut suggest_T, (*gap).ga_len as usize)
        }
    }
}

/// Add a suggestion to the list, or lower the score of the one already
/// there.
///
/// `badlenarg` is how much of the bad word `goodword` replaces; `maxsf`
/// says whether the caller's score ceiling is the sound-fold one.
///
/// # Safety
///
/// `su` and `gap` must be valid, `goodword` NUL-terminated, and `su`'s bad
/// word must still point into the line it came from.
#[allow(clippy::too_many_arguments)]
pub unsafe fn add_suggestion(
    su: *mut suginfo_T,
    gap: *mut garray_T,
    goodword: *const c_char,
    badlenarg: c_int,
    score: c_int,
    altscore: c_int,
    had_bonus: bool,
    slang: *mut slang_T,
    maxsf: bool,
) {
    // SAFETY: the caller guarantees the pointers; every read below stays
    // inside the two words and the suggestion list.
    unsafe {
        // Minimise "badlen" for consistency: changing "the the" to "thee
        // the" should not be listed next to changing the first "the" to
        // "thee". Walk both tails back over the characters they share.
        // The lengths are the ones measured before the last step back: a
        // shared trailing character is not part of the replacement.
        let mut pgood = goodword.add(strlen(goodword) as usize);
        let mut pbad = (*su).su_badptr.offset(badlenarg as isize);
        let (goodlen, badlen) = loop {
            let lens = (
                pgood.offset_from(goodword) as c_int,
                pbad.offset_from((*su).su_badptr) as c_int,
            );
            if lens.0 <= 0 || lens.1 <= 0 {
                break lens;
            }
            pgood = pgood.sub(utf_head_off(goodword as *mut c_char, pgood.sub(1)) as usize + 1);
            pbad = pbad.sub(utf_head_off((*su).su_badptr, pbad.sub(1)) as usize + 1);
            if utf_ptr2char(pgood) != utf_ptr2char(pbad) {
                break lens;
            }
        };

        if badlen == 0 && goodlen == 0 {
            // The good word changes nothing; happens when "the the" has its
            // first "the" replaced by itself.
            return;
        }

        // Already in the list? The replaced length is part of the identity:
        // "thes," -> "these" is a different suggestion from "thes" ->
        // "these".
        for stp in suggestions(gap) {
            if stp.st_wordlen != goodlen
                || stp.st_orglen != badlen
                || strncmp(stp.st_word, goodword, goodlen as size_t) != 0
            {
                continue;
            }

            if stp.st_slang.is_null() {
                stp.st_slang = slang;
            }

            let mut new_sug = *stp;
            new_sug.st_score = score;
            new_sug.st_altscore = altscore;
            new_sug.st_had_bonus = had_bonus;

            if stp.st_had_bonus != had_bonus {
                // Only one of the two has a sound-a-like score, so they
                // cannot be compared yet. `suggest_try_change` leaves it
                // out to stay fast, while some of the special methods set
                // it to zero.
                if had_bonus {
                    rescore_one(su, stp);
                } else {
                    new_sug.st_orglen = badlen;
                    rescore_one(su, &mut new_sug);
                }
            }

            if stp.st_score > new_sug.st_score {
                stp.st_score = new_sug.st_score;
                stp.st_altscore = new_sug.st_altscore;
                stp.st_had_bonus = new_sug.st_had_bonus;
            }
            return;
        }

        let stp = ga_append_via_ptr(gap, mem::size_of::<suggest_T>()) as *mut suggest_T;
        *stp = suggest_T {
            st_word: xmemdupz(goodword as *const c_void, goodlen as usize) as *mut c_char,
            st_wordlen: goodlen,
            st_orglen: badlen,
            st_score: score,
            st_altscore: altscore,
            st_salscore: false,
            st_had_bonus: had_bonus,
            st_slang: slang,
        };

        // Far enough over the display count that sorting pays for itself.
        if (*gap).ga_len > max_count(&*su) {
            let keep = clean_count(&*su);
            if maxsf {
                (*su).su_sfmaxscore = cleanup_suggestions(gap, (*su).su_sfmaxscore, keep);
            } else {
                (*su).su_maxscore = cleanup_suggestions(gap, (*su).su_maxscore, keep);
            }
        }
    }
}

/// Drop suggestions that are themselves misspelled.
///
/// Banned words and split words -- "the the" -- only show up as errors
/// once the replacement is checked against the rest of the line, which is
/// why this cannot be done as candidates arrive.
///
/// # Safety
///
/// `su` and `gap` must be valid and `gap` must hold `suggest_T`s.
pub unsafe fn check_suggestions(su: *mut suginfo_T, gap: *mut garray_T) {
    // SAFETY: the caller guarantees the pointers; `longword` is sized for
    // any word plus a terminator and every copy into it is bounded.
    unsafe {
        let mut longword = [0 as c_char; MAXWLEN as usize + 1];
        let stp = (*gap).ga_data as *mut suggest_T;
        for i in (0..(*gap).ga_len).rev() {
            let sug = &*stp.offset(i as isize);
            // Append what follows in the line, so that "the the" is
            // recognisable.
            xstrlcpy(longword.as_mut_ptr(), sug.st_word, MAXWLEN as usize + 1);
            let len = sug.st_wordlen;
            xstrlcpy(
                longword.as_mut_ptr().offset(len as isize),
                (*su).su_badptr.offset(sug.st_orglen as isize),
                MAXWLEN as usize + 1 - len as usize,
            );

            let mut attr: hlf_T = HLF_COUNT;
            spell_check(
                curwin.get(),
                longword.as_mut_ptr(),
                &raw mut attr,
                ptr::null_mut(),
                false,
            );
            if attr == HLF_COUNT {
                continue;
            }

            xfree(sug.st_word as *mut c_void);
            (*gap).ga_len -= 1;
            if i < (*gap).ga_len {
                ptr::copy(
                    stp.offset(i as isize + 1),
                    stp.offset(i as isize),
                    ((*gap).ga_len - i) as usize,
                );
            }
        }
    }
}

/// Remember a word that must never be suggested.
///
/// # Safety
///
/// `su` must be valid and `word` NUL-terminated.
pub unsafe fn add_banned(su: *mut suginfo_T, word: *mut c_char) {
    // SAFETY: the caller guarantees the pointers; the copy handed to the
    // table is owned by it until `hash_clear_all` frees it.
    unsafe {
        let hash = hash_hash(word);
        let word_len = strlen(word) as usize;
        let hi = hash_lookup(&raw mut (*su).su_banned, word, word_len, hash);
        let key = (*hi).hi_key;
        if !(key.is_null() || key == &raw const hash_removed as *mut c_char) {
            return; // already present
        }
        let owned = xmemdupz(word as *const c_void, word_len) as *mut c_char;
        hash_add_item(&raw mut (*su).su_banned, hi, owned, hash);
    }
}

/// Recompute every suggestion's score with sound folding taken into
/// account. Slow, so only done once the list is final.
///
/// # Safety
///
/// `su` must be valid.
pub unsafe fn rescore_suggestions(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees the pointer.
    unsafe {
        if (*su).su_sallang.is_null() {
            return;
        }
        let gap = &raw mut (*su).su_ga;
        for i in 0..(*gap).ga_len {
            let stp = ((*gap).ga_data as *mut suggest_T).offset(i as isize);
            rescore_one(su, &mut *stp);
        }
    }
}

/// Recompute one suggestion's score with sound folding taken into account.
///
/// # Safety
///
/// `su` must be valid.
pub unsafe fn rescore_one(su: *mut suginfo_T, stp: &mut suggest_T) {
    let slang = stp.st_slang;
    // SAFETY: the caller guarantees `su`; `st_slang` is either null or a
    // loaded language that outlives the suggestion list.
    unsafe {
        // Only worth doing for a suggestion that has no sound-a-like score
        // yet and knows which language it came from.
        if slang.is_null() || (*slang).sl_sal.ga_len <= 0 || stp.st_had_bonus {
            return;
        }

        let mut sal_badword = EMPTY_SOUND;
        let badsound = if slang == (*su).su_sallang {
            &(*su).su_sal_badword
        } else {
            spell_soundfold(
                slang,
                &raw mut (*su).su_fbadword as *mut c_char,
                true,
                sal_badword.as_mut_ptr(),
            );
            &sal_badword
        };

        stp.st_altscore = stp_sal_score(stp, &*su, slang, badsound);
        if stp.st_altscore == SCORE_MAXMAX as c_int {
            stp.st_altscore = SCORE_BIG;
        }
        stp.st_score = rescore(stp.st_score, stp.st_altscore);
        stp.st_had_bonus = true;
    }
}

/// Order suggestions by score, then by sound-a-like score, then by word.
///
/// Still spelled for `qsort`: replacing the sort with `slice::sort_by`
/// would be a behaviour change, because the last comparison is
/// case-insensitive and two entries differing only in case therefore tie,
/// leaving their relative order up to the algorithm.
///
/// # Safety
///
/// Both pointers must be to `suggest_T`s, as `qsort` guarantees.
pub unsafe extern "C" fn sug_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: `qsort` passes pointers to the elements of the array it was
    // given, which is an array of `suggest_T`.
    unsafe {
        let (p1, p2) = (&*(s1 as *const suggest_T), &*(s2 as *const suggest_T));
        match p1.st_score.cmp(&p2.st_score) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Greater => 1,
            core::cmp::Ordering::Equal => match p1.st_altscore.cmp(&p2.st_altscore) {
                core::cmp::Ordering::Less => -1,
                core::cmp::Ordering::Greater => 1,
                core::cmp::Ordering::Equal => strcasecmp(p1.st_word, p2.st_word),
            },
        }
    }
}

/// Sort the suggestions and drop the ones that will not be displayed.
///
/// Returns the new score ceiling: the score of the worst suggestion still
/// in the list, which nothing worse can now beat.
///
/// # Safety
///
/// `gap` must hold `suggest_T`s.
pub unsafe fn cleanup_suggestions(gap: *mut garray_T, maxscore: c_int, keep: c_int) -> c_int {
    // SAFETY: the caller guarantees the element type; the comparator reads
    // exactly the elements `qsort` hands it.
    unsafe {
        if (*gap).ga_len <= 0 {
            return maxscore;
        }

        qsort(
            (*gap).ga_data,
            (*gap).ga_len as size_t,
            mem::size_of::<suggest_T>() as size_t,
            Some(sug_compare as unsafe extern "C" fn(*const c_void, *const c_void) -> c_int)
                as __compar_fn_t,
        );

        if (*gap).ga_len <= keep {
            return maxscore;
        }

        let stp = (*gap).ga_data as *mut suggest_T;
        for i in keep..(*gap).ga_len {
            xfree((*stp.offset(i as isize)).st_word as *mut c_void);
        }
        (*gap).ga_len = keep;
        if keep >= 1 {
            return (*stp.offset(keep as isize - 1)).st_score;
        }
        maxscore
    }
}

/// Score every suggestion in `su_ga` by sound and put the ones that are
/// close enough into `su_sga`.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
pub unsafe fn score_comp_sal(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees `su`; the languages come from the
    // current window's loaded list.
    unsafe {
        ga_grow(&raw mut (*su).su_sga, (*su).su_ga.ga_len);

        // Use the sound folding of the first language that has any.
        let Some(lp) = window_langs()
            .iter()
            .find(|lp| (*(*lp).lp_slang).sl_sal.ga_len > 0)
        else {
            return;
        };
        let slang = lp.lp_slang;

        let mut badsound = EMPTY_SOUND;
        spell_soundfold(
            slang,
            &raw mut (*su).su_fbadword as *mut c_char,
            true,
            badsound.as_mut_ptr(),
        );

        for i in 0..(*su).su_ga.ga_len {
            let stp = &*((*su).su_ga.ga_data as *mut suggest_T).offset(i as isize);
            let score = stp_sal_score(stp, &*su, slang, &badsound);
            if score >= SCORE_MAXMAX as c_int {
                continue;
            }
            let sstp =
                ((*su).su_sga.ga_data as *mut suggest_T).offset((*su).su_sga.ga_len as isize);
            (*sstp).st_word = xstrdup(stp.st_word);
            (*sstp).st_wordlen = stp.st_wordlen;
            (*sstp).st_score = score;
            (*sstp).st_altscore = 0;
            (*sstp).st_orglen = stp.st_orglen;
            (*su).su_sga.ga_len += 1;
        }
    }
}

/// Merge the edit-distance list and the sound-a-like list into one.
///
/// Each list is first given the other measure's score so that both are
/// comparable, then the two are taken from alternately -- best of one,
/// best of the other -- so that "double" mode shows a mixture rather than
/// whichever measure happened to score lower.
///
/// # Safety
///
/// `su` must be valid.
pub unsafe fn score_combine(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees `su`; both lists hold `suggest_T`s and
    // own their words until they are moved into the merged list.
    unsafe {
        let mut badsound = EMPTY_SOUND;
        let mut slang: *mut slang_T = ptr::null_mut();

        // Give the edit-distance list a sound-a-like score.
        if let Some(lp) = window_langs()
            .iter()
            .find(|lp| (*(*lp).lp_slang).sl_sal.ga_len > 0)
        {
            slang = lp.lp_slang;
            spell_soundfold(
                slang,
                &raw mut (*su).su_fbadword as *mut c_char,
                true,
                badsound.as_mut_ptr(),
            );

            for stp in suggestions(&raw mut (*su).su_ga) {
                stp.st_altscore = stp_sal_score(stp, &*su, slang, &badsound);
                let alt = if stp.st_altscore == SCORE_MAXMAX as c_int {
                    SCORE_BIG
                } else {
                    stp.st_altscore
                };
                stp.st_score = rescore(stp.st_score, alt);
                stp.st_salscore = false;
            }
        }

        if slang.is_null() {
            // "double" without a language that can sound-fold.
            cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
            return;
        }

        // Give the sound-a-like list an edit-distance score. Here the
        // sound score is the one that dominates.
        for stp in suggestions(&raw mut (*su).su_sga) {
            stp.st_altscore = spell_edit_score(
                Some(&*slang),
                &raw const (*su).su_badword as *const c_char,
                stp.st_word,
            );
            let base = if stp.st_score == SCORE_MAXMAX as c_int {
                SCORE_BIG
            } else {
                stp.st_score
            };
            stp.st_score = (base * 7 + stp.st_altscore) / 8;
            stp.st_salscore = true;
        }

        check_suggestions(su, &raw mut (*su).su_ga);
        cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
        check_suggestions(su, &raw mut (*su).su_sga);
        cleanup_suggestions(&raw mut (*su).su_sga, (*su).su_maxscore, (*su).su_maxcount);

        let mut ga: garray_T = mem::zeroed();
        ga_init(&raw mut ga, mem::size_of::<suggest_T>() as c_int, 1);
        ga_grow(&raw mut ga, (*su).su_ga.ga_len + (*su).su_sga.ga_len);
        let merged = ga.ga_data as *mut suggest_T;

        let rounds = [&raw mut (*su).su_ga, &raw mut (*su).su_sga];
        let longest = (*su).su_ga.ga_len.max((*su).su_sga.ga_len);
        for i in 0..longest {
            for gap in rounds {
                if i >= (*gap).ga_len {
                    continue;
                }
                let candidate = *((*gap).ga_data as *mut suggest_T).offset(i as isize);
                // Skip a word that is already in the merged list; its copy
                // of the word is then nobody's, so free it.
                let seen = (0..ga.ga_len)
                    .any(|j| same_word((*merged.offset(j as isize)).st_word, candidate.st_word));
                if seen {
                    xfree(candidate.st_word as *mut c_void);
                } else {
                    *merged.offset(ga.ga_len as isize) = candidate;
                    ga.ga_len += 1;
                }
            }
        }

        ga_clear(&raw mut (*su).su_ga);
        ga_clear(&raw mut (*su).su_sga);

        // Keep only what will be displayed.
        if ga.ga_len > (*su).su_maxcount {
            for i in (*su).su_maxcount..ga.ga_len {
                xfree((*merged.offset(i as isize)).st_word as *mut c_void);
            }
            ga.ga_len = (*su).su_maxcount;
        }

        (*su).su_ga = ga;
    }
}

/// Exact string equality, as the merge's duplicate test uses.
///
/// # Safety
///
/// Both pointers must be NUL-terminated.
unsafe fn same_word(a: *const c_char, b: *const c_char) -> bool {
    // SAFETY: the caller guarantees both strings.
    unsafe { strcmp(a, b) == 0 }
}
