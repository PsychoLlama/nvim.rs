//! The match list, driven through the functions the completion uses: the
//! order adds land in, which duplicates are refused, how `<C-N>`/`<C-P>` walk
//! the ring and come back to the original text, how the fuzzy and nearest
//! orderings sort it, the numbering, a source's matches leaving, the
//! `complete_info()` view of one match, and the free.
//!
//! No buffer and no event loop: every add passes `CP_FAST` with the
//! breakcheck count reset, and `'completeopt'` is the global value, which is
//! what [`completeopt_flags`] answers with no current buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::dict_find;
use crate::global_cell::{editor_state::Held, editor_state_lock};
use crate::memory::xstrdup;
use crate::options::kOptCotFlagNearest;
use crate::os::input::reset_breakcheck_count;
use crate::types::{FAIL, OK};
use std::ffi::CString;

/// The completion's list state cleared, `'completeopt'` set to `cot`, and
/// both put back on drop.
struct Fixture {
    cot: c_uint,
    _held: Held,
}

impl Fixture {
    fn new(cot: c_uint) -> Fixture {
        let held = editor_state_lock();
        reset_breakcheck_count();
        got_int.set(false);
        let saved = cot_flags.replace(cot);
        reset();
        Fixture {
            cot: saved,
            _held: held,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        reset();
        cot_flags.set(self.cot);
    }
}

fn reset() {
    ins_compl_free();
    compl_orig_text().clear();
    cpt_sources().clear();
    compl_direction.set(FORWARD);
    compl_shows_dir.set(FORWARD);
    compl_get_longest.set(false);
    compl_autocomplete.set(false);
    compl_pending.set(0);
    compl_matches.set(0);
}

/// Add `text` as a match: `dir` side of the current one, `adup` allowing a
/// duplicate, scored `score`.
fn add_with(text: &str, dir: Direction, flags: c_int, adup: bool, score: c_int) -> c_int {
    let flags = flags | CP_FAST;
    ins_compl_add(
        text.as_bytes(),
        None,
        NO_EXTRA,
        None,
        dir,
        flags,
        adup,
        NO_HL,
        score,
    )
}

fn add(text: &str) -> c_int {
    add_with(text, kDirectionNotSet, 0, false, FUZZY_SCORE_NONE)
}

fn add_scored(text: &str, score: c_int) -> c_int {
    add_with(text, kDirectionNotSet, 0, false, score)
}

/// Start a completion over `text`: the original-text match every completion
/// begins with.
fn start(text: &str) {
    compl_orig_text().set(String_0::from_bytes(text.as_bytes()));
    ins_compl_add_orig_text(CP_ORIGINAL_TEXT | CP_FAST).expect("an empty list takes the original");
}

/// Start over `orig`, add `words` forward and close the ring, as a finished
/// collection leaves it.
fn ring(orig: &str, words: &[&str]) {
    start(orig);
    for word in words {
        assert_eq!(add(word), OK);
    }
    compl_matches.set(ins_compl_make_cyclic());
}

fn text_of(m: MatchId) -> String {
    String::from_utf8_lossy(&m.text_copy()).into_owned()
}

fn score_of(m: MatchId) -> c_int {
    m.with(|m| m.score)
}

fn number_of(m: MatchId) -> c_int {
    m.with(|m| m.number)
}

fn set_in_array(m: MatchId, in_array: bool) {
    m.update(|m| m.in_match_array = in_array);
}

/// The texts in list order, from the head; the original text is `"<o>"`.
fn texts() -> Vec<String> {
    matches_from(first_match())
        .map(|m| {
            if m.is_original() {
                "<o>".to_owned()
            } else {
                text_of(m)
            }
        })
        .collect()
}

/// The match at list position `at`.
fn nth(at: usize) -> MatchId {
    matches_from(first_match())
        .nth(at)
        .expect("a match at that position")
}

/// The list position of `m`.
fn position(m: Option<MatchId>) -> Option<usize> {
    let m = m?;
    matches_from(first_match()).position(|x| x == m)
}

fn shown_at() -> Option<usize> {
    position(shown_match())
}

/// One `<C-N>`/`<C-P>` press of `todo` steps, in `compl_shows_dir`.
fn step(todo: c_int) {
    let mut num_matches = -1;
    let status = find_next_completion_match(false, todo, true, &mut num_matches);
    assert_eq!(status, OK);
}

fn sort_fuzzy() {
    sort_compl_match_list(MatchOrder::Fuzzy);
}

fn sort_nearest() {
    sort_compl_match_list(MatchOrder::Nearest);
}

fn order_by_fuzzy_score(scores: &mut [c_int], indices: &mut [c_int]) {
    sort_by_fuzzy_score(scores, indices);
}

/// The ring is closed: the tail links to the head and back.
fn assert_cyclic() {
    let first = first_match().expect("a list");
    let last = first.prev().expect("a closed ring has a tail");
    assert!(last.next() == Some(first));
    assert_eq!(position(Some(last)), Some(texts().len() - 1));
}

#[test]
fn forward_adds_follow_the_current_match() {
    let _f = Fixture::new(0);
    start("ap");
    for word in ["apple", "apricot", "ape"] {
        assert_eq!(add(word), OK);
        assert_eq!(position(curr_match()), Some(texts().len() - 1));
    }
    assert_eq!(texts(), ["<o>", "apple", "apricot", "ape"]);
    assert_eq!(number_of(nth(0)), 0);
    assert_eq!(number_of(nth(1)), -1);
    // The list is open until the collection closes it.
    assert!(nth(3).next().is_none());
    assert!(nth(0).prev().is_none());
    assert_eq!(ins_compl_make_cyclic(), 3);
    assert_cyclic();
    ins_compl_make_linear();
    assert!(nth(3).next().is_none());
    assert!(nth(0).prev().is_none());
    assert_eq!(texts(), ["<o>", "apple", "apricot", "ape"]);
}

#[test]
fn backward_adds_go_before_the_current_match() {
    let _f = Fixture::new(0);
    compl_direction.set(BACKWARD);
    start("x");
    for word in ["x1", "x2", "x3"] {
        assert_eq!(add(word), OK);
        assert_eq!(position(curr_match()), Some(0));
    }
    // The original text ends up last, which is where backward completion
    // looks for it.
    assert_eq!(texts(), ["x3", "x2", "x1", "<o>"]);
    assert!(first_match().is_some_and(|m| !m.is_original()));
    // An explicit direction overrides `compl_direction`.
    assert_eq!(add_with("x4", FORWARD, 0, false, FUZZY_SCORE_NONE), OK);
    assert_eq!(texts(), ["x3", "x4", "x2", "x1", "<o>"]);
    assert_eq!(position(curr_match()), Some(1));
}

/// Add `text` with the extras a source can hand over: a file name, the four
/// `abbr`/`kind`/`menu`/`info` strings (an empty one is dropped) and a string
/// as user data.
fn add_full(text: &str, fname: Option<&str>, extra: [&str; 4], user_data: Option<&str>) -> c_int {
    let fname = fname.map(|f| CString::new(f).expect("no NUL in a test name"));
    let extra = extra.map(|s| Some(XString::from(s)));
    let mut data = user_data.map(|d| {
        let d = CString::new(d).expect("no NUL in test data");
        // SAFETY: a NUL-terminated string; the copy is the value's own.
        TypVal::string_raw(unsafe { xstrdup(d.as_ptr()) })
    });
    let (dir, score) = (kDirectionNotSet, FUZZY_SCORE_NONE);
    let fname = fname.as_deref();
    ins_compl_add(
        text.as_bytes(),
        fname,
        extra,
        data.as_mut(),
        dir,
        CP_FAST,
        false,
        NO_HL,
        score,
    )
}

/// Point `compl_shown_match` at list position `at`.
fn set_shown(at: usize) {
    compl_shown_match.set(Some(nth(at)));
}

/// Point `compl_curr_match` at list position `at`.
fn set_curr(at: usize) {
    compl_curr_match.set(Some(nth(at)));
}

/// `m`'s file name: its address (shared or not) and its text.
fn fname_of(m: MatchId) -> Option<(*const c_char, String)> {
    m.with(|m| {
        m.fname
            .as_ref()
            .map(|name| (name.as_ptr(), String::from_utf8_lossy(name).into_owned()))
    })
}

/// `m` as `v:completed_item` reads it.
fn completed_item(m: MatchId) -> crate::eval::typval::DictRef {
    ins_compl_dict_alloc(m)
}

/// A string member of `dict`: `None` when absent, `"<null>"` for a NULL
/// string.
fn member(dict: &Dict, key: &[u8]) -> Option<String> {
    let item = dict_find(Some(dict), key)?;
    let text = item.di_tv.string_or_null();
    Some(if text.is_null() {
        "<null>".to_owned()
    } else {
        // SAFETY: a dict string is NUL-terminated.
        String::from_utf8_lossy(unsafe { CStr::from_ptr(text) }.to_bytes()).into_owned()
    })
}

fn numbers() -> Vec<c_int> {
    matches_from(first_match()).map(number_of).collect()
}

#[test]
fn an_expansion_takes_its_direction_once() {
    let _f = Fixture::new(0);
    compl_direction.set(BACKWARD);
    start("y");
    let words = ["y1", "y2", "y3"];
    // SAFETY: an `xmalloc`ed array of `xstrdup`ed strings, which the add
    // takes over and frees.
    unsafe {
        let array = xmalloc(words.len() * size_of::<*mut c_char>()).cast::<*mut c_char>();
        for (i, word) in words.iter().enumerate() {
            let word = CString::new(*word).expect("no NUL");
            *array.add(i) = xstrdup(word.as_ptr());
        }
        ins_compl_add_matches(words.len() as c_int, array, 0);
    }
    // The first goes before the original, the rest follow it.
    assert_eq!(texts(), ["y1", "y2", "y3", "<o>"]);
}

#[test]
fn duplicates_are_refused_unless_allowed() {
    let _f = Fixture::new(0);
    start("ap");
    assert_eq!(add("apple"), OK);
    assert_eq!(add("apple"), NOTDONE);
    // A prefix and an extension are different words.
    assert_eq!(add("app"), OK);
    assert_eq!(add("apples"), OK);
    // The original text is never a duplicate.
    assert_eq!(add("ap"), OK);
    assert_eq!(add("ap"), NOTDONE);
    // `dup` lets the same word in twice.
    assert_eq!(
        add_with("apple", kDirectionNotSet, 0, true, FUZZY_SCORE_NONE),
        OK
    );
    assert_eq!(texts(), ["<o>", "apple", "app", "apples", "ap", "apple"]);
    // An interrupt refuses everything.
    got_int.set(true);
    assert_eq!(add("apricot"), FAIL);
    got_int.set(false);
    assert_eq!(texts().len(), 6);
}

#[test]
fn a_nearer_duplicate_lowers_the_score() {
    let _f = Fixture::new(kOptCotFlagNearest);
    start("w");
    assert_eq!(add_scored("word", 5), OK);
    assert_eq!(add_scored("word", 3), NOTDONE);
    assert_eq!(score_of(nth(1)), 3);
    assert_eq!(add_scored("word", 9), NOTDONE);
    assert_eq!(add_scored("word", 0), NOTDONE);
    assert_eq!(score_of(nth(1)), 3);
    // Without `nearest` the first score stands.
    cot_flags.set(0);
    assert_eq!(add_scored("word", 1), NOTDONE);
    assert_eq!(score_of(nth(1)), 3);
}

#[test]
fn fuzzy_longest_inserts_by_score() {
    let _f = Fixture::new(kOptCotFlagFuzzy);
    compl_get_longest.set(true);
    start("w");
    for (word, score) in [("a", 10), ("b", 30), ("c", 20), ("d", 30)] {
        assert_eq!(add_scored(word, score), OK);
    }
    // Highest first, a tie after the earlier one; the ring is already closed.
    assert_eq!(texts(), ["<o>", "b", "d", "c", "a"]);
    assert_eq!(position(curr_match()), Some(2));
    assert_cyclic();
}

#[test]
fn ctrl_n_walks_the_ring_and_back_to_the_original() {
    let _f = Fixture::new(0);
    ring("", &["a", "b", "c"]);
    assert_eq!(compl_matches.get(), 3);
    set_shown(0);
    let mut seen = Vec::new();
    for _ in 0..5 {
        step(1);
        seen.push(shown_at());
    }
    assert_eq!(seen, [Some(1), Some(2), Some(3), Some(0), Some(1)]);
    // A page at once stops at the last match rather than wrap past it.
    step(10);
    assert_eq!(shown_at(), Some(3));
    step(2);
    assert_eq!(shown_at(), Some(0));
    assert_eq!(compl_pending.get(), 0);
}

#[test]
fn ctrl_p_walks_the_ring_the_other_way() {
    let _f = Fixture::new(0);
    ring("", &["a", "b", "c"]);
    compl_shows_dir.set(BACKWARD);
    set_shown(1);
    let mut seen = Vec::new();
    for _ in 0..4 {
        step(1);
        seen.push(shown_at());
    }
    assert_eq!(seen, [Some(0), Some(3), Some(2), Some(1)]);
}

#[test]
fn the_leader_hides_what_it_does_not_start() {
    let _f = Fixture::new(0);
    ring("", &["apple", "banana", "blueberry", "cherry"]);
    compl_leader().set(String_0::from_bytes(b"b"));
    set_shown(0);
    step(1);
    assert_eq!(shown_at(), Some(2));
    step(1);
    assert_eq!(shown_at(), Some(3));
    step(1);
    assert_eq!(shown_at(), Some(0));
    // A shown match the leader hides moves on to the next it does not.
    set_shown(1);
    ins_compl_update_shown_match();
    assert_eq!(shown_at(), Some(2));
}

#[test]
fn the_menu_walk_skips_what_the_menu_left_out() {
    let _f = Fixture::new(0);
    ring("", &["a", "b", "c", "d"]);
    for (at, inside) in [(1, true), (2, false), (3, false), (4, true)] {
        set_in_array(nth(at), inside);
    }
    set_shown(1);
    assert_eq!(position(Some(find_next_match_in_menu())), Some(4));
    set_shown(4);
    assert_eq!(position(Some(find_next_match_in_menu())), Some(0));
    compl_shows_dir.set(BACKWARD);
    assert_eq!(position(Some(find_next_match_in_menu())), Some(1));
}

#[test]
fn numbers_count_out_from_the_original_in_the_running_direction() {
    let _f = Fixture::new(0);
    ring("", &["a", "b", "c"]);
    set_curr(2);
    // Nothing numbered yet: every match reads as the current one.
    assert!(compl_match_curr_select(0));
    ins_compl_update_sequence_numbers();
    assert_eq!(numbers(), [0, 1, 2, 3]);
    assert!(compl_match_curr_select(1));
    assert!(!compl_match_curr_select(0));
    assert!(!compl_match_curr_select(-1));
    reset();

    compl_direction.set(BACKWARD);
    ring("", &["a", "b", "c"]);
    assert_eq!(texts(), ["c", "b", "a", "<o>"]);
    set_curr(1);
    ins_compl_update_sequence_numbers();
    assert_eq!(numbers(), [3, 2, 1, 0]);
}

#[test]
fn fuzzy_scores_sort_the_matches_and_leave_the_original() {
    let _f = Fixture::new(kOptCotFlagFuzzy);
    let words = ["grape", "apple", "map", "apricot", "pear", "papaya"];
    let score =
        |word: &str, pattern: &CStr| fuzzy_match_str(&CString::new(word).expect("no NUL"), pattern);
    // A stable sort of the list as it stands, best score first.
    let sorted = |pattern: &CStr, list: &[String]| {
        let mut sorted = list.to_vec();
        sorted.sort_by_key(|word| ::core::cmp::Reverse(score(word, pattern)));
        sorted
    };

    // Forward: the original text heads the list and stays there.
    ring("ap", &words);
    set_fuzzy_score();
    for (i, word) in words.iter().enumerate() {
        assert_eq!(score_of(nth(i + 1)), score(word, c"ap"));
    }
    let before = texts()[1..].to_vec();
    sort_fuzzy();
    assert_eq!(texts()[0], "<o>");
    assert_eq!(texts()[1..], sorted(c"ap", &before));
    assert_cyclic();

    // A leader scores in place of the original text.
    compl_leader().set(String_0::from_bytes(b"pa"));
    set_fuzzy_score();
    let before = texts()[1..].to_vec();
    sort_fuzzy();
    assert_eq!(texts()[1..], sorted(c"pa", &before));
    reset();

    // Backward: the original text is the tail and stays there.
    compl_direction.set(BACKWARD);
    compl_shows_dir.set(BACKWARD);
    ring("ap", &words);
    set_fuzzy_score();
    let n = words.len();
    let before = texts()[..n].to_vec();
    sort_fuzzy();
    assert_eq!(texts()[n], "<o>");
    assert_eq!(texts()[..n], sorted(c"ap", &before));
    assert_cyclic();
}

#[test]
fn a_fuzzy_resort_moves_the_shown_match_under_noinsert() {
    let _f = Fixture::new(kOptCotFlagFuzzy | kOptCotFlagNoinsert);
    ring("ap", &["grape", "apple", "map"]);
    set_shown(3);
    ins_compl_fuzzy_sort();
    // The shown match is reset to the first one after the original.
    assert_eq!(shown_at(), Some(1));
    assert_eq!(texts()[1], "apple");
}

#[test]
fn nearest_sorts_scored_matches_and_leaves_unscored_in_runs() {
    let _f = Fixture::new(kOptCotFlagNearest);
    const NONE: c_int = FUZZY_SCORE_NONE;
    start("");
    for (word, score) in [
        ("n1", NONE),
        ("s5", 5),
        ("n2", NONE),
        ("s2", 2),
        ("s8", 8),
        ("n3", NONE),
        ("s1", 1),
    ] {
        assert_eq!(add_scored(word, score), OK);
    }
    ins_compl_make_cyclic();
    sort_nearest();
    // An unscored match compares equal to everything, so the order is the
    // bottom-up merge's and nothing else's.
    assert_eq!(texts(), ["<o>", "n1", "s1", "s5", "n2", "s2", "s8", "n3"]);
    assert_cyclic();
}

#[test]
fn file_matches_order_by_score_then_index() {
    let _f = Fixture::new(0);
    let mut scores = [7, FUZZY_SCORE_NONE, 9, 7, 3, 9];
    let mut indices = [0, 2, 3, 4, 5];
    order_by_fuzzy_score(&mut scores, &mut indices);
    assert_eq!(indices, [2, 5, 0, 3, 4]);
}

#[test]
fn a_refresh_drops_one_source_and_keeps_the_rest_in_place() {
    let _f = Fixture::new(0);
    cpt_sources().set_rows(vec![CPT_SOURCE_INIT; 3]);
    start("");
    for (source, word) in [(0, "a0"), (0, "b0"), (1, "a1"), (1, "b1"), (2, "a2")] {
        cpt_sources().set_index(source);
        assert_eq!(add(word), OK);
    }
    set_shown(3);
    cpt_sources().set_index(1);
    remove_old_matches();
    assert_eq!(texts(), ["<o>", "a0", "b0", "a2"]);
    // The shown match went with its source; the walk restarts at the head,
    // and the current match is the last one from an earlier source.
    assert_eq!(shown_at(), Some(0));
    assert_eq!(position(curr_match()), Some(2));
    assert_eq!(compl_direction.get(), FORWARD);
}

#[test]
fn a_match_shares_the_file_name_of_the_current_one() {
    let _f = Fixture::new(0);
    start("w");
    let none = ["", "", "", ""];
    assert_eq!(add_full("w1", Some("one.txt"), none, None), OK);
    assert_eq!(add_full("w2", Some("one.txt"), none, None), OK);
    assert_eq!(add_full("w3", Some("two.txt"), none, None), OK);
    assert_eq!(add_full("w4", None, none, None), OK);
    let (one, two, three) = (fname_of(nth(1)), fname_of(nth(2)), fname_of(nth(3)));
    let (one, two, three) = (
        one.expect("a name"),
        two.expect("a name"),
        three.expect("a name"),
    );
    assert_eq!(one.1, "one.txt");
    assert_eq!(one, two);
    assert_eq!(three.1, "two.txt");
    assert_ne!(three.0, one.0);
    assert!(fname_of(nth(4)).is_none());
}

#[test]
fn a_match_reads_back_as_a_completed_item() {
    let _f = Fixture::new(0);
    start("f");
    assert_eq!(
        add_full(
            "foo",
            Some("f.c"),
            ["FOO", "", "menu", "info"],
            Some("data")
        ),
        OK
    );
    assert_eq!(add("fob"), OK);
    let item = completed_item(nth(1));
    let get = |key: &[u8]| member(&item, key);
    assert_eq!(get(b"word").as_deref(), Some("foo"));
    assert_eq!(get(b"abbr").as_deref(), Some("FOO"));
    assert_eq!(get(b"kind").as_deref(), Some("<null>"));
    assert_eq!(get(b"menu").as_deref(), Some("menu"));
    assert_eq!(get(b"info").as_deref(), Some("info"));
    assert_eq!(get(b"user_data").as_deref(), Some("data"));
    assert_eq!(get(b"match"), None);
    // No user data reads back as an empty string.
    let plain = completed_item(nth(2));
    assert_eq!(member(&plain, b"user_data").as_deref(), Some(""));
    assert_eq!(member(&plain, b"abbr").as_deref(), Some("<null>"));
}

#[test]
fn the_free_takes_every_match_and_what_it_owns() {
    let _f = Fixture::new(0);
    start("f");
    assert_eq!(
        add_full("foo", Some("f.c"), ["A", "K", "M", "I"], Some("data")),
        OK
    );
    assert_eq!(
        add_full("fob", Some("f.c"), ["", "", "", ""], Some("more")),
        OK
    );
    assert_eq!(add("fig"), OK);
    ins_compl_make_cyclic();
    set_shown(2);
    compl_old_match.set(Some(nth(1)));
    compl_leader().set(String_0::from_bytes(b"fo"));
    compl_pattern().set(String_0::from_bytes(b"\\<fo"));
    ins_compl_free();
    assert!(first_match().is_none());
    assert!(curr_match().is_none());
    assert!(shown_match().is_none());
    assert!(old_match().is_none());
    assert!(compl_leader().is_unset());
    assert!(compl_pattern().is_unset());
    assert!(texts().is_empty());
    // A second free of an empty list is a no-op.
    ins_compl_free();
}

#[test]
fn complete_option_entries_split_as_copy_option_part_does() {
    let entries: Vec<(Vec<u8>, Vec<u8>)> = cpt_entries(b".,w^5, b,,u,k/a\\,b,Ffunc^3")
        .map(|entry| (entry.part, entry.after.to_vec()))
        .collect();
    let parts: Vec<&[u8]> = entries.iter().map(|(part, _)| &part[..]).collect();
    assert_eq!(parts, [&b"."[..], b"w^5", b"b", b"u", b"k/a,b", b"Ffunc^3"]);
    assert_eq!(entries[1].1, b"b,,u,k/a\\,b,Ffunc^3");
    assert!(entries[5].1.is_empty());
    assert_eq!(cpt_entries(b" , ,").count(), 0);
    // A long entry is cut where upstream's `LSIZE` buffer ends.
    let long = vec![b'x'; LSIZE as usize + 10];
    let entry = cpt_entries(&long).next().expect("one entry");
    assert_eq!(entry.part.len(), LSIZE as usize - 1);
    assert!(entry.after.is_empty());
}

#[test]
fn caret_counts_go_and_other_carets_stay() {
    assert_eq!(&*strip_caret_numbers(b".^3,w,b^12,u"), b".,w,b,u");
    assert_eq!(&*strip_caret_numbers(b"k^x,t^,F^3f"), b"k^x,t^,F^3f");
    assert_eq!(&*strip_caret_numbers(b"o^7"), b"o");
}

#[test]
fn the_next_entry_is_only_there_past_separators() {
    let _f = Fixture::new(0);
    cpt_sources().set_index(0);
    assert!(may_advance_cpt_index(b", ,w"));
    assert!(!may_advance_cpt_index(b", , "));
    cpt_sources().set_index(-1);
    assert!(!may_advance_cpt_index(b", ,w"));
}

#[test]
fn a_max_matches_count_reads_as_atoi_does() {
    for (text, value) in [
        (&b"3"[..], 3),
        (b" 12x", 12),
        (b"-2", -2),
        (b"+7", 7),
        (b"", 0),
        (b"x", 0),
    ] {
        assert_eq!(leading_number(text), value);
    }
}

#[test]
fn a_match_from_rebuilt_complete_rows_counts_against_no_source() {
    let _f = Fixture::new(0);
    // Three rows, the last capped at one match, as `'complete'` was when
    // the first match came in...
    let capped = CptSource {
        cs_max_matches: 1,
        ..CPT_SOURCE_INIT
    };
    cpt_sources().set_rows(vec![CPT_SOURCE_INIT, CPT_SOURCE_INIT, capped]);
    start("");
    cpt_sources().set_index(2);
    assert_eq!(add("a2"), OK);
    assert_eq!(add("b2"), OK);
    // ...and a single uncapped row after a `:set complete=` rebuilt them.
    cpt_sources().set_rows(vec![CPT_SOURCE_INIT]);
    cpt_sources().set_index(0);
    assert_eq!(add("a0"), OK);

    assert_eq!(nth(1).with(ComplItem::cpt_source), None);
    assert_eq!(nth(3).with(ComplItem::cpt_source), Some(0));
    // Upstream indexes its per-source counts with the stale row number.
    // Here the old matches count against no row, so the old cap is gone.
    ins_compl_build_pum();
    assert_eq!(compl_match_array().len(), 3);
    assert_eq!(find_common_prefix(false), None);
}
