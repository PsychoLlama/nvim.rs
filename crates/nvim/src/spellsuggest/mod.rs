//! Spelling suggestions: what to offer instead of a badly spelled word.
//!
//! One run starts with a bad word and ends with a scored, sorted list of
//! replacements. [`spell_find_suggest`] is the top of it: it fills a
//! [`SugInfo`] with everything the search needs — the bad word as typed,
//! case-folded and sound-folded, its capitalisation flags and the score
//! ceiling — and then walks `'spellsuggest'` to decide which methods run.
//!
//! `'spellsuggest'` is a comma-separated list, and each item is either a
//! source of suggestions or a knob on the internal one:
//!
//! - `best`, `fast` and `double` choose how the internal search scores
//!   what it finds. [`spell_check_sps`] turns them into [`sps_flags`],
//!   which [`spell_suggest_intern`] then reads.
//! - `expr:{expr}` and `file:{fname}` are outside sources, handled by
//!   [`spell_suggest_expr`] and [`spell_suggest_file`].
//! - `timeout:{ms}` bounds how long the trie walk may run for.
//! - A bare number caps how many suggestions `z=` lists.
//!
//! The two entry points are [`spell_suggest`], the interactive `z=`
//! command, and [`spell_suggest_list`], which is what the `spellsuggest()`
//! Vimscript function and `:spellrepall` are built on.
//!
//! # The submodules
//!
//! - [`prompt`] — the `z=` command itself: find the bad word, list the
//!   numbered choices, ask, and make the replacement.
//! - [`walk`] — the edit-distance search, which walks the language's word
//!   tree trying insertions, deletions, swaps and `REP` items.
//! - [`soundalike`] — the search over the sound-folded tree, which finds
//!   words that are spelled differently but sound the same.
//! - [`score`] — how far one word is from another, by letters and by
//!   sound.
//! - [`collect`] — keeping, deduplicating, rescoring and sorting the
//!   candidates the two searches produce.
//! - [`sps`] — the `'spellsuggest'` option itself: parsing its items, and
//!   the two outside sources (`expr:`, `file:`) they can name.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod collect;
mod prompt;
mod score;
mod soundalike;
mod sps;
mod walk;

#[cfg(test)]
mod tests;

use crate::cstr::until_nul;
use crate::memory::XString;
use crate::spell::{WordFlags, WordHasher};
use crate::winlayer::{Buf, Win};
pub(crate) use prompt::spell_suggest;
pub(crate) use sps::spell_check_sps;
use sps::{spell_suggest_expr, spell_suggest_file};

use crate::charset::atoi;
use crate::getchar::state::got_int;
use crate::getchar::vgetc;
use crate::global_cell::GlobalCell;
use crate::mbyte::{c_cluster_len, char_at};
use crate::option::split_option_part;
use crate::option::vars::P_SPS;
use crate::os::input::os_breakcheck;
use crate::spell::{
    captype, loaded_langs, make_case_word, spell_casefold, spell_check, spell_soundfold,
};
use crate::spellfile::suggest_load_files;
use crate::spellsuggest::collect::{
    List, add_banned, add_suggestion, check_suggestions, clean_count, cleanup_suggestions,
    rescore_suggestions, score_combine, score_comp_sal,
};
use crate::spellsuggest::score::spell_isupper;
use crate::spellsuggest::soundalike::{SoundDone, suggest_try_soundalike};
use crate::spellsuggest::walk::suggest_trie_walk;
use crate::types::{Hlf, LangId, LangP, MAXPATHL, NUL};
use core::ffi::{CStr, c_int};
use core::hash::BuildHasherDefault;
use std::collections::HashSet;

use crate::highlight_group::HLF_COUNT;
/// The longest word the spell code handles, and so the size of every word
/// buffer in this module tree.
pub(crate) use crate::spell::MAXWLEN;

/// A tab, which sound folding treats as a space.
pub(crate) const TAB: c_int = '\t' as c_int;

// Word flags. These live beside each word in the tree, except for
// `WordFlags::MIXCAP`, which only ever appears in `su_badflags`.

// What each kind of change costs. A suggestion's score is the sum over
// the changes that reach it, and lower is offered first.
/// Split the bad word in two.
pub(crate) const SCORE_SPLIT: c_int = 149;
/// Split it where the language says not to (`NOSPLITSUGS`).
pub(crate) const SCORE_SPLIT_NO: c_int = 249;
/// Only the case differs.
pub(crate) const SCORE_ICASE: c_int = 52;
/// The word belongs to another region.
pub(crate) const SCORE_REGION: c_int = 200;
/// The word is marked rare.
pub(crate) const SCORE_RARE: c_int = 180;
/// Swap two characters.
pub(crate) const SCORE_SWAP: c_int = 75;
/// Swap two characters that have a third between them.
pub(crate) const SCORE_SWAP3: c_int = 110;
/// Apply one `REP` item from the `.aff` file.
pub(crate) const SCORE_REP: c_int = 65;
/// Substitute a character.
pub(crate) const SCORE_SUBST: c_int = 93;
/// Substitute a character the language's `MAP` lines call similar.
pub(crate) const SCORE_SIMILAR: c_int = 33;
/// Substitute a composing character.
pub(crate) const SCORE_SUBCOMP: c_int = 33;
/// Delete a character.
pub(crate) const SCORE_DEL: c_int = 94;
/// Delete one of two identical characters.
pub(crate) const SCORE_DELDUP: c_int = 66;
/// Delete a composing character.
pub(crate) const SCORE_DELCOMP: c_int = 28;
/// Insert a character.
pub(crate) const SCORE_INS: c_int = 96;
/// Insert a character that duplicates its neighbour.
pub(crate) const SCORE_INSDUP: c_int = 67;
/// Insert a composing character.
pub(crate) const SCORE_INSCOMP: c_int = 30;
/// Turn a non-word character into a word character.
pub(crate) const SCORE_NONWORD: c_int = 103;

/// A suggestion that came out of a `file:` item.
pub(crate) const SCORE_FILE: c_int = 30;
/// The score ceiling a run starts with. Higher means slower; this allows
/// about three changes.
pub(crate) const SCORE_MAXINIT: c_int = 350;

// Discounts for words the dictionary has seen before, and the word counts
// that earn them.
pub(crate) const SCORE_COMMON1: c_int = 30;
pub(crate) const SCORE_COMMON2: c_int = 40;
pub(crate) const SCORE_COMMON3: c_int = 50;
pub(crate) const SCORE_THRES2: c_int = 10;
pub(crate) const SCORE_THRES3: c_int = 100;

// Trying changed sound-folded words gets slow past two changes, and
// stopping at one misses a few good suggestions, so the sound-a-like pass
// runs up to three times with a rising ceiling.
pub(crate) const SCORE_SFMAX1: c_int = 200;
pub(crate) const SCORE_SFMAX2: c_int = 300;
pub(crate) const SCORE_SFMAX3: c_int = 400;

/// Any score at all; used where a score could not be computed.
pub(crate) const SCORE_MAXMAX: c_int = 999999;
/// Past this, `spell_edit_score_limit`'s depth-first search costs more
/// than the full table would.
pub(crate) const SCORE_LIMITMAX: c_int = 350;

// Values for `sps_flags`, one per `'spellsuggest'` method.
/// Weigh the sound-a-like score into the final order.
pub(crate) const SPS_BEST: c_int = 1;
/// Skip the sound-a-like search entirely.
pub(crate) const SPS_FAST: c_int = 2;
/// Score the two searches separately and interleave the results.
pub(crate) const SPS_DOUBLE: c_int = 4;

/// Words that must never be suggested, keyed the way the `COMMON` table is.
pub(crate) type Banned = HashSet<Box<[u8]>, BuildHasherDefault<WordHasher>>;

/// What is known while looking for suggestions: the search's whole state,
/// which every step of it is handed by reference.
pub(crate) struct SugInfo {
    /// The suggestions found so far, best last until they are sorted.
    pub su_ga: Vec<Suggest>,
    /// How many suggestions will be displayed.
    pub su_maxcount: c_int,
    /// The score ceiling for adding to `su_ga`.
    pub su_maxscore: c_int,
    /// The same, while working on sound-folded words.
    pub su_sfmaxscore: c_int,
    /// Like `su_ga`, but scored by sound; only used in "double" mode.
    pub su_sga: Vec<Suggest>,
    /// The bad word and the rest of its line after it, without a NUL.
    ///
    /// A copy, never the buffer's line: `z=`'s prompt redraws the screen
    /// and an `expr:` item runs user code, either of which may change or
    /// free the line while the search still reads the text after the bad
    /// word.
    pub su_bad: Vec<u8>,
    /// How much of `su_bad` the bad word covers.
    pub su_badlen: c_int,
    /// The bad word's capitalisation, as `WF_` bits.
    pub su_badflags: WordFlags,
    /// The bad word, truncated at `su_badlen` and NUL-terminated.
    pub su_badword: [u8; MAXWLEN],
    /// `su_badword`, case-folded.
    pub su_fbadword: [u8; MAXWLEN],
    /// `su_badword`, sound-folded.
    pub su_sal_badword: [u8; MAXWLEN],
    /// Words that must never be suggested.
    pub su_banned: Banned,
    /// The language sound folding defaults to.
    pub su_sallang: Option<LangId>,
    /// Per language, the soundfolds the sound-a-like passes have already
    /// turned back into words; see [`soundalike`].
    pub su_sounddone: Vec<(LangId, SoundDone)>,
}

impl SugInfo {
    /// An empty one, as `spell_find_suggest` expects to be handed.
    pub(crate) fn new() -> Self {
        Self {
            su_ga: Vec::new(),
            su_maxcount: 0,
            su_maxscore: 0,
            su_sfmaxscore: 0,
            su_sga: Vec::new(),
            su_bad: Vec::new(),
            su_badlen: 0,
            su_badflags: WordFlags::NONE,
            su_badword: [0; MAXWLEN],
            su_fbadword: [0; MAXWLEN],
            su_sal_badword: [0; MAXWLEN],
            su_banned: Banned::default(),
            su_sallang: None,
            su_sounddone: Vec::new(),
        }
    }

    /// The line from `at` bytes into the bad word to its end; empty past it.
    pub(crate) fn bad_from(&self, at: usize) -> &[u8] {
        self.su_bad.get(at..).unwrap_or_default()
    }

    /// The byte `at` bytes into the bad word, NUL past the line's end -- what
    /// reading the C string the line was gave.
    pub(crate) fn bad_byte(&self, at: usize) -> u8 {
        self.su_bad.get(at).copied().unwrap_or(0)
    }

    /// The bad word as typed, without its NUL.
    pub(crate) fn badword(&self) -> &[u8] {
        until_nul(&self.su_badword)
    }

    /// The caps type of the bad word from `from` to `to` bytes into it.
    pub(crate) fn captype_between(&self, from: usize, to: usize) -> WordFlags {
        badword_captype(self.bad_from(from), to.saturating_sub(from))
    }
}

/// One suggestion.
pub(crate) struct Suggest {
    /// The suggested word.
    pub st_word: Box<[u8]>,
    /// `st_word`'s length.
    pub st_wordlen: c_int,
    /// How much of the bad word it replaces.
    pub st_orglen: c_int,
    /// Lower is better.
    pub st_score: c_int,
    /// The tie-breaker when `st_score` compares equal.
    pub st_altscore: c_int,
    /// `st_score` is a sound-a-like score.
    pub st_salscore: bool,
    /// The sound-a-like bonus is already in `st_score`.
    pub st_had_bonus: bool,
    /// The language the word was sound-folded with.
    pub st_slang: Option<LangId>,
}

impl Suggest {
    /// The suggested word.
    pub(crate) fn word_bytes(&self) -> &[u8] {
        &self.st_word
    }
}

/// How long the trie walk may run for, in milliseconds; `timeout:` in
/// `'spellsuggest'` sets it, and a fresh `z=` resets it.
pub(crate) static spell_suggest_timeout: GlobalCell<c_int> = GlobalCell::new(5000);

/// Which internal method `'spellsuggest'` asks for, as `SPS_` bits.
pub(crate) static sps_flags: GlobalCell<c_int> = GlobalCell::new(SPS_BEST);
/// How many suggestions `z=` may list; the number in `'spellsuggest'`.
static sps_limit: GlobalCell<c_int> = GlobalCell::new(9999);

/// The spell languages the current window has loaded, in the order
/// `'spelllang'` put them in: a copy, because a search reaches back into
/// the window for its word characters while it walks them.
pub(crate) fn window_langs() -> Vec<LangP> {
    Win::current().syn_block().b_langp.clone()
}

/// The capitalisation of a bad word, for reproducing it in the
/// replacement.
///
/// Like `captype`, except that a `KEEPCAP` word is looked at more closely
/// so that `make_case_word` can turn "WOrd" into "Word" and "WOrD" into
/// "WORD". The word is the first `end` bytes of `word`, which runs on to
/// the end of its line.
pub(crate) fn badword_captype(word: &[u8], end: usize) -> WordFlags {
    let mut flags = captype(word, end);
    if !flags.has(WordFlags::KEEPCAP) {
        return flags;
    }

    // Count the upper and lower case letters.
    let (mut lower, mut upper) = (0, 0);
    let mut first = false;
    let end = end.min(word.len());
    let mut p = 0;
    while p < end {
        if spell_isupper(char_at(&word[p..])) {
            upper += 1;
            if p == 0 {
                first = true;
            }
        } else {
            lower += 1;
        }
        p += c_cluster_len(&word[p..]);
    }

    // More upper than lower case suggests an all-capitals word;
    // otherwise a leading capital suggests one capital. "ALl" is most
    // likely "All", hence the three-capital floor.
    if upper > lower && upper > 2 {
        flags |= WordFlags::ALLCAP;
    } else if first {
        flags |= WordFlags::ONECAP;
    }
    if upper >= 2 && lower >= 2 {
        // maCARONI, maCAroni
        flags |= WordFlags::MIXCAP;
    }
    flags
}

/// Find suggestions for `word`, at most `maxcount` of them.
///
/// This is what the `spellsuggest()` Vimscript function is built on.
pub(crate) fn spell_suggest_list(
    word: &CStr,
    maxcount: c_int,
    need_cap: bool,
    interactive: bool,
) -> Vec<XString> {
    let mut sug = SugInfo::new();
    spell_find_suggest(
        word.to_bytes(),
        0,
        &mut sug,
        maxcount,
        false,
        need_cap,
        interactive,
    );

    sug.su_ga
        .iter()
        .map(|stp| {
            // A suggestion may replace only part of `word`; what it does
            // not replace goes on the end.
            let mut word = XString::from_bytes(stp.word_bytes());
            word.push_bytes(sug.bad_from(stp.st_orglen as usize));
            word
        })
        .collect()
}

/// Copy `src` into `dest` with a NUL after it, cut to fit.
pub(crate) fn copy_terminated(dest: &mut [u8], src: &[u8]) {
    let n = src.len().min(dest.len() - 1);
    dest[..n].copy_from_slice(&src[..n]);
    dest[n] = NUL as u8;
}

/// Find suggestions for the word at the start of `bad` -- the line from the
/// bad word to its end -- and leave them in `su.su_ga`.
///
/// `badlen` is how much of the line the bad word covers, or 0 to let the
/// spell checker decide. `banbadword` keeps the bad word itself out of the
/// list, and `need_cap` says `'spellcapcheck'` wants a capital.
///
/// The mechanism is Aspell's, reimplemented.
#[allow(clippy::too_many_arguments)]
fn spell_find_suggest(
    bad: &[u8],
    badlen: c_int,
    su: &mut SugInfo,
    maxcount: c_int,
    banbadword: bool,
    need_cap: bool,
    interactive: bool,
) {
    // A `expr:` item may itself call `spellsuggest()`, which lands
    // back here; the inner call must not evaluate the expression
    // again.
    static EXPR_BUSY: GlobalCell<bool> = GlobalCell::new(false);

    let mut attr: Hlf = HLF_COUNT;

    // Start clean.
    *su = SugInfo::new();
    let bad = until_nul(bad);
    if bad.is_empty() {
        return;
    }
    su.su_bad = bad.to_vec();

    su.su_badlen = if badlen != 0 {
        badlen
    } else {
        let len;
        (len, attr) = spell_check(Win::current(), bad, None, false);
        len as c_int
    };
    su.su_maxcount = maxcount;
    su.su_maxscore = SCORE_MAXINIT;
    su.su_badlen = su.su_badlen.min(MAXWLEN as c_int - 1); // just in case
    let badlen = su.su_badlen as usize;
    copy_terminated(&mut su.su_badword, &bad[..badlen.min(bad.len())]);
    let _ = spell_casefold(Win::current(), bad, badlen, &mut su.su_fbadword);
    // Upstream note: this breaks if the case-folded text comes out
    // longer than the original, because an illegal byte then throws
    // the pointer arithmetic off.
    su.su_fbadword[badlen] = NUL as u8;

    su.su_badflags = badword_captype(bad, badlen);
    if need_cap {
        su.su_badflags |= WordFlags::ONECAP;
    }

    // Sound-fold with the first language in 'spelllang' that can. That
    // is right for several files of one language and not too bad for a
    // mixture like "pl,en". Note this is the buffer's list of
    // languages rather than the window's.
    su.su_sallang = Buf::current()
        .syntax()
        .b_langp
        .iter()
        .find_map(|lp| lp.sallang);
    if let Some(id) = su.su_sallang {
        // Once here, rather than once per candidate.
        let SugInfo {
            su_fbadword,
            su_sal_badword,
            ..
        } = su;
        loaded_langs.with(|langs| {
            if let Some(lang) = langs.get(id) {
                spell_soundfold(lang, &su_fbadword[..], true, su_sal_badword);
            }
        });
    }

    // A word the spell checker is happy with, spelled lower case, may
    // simply be missing its capital.
    if !spell_isupper(char_at(bad)) && attr == HLF_COUNT {
        let mut buf = [0u8; MAXPATHL as usize];
        make_case_word(&su.su_badword, &mut buf, WordFlags::ONECAP);
        let (badlen, lang) = (su.su_badlen, su.su_sallang);
        add_suggestion(
            su,
            List::Main,
            &buf,
            badlen,
            SCORE_ICASE,
            0,
            true,
            lang,
            false,
        );
    }

    // Ban the bad word itself; it may be valid in another region.
    if banbadword {
        let word = su.badword().to_vec();
        add_banned(su, &word);
    }

    // An expression may change 'spellsuggest' while it runs, so this walks
    // a copy.
    let sps = P_SPS.get();
    let mut rest: &[u8] = until_nul(&sps);
    let mut do_combine = false;
    let mut did_intern = false;
    while !rest.is_empty() {
        let (part, after) = split_option_part(rest, MAXPATHL as usize, b",");
        rest = after;

        if let Some(expr) = part.strip_prefix(b"expr:") {
            if !EXPR_BUSY.get() {
                EXPR_BUSY.set(true);
                spell_suggest_expr(su, expr);
                EXPR_BUSY.set(false);
            }
        } else if let Some(fname) = part.strip_prefix(b"file:") {
            spell_suggest_file(su, fname);
        } else if let Some(ms) = part.strip_prefix(b"timeout:") {
            spell_suggest_timeout.set(atoi(ms));
        } else if !did_intern {
            // The internal method runs at most once.
            spell_suggest_intern(su, interactive);
            do_combine = sps_flags.get() & SPS_DOUBLE != 0;
            did_intern = true;
        }
    }

    if do_combine {
        // Last, because sorting would undo the interleaving.
        score_combine(su);
    }
}

/// Run the internal search, in whichever form [`sps_flags`] asks for.
fn spell_suggest_intern(su: &mut SugInfo, interactive: bool) {
    suggest_load_files();

    // 1. Special cases, such as a repeated word: "the the" -> "the".
    suggest_try_special(su);

    // 2. Inserting, deleting, swapping and changing letters, `REP`
    //    items from the `.aff` file, and splitting the word in two.
    suggest_try_change(su);

    // Give the top scorers a sound-a-like score to interleave on.
    if sps_flags.get() & SPS_DOUBLE != 0 {
        score_comp_sal(su);
    }

    // 3. Words that sound alike.
    if sps_flags.get() & SPS_FAST == 0 {
        if sps_flags.get() & SPS_BEST != 0 {
            rescore_suggestions(su);
        }

        // Through the sound-fold tree `su_maxscore` bounds the changes
        // tried to the sound-folded word, while `su_sfmaxscore` bounds
        // the rescored result. Small edit distances come first because
        // they are much faster and usually enough; only if too little
        // turns up is a wider search worth its time. `su_sounddone`
        // keeps the passes from redoing each other's work.
        su.su_sounddone.clear();
        su.su_maxscore = SCORE_SFMAX1;
        su.su_sfmaxscore = SCORE_MAXINIT * 3;
        suggest_try_soundalike(su);
        for ceiling in [SCORE_SFMAX2, SCORE_SFMAX3] {
            if su.su_ga.len() as c_int >= clean_count(su) {
                break;
            }
            su.su_maxscore = ceiling;
            suggest_try_soundalike(su);
        }
        su.su_maxscore = su.su_sfmaxscore;
        su.su_sounddone = Vec::new();
    }

    // Interrupted searches still show what they found. `got_int` is
    // only cleared for a command, not for `spellsuggest()`.
    os_breakcheck();
    if interactive && got_int.get() {
        vgetc();
        got_int.set(false);
    }

    if sps_flags.get() & SPS_DOUBLE == 0 && !su.su_ga.is_empty() {
        if sps_flags.get() & SPS_BEST != 0 {
            rescore_suggestions(su);
        }
        check_suggestions(su, List::Main);
        let (maxscore, maxcount) = (su.su_maxscore, su.su_maxcount);
        cleanup_suggestions(&mut su.su_ga, maxscore, maxcount);
    }
}

/// Find suggestions by recognising specific situations.
///
/// There is only one: a word typed twice, "the the", whose suggestion is
/// the word once.
fn suggest_try_special(su: &mut SugInfo) {
    let fbadword = until_nul(&su.su_fbadword);
    let len = fbadword
        .iter()
        .position(|&b| b == b' ' || b == b'\t')
        .unwrap_or(fbadword.len());
    let second = &fbadword[len..];
    let second = &second[second
        .iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()..];
    if second != &fbadword[..len] {
        return;
    }

    // Take the bad word's case with it, so that "The the" -> "The".
    let mut word = [0u8; MAXWLEN];
    make_case_word(&fbadword[..len], &mut word, su.su_badflags);

    // Score it as one deletion, with a sound-a-like score of zero.
    let (badlen, lang) = (su.su_badlen, su.su_sallang);
    let score = 3 * SCORE_REP / 4;
    add_suggestion(su, List::Main, &word, badlen, score, 0, true, lang, false);
}

/// Find suggestions by adding, removing and swapping letters, in every
/// language the window has loaded.
fn suggest_try_change(su: &mut SugInfo) {
    // The walk rewrites the case-folded bad word in place (for `REP`
    // items especially), so it gets a copy. What follows the bad word
    // is appended: changing characters after it may help.
    let mut fword = [0u8; MAXWLEN];
    let folded = until_nul(&su.su_fbadword);
    let n = folded.len();
    fword[..n].copy_from_slice(folded);
    let tail = su.bad_from(su.su_badlen as usize);
    let _ = spell_casefold(Win::current(), tail, tail.len(), &mut fword[n..]);

    // Keep the result no longer than the original text.
    let n = su.su_bad.len();
    if n < MAXWLEN {
        fword[n] = NUL as u8;
    }

    for lp in window_langs() {
        loaded_langs.with(|langs| {
            // A spell file that failed to reload is still in the list, but
            // everything in it has been cleared.
            if langs
                .get(lp.slang)
                .is_some_and(|lang| !lang.sl_fold_tree.is_empty())
            {
                suggest_trie_walk(su, lp, langs, &mut fword, false);
            }
        });
    }
}
