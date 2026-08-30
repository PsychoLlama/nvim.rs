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
use crate::garray::{ga_append_via_ptr, ga_clear, ga_grow, ga_init};
use crate::hashtab::{hash_add_item, hash_hash, hash_lookup, hash_removed};
use crate::highlight_group::HLF_COUNT;
use crate::main::curwin;
use crate::mbyte::{utf_head_off, utf_ptr2char};
use crate::memory::{xfree, xmemdupz, xstrdup, xstrlcpy};
use crate::spell::{spell_check, spell_soundfold};
use crate::spellsuggest::score::{EMPTY_SOUND, spell_edit_score, stp_sal_score};
use crate::spellsuggest::{MAXWLEN, SCORE_INS, SCORE_MAXMAX, suggest_T, suginfo_T, window_langs};
use crate::types::{__compar_fn_t, garray_T, hlf_T, size_t, slang_T};
use ::libc::{qsort, strcasecmp, strlen};
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr, slice};

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

/// The suggestions a garray holds.
///
/// # Safety
///
/// `gap` must be a garray of `suggest_T`.
pub(super) unsafe fn suggestions<'a>(gap: *mut garray_T) -> &'a mut [suggest_T] {
    // SAFETY: the caller guarantees the element type; an empty garray has a
    // null data pointer, which `from_raw_parts_mut` rejects even at length
    // zero, so both fields are tested before the slice is built.
    if unsafe { (*gap).ga_data.is_null() } || unsafe { (*gap).ga_len } <= 0 {
        &mut []
    } else {
        let data = unsafe { (*gap).ga_data } as *mut suggest_T;
        let len = unsafe { (*gap).ga_len } as usize;
        unsafe { slice::from_raw_parts_mut(data, len) }
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
pub(super) unsafe fn add_suggestion(
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
    let mut pgood = unsafe { goodword.add(strlen(goodword) as usize) };
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
    // SAFETY: `gap` is a garray of `suggest_T` by the contract above, and
    // `goodlen` bytes of `goodword` are its own.
    for stp in unsafe { suggestions(gap) } {
        if stp.st_wordlen != goodlen
            || stp.st_orglen != badlen
            || !unsafe { cstr::prefix_eq(stp.st_word, goodword, goodlen as size_t) }
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
            //
            // SAFETY: `su` is valid by the contract above.
            if had_bonus {
                unsafe { rescore_one(su, stp) };
            } else {
                new_sug.st_orglen = badlen;
                unsafe { rescore_one(su, &mut new_sug) };
            }
        }

        if stp.st_score > new_sug.st_score {
            stp.st_score = new_sug.st_score;
            stp.st_altscore = new_sug.st_altscore;
            stp.st_had_bonus = new_sug.st_had_bonus;
        }
        return;
    }

    // SAFETY: `gap` is a garray of `suggest_T`, so the slot appended holds
    // one, and `goodlen` bytes of `goodword` are its own.
    let stp = unsafe { ga_append_via_ptr(gap, size_of::<suggest_T>()) } as *mut suggest_T;
    let word = unsafe { xmemdupz(goodword as *const c_void, goodlen as usize) } as *mut c_char;
    let sug = suggest_T {
        st_word: word,
        st_wordlen: goodlen,
        st_orglen: badlen,
        st_score: score,
        st_altscore: altscore,
        st_salscore: false,
        st_had_bonus: had_bonus,
        st_slang: slang,
    };
    unsafe { *stp = sug };

    // Far enough over the display count that sorting pays for itself.
    //
    // SAFETY: `su` is valid by the contract above.
    if unsafe { (*gap).ga_len } > max_count(unsafe { &*su }) {
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
pub(super) unsafe fn check_suggestions(su: *mut suginfo_T, gap: *mut garray_T) {
    let mut longword = [0 as c_char; MAXWLEN + 1];
    // SAFETY: `gap` holds `suggest_T`s by the contract above, so `i` below
    // -- taken from `0..ga_len` -- indexes one of them.
    let stp = unsafe { (*gap).ga_data } as *mut suggest_T;
    for i in (0..unsafe { (*gap).ga_len }).rev() {
        // SAFETY: as above.
        let sug = unsafe { &*stp.offset(i as isize) };
        // Append what follows in the line, so that "the the" is
        // recognisable.
        //
        // SAFETY: `longword` is `MAXWLEN + 1` bytes and both copies are
        // told how much of it is left; `su_badptr` points into the line the
        // bad word came from, of which `st_orglen` bytes are replaced.
        unsafe { xstrlcpy(longword.as_mut_ptr(), sug.st_word, MAXWLEN + 1) };
        let len = sug.st_wordlen;
        let tail = unsafe { longword.as_mut_ptr().offset(len as isize) };
        let rest = unsafe { (*su).su_badptr.offset(sug.st_orglen as isize) };
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

        // SAFETY: the suggestion owns its word; dropping it leaves a hole
        // the entries after `i` are shifted down over, and there are
        // exactly `ga_len - i` of those.
        unsafe { xfree(sug.st_word as *mut c_void) };
        unsafe { (*gap).ga_len -= 1 };
        if i < unsafe { (*gap).ga_len } {
            let moved = unsafe { (*gap).ga_len - i } as usize;
            unsafe { ptr::copy(stp.offset(i as isize + 1), stp.offset(i as isize), moved) };
        }
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
    let word_len = unsafe { strlen(word) } as usize;
    let hi = unsafe { hash_lookup(&raw mut (*su).su_banned, word, word_len, hash) };
    let key = unsafe { (*hi).hi_key };
    if !(key.is_null() || ptr::eq(key, &raw const hash_removed)) {
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
    // SAFETY: `su` is valid by the contract above, so `su_ga` is one of its
    // own growarrays of `suggest_T` and `i` indexes one of its entries.
    if unsafe { (*su).su_sallang }.is_null() {
        return;
    }
    let gap = unsafe { &raw mut (*su).su_ga };
    for i in 0..unsafe { (*gap).ga_len } {
        let stp = unsafe { ((*gap).ga_data as *mut suggest_T).offset(i as isize) };
        unsafe { rescore_one(su, &mut *stp) };
    }
}

/// Recompute one suggestion's score with sound folding taken into account.
///
/// # Safety
///
/// `su` must be valid.
pub(super) unsafe fn rescore_one(su: *mut suginfo_T, stp: &mut suggest_T) {
    let slang = stp.st_slang;
    // Only worth doing for a suggestion that has no sound-a-like score
    // yet and knows which language it came from.
    //
    // SAFETY: `st_slang` is either null -- which the `||` tests first -- or
    // a loaded language that outlives the suggestion list.
    if slang.is_null() || unsafe { (*slang).sl_sal.ga_len } <= 0 || stp.st_had_bonus {
        return;
    }

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
    stp.st_altscore = unsafe { stp_sal_score(stp, &*su, slang, badsound) };
    if stp.st_altscore == SCORE_MAXMAX {
        stp.st_altscore = SCORE_BIG;
    }
    stp.st_score = rescore(stp.st_score, stp.st_altscore);
    stp.st_had_bonus = true;
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
            core::cmp::Ordering::Equal => unsafe { strcasecmp(p1.st_word, p2.st_word) },
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
/// `gap` must hold `suggest_T`s.
pub(super) unsafe fn cleanup_suggestions(
    gap: *mut garray_T,
    maxscore: c_int,
    keep: c_int,
) -> c_int {
    // SAFETY: the caller guarantees the element type.
    if unsafe { (*gap).ga_len } <= 0 {
        return maxscore;
    }

    let data = unsafe { (*gap).ga_data };
    let len = unsafe { (*gap).ga_len } as size_t;
    let cmp = Some(sug_compare as unsafe extern "C" fn(*const c_void, *const c_void) -> c_int)
        as __compar_fn_t;
    // SAFETY: the array really is `len` `suggest_T`s, which is the size the
    // comparator reads at each of the pointers `qsort` hands it.
    unsafe { qsort(data, len, size_of::<suggest_T>() as size_t, cmp) };

    if unsafe { (*gap).ga_len } <= keep {
        return maxscore;
    }

    // SAFETY: as above; `keep` is below `ga_len` here, so every index the
    // two reads below use is inside the array, and each entry owns its
    // word until it is dropped from the list.
    let stp = unsafe { (*gap).ga_data } as *mut suggest_T;
    for i in keep..unsafe { (*gap).ga_len } {
        unsafe { xfree((*stp.offset(i as isize)).st_word as *mut c_void) };
    }
    unsafe { (*gap).ga_len = keep };
    if keep >= 1 {
        return unsafe { (*stp.offset(keep as isize - 1)).st_score };
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
    // SAFETY: `su` is valid by the contract above, so `su_sga` is one of
    // its own growarrays; growing it to `su_ga`'s length reserves a slot
    // for every entry the loop below can copy across.
    unsafe { ga_grow(&raw mut (*su).su_sga, (*su).su_ga.ga_len) };

    // Use the sound folding of the first language that has any.
    //
    // SAFETY: the languages come from the current window's loaded list.
    let langs = unsafe { window_langs() };
    let Some(lp) = langs
        .iter()
        .find(|lp| unsafe { (*lp.lp_slang).sl_sal.ga_len } > 0)
    else {
        return;
    };
    let slang = lp.lp_slang;

    let mut badsound = EMPTY_SOUND;
    // SAFETY: `su_fbadword` is the bad word's NUL-terminated fold and
    // `badsound` has room for a soundfold.
    let fbadword = unsafe { &raw mut (*su).su_fbadword } as *mut c_char;
    unsafe { spell_soundfold(slang, fbadword, true, badsound.as_mut_ptr()) };

    // SAFETY: `i` is inside `su_ga`, and `su_sga` has the slot reserved
    // above for each entry taken from it.
    for i in 0..unsafe { (*su).su_ga.ga_len } {
        let stp = unsafe { &*((*su).su_ga.ga_data as *mut suggest_T).offset(i as isize) };
        let score = unsafe { stp_sal_score(stp, &*su, slang, &badsound) };
        if score >= SCORE_MAXMAX {
            continue;
        }
        let sga = unsafe { &raw mut (*su).su_sga };
        let sstp = unsafe { ((*sga).ga_data as *mut suggest_T).offset((*sga).ga_len as isize) };
        unsafe { (*sstp).st_word = xstrdup(stp.st_word) };
        unsafe { (*sstp).st_wordlen = stp.st_wordlen };
        unsafe { (*sstp).st_score = score };
        unsafe { (*sstp).st_altscore = 0 };
        unsafe { (*sstp).st_orglen = stp.st_orglen };
        unsafe { (*su).su_sga.ga_len += 1 };
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
        .find(|lp| unsafe { (*lp.lp_slang).sl_sal.ga_len } > 0)
    {
        slang = lp.lp_slang;
        let fbadword = unsafe { &raw mut (*su).su_fbadword } as *mut c_char;
        unsafe { spell_soundfold(slang, fbadword, true, badsound.as_mut_ptr()) };

        for stp in unsafe { suggestions(&raw mut (*su).su_ga) } {
            stp.st_altscore = unsafe { stp_sal_score(stp, &*su, slang, &badsound) };
            let alt = if stp.st_altscore == SCORE_MAXMAX {
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
        //
        // SAFETY: as above.
        unsafe { cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount) };
        return;
    }

    // Give the sound-a-like list an edit-distance score. Here the
    // sound score is the one that dominates.
    //
    // SAFETY: `su_sga` is one of `su`'s own growarrays of `suggest_T`,
    // `slang` is the loaded language found above, and `su_badword` and
    // every `st_word` are NUL-terminated.
    for stp in unsafe { suggestions(&raw mut (*su).su_sga) } {
        let badword = unsafe { &raw const (*su).su_badword } as *const c_char;
        stp.st_altscore = unsafe { spell_edit_score(Some(&*slang), badword, stp.st_word) };
        let base = if stp.st_score == SCORE_MAXMAX {
            SCORE_BIG
        } else {
            stp.st_score
        };
        stp.st_score = (base * 7 + stp.st_altscore) / 8;
        stp.st_salscore = true;
    }

    // SAFETY: both lists are `su`'s own growarrays of `suggest_T`.
    unsafe { check_suggestions(su, &raw mut (*su).su_ga) };
    unsafe { cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount) };
    unsafe { check_suggestions(su, &raw mut (*su).su_sga) };
    unsafe { cleanup_suggestions(&raw mut (*su).su_sga, (*su).su_maxscore, (*su).su_maxcount) };

    // SAFETY: `garray_T` is all-integer plus a pointer, for which an
    // all-zero value is the empty garray `ga_init` then describes; growing
    // it to both lists' lengths reserves a slot for every entry merged.
    let mut ga: garray_T = unsafe { mem::zeroed() };
    unsafe { ga_init(&raw mut ga, size_of::<suggest_T>() as c_int, 1) };
    unsafe { ga_grow(&raw mut ga, (*su).su_ga.ga_len + (*su).su_sga.ga_len) };
    let merged = ga.ga_data as *mut suggest_T;

    let su_ga = unsafe { &raw mut (*su).su_ga };
    let su_sga = unsafe { &raw mut (*su).su_sga };
    let rounds = [su_ga, su_sga];
    let longest = unsafe { (*su).su_ga.ga_len.max((*su).su_sga.ga_len) };
    for i in 0..longest {
        for gap in rounds {
            // SAFETY: as above -- `gap` is one of the two lists.
            if i >= unsafe { (*gap).ga_len } {
                continue;
            }
            // SAFETY: `i` is inside `gap`'s list, and `j` inside `merged`,
            // whose entries own NUL-terminated words.
            let candidate = unsafe { *((*gap).ga_data as *mut suggest_T).offset(i as isize) };
            // Skip a word that is already in the merged list; its copy
            // of the word is then nobody's, so free it.
            let seen = (0..ga.ga_len).any(|j| unsafe {
                same_word((*merged.offset(j as isize)).st_word, candidate.st_word)
            });
            if seen {
                unsafe { xfree(candidate.st_word as *mut c_void) };
            } else {
                unsafe { *merged.offset(ga.ga_len as isize) = candidate };
                ga.ga_len += 1;
            }
        }
    }

    // SAFETY: every entry of both lists has been moved into `ga` or freed.
    unsafe { ga_clear(&raw mut (*su).su_ga) };
    unsafe { ga_clear(&raw mut (*su).su_sga) };

    // Keep only what will be displayed.
    //
    // SAFETY: the entries past `su_maxcount` are inside `merged` and own
    // their words; `su_ga` was cleared above, so it may be overwritten.
    if ga.ga_len > unsafe { (*su).su_maxcount } {
        for i in unsafe { (*su).su_maxcount }..ga.ga_len {
            unsafe { xfree((*merged.offset(i as isize)).st_word as *mut c_void) };
        }
        ga.ga_len = unsafe { (*su).su_maxcount };
    }

    unsafe { (*su).su_ga = ga };
}

/// Exact string equality, as the merge's duplicate test uses.
///
/// # Safety
///
/// Both pointers must be NUL-terminated.
unsafe fn same_word(a: *const c_char, b: *const c_char) -> bool {
    // SAFETY: the caller guarantees both strings.
    unsafe { cstr::eq(a, b) }
}
