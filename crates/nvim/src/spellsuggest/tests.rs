//! The suggestion search's pure parts, over trees built by hand: the `.sug`
//! word-number decoding, the two edit distances and the sound-a-like score,
//! the keep-case search, and the suggestion list's bookkeeping. The walk
//! itself asks the current window which bytes are word characters, so it
//! is covered by the unit suite's editor cases instead.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::collect::{List, add_banned, add_suggestion, cleanup_suggestions};
use super::score::{
    EMPTY_SOUND, SoundBuf, soundalike_score, spell_edit_score, spell_edit_score_limit,
};
use super::soundalike::{bytes2offset, find_keepcap_word, soundfold_find, word_number_to_letters};
use super::{MAXWLEN, SCORE_DEL, SCORE_MAXMAX, SCORE_SUBST, SCORE_SWAP, SugInfo, Suggest};
use crate::cstr::until_nul;
use crate::memory::XString;
use crate::spell::WordTree;
use crate::types::{SpellIdx, SpellLang};

/// A tree of the words "ab", "ac" and "b", with every node's own entry the
/// number of words below it, as the `.sug` reader leaves a case-folded tree.
fn ab_ac_b() -> WordTree {
    let byts = [2, b'a', b'b', 2, b'b', b'c', 1, 0, 1, 0, 1, 0];
    let idxs: [SpellIdx; 12] = [3, 3, 6, 2, 8, 10, 1, 0, 1, 0, 1, 0];
    WordTree::from_parts(byts.into(), idxs.into())
}

/// A language whose case-folded and sound-folded trees are both
/// [`ab_ac_b`], and whose keep-case tree holds "éb": a multi-byte word,
/// matched a byte per node. (Its upper-case round is not exercised: the
/// case tables are only filled in by an editor's start-up.)
fn lang() -> Box<SpellLang> {
    let mut lang = SpellLang::new(b"test", XString::from_bytes(b"test.spl"));
    lang.sl_fold_tree = ab_ac_b();
    lang.sl_sound_tree = ab_ac_b();
    let keep = [1, 0xc3, 1, 0xa9, 1, b'b', 1, 0];
    let keep_idxs: [SpellIdx; 8] = [0, 2, 0, 4, 0, 6, 0, 0];
    lang.sl_keep_tree = WordTree::from_parts(keep.into(), keep_idxs.into());
    lang
}

fn sound(word: &[u8]) -> SoundBuf {
    let mut buf = EMPTY_SOUND;
    buf[..word.len()].copy_from_slice(word);
    buf
}

#[test]
fn word_numbers_are_biased_and_sized_by_their_first_byte() {
    // One byte: the value plus one. Two: 0x80 | high, then low, each + 1.
    let line = [5 + 1, 0x80 | 2, 3 + 1];
    let mut pos = 0;
    assert_eq!(bytes2offset(&line, &mut pos), 5);
    assert_eq!(bytes2offset(&line, &mut pos), 255 + 3);
    assert_eq!(pos, 3);
    // A damaged line reads the missing byte as zero and stays put.
    let mut pos = 0;
    assert_eq!(bytes2offset(&[0x80 | 1], &mut pos), -1);
}

#[test]
fn a_word_number_walks_back_to_its_letters() {
    let lang = lang();
    for (nr, word) in [(0, &b"ab"[..]), (1, b"ac"), (2, b"b")] {
        let (letters, _, _) = word_number_to_letters(&lang, nr);
        assert_eq!(until_nul(&letters), word);
        assert_eq!(soundfold_find(&lang, word), nr);
    }
    // Not words: a prefix, a word continued, a byte the tree lacks.
    assert_eq!(soundfold_find(&lang, b"a"), -1);
    assert_eq!(soundfold_find(&lang, b"abc"), -1);
    assert_eq!(soundfold_find(&lang, b"z"), -1);
    // The word ends at a NUL as well as at its end.
    assert_eq!(soundfold_find(&lang, b"ac\0zz"), 1);
}

#[test]
fn the_keep_case_spelling_is_found_a_byte_per_node() {
    let lang = lang();
    let mut kword = [0xffu8; MAXWLEN];
    find_keepcap_word(&lang, "éb\0".as_bytes(), &mut kword);
    assert_eq!(until_nul(&kword), "éb".as_bytes());
    // A word the keep-case tree does not hold comes back empty.
    find_keepcap_word(&lang, b"b", &mut kword);
    assert_eq!(until_nul(&kword), b"");
}

#[test]
fn the_two_edit_distances_agree_under_the_limit() {
    let pairs: [(&[u8], &[u8], i32); 5] = [
        (b"the", b"the", 0),
        (b"teh", b"the", SCORE_SWAP),
        ("xé".as_bytes(), "xö".as_bytes(), SCORE_SUBST),
        (b"thee", b"the", SCORE_DEL),
        (b"ab\0junk", b"ab", 0),
    ];
    for (bad, good, want) in pairs {
        assert_eq!(spell_edit_score(None, bad, good), want, "{bad:?} {good:?}");
        assert_eq!(spell_edit_score_limit(None, bad, good, 300), want);
    }
    // Past the limit the cut-off answer is "too far".
    assert_eq!(
        spell_edit_score_limit(None, b"abcdef", b"uvwxyz", 100),
        SCORE_MAXMAX
    );
}

#[test]
fn sound_alike_words_score_by_their_differences() {
    assert_eq!(soundalike_score(&sound(b"TK"), &sound(b"TK")), 0);
    assert_eq!(soundalike_score(&sound(b"TK"), &sound(b"KT")), SCORE_SWAP);
    assert_eq!(soundalike_score(&sound(b"TKS"), &sound(b"TK")), SCORE_DEL);
    assert_eq!(
        soundalike_score(&sound(b"TKSXX"), &sound(b"TK")),
        SCORE_MAXMAX
    );
}

fn sug_for(line: &[u8], badlen: i32) -> SugInfo {
    let mut su = SugInfo::new();
    su.su_bad = line.to_vec();
    su.su_badlen = badlen;
    su.su_maxcount = 10;
    su.su_maxscore = 350;
    su
}

#[test]
fn a_suggestion_replaces_no_more_than_it_changes() {
    let mut su = sug_for(b"teh word", 3);
    // The shared " word" is not part of the replacement.
    add_suggestion(
        &mut su,
        List::Main,
        b"the word\0",
        8,
        100,
        0,
        false,
        None,
        false,
    );
    assert_eq!(su.su_ga.len(), 1);
    assert_eq!(su.su_ga[0].word_bytes(), b"the");
    assert_eq!(su.su_ga[0].st_orglen, 3);
    // The same suggestion again only lowers the score.
    add_suggestion(&mut su, List::Main, b"the", 3, 90, 0, false, None, false);
    add_suggestion(&mut su, List::Main, b"the", 3, 95, 0, false, None, false);
    assert_eq!(su.su_ga.len(), 1);
    assert_eq!(su.su_ga[0].st_score, 90);
    // Replacing the bad word by itself is no suggestion at all.
    add_suggestion(&mut su, List::Main, b"teh", 3, 10, 0, false, None, false);
    assert_eq!(su.su_ga.len(), 1);
    // A banned word is remembered without its NUL and only once.
    add_banned(&mut su, b"teh\0x");
    add_banned(&mut su, b"teh");
    assert_eq!(su.su_banned.len(), 1);
    assert!(su.su_banned.contains(&b"teh"[..]));
}

fn suggest(word: &[u8], score: i32, altscore: i32) -> Suggest {
    Suggest {
        st_word: word.into(),
        st_wordlen: i32::try_from(word.len()).expect("a short word"),
        st_orglen: 3,
        st_score: score,
        st_altscore: altscore,
        st_salscore: false,
        st_had_bonus: false,
        st_slang: None,
    }
}

#[test]
fn cleaning_up_sorts_stably_and_lowers_the_ceiling() {
    let mut list = vec![
        suggest(b"b", 50, 0),
        suggest(b"The", 20, 0),
        suggest(b"a", 50, 0),
        suggest(b"the", 20, 0),
        suggest(b"c", 10, 5),
    ];
    // Case-insensitive ties keep the order they were found in.
    assert_eq!(cleanup_suggestions(&mut list, 350, 4), 50);
    let words: Vec<&[u8]> = list.iter().map(Suggest::word_bytes).collect();
    assert_eq!(words, [&b"c"[..], b"The", b"the", b"a"]);
    // Nothing dropped: the ceiling stays.
    assert_eq!(cleanup_suggestions(&mut list, 350, 9), 350);
}
