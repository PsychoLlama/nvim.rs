//! Keeping the list of candidate suggestions.
//!
//! Everything that finds a candidate word funnels into [`add_suggestion`],
//! which is the only place a `Suggest` is created. It also decides what
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

#![forbid(unsafe_code)]

use crate::cstr::until_nul;
use crate::winlayer::Win;

use crate::highlight_group::HLF_COUNT;
use crate::mbyte::{char_at, head_off};
use crate::spell::{loaded_langs, spell_check, spell_soundfold};
use crate::spellsuggest::score::{EMPTY_SOUND, spell_edit_score, stp_sal_score};
use crate::spellsuggest::{MAXWLEN, SCORE_INS, SCORE_MAXMAX, SugInfo, Suggest, window_langs};
use crate::types::{LangId, SpellLang};
use core::cmp::Ordering;
use core::ffi::c_int;
use core::mem;

/// A sound-a-like score that could not be computed stands in as "three
/// insertions apart", which is far but not infinitely so.
const SCORE_BIG: c_int = SCORE_INS * 3;

/// Which of the two suggestion lists a candidate goes into.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum List {
    /// `su_ga`, the list that is shown.
    Main,
    /// `su_sga`, the sound-a-like list of "double" mode.
    Sound,
}

impl SugInfo {
    fn list(&self, list: List) -> &Vec<Suggest> {
        match list {
            List::Main => &self.su_ga,
            List::Sound => &self.su_sga,
        }
    }

    fn list_mut(&mut self, list: List) -> &mut Vec<Suggest> {
        match list {
            List::Main => &mut self.su_ga,
            List::Sound => &mut self.su_sga,
        }
    }
}

/// Blend a word's edit-distance score with its sound-a-like score. The
/// edit distance dominates; the sound only nudges.
fn rescore(word_score: c_int, sound_score: c_int) -> c_int {
    (3 * word_score + sound_score) / 4
}

/// How many suggestions to keep when the list is cleaned up. Always
/// comfortably more than will be displayed, because a later pass can
/// rescore them and change the order.
pub(super) fn clean_count(su: &SugInfo) -> c_int {
    if su.su_maxcount < 130 {
        150
    } else {
        su.su_maxcount + 20
    }
}

/// The size the list is allowed to reach before it gets cleaned up.
fn max_count(su: &SugInfo) -> c_int {
    clean_count(su) + 50
}

/// Add a suggestion to the list, or lower the score of the one already
/// there.
///
/// `goodword` ends at its end or at a NUL. `badlenarg` is how much of the
/// bad word it replaces; `maxsf` says whether the caller's score ceiling
/// is the sound-fold one.
#[allow(clippy::too_many_arguments)]
pub(super) fn add_suggestion(
    su: &mut SugInfo,
    list: List,
    goodword: &[u8],
    badlenarg: c_int,
    score: c_int,
    altscore: c_int,
    had_bonus: bool,
    slang: Option<LangId>,
    maxsf: bool,
) {
    let goodword = until_nul(goodword);
    // Minimise "badlen" for consistency: changing "the the" to "thee
    // the" should not be listed next to changing the first "the" to
    // "thee". Walk both tails back over the characters they share.
    // The lengths are the ones measured before the last step back: a
    // shared trailing character is not part of the replacement.
    let bad = &su.su_bad[..];
    let mut gi = goodword.len();
    let mut bi = badlenarg.max(0) as usize;
    let (goodlen, badlen) = loop {
        let lens = (gi as c_int, bi as c_int);
        // The second test only bites on a replaced length past the end of
        // the line, which the C read on into whatever followed it.
        if gi == 0 || bi == 0 || bi > bad.len() {
            break lens;
        }
        gi -= head_off(goodword, gi - 1) + 1;
        bi -= head_off(bad, bi - 1) + 1;
        if char_at(&goodword[gi..]) != char_at(&bad[bi..]) {
            break lens;
        }
    };

    if badlen == 0 && goodlen == 0 {
        // The good word changes nothing; happens when "the the" has its
        // first "the" replaced by itself.
        return;
    }
    let good = &goodword[..goodlen as usize];

    // Already in the list? The replaced length is part of the identity:
    // "thes," -> "these" is a different suggestion from "thes" ->
    // "these".
    let found = su
        .list(list)
        .iter()
        .position(|stp| stp.st_orglen == badlen && stp.word_bytes() == good);
    if let Some(at) = found {
        let had_bonus_before = {
            let stp = &mut su.list_mut(list)[at];
            if stp.st_slang.is_none() {
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
            if had_bonus {
                rescore_one(su, list, at);
                let stp = &su.list(list)[at];
                new_score = stp.st_score;
                new_altscore = stp.st_altscore;
                new_had_bonus = stp.st_had_bonus;
            } else if let Some(id) = slang {
                let rescored = loaded_langs.with(|langs| {
                    let lang = langs.get(id).filter(|lang| lang.has_soundfold())?;
                    Some(sal_rescore(su, id, lang, good, badlen, score))
                });
                if let Some((s, a)) = rescored {
                    new_score = s;
                    new_altscore = a;
                    new_had_bonus = true;
                }
            }
        }

        let stp = &mut su.list_mut(list)[at];
        if stp.st_score > new_score {
            stp.st_score = new_score;
            stp.st_altscore = new_altscore;
            stp.st_had_bonus = new_had_bonus;
        }
        return;
    }

    let entry = Suggest {
        st_word: good.into(),
        st_wordlen: goodlen,
        st_orglen: badlen,
        st_score: score,
        st_altscore: altscore,
        st_salscore: false,
        st_had_bonus: had_bonus,
        st_slang: slang,
    };
    su.list_mut(list).push(entry);

    // Far enough over the display count that sorting pays for itself.
    if su.list(list).len() as c_int > max_count(su) {
        let keep = clean_count(su);
        let ceiling = if maxsf {
            su.su_sfmaxscore
        } else {
            su.su_maxscore
        };
        let ceiling = cleanup_suggestions(su.list_mut(list), ceiling, keep);
        if maxsf {
            su.su_sfmaxscore = ceiling;
        } else {
            su.su_maxscore = ceiling;
        }
    }
}

/// Drop suggestions that are themselves misspelled.
///
/// Banned words and split words -- "the the" -- only show up as errors
/// once the replacement is checked against the rest of the line, which is
/// why this cannot be done as candidates arrive.
pub(super) fn check_suggestions(su: &mut SugInfo, list: List) {
    let mut longword = Vec::with_capacity(MAXWLEN);
    for i in (0..su.list(list).len()).rev() {
        let stp = &su.list(list)[i];
        // Append what follows in the line, so that "the the" is
        // recognisable; at most `MAXWLEN` bytes of the two.
        let word = stp.word_bytes();
        let word = &word[..word.len().min(MAXWLEN)];
        let rest = su.bad_from(stp.st_orglen as usize);
        longword.clear();
        longword.extend_from_slice(word);
        longword.extend_from_slice(&rest[..rest.len().min(MAXWLEN - word.len())]);

        let (_, attr) = spell_check(Win::current(), &longword, None, false);
        if attr != HLF_COUNT {
            su.list_mut(list).remove(i);
        }
    }
}

/// Remember a word that must never be suggested. It ends at its end or at a
/// NUL.
pub(super) fn add_banned(su: &mut SugInfo, word: &[u8]) {
    let word = until_nul(word);
    if !su.su_banned.contains(word) {
        su.su_banned.insert(word.into());
    }
}

/// Recompute every suggestion's score with sound folding taken into
/// account. Slow, so only done once the list is final.
pub(super) fn rescore_suggestions(su: &mut SugInfo) {
    if su.su_sallang.is_none() {
        return;
    }
    for at in 0..su.su_ga.len() {
        rescore_one(su, List::Main, at);
    }
}

/// Recompute one suggestion's score with sound folding taken into account.
pub(super) fn rescore_one(su: &mut SugInfo, list: List, at: usize) {
    let stp = &su.list(list)[at];
    // Only worth doing for a suggestion that has no sound-a-like score
    // yet and knows which language it came from.
    let Some(id) = stp.st_slang.filter(|_| !stp.st_had_bonus) else {
        return;
    };
    let rescored = loaded_langs.with(|langs| {
        let lang = langs.get(id).filter(|lang| lang.has_soundfold())?;
        let stp = &su.list(list)[at];
        Some(sal_rescore(
            su,
            id,
            lang,
            stp.word_bytes(),
            stp.st_orglen,
            stp.st_score,
        ))
    });
    let Some((score, altscore)) = rescored else {
        return;
    };

    let stp = &mut su.list_mut(list)[at];
    stp.st_score = score;
    stp.st_altscore = altscore;
    stp.st_had_bonus = true;
}

/// What sound folding scores one suggestion, the language `id` being
/// `lang`: the pair that replaces its `st_score` and `st_altscore`.
fn sal_rescore(
    su: &SugInfo,
    id: LangId,
    lang: &SpellLang,
    word: &[u8],
    orglen: c_int,
    score: c_int,
) -> (c_int, c_int) {
    let mut sal_badword = EMPTY_SOUND;
    let badsound = if Some(id) == su.su_sallang {
        &su.su_sal_badword
    } else {
        spell_soundfold(lang, &su.su_fbadword, true, &mut sal_badword);
        &sal_badword
    };

    let mut altscore = stp_sal_score(word, orglen, su, lang, badsound);
    if altscore == SCORE_MAXMAX {
        altscore = SCORE_BIG;
    }
    (rescore(score, altscore), altscore)
}

/// `strcasecmp` over two words: bytes compared with ASCII letters folded
/// to lower case, the end of the shorter one reading as a NUL.
pub(super) fn ascii_casecmp(a: &[u8], b: &[u8]) -> Ordering {
    let mut i = 0;
    loop {
        let x = a.get(i).map_or(0, u8::to_ascii_lowercase);
        let y = b.get(i).map_or(0, u8::to_ascii_lowercase);
        if x != y || x == 0 {
            return x.cmp(&y);
        }
        i += 1;
    }
}

/// Order suggestions by score, then by sound-a-like score, then by word.
///
/// The last comparison is case-insensitive, so two entries differing only
/// in case tie; the sort is stable, as glibc's `qsort` (a merge sort) was,
/// so they keep the order they were found in.
fn sug_compare(p1: &Suggest, p2: &Suggest) -> Ordering {
    p1.st_score
        .cmp(&p2.st_score)
        .then(p1.st_altscore.cmp(&p2.st_altscore))
        .then_with(|| ascii_casecmp(p1.word_bytes(), p2.word_bytes()))
}

/// Sort the suggestions and drop the ones that will not be displayed.
///
/// Returns the new score ceiling: the score of the worst suggestion still
/// in the list, which nothing worse can now beat.
pub(super) fn cleanup_suggestions(list: &mut Vec<Suggest>, maxscore: c_int, keep: c_int) -> c_int {
    if list.is_empty() {
        return maxscore;
    }
    list.sort_by(sug_compare);

    let keep = keep.max(0) as usize;
    if list.len() <= keep {
        return maxscore;
    }
    list.truncate(keep);
    if keep >= 1 {
        return list[keep - 1].st_score;
    }
    maxscore
}

/// The first language the window has that can sound-fold, and the bad word
/// folded by it -- for `f` to score with.
fn with_first_soundfold<R>(
    su: &SugInfo,
    f: impl FnOnce(&SpellLang, &[u8; MAXWLEN]) -> R,
) -> Option<R> {
    let langs = window_langs();
    loaded_langs.with(|table| {
        let lang = langs
            .iter()
            .filter_map(|lp| table.get(lp.slang))
            .find(|lang| lang.has_soundfold())?;
        let mut badsound = EMPTY_SOUND;
        spell_soundfold(lang, &su.su_fbadword, true, &mut badsound);
        Some(f(lang, &badsound))
    })
}

/// Score every suggestion in `su_ga` by sound and put the ones that are
/// close enough into `su_sga`.
pub(super) fn score_comp_sal(su: &mut SugInfo) {
    // Use the sound folding of the first language that has any.
    let found = with_first_soundfold(su, |lang, badsound| {
        su.su_ga
            .iter()
            .filter_map(|stp| {
                let score = stp_sal_score(stp.word_bytes(), stp.st_orglen, su, lang, badsound);
                (score < SCORE_MAXMAX).then(|| Suggest {
                    st_word: stp.st_word.clone(),
                    st_wordlen: stp.st_wordlen,
                    st_orglen: stp.st_orglen,
                    st_score: score,
                    st_altscore: 0,
                    st_salscore: false,
                    st_had_bonus: false,
                    st_slang: None,
                })
            })
            .collect::<Vec<_>>()
    });
    if let Some(found) = found {
        su.su_sga.extend(found);
    }
}

/// Merge the edit-distance list and the sound-a-like list into one.
///
/// Each list is first given the other measure's score so that both are
/// comparable, then the two are taken from alternately -- best of one,
/// best of the other -- so that "double" mode shows a mixture rather than
/// whichever measure happened to score lower.
pub(super) fn score_combine(su: &mut SugInfo) {
    // Give the edit-distance list a sound-a-like score, and the sound-a-like
    // list an edit-distance one. Here the sound score is the one that
    // dominates.
    let scores = with_first_soundfold(su, |lang, badsound| {
        let main: Vec<c_int> = su
            .su_ga
            .iter()
            .map(|stp| stp_sal_score(stp.word_bytes(), stp.st_orglen, su, lang, badsound))
            .collect();
        let sound: Vec<c_int> = su
            .su_sga
            .iter()
            .map(|stp| spell_edit_score(Some(lang), su.badword(), stp.word_bytes()))
            .collect();
        (main, sound)
    });

    let Some((main, sound)) = scores else {
        // "double" without a language that can sound-fold.
        let (maxscore, maxcount) = (su.su_maxscore, su.su_maxcount);
        cleanup_suggestions(&mut su.su_ga, maxscore, maxcount);
        return;
    };
    for (stp, altscore) in su.su_ga.iter_mut().zip(main) {
        let alt = if altscore == SCORE_MAXMAX {
            SCORE_BIG
        } else {
            altscore
        };
        stp.st_altscore = altscore;
        stp.st_score = rescore(stp.st_score, alt);
        stp.st_salscore = false;
    }
    for (stp, altscore) in su.su_sga.iter_mut().zip(sound) {
        let base = if stp.st_score == SCORE_MAXMAX {
            SCORE_BIG
        } else {
            stp.st_score
        };
        stp.st_altscore = altscore;
        stp.st_score = (base * 7 + altscore) / 8;
        stp.st_salscore = true;
    }

    let (maxscore, maxcount) = (su.su_maxscore, su.su_maxcount);
    check_suggestions(su, List::Main);
    cleanup_suggestions(&mut su.su_ga, maxscore, maxcount);
    check_suggestions(su, List::Sound);
    cleanup_suggestions(&mut su.su_sga, maxscore, maxcount);

    // Take from the two lists alternately, skipping a word already
    // merged.
    let mut main = mem::take(&mut su.su_ga).into_iter();
    let mut sound = mem::take(&mut su.su_sga).into_iter();
    let mut merged: Vec<Suggest> = Vec::new();
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

    // Keep only what will be displayed.
    merged.truncate(su.su_maxcount.max(0) as usize);
    su.su_ga = merged;
}
