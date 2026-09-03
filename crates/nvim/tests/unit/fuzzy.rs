//! The fzy scorer, pinned to the numbers the editor answered before it was
//! rewritten.
//!
//! `fuzzy.rs` had no test of its own: `test_matchfuzzy.vim` and
//! `029_fuzzy_spec.lua` only ever assert the *order* `matchfuzzy()` puts
//! candidates in, so every score below the top one — and every match
//! position — was unobserved. A gap penalty that changed by a tenth, or a
//! CamelCase bonus that stopped applying, would move no test at all as long
//! as the ranking survived.
//!
//! # Where the table comes from
//!
//! Every expected score and position list in [`SCORES`] was **captured from
//! the binary**, not derived: a script ran `matchfuzzypos([haystack], pat)`
//! (and again with `{'matchseq': 1}`) over the pairs below and wrote what
//! came back. So the table is a photograph of the scorer as it stood, and
//! any later disagreement is a change in behaviour rather than a
//! disagreement about what the behaviour should be.
//!
//! The pairs themselves are chosen to reach each term of the score
//! separately: the CamelCase bonus (`oneTwo` in `oneTwo` vs `one_two`), the
//! word-boundary bonus and its three separators (`_`, space, `-`), the path
//! separator and the dot (`color/setup.vim` against `color.setup.vim`), the
//! leading, inner and trailing gap penalties (`xxayybxxxx` through
//! `xxayyyybxx`, and `one`/`onex`/`onexx`), the consecutive-run bonus
//! (`onetwo` against `oanbectdweo`), the two infinities (an exact match
//! scores `INT_MAX`, a rejection is not scored at all), multi-word patterns
//! in both `matchseq` modes, and multibyte candidates, where a position is
//! a character index and not a byte offset.
//!
//! [`ORDERING`] is the other half: the relative order the preference cases
//! of `test_matchfuzzy.vim` assert, restated as "these candidates score in
//! this order". The old test can only see the order after the sort, so a
//! score that is wrong but still sorts correctly passes it; here the scores
//! themselves have to descend.

#![cfg(not(miri))]

use std::ffi::c_int;

use neovim::fuzzy::{FUZZY_MATCH_MAX_LEN, FUZZY_SCORE_NONE, fuzzy_match};

use crate::support::{Sandbox, cstr};

/// One row of [`SCORES`]: a pattern, a candidate, whether the pattern is
/// matched as one sequence (`matchseq`), and what the editor answered —
/// `None` for a candidate it rejected, otherwise the score and the haystack
/// *character* index each pattern character landed on.
struct Case {
    pat: &'static str,
    hay: &'static str,
    seq: bool,
    want: Option<(c_int, &'static [u32])>,
}

const fn case(
    pat: &'static str,
    hay: &'static str,
    seq: bool,
    want: Option<(c_int, &'static [u32])>,
) -> Case {
    Case {
        pat,
        hay,
        seq,
        want,
    }
}

/// Score `pat` against `hay`, as the editor's own entry point does.
///
/// Answers whether anything matched, the score, and as many positions as the
/// caller says were filled — `fuzzy_match` reports how many only through the
/// pattern, so the expected list's length is what is read back.
fn scored(pat: &str, hay: &str, matchseq: bool, positions: usize) -> (bool, c_int, Vec<u32>) {
    // The editor lock is held for the keyword table `compute_bonus` reads.
    let _sandbox = Sandbox::globals();
    let (hay, pat) = (cstr(hay), cstr(pat));
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    let (score, filled) = fuzzy_match(&hay, &pat, matchseq, &mut matches);
    (filled != 0, score, matches[..positions].to_vec())
}

#[test]
fn scores_and_positions_are_what_they_were() {
    let mut wrong = Vec::new();
    for c in SCORES {
        let want_pos: &[u32] = c.want.map_or(&[], |(_, pos)| pos);
        let (matched, got_score, got_pos) = scored(c.pat, c.hay, c.seq, want_pos.len());
        let got = matched.then_some((got_score, got_pos));
        let want = c.want.map(|(s, pos)| (s, pos.to_vec()));
        if got != want {
            wrong.push(format!(
                "  {:?} in {:?} (matchseq {}): want {want:?}, got {got:?}",
                c.pat, c.hay, c.seq
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} rows moved:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// The captured table. Rows come in pairs — the same pattern and
/// candidate with `matchseq` off and on — so that the two paths through
/// `fuzzy_match_words` are always compared on the same input.
const SCORES: &[Case] = &[
    case("abc", "abc", false, Some((2147483647, &[0, 1, 2]))),
    case("abc", "abc", true, Some((2147483647, &[0, 1, 2]))),
    case("ac", "abc", false, Some((890, &[0, 2]))),
    case("ac", "abc", true, Some((890, &[0, 2]))),
    case("cra", "crayon", false, Some((2885, &[0, 1, 2]))),
    case("cra", "crayon", true, Some((2885, &[0, 1, 2]))),
    case("cra", "camera", false, Some((1870, &[0, 4, 5]))),
    case("cra", "camera", true, Some((1870, &[0, 4, 5]))),
    case("aa", "aba", false, Some((890, &[0, 2]))),
    case("aa", "aba", true, Some((890, &[0, 2]))),
    case("aa", "aabbaa", false, Some((1880, &[0, 1]))),
    case("aa", "aabbaa", true, Some((1880, &[0, 1]))),
    case("aa", "aaabbbaaa", false, Some((1865, &[0, 1]))),
    case("aa", "aaabbbaaa", true, Some((1865, &[0, 1]))),
    case("aa", "aaaabbbbaaaa", false, Some((1850, &[0, 1]))),
    case("aa", "aaaabbbbaaaa", true, Some((1850, &[0, 1]))),
    case("oneTwo", "onetwo", false, None),
    case("oneTwo", "onetwo", true, None),
    case(
        "oneTwo",
        "oneTwo",
        false,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "oneTwo",
        "oneTwo",
        true,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case("oneTwo", "one_two", false, None),
    case("oneTwo", "one_two", true, None),
    case(
        "onetwo",
        "onetwo",
        false,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "onetwo",
        "onetwo",
        true,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "onetwo",
        "oneTwo",
        false,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "onetwo",
        "oneTwo",
        true,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "onetwo",
        "one_two",
        false,
        Some((5690, &[0, 1, 2, 4, 5, 6])),
    ),
    case("onetwo", "one_two", true, Some((5690, &[0, 1, 2, 4, 5, 6]))),
    case(
        "onetwo",
        "one two",
        false,
        Some((5690, &[0, 1, 2, 4, 5, 6])),
    ),
    case("onetwo", "one two", true, Some((5690, &[0, 1, 2, 4, 5, 6]))),
    case(
        "onetwo",
        "xonetwo",
        false,
        Some((4995, &[1, 2, 3, 4, 5, 6])),
    ),
    case("onetwo", "xonetwo", true, Some((4995, &[1, 2, 3, 4, 5, 6]))),
    case(
        "onetwo",
        "xxonetwo",
        false,
        Some((4990, &[2, 3, 4, 5, 6, 7])),
    ),
    case(
        "onetwo",
        "xxonetwo",
        true,
        Some((4990, &[2, 3, 4, 5, 6, 7])),
    ),
    case(
        "onetwo",
        "oanbectdweo",
        false,
        Some((850, &[0, 2, 4, 6, 8, 10])),
    ),
    case(
        "onetwo",
        "oanbectdweo",
        true,
        Some((850, &[0, 2, 4, 6, 8, 10])),
    ),
    case("one", "one", false, Some((2147483647, &[0, 1, 2]))),
    case("one", "one", true, Some((2147483647, &[0, 1, 2]))),
    case("one", "onex", false, Some((2895, &[0, 1, 2]))),
    case("one", "onex", true, Some((2895, &[0, 1, 2]))),
    case("one", "onexx", false, Some((2890, &[0, 1, 2]))),
    case("one", "onexx", true, Some((2890, &[0, 1, 2]))),
    case("vimrc", ".vim/vimrc", false, Some((4875, &[5, 6, 7, 8, 9]))),
    case("vimrc", ".vim/vimrc", true, Some((4875, &[5, 6, 7, 8, 9]))),
    case(
        "vimrc",
        ".vim/vimrc_colors",
        false,
        Some((4840, &[5, 6, 7, 8, 9])),
    ),
    case(
        "vimrc",
        ".vim/vimrc_colors",
        true,
        Some((4840, &[5, 6, 7, 8, 9])),
    ),
    case(
        "vimrc",
        ".vim/v_i_m_r_c",
        false,
        Some((4115, &[1, 2, 3, 11, 13])),
    ),
    case(
        "vimrc",
        ".vim/v_i_m_r_c",
        true,
        Some((4115, &[1, 2, 3, 11, 13])),
    ),
    case("ab", "xxayybxxxx", false, Some((-50, &[2, 5]))),
    case("ab", "xxayybxxxx", true, Some((-50, &[2, 5]))),
    case("ab", "xxayyybxxx", false, Some((-55, &[2, 6]))),
    case("ab", "xxayyybxxx", true, Some((-55, &[2, 6]))),
    case("ab", "xxayyyybxx", false, Some((-60, &[2, 7]))),
    case("ab", "xxayyyybxx", true, Some((-60, &[2, 7]))),
    case(
        "setup.vim",
        "colorsetup.vim",
        false,
        Some((7975, &[5, 6, 7, 8, 9, 10, 11, 12, 13])),
    ),
    case(
        "setup.vim",
        "colorsetup.vim",
        true,
        Some((7975, &[5, 6, 7, 8, 9, 10, 11, 12, 13])),
    ),
    case(
        "setup.vim",
        "color setup.vim",
        false,
        Some((8770, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color setup.vim",
        true,
        Some((8770, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color/setup.vim",
        false,
        Some((8870, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color/setup.vim",
        true,
        Some((8870, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color_setup.vim",
        false,
        Some((8770, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color_setup.vim",
        true,
        Some((8770, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color\\setup.vim",
        false,
        Some((7970, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case(
        "setup.vim",
        "color\\setup.vim",
        true,
        Some((7970, &[6, 7, 8, 9, 10, 11, 12, 13, 14])),
    ),
    case("Cursor", "hello", false, None),
    case("Cursor", "hello", true, None),
    case(
        "Cursor",
        "lCursor",
        false,
        Some((5695, &[1, 2, 3, 4, 5, 6])),
    ),
    case("Cursor", "lCursor", true, Some((5695, &[1, 2, 3, 4, 5, 6]))),
    case(
        "Cursor",
        "Cursor",
        false,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "Cursor",
        "Cursor",
        true,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "cursor",
        "Cursor",
        false,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case(
        "cursor",
        "Cursor",
        true,
        Some((2147483647, &[0, 1, 2, 3, 4, 5])),
    ),
    case("CURSOR", "cursor", false, None),
    case("CURSOR", "cursor", true, None),
    case(
        "baz foo",
        "foo bar baz",
        false,
        Some((5620, &[8, 9, 10, 0, 1, 2])),
    ),
    case("baz foo", "foo bar baz", true, None),
    case("baz foo", "foo", false, None),
    case("baz foo", "foo", true, None),
    case("baz foo", "foo bar", false, None),
    case("baz foo", "foo bar", true, None),
    case("baz foo", "baz bar", false, None),
    case("baz foo", "baz bar", true, None),
    case("one two", "foo bar baz", false, None),
    case("one two", "foo bar baz", true, None),
    case(
        "bar foo",
        "foo bar",
        false,
        Some((5660, &[4, 5, 6, 0, 1, 2])),
    ),
    case("bar foo", "foo bar", true, None),
    case(
        "bar foo",
        "bar foo",
        false,
        Some((5660, &[0, 1, 2, 4, 5, 6])),
    ),
    case(
        "bar foo",
        "bar foo",
        true,
        Some((2147483647, &[0, 1, 2, 3, 4, 5, 6])),
    ),
    case(
        "bar foo",
        "foobar",
        false,
        Some((4870, &[3, 4, 5, 0, 1, 2])),
    ),
    case("bar foo", "foobar", true, None),
    case(
        "bar foo",
        "barfoo",
        false,
        Some((4870, &[0, 1, 2, 3, 4, 5])),
    ),
    case("bar foo", "barfoo", true, None),
    case("  \t ", "foo bar", false, None),
    case("  \t ", "foo bar", true, None),
    case("", "abc", false, None),
    case("", "abc", true, None),
    case("abcd", "abc", false, None),
    case("abcd", "abc", true, None),
    case("xyz", "abc", false, None),
    case("xyz", "abc", true, None),
    case("ndl", "needle", false, Some((1875, &[0, 3, 4]))),
    case("ndl", "needle", true, Some((1875, &[0, 3, 4]))),
    case("ndl", "somebuf", false, None),
    case("ndl", "somebuf", true, None),
    case("a.b", "a.b", false, Some((2147483647, &[0, 1, 2]))),
    case("a.b", "a.b", true, Some((2147483647, &[0, 1, 2]))),
    case("ab", "a.b", false, Some((1490, &[0, 2]))),
    case("ab", "a.b", true, Some((1490, &[0, 2]))),
    case("ab", "a-b", false, Some((1690, &[0, 2]))),
    case("ab", "a-b", true, Some((1690, &[0, 2]))),
    case("ab", "a_b", false, Some((1690, &[0, 2]))),
    case("ab", "a_b", true, Some((1690, &[0, 2]))),
    case("ab", "a b", false, Some((1690, &[0, 2]))),
    case("ab", "a b", true, Some((1690, &[0, 2]))),
    case("ab", "a/b", false, Some((1790, &[0, 2]))),
    case("ab", "a/b", true, Some((1790, &[0, 2]))),
    case("ab", "aXb", false, Some((890, &[0, 2]))),
    case("ab", "aXb", true, Some((890, &[0, 2]))),
    case("abc", "aBcD", false, Some((2895, &[0, 1, 2]))),
    case("abc", "aBcD", true, Some((2895, &[0, 1, 2]))),
    case("fbb", "fooBarBaz", false, Some((2250, &[0, 3, 6]))),
    case("fbb", "fooBarBaz", true, Some((2250, &[0, 3, 6]))),
    case("fbb", "foo_bar_baz", false, Some((2430, &[0, 4, 8]))),
    case("fbb", "foo_bar_baz", true, Some((2430, &[0, 4, 8]))),
    case("fbb", "foo/bar/baz", false, Some((2630, &[0, 4, 8]))),
    case("fbb", "foo/bar/baz", true, Some((2630, &[0, 4, 8]))),
    case("fbb", "foo.bar.baz", false, Some((2030, &[0, 4, 8]))),
    case("fbb", "foo.bar.baz", true, Some((2030, &[0, 4, 8]))),
    case("fbb", "foobarbaz", false, Some((850, &[0, 3, 6]))),
    case("fbb", "foobarbaz", true, Some((850, &[0, 3, 6]))),
    case("é", "café", false, Some((-15, &[3]))),
    case("é", "café", true, Some((-15, &[3]))),
    case("caf", "café", false, Some((2895, &[0, 1, 2]))),
    case("caf", "café", true, Some((2895, &[0, 1, 2]))),
    case("éà", "éxàx", false, Some((885, &[0, 2]))),
    case("éà", "éxàx", true, Some((885, &[0, 2]))),
    case("日本", "x日x本x", false, Some((-20, &[1, 3]))),
    case("日本", "x日x本x", true, Some((-20, &[1, 3]))),
    case("日本", "日本語", false, Some((1895, &[0, 1]))),
    case("日本", "日本語", true, Some((1895, &[0, 1]))),
    case("ab", "ab", false, Some((2147483647, &[0, 1]))),
    case("ab", "ab", true, Some((2147483647, &[0, 1]))),
    case("a", "a", false, Some((2147483647, &[0]))),
    case("a", "a", true, Some((2147483647, &[0]))),
    case("a", "ba", false, Some((-5, &[1]))),
    case("a", "ba", true, Some((-5, &[1]))),
    case("a", "bba", false, Some((-10, &[2]))),
    case("a", "bba", true, Some((-10, &[2]))),
    case("a", "bbba", false, Some((-15, &[3]))),
    case("a", "bbba", true, Some((-15, &[3]))),
    case("ab cd", "abcd", false, Some((2880, &[0, 1, 2, 3]))),
    case("ab cd", "abcd", true, None),
    case("ab cd", "ab cd", false, Some((3670, &[0, 1, 3, 4]))),
    case("ab cd", "ab cd", true, Some((2147483647, &[0, 1, 2, 3, 4]))),
    case("ab cd", "cdab", false, Some((2880, &[2, 3, 0, 1]))),
    case("ab cd", "cdab", true, None),
];

/// The preference cases of `test_matchfuzzy.vim`, as an ordering over
/// scores: a pattern, then the candidates in the order `matchfuzzy()` puts
/// them, grouped so that each group scores strictly above the next and the
/// candidates *within* a group score the same.
///
/// The groups are the point. `matchfuzzy()` reports a total order, but part
/// of it comes from the tie-breaks in the sort — an exact substring wins a
/// tie, and the input order settles what is left — so an order the old test
/// observes is not by itself a claim about the scores. Everything below is
/// a claim about the scores, with each tie the scorer really produces
/// written down rather than papered over.
const ORDERING: &[(&str, &[&[&str]])] = &[
    // A full match beats everything, and a leading match beats a late one.
    ("Cursor", &[&["Cursor"], &["lCursor"]]),
    // Case is folded before scoring, so an exact-case candidate and one
    // that only matches folded score the *same*: `matchfuzzy()` puts
    // `onetwo` first because the pattern occurs in it literally, which is
    // the sort's `exact` tie-break and not a score at all.
    ("onetwo", &[&["onetwo", "oneTwo"]]),
    // A match after a separator is worth more than one that is buried, and
    // `_` and space are the same separator as each other.
    ("onetwo", &[&["onetwo"], &["one_two", "one two"]]),
    // Leading gaps cost.
    ("onetwo", &[&["onetwo"], &["xonetwo"], &["xxonetwo"]]),
    // A run of consecutive characters beats the same characters scattered.
    ("onetwo", &[&["onetwo"], &["oanbectdweo"]]),
    // Trailing gaps cost, less than inner ones.
    ("one", &[&["one"], &["onex"], &["onexx"]]),
    // A complete match beats one made of separator bonuses.
    (
        "vimrc",
        &[&[".vim/vimrc"], &[".vim/vimrc_colors"], &[".vim/v_i_m_r_c"]],
    ),
    // The inner gap penalty grows with the gap.
    ("ab", &[&["xxayybxxxx"], &["xxayyybxxx"], &["xxayyyybxx"]]),
    // The path separator outranks the word separators, which outrank a
    // plain consecutive match; a backslash is not a separator at all.
    (
        "setup.vim",
        &[
            &["color/setup.vim"],
            &["color setup.vim", "color_setup.vim"],
            &["colorsetup.vim"],
            &["color\\setup.vim"],
        ],
    ),
];

#[test]
fn candidates_score_in_the_order_matchfuzzy_reports() {
    for (pat, groups) in ORDERING {
        let scored_group = |group: &&[&str]| -> Vec<(String, c_int)> {
            group
                .iter()
                .map(|hay| {
                    let (matched, score, _) = scored(pat, hay, false, 0);
                    assert!(matched, "{pat:?} should match {hay:?}");
                    ((*hay).to_string(), score)
                })
                .collect()
        };
        let rows: Vec<Vec<(String, c_int)>> = groups.iter().map(scored_group).collect();
        for group in &rows {
            let (first, want) = &group[0];
            for (hay, got) in &group[1..] {
                assert_eq!(got, want, "{pat:?}: {hay:?} should tie with {first:?}");
            }
        }
        for pair in rows.windows(2) {
            let ((above, higher), (below, lower)) = (&pair[0][0], &pair[1][0]);
            assert!(
                higher > lower,
                "{pat:?}: {above:?} ({higher}) should outscore {below:?} ({lower})",
            );
        }
    }
}

#[test]
fn a_rejected_candidate_scores_the_sentinel() {
    // The one score callers test for by name: every `fuzzy_match_str` caller
    // in the tree compares against it rather than against the match flag.
    let (matched, score, _) = scored("xyz", "abc", true, 0);
    assert!(!matched);
    assert_eq!(score, FUZZY_SCORE_NONE);
}

#[test]
fn an_empty_pattern_matches_nothing_but_is_not_the_sentinel() {
    // Word-at-a-time, an empty pattern has no words, so the loop that would
    // reject the candidate never runs and the total stays zero. `matchfuzzy`
    // drops the candidate on the count of filled positions instead.
    let (matched, score, _) = scored("", "abc", false, 0);
    assert!(!matched);
    assert_eq!(score, 0);
    // Whitespace is not a word either.
    let (matched, score, _) = scored(" \t ", "abc", false, 0);
    assert!(!matched);
    assert_eq!(score, 0);
    // As one sequence there is no word loop, and an empty needle is a
    // rejection like any other.
    let (matched, score, _) = scored("", "abc", true, 0);
    assert!(!matched);
    assert_eq!(score, FUZZY_SCORE_NONE);
}

#[test]
fn a_summed_score_saturates_rather_than_wrapping() {
    // Two words that each match exactly score `INT_MAX` apiece; their sum
    // has to stay there rather than wrap to a rejection.
    let (matched, score, _) = scored("ab ab", "ab", false, 4);
    assert!(matched);
    assert_eq!(score, c_int::MAX);
}

#[test]
fn positions_are_bounded_by_the_array_the_caller_gave() {
    // Upstream writes one position per pattern character with no bound at
    // all; two words that together outrun the caller's array overrun it.
    let _sandbox = Sandbox::globals();
    let (hay, pat) = (cstr("abcdef"), cstr("abc def"));
    let mut matches = [0u32; 8];
    // An array deliberately shorter than the pattern's six matching
    // characters would need.
    let (_, filled) = fuzzy_match(&hay, &pat, false, &mut matches[..4]);
    assert!(filled != 0);
    assert_eq!(matches[4..], [0; 4], "wrote past the caller's four entries");
}
