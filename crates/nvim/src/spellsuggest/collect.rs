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

use crate::cstr;

use crate::hashtab::{hash_add_item, hash_hash, hash_lookup};
use crate::highlight_group::HLF_COUNT;
use crate::main::curwin;
use crate::mbyte::{utf_head_off, utf_ptr2char};
use crate::memory::{xmemdupz, xstrlcpy};
use crate::spell::{spell_check, spell_soundfold};
use crate::spellsuggest::score::{EMPTY_SOUND, spell_edit_score, stp_sal_score};
use crate::spellsuggest::{MAXWLEN, SCORE_INS, SCORE_MAXMAX, suggest_T, suginfo_T, window_langs};
use crate::types::{__compar_fn_t, hlf_T, size_t, slang_T};
use ::libc::{qsort, strcasecmp};
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr};

/// A sound-a-like score that could not be computed stands in as "three
/// insertions apart", which is far but not infinitely so.
const SCORE_BIG: c_int = SCORE_INS * 3;

/// Blend a word's edit-distance score with its sound-a-like score. The
/// edit distance dominates; the sound only nudges.
fn rescore(word_score: c_int, sound_score: c_int) -> c_int {
    (3 * word_score + sound_score) / 4
}

/// How many suggestions to keep when the list is cleaned up. Always
/// comfortably more than will be displayed, because a later pass can
/// rescore them and change the order.
pub(super) fn clean_count(su: &suginfo_T) -> c_int {
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
pub(super) unsafe fn add_suggestion(
    su: *mut suginfo_T,
    gap: *mut Vec<suggest_T>,
    goodword: *const c_char,
    badlenarg: c_int,
    score: c_int,
    altscore: c_int,
    had_bonus: bool,
    slang: *mut slang_T,
    maxsf: bool,
) {
    // Minimise "badlen" for consistency: changing "the the" to "thee
    // the" should not be listed next to changing the first "the" to
    // "thee". Walk both tails back over the characters they share.
    // The lengths are the ones measured before the last step back: a
    // shared trailing character is not part of the replacement.
    //
    // SAFETY: `goodword` is NUL-terminated and `su_badptr` still points
    // into the line the bad word came from, so both walks start at the end
    // of a live string and only step back over characters already matched;
    // the loop stops as soon as either offset reaches zero.
    let mut pgood = unsafe { goodword.add(cstr::bytes_at(goodword).len()) };
    let mut pbad = unsafe { (*su).su_badptr.offset(badlenarg as isize) };
    let (goodlen, badlen) = loop {
        let lens = (unsafe { pgood.offset_from(goodword) } as c_int, unsafe {
            pbad.offset_from((*su).su_badptr)
        }
            as c_int);
        if lens.0 <= 0 || lens.1 <= 0 {
            break lens;
        }
        // SAFETY: as above.
        unsafe {
            pgood = pgood.sub(utf_head_off(goodword as *mut c_char, pgood.sub(1)) as usize + 1);
            pbad = pbad.sub(utf_head_off((*su).su_badptr, pbad.sub(1)) as usize + 1);
        }
        // SAFETY: both now sit on a character head inside their word.
        if unsafe { utf_ptr2char(pgood) } != unsafe { utf_ptr2char(pbad) } {
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
    //
    // SAFETY: `gap` is one of `su`'s two lists by the contract above, and
    // `goodlen` bytes of `goodword` are its own. Each step below takes the
    // list afresh rather than holding it across the scoring, which reads
    // the rest of `su`.
    let found = unsafe { &*gap }.iter().position(|stp| {
        stp.st_wordlen == goodlen
            && stp.st_orglen == badlen
            && unsafe { cstr::prefix_eq(stp.word(), goodword, goodlen as size_t) }
    });
    if let Some(at) = found {
        let had_bonus_before = {
            let stp = &mut unsafe { &mut *gap }[at];
            if stp.st_slang.is_null() {
                stp.st_slang = slang;
            }
            stp.st_had_bonus
        };

        let mut new_score = score;
        let mut new_altscore = altscore;
        let mut new_had_bonus = had_bonus;

        if had_bonus_before != had_bonus {
            // Only one of the two has a sound-a-like score, so they
            // cannot be compared yet. `suggest_try_change` leaves it
            // out to stay fast, while some of the special methods set
            // it to zero.
            //
            // SAFETY: `su` is valid by the contract above. Neither call
            // holds the list, which is one of `su`'s own fields.
            if had_bonus {
                unsafe { rescore_one(su, gap, at) };
                let stp = &unsafe { &*gap }[at];
                new_score = stp.st_score;
                new_altscore = stp.st_altscore;
                new_had_bonus = stp.st_had_bonus;
            } else if !slang.is_null() && unsafe { (*slang).has_soundfold() } {
                let (word, wordlen) = {
                    let stp = &unsafe { &*gap }[at];
                    (stp.word(), stp.st_wordlen)
                };
                let (s, a) = unsafe { sal_rescore(su, slang, word, wordlen, badlen, score) };
                new_score = s;
                new_altscore = a;
                new_had_bonus = true;
            }
        }

        let stp = &mut unsafe { &mut *gap }[at];
        if stp.st_score > new_score {
            stp.st_score = new_score;
            stp.st_altscore = new_altscore;
            stp.st_had_bonus = new_had_bonus;
        }
        return;
    }

    // SAFETY: `goodlen` bytes of `goodword` are its own.
    let bytes = unsafe { core::slice::from_raw_parts(goodword.cast::<u8>(), goodlen as usize) };
    let mut word = Vec::with_capacity(goodlen as usize + 1);
    word.extend_from_slice(bytes);
    word.push(0);
    // SAFETY: as above.
    let list = unsafe { &mut *gap };
    list.push(suggest_T {
        st_word: word.into_boxed_slice(),
        st_wordlen: goodlen,
        st_orglen: badlen,
        st_score: score,
        st_altscore: altscore,
        st_salscore: false,
        st_had_bonus: had_bonus,
        st_slang: slang,
    });

    // Far enough over the display count that sorting pays for itself.
    //
    // SAFETY: `su` is valid by the contract above.
    if list.len() as c_int > max_count(unsafe { &*su }) {
        let keep = clean_count(unsafe { &*su });
        if maxsf {
            unsafe { (*su).su_sfmaxscore = cleanup_suggestions(gap, (*su).su_sfmaxscore, keep) };
        } else {
            unsafe { (*su).su_maxscore = cleanup_suggestions(gap, (*su).su_maxscore, keep) };
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
pub(super) unsafe fn check_suggestions(su: *mut suginfo_T, gap: *mut Vec<suggest_T>) {
    let mut longword = [0 as c_char; MAXWLEN + 1];
    // SAFETY: `gap` is one of `su`'s two lists by the contract above, and
    // the list is taken afresh at each step rather than held across
    // `spell_check`.
    for i in (0..unsafe { (*gap).len() }).rev() {
        let (word, len, orglen) = {
            let sug = &unsafe { &*gap }[i];
            (sug.word(), sug.st_wordlen, sug.st_orglen)
        };
        // Append what follows in the line, so that "the the" is
        // recognisable.
        //
        // SAFETY: `longword` is `MAXWLEN + 1` bytes and both copies are
        // told how much of it is left; `su_badptr` points into the line the
        // bad word came from, of which `st_orglen` bytes are replaced.
        unsafe { xstrlcpy(longword.as_mut_ptr(), word, MAXWLEN + 1) };
        let tail = unsafe { longword.as_mut_ptr().offset(len as isize) };
        let rest = unsafe { (*su).su_badptr.offset(orglen as isize) };
        unsafe { xstrlcpy(tail, rest, MAXWLEN + 1 - len as usize) };

        let mut attr: hlf_T = HLF_COUNT;
        let win = curwin.get();
        let longwordp = longword.as_mut_ptr();
        let attrp = &raw mut attr;
        // SAFETY: `longword` is NUL-terminated by the copies above and
        // `attr` is a live local.
        unsafe { spell_check(win, longwordp, attrp, ptr::null_mut(), false) };
        if attr == HLF_COUNT {
            continue;
        }

        // The suggestion owns its word, so removing it frees it.
        //
        // SAFETY: as above.
        unsafe { &mut *gap }.remove(i);
    }
}

/// Remember a word that must never be suggested.
///
/// # Safety
///
/// `su` must be valid and `word` NUL-terminated.
pub(super) unsafe fn add_banned(su: *mut suginfo_T, word: *mut c_char) {
    // SAFETY: `word` is NUL-terminated and `su` is valid by the contract
    // above, so `su_banned` is a live hash table; `hash_add_item` is handed
    // back the `hi`/`hash` pair `hash_lookup` just produced for it, and the
    // copy it takes is owned by the table until `hash_clear_all` frees it.
    let hash = unsafe { hash_hash(word) };
    let word_len = unsafe { cstr::bytes_at(word) }.len();
    let hi = unsafe { hash_lookup(&raw mut (*su).su_banned, word, word_len, hash) };
    if hi.is_kept() {
        return; // already present
    }
    let owned = unsafe { xmemdupz(word as *const c_void, word_len) } as *mut c_char;
    unsafe { hash_add_item(&raw mut (*su).su_banned, hi, owned, hash) };
}

/// Recompute every suggestion's score with sound folding taken into
/// account. Slow, so only done once the list is final.
///
/// # Safety
///
/// `su` must be valid.
pub(super) unsafe fn rescore_suggestions(su: *mut suginfo_T) {
    // SAFETY: `su` is valid by the contract above, so `su_ga` is its own
    // suggestion list.
    if unsafe { (*su).su_sallang }.is_null() {
        return;
    }
    let gap = unsafe { &raw mut (*su).su_ga };
    for at in 0..unsafe { (*gap).len() } {
        unsafe { rescore_one(su, gap, at) };
    }
}

/// Recompute one suggestion's score with sound folding taken into account.
///
/// The entry is named by its position rather than borrowed: the scoring
/// reads the rest of `su`, and the list is one of `su`'s own fields, so the
/// numbers come out, the score is worked out, and the entry is written back.
///
/// # Safety
///
/// `su` must be valid and `gap` be one of its two lists, with `at` inside it.
pub(super) unsafe fn rescore_one(su: *mut suginfo_T, gap: *mut Vec<suggest_T>, at: usize) {
    // SAFETY: the caller guarantees `gap` and `at`.
    let stp = &unsafe { &*gap }[at];
    let (slang, had_bonus) = (stp.st_slang, stp.st_had_bonus);
    // Only worth doing for a suggestion that has no sound-a-like score
    // yet and knows which language it came from.
    //
    // SAFETY: `st_slang` is either null -- which the `||` tests first -- or
    // a loaded language that outlives the suggestion list.
    if slang.is_null() || !unsafe { (*slang).has_soundfold() } || had_bonus {
        return;
    }
    let (word, wordlen, orglen, score) = (stp.word(), stp.st_wordlen, stp.st_orglen, stp.st_score);

    // SAFETY: as above; the word outlives the call because nothing here
    // adds to or drops from the list.
    let (score, altscore) = unsafe { sal_rescore(su, slang, word, wordlen, orglen, score) };

    // SAFETY: the caller guarantees `gap` and `at`.
    let stp = &mut unsafe { &mut *gap }[at];
    stp.st_score = score;
    stp.st_altscore = altscore;
    stp.st_had_bonus = true;
}

/// What sound folding scores one suggestion: the pair that replaces its
/// `st_score` and `st_altscore`.
///
/// Takes the entry's own numbers rather than the entry, so that the
/// suggestion list -- a field of `su`, all of which the scoring reads --
/// need not be borrowed across the call.
///
/// # Safety
///
/// `su` must be valid, `slang` loaded, and `word` a NUL-terminated string
/// of `wordlen` bytes that outlives the call.
unsafe fn sal_rescore(
    su: *mut suginfo_T,
    slang: *mut slang_T,
    word: *mut c_char,
    wordlen: c_int,
    orglen: c_int,
    score: c_int,
) -> (c_int, c_int) {
    let mut sal_badword = EMPTY_SOUND;
    // SAFETY: `su` is valid by the contract above, so `su_sal_badword` is
    // its own soundfold of the bad word and `su_fbadword` the bad word's
    // NUL-terminated fold; `sal_badword` has room for a soundfold.
    let badsound = if slang == unsafe { (*su).su_sallang } {
        unsafe { &(*su).su_sal_badword }
    } else {
        let fbadword = unsafe { &raw mut (*su).su_fbadword } as *mut c_char;
        unsafe { spell_soundfold(slang, fbadword, true, sal_badword.as_mut_ptr()) };
        &sal_badword
    };

    // SAFETY: as above; `badsound` is one of the two soundfolds.
    let mut altscore = unsafe { stp_sal_score(word, wordlen, orglen, &*su, slang, badsound) };
    if altscore == SCORE_MAXMAX {
        altscore = SCORE_BIG;
    }
    (rescore(score, altscore), altscore)
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
pub(super) unsafe extern "C" fn sug_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: `qsort` passes pointers to the elements of the array it was
    // given, which is an array of `suggest_T`.
    let (p1, p2) = unsafe { (&*(s1 as *const suggest_T), &*(s2 as *const suggest_T)) };
    match p1.st_score.cmp(&p2.st_score) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Greater => 1,
        core::cmp::Ordering::Equal => match p1.st_altscore.cmp(&p2.st_altscore) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Greater => 1,
            // SAFETY: each entry's word is NUL-terminated.
            core::cmp::Ordering::Equal => unsafe { strcasecmp(p1.word(), p2.word()) },
        },
    }
}

/// Sort the suggestions and drop the ones that will not be displayed.
///
/// Returns the new score ceiling: the score of the worst suggestion still
/// in the list, which nothing worse can now beat.
///
/// # Safety
///
/// `gap` must be a live suggestion list.
pub(super) unsafe fn cleanup_suggestions(
    gap: *mut Vec<suggest_T>,
    maxscore: c_int,
    keep: c_int,
) -> c_int {
    // SAFETY: the caller guarantees the list.
    let list = unsafe { &mut *gap };
    if list.is_empty() {
        return maxscore;
    }

    let cmp = Some(sug_compare as unsafe extern "C" fn(*const c_void, *const c_void) -> c_int)
        as __compar_fn_t;
    // SAFETY: the buffer really is `len` `suggest_T`s, which is the size
    // the comparator reads at each of the pointers `qsort` hands it, and
    // `qsort` only permutes them -- which for these is a move.
    let (data, len) = (list.as_mut_ptr().cast::<c_void>(), list.len());
    unsafe { qsort(data, len as size_t, size_of::<suggest_T>() as size_t, cmp) };

    let keep = keep.max(0) as usize;
    if list.len() <= keep {
        return maxscore;
    }
    // Each entry dropped here owns its word, so truncating frees them.
    list.truncate(keep);
    if keep >= 1 {
        return list[keep - 1].st_score;
    }
    maxscore
}

/// Score every suggestion in `su_ga` by sound and put the ones that are
/// close enough into `su_sga`.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
pub(super) unsafe fn score_comp_sal(su: *mut suginfo_T) {
    // Use the sound folding of the first language that has any.
    //
    // SAFETY: the languages come from the current window's loaded list.
    let langs = unsafe { window_langs() };
    let Some(lp) = langs
        .iter()
        .find(|lp| unsafe { (*lp.lp_slang).has_soundfold() })
    else {
        return;
    };
    let slang = lp.lp_slang;

    let mut badsound = EMPTY_SOUND;
    // SAFETY: `su_fbadword` is the bad word's NUL-terminated fold and
    // `badsound` has room for a soundfold.
    let fbadword = unsafe { &raw mut (*su).su_fbadword } as *mut c_char;
    unsafe { spell_soundfold(slang, fbadword, true, badsound.as_mut_ptr()) };

    // SAFETY: `su` is valid by the contract above, so both lists are its
    // own. Neither is borrowed across the scoring, which reads the rest of
    // `su`; the entries copied across are new ones, so the word is copied
    // rather than moved.
    for i in 0..unsafe { (*su).su_ga.len() } {
        let (word, wordlen, orglen) = {
            let stp = &unsafe { &(*su).su_ga }[i];
            (stp.word(), stp.st_wordlen, stp.st_orglen)
        };
        let score = unsafe { stp_sal_score(word, wordlen, orglen, &*su, slang, &badsound) };
        if score >= SCORE_MAXMAX {
            continue;
        }
        let copy = unsafe { &(*su).su_ga }[i].st_word.clone();
        unsafe {
            (*su).su_sga.push(suggest_T {
                st_word: copy,
                st_wordlen: wordlen,
                st_orglen: orglen,
                st_score: score,
                st_altscore: 0,
                st_salscore: false,
                st_had_bonus: false,
                st_slang: ptr::null_mut(),
            });
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
pub(super) unsafe fn score_combine(su: *mut suginfo_T) {
    let mut badsound = EMPTY_SOUND;
    let mut slang: *mut slang_T = ptr::null_mut();

    // Give the edit-distance list a sound-a-like score.
    //
    // SAFETY: the languages come from the current window's loaded list;
    // `su` is valid by the contract above, so `su_fbadword` is the bad
    // word's NUL-terminated fold and `su_ga` one of its own growarrays of
    // `suggest_T`. `badsound` has room for a soundfold.
    let langs = unsafe { window_langs() };
    if let Some(lp) = langs
        .iter()
        .find(|lp| unsafe { (*lp.lp_slang).has_soundfold() })
    {
        slang = lp.lp_slang;
        let fbadword = unsafe { &raw mut (*su).su_fbadword } as *mut c_char;
        unsafe { spell_soundfold(slang, fbadword, true, badsound.as_mut_ptr()) };

        for i in 0..unsafe { (*su).su_ga.len() } {
            let (word, wordlen, orglen, score) = {
                let stp = &unsafe { &(*su).su_ga }[i];
                (stp.word(), stp.st_wordlen, stp.st_orglen, stp.st_score)
            };
            let altscore = unsafe { stp_sal_score(word, wordlen, orglen, &*su, slang, &badsound) };
            let alt = if altscore == SCORE_MAXMAX {
                SCORE_BIG
            } else {
                altscore
            };
            let stp = &mut unsafe { &mut (*su).su_ga }[i];
            stp.st_altscore = altscore;
            stp.st_score = rescore(score, alt);
            stp.st_salscore = false;
        }
    }

    if slang.is_null() {
        // "double" without a language that can sound-fold.
        //
        // SAFETY: as above.
        unsafe { cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount) };
        return;
    }

    // Give the sound-a-like list an edit-distance score. Here the
    // sound score is the one that dominates.
    //
    // SAFETY: `su_sga` is one of `su`'s own lists, `slang` is the loaded
    // language found above, and `su_badword` and every word are
    // NUL-terminated. The list is taken afresh at each step rather than
    // held across the scoring, which reads the rest of `su`.
    for i in 0..unsafe { (*su).su_sga.len() } {
        let (word, score) = {
            let stp = &unsafe { &(*su).su_sga }[i];
            (stp.word(), stp.st_score)
        };
        let badword = unsafe { &raw const (*su).su_badword } as *const c_char;
        let altscore = unsafe { spell_edit_score(Some(&*slang), badword, word) };
        let base = if score == SCORE_MAXMAX {
            SCORE_BIG
        } else {
            score
        };
        let stp = &mut unsafe { &mut (*su).su_sga }[i];
        stp.st_altscore = altscore;
        stp.st_score = (base * 7 + altscore) / 8;
        stp.st_salscore = true;
    }

    // SAFETY: both lists are `su`'s own.
    unsafe { check_suggestions(su, &raw mut (*su).su_ga) };
    unsafe { cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount) };
    unsafe { check_suggestions(su, &raw mut (*su).su_sga) };
    unsafe { cleanup_suggestions(&raw mut (*su).su_sga, (*su).su_maxscore, (*su).su_maxcount) };

    // Take from the two lists alternately, skipping a word already
    // merged; its copy is then nobody's, and dropping it frees it.
    //
    // SAFETY: `su` is valid by the contract above.
    let (mut main, mut sound) = unsafe {
        (
            mem::take(&mut (*su).su_ga).into_iter(),
            mem::take(&mut (*su).su_sga).into_iter(),
        )
    };
    let mut merged: Vec<suggest_T> = Vec::new();
    loop {
        let round = [main.next(), sound.next()];
        if round.iter().all(Option::is_none) {
            break;
        }
        for candidate in round.into_iter().flatten() {
            if !merged.iter().any(|seen| seen.st_word == candidate.st_word) {
                merged.push(candidate);
            }
        }
    }

    // Keep only what will be displayed; the entries dropped own their
    // words.
    //
    // SAFETY: as above.
    let maxcount = unsafe { (*su).su_maxcount }.max(0) as usize;
    merged.truncate(maxcount);
    unsafe { (*su).su_ga = merged };
}
