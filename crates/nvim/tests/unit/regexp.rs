//! The regexp engines, driven straight through their entry points.
//!
//! A port of `test/unit/regexp_spec.lua`, which the phase-20 rewrite of
//! `regexp/` was landed against. It calls `vim_regcomp`/`vim_regexec` rather
//! than going through Vimscript, so a failure points at the engine instead
//! of at whatever called it.
//!
//! Three overlapping nets, each catching a different kind of regression:
//!
//!   1. Golden expectations. Hand-written from the documented semantics
//!      (`:help pattern`), not captured from a run, so they still mean
//!      something when the implementation changes underneath them.
//!   2. A differential oracle. The backtracking and NFA engines share almost
//!      nothing but the entry point, so "both agree" is a cheap,
//!      self-maintaining check over a corpus far larger than anyone would
//!      write expectations for. Every golden case is also run on both.
//!   3. Termination and liveness. Pathological patterns must finish and must
//!      not take the process down.
//!
//! **What the move off `itp` cost.** The LuaJIT harness forked a child per
//! case and armed `SIGALRM`, so a hang killed the child and the parent
//! reported one failed test. `cargo test` is one process with no per-test
//! timeout, so a hang would wedge the whole run; [`deadline`] restores the
//! guarantee with a watchdog thread. A crash is no longer contained either —
//! it takes the test binary with it — which is why the corpus slices the
//! spec ran in four children are one case here: the reason to split them is
//! gone.
//!
//! Sibling spec: `test/functional/editor/regexp_spec.lua` covers what needs
//! a live buffer (multi-line matching, `:substitute`, search offsets).

#![cfg(not(miri))]

use std::ffi::{CString, c_char};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use c2rust_neovim::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec, vim_regfree};
use c2rust_neovim::types::regmatch_T;

use crate::support::Sandbox;

/// Prefixes that pin the engine. `\%#=N` is stripped by `vim_regcomp` before
/// the pattern proper, so it composes with a following `\v`/`\M`/`\V`.
const BT: &str = r"\%#=1";
const NFA: &str = r"\%#=2";
/// `\%#=0` lets `vim_regcomp` choose, and fall back to the backtracking
/// engine when the NFA one refuses the pattern.
const AUTO: &str = r"\%#=0";
/// The two real engines, named for a failure message.
const ENGINES: [(&str, &str); 2] = [("bt", BT), ("nfa", NFA)];

/// Compile `pat` under `engine` and match it against `line`, as a string
/// that reads well in a failure diff: `compile-error`, `nomatch`, or
/// `<start>-<end>` followed by ` <n>:<start>-<end>` for each set submatch.
/// Offsets are byte offsets into `line`.
///
/// The caller must hold the editor lock: compiling reads `'regexpengine'`
/// and matching runs in the process-wide `rex` context.
fn run(engine: &str, pat: impl AsRef<[u8]>, line: impl AsRef<[u8]>, ic: bool) -> String {
    let pattern =
        CString::new([engine.as_bytes(), pat.as_ref()].concat()).expect("a pattern holds no NUL");
    // SAFETY: a NUL-terminated pattern that outlives the call.
    let prog = unsafe { vim_regcomp(pattern.as_ptr(), RE_MAGIC | RE_STRING) };
    if prog.is_null() {
        return "compile-error".to_string();
    }
    let mut rm = regmatch_T {
        regprog: prog,
        rm_ic: ic,
        ..Default::default()
    };
    // The subject must outlive the match: `startp`/`endp` point into it.
    let text = CString::new(line.as_ref().to_vec()).expect("a subject holds no NUL");
    let base = text.as_ptr();
    // SAFETY: `rm` holds a program this call compiled, `base` is
    // NUL-terminated and outlives the match, and column 0 is in range.
    let hit = unsafe { vim_regexec(&raw mut rm, base, 0) };
    let at = |p: *mut c_char| {
        // SAFETY: on a hit the engine sets both ends to positions inside
        // `text`, which is the allocation `base` points at.
        unsafe { p.cast_const().offset_from(base) }
    };
    let answer = if hit {
        let mut answer = format!("{}-{}", at(rm.startp[0]), at(rm.endp[0]));
        for i in 1..10 {
            if !rm.startp[i].is_null() && !rm.endp[i].is_null() {
                answer += &format!(" {i}:{}-{}", at(rm.startp[i]), at(rm.endp[i]));
            }
        }
        answer
    } else {
        "nomatch".to_string()
    };
    // SAFETY: `regprog` is what the match left behind -- possibly a
    // recompile of the program above, which is why it is read back.
    unsafe { vim_regfree(rm.regprog) };
    answer
}

/// Describes a case the way it should appear in a failure message: the
/// pattern and input as one would type them.
fn label(pat: impl AsRef<[u8]>, line: impl AsRef<[u8]>) -> String {
    format!(
        "/{}/ on {:?}",
        String::from_utf8_lossy(pat.as_ref()),
        String::from_utf8_lossy(line.as_ref())
    )
}

/// Assert a golden expectation, and assert the two engines agree on it.
/// Both engines run for every case: an expectation that only holds on one of
/// them is itself a finding.
fn both(want: &str, pat: &str, line: &str, ic: bool) {
    let ctx = label(pat, line);
    for (name, engine) in ENGINES {
        assert_eq!(run(engine, pat, line, ic), want, "{name} {ctx}");
    }
}

/// Run a whole table of `(pattern, input, expected)` rows through [`both`].
fn golden(rows: &[(&str, &str, &str)]) {
    let _sandbox = Sandbox::globals();
    golden_locked(rows);
}

/// [`golden`] for a caller that has already taken the editor lock — which
/// every [`deadline`] case has to, because the lock is not reentrant and
/// because time spent waiting for it must not come out of the deadline.
fn golden_locked(rows: &[(&str, &str, &str)]) {
    for &(pat, line, want) in rows {
        both(want, pat, line, false);
    }
}

/// [`golden`] with the ignore-case flag the caller would have passed.
fn golden_ic(rows: &[(&str, &str, &str, bool)]) {
    let _sandbox = Sandbox::globals();
    for &(pat, line, want, ic) in rows {
        both(want, pat, line, ic);
    }
}

/// Run `f`, and take the process down if it has not finished in `seconds`.
///
/// This is the `SIGALRM` the spec armed inside its forked child, which is
/// the only hang guard available in a single process: nothing can interrupt
/// a `vim_regexec` that will not return, so the choice is between a loud
/// abort that names the case and a run that never ends. A pathological
/// pattern is exactly what this file exists to catch, so it aborts.
///
/// The bound is wall clock and therefore generous: it is there to turn
/// "never" into "reported", not to measure anything. **Arm it after taking
/// the editor lock, never before** — every case here queues behind that one
/// mutex, and a deadline armed in the queue measures the queue.
fn deadline<R>(what: &str, seconds: u64, f: impl FnOnce() -> R) -> R {
    let done = Arc::new(AtomicBool::new(false));
    let watch = Arc::clone(&done);
    let what = what.to_string();
    std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(seconds);
        while Instant::now() < until {
            if watch.load(Ordering::Relaxed) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !watch.load(Ordering::Relaxed) {
            eprintln!("regexp: {what} did not finish within {seconds}s");
            std::process::abort();
        }
    });
    let answer = f();
    done.store(true, Ordering::Relaxed);
    answer
}

/// Magic — the default — makes `.`, `*` and `[]` special, while `+ ? ( ) |
/// {` each need a backslash.
#[test]
fn magic_makes_dot_star_and_collections_special() {
    golden(&[
        ("a.c", "abc", "0-3"),
        ("a.c", "a.c", "0-3"),
        ("ab*c", "ac", "0-2"),
        ("ab*c", "abbbc", "0-5"),
        (r"[abc]\+", "xxabcxx", "2-5"),
        // while + ? ( ) | { need escaping
        ("a+", "a+", "0-2"),
        (r"a\+", "aaa", "0-3"),
        ("a?", "a?", "0-2"),
        (r"a\?", "a", "0-1"),
        ("(a)", "(a)", "0-3"),
        (r"\(a\)", "a", "0-1 1:0-1"),
        ("a|b", "a|b", "0-3"),
        (r"a\|b", "b", "0-1"),
        ("a{2}", "a{2}", "0-4"),
        (r"a\{2}", "aa", "0-2"),
    ]);
}

/// `\v` makes every punctuation character special, so the escapes go the
/// other way round.
#[test]
fn very_magic_makes_everything_but_word_characters_special() {
    golden(&[
        (r"\va+", "aaa", "0-3"),
        (r"\v(a|b)+", "abab", "0-4 1:3-4"),
        (r"\v%(ab)+", "abab", "0-4"),
        (r"\va{2,3}", "aaaa", "0-3"),
        (r"\va{-2,3}", "aaaa", "0-2"),
        (r"\v<word>", "a word here", "2-6"),
        (r"\v.{3}", "abcd", "0-3"),
        (r"\v\.", "a.b", "1-2"),
        (r"\v(foo)@<=bar", "foobar", "3-6 1:0-3"),
        (r"\vx=y", "y", "0-1"),
        (r"\v[[:digit:]]+", "ab12", "2-4"),
    ]);
}

/// `\M` leaves only `^` and `$` special; `\+ \? \( \| \{` keep working,
/// because they are backslashed at every level.
#[test]
fn nomagic_leaves_only_the_two_anchors_special() {
    golden(&[
        (r"\Ma.c", "a.c", "0-3"),
        (r"\Ma.c", "abc", "nomatch"),
        (r"\Ma\.c", "abc", "0-3"),
        (r"\Mab*c", "ab*c", "0-4"),
        (r"\Mab\*c", "abbc", "0-4"),
        (r"\M[abc]", "[abc]", "0-5"),
        (r"\M\[abc]", "b", "0-1"),
        (r"\M^abc$", "abc", "0-3"),
        // \+ \? \( \| \{ keep working: they are backslashed at every level
        (r"\Ma\+", "aaa", "0-3"),
        (r"\M\(a\)\1", "aa", "0-2 1:0-1"),
    ]);
}

/// `\V` leaves nothing special but the backslash itself.
#[test]
fn very_nomagic_leaves_only_the_backslash_special() {
    golden(&[
        (r"\Va.c", "a.c", "0-3"),
        (r"\Va*c", "a*c", "0-3"),
        (r"\Va\*", "aaa", "0-3"),
        (r"\V[abc]", "[abc]", "0-5"),
        (r"\V^abc", "^abc", "0-4"),
        (r"\V\^abc", "abc", "0-3"),
        (r"\Vabc\$", "abc", "0-3"),
        (r"\V\(a\)\1", "aa", "0-2 1:0-1"),
    ]);
}

/// A magic level is a switch inside the pattern, not a property of the whole
/// of it.
#[test]
fn a_magic_level_applies_from_where_it_appears() {
    golden(&[
        // \v..\V.. — the switch takes effect mid-pattern
        (r"a\v(b|c)", "ac", "0-2 1:1-2"),
        (r"\va\M(b)", "a(b)", "0-4"),
        (r"\Ma.\vb+", "a.bbb", "0-5"),
    ]);
}

/// `^` anchors at the start of a branch and is a literal anywhere else; `$`
/// likewise at the end.
#[test]
fn caret_and_dollar_anchor_to_the_ends_of_the_string() {
    golden(&[
        ("^abc", "abcdef", "0-3"),
        ("^abc", "xabcdef", "nomatch"),
        ("abc$", "xyzabc", "3-6"),
        ("abc$", "abcx", "nomatch"),
        ("^$", "", "0-0"),
        ("^$", "x", "nomatch"),
        ("^", "abc", "0-0"),
        ("$", "abc", "3-3"),
        ("^abc$", "abc", "0-3"),
        // ^ is only an anchor at the start of a branch; elsewhere it is literal
        ("a^b", "a^b", "0-3"),
        ("a$b", "a$b", "0-3"),
        (r"\(^a\|^b\)", "b", "0-1 1:0-1"),
    ]);
}

/// `\%^` and `\%$` are the ends of the *text*, which for a string match is
/// the string.
#[test]
fn the_text_anchors_hold_the_ends_of_the_whole_text() {
    golden(&[
        (r"\%^abc", "abcdef", "0-3"),
        (r"\%^bc", "abcdef", "nomatch"),
        (r"def\%$", "abcdef", "3-6"),
        (r"de\%$", "abcdef", "nomatch"),
    ]);
}

/// `\<` and `\>` are zero-width word boundaries.
#[test]
fn the_word_anchors_hold_word_boundaries() {
    golden(&[
        (r"\<mat\>", "on the mat", "7-10"),
        (r"\<mat\>", "on the matt", "nomatch"),
        (r"\<the", "blithe theory", "7-10"),
        (r"the\>", "blithe theory", "3-6"),
        (r"\<\w\+\>", "  word  ", "2-6"),
        // \zs after a boundary must not shift the boundary itself
        (r"\<\zsword", "a word", "2-6"),
    ]);
}

/// `*`, `\+`, `\?` and `\=` take as much as they can and give back only
/// when the rest of the pattern needs it.
#[test]
fn a_greedy_quantifier_takes_as_much_as_it_can() {
    golden(&[
        ("a*", "aaa", "0-3"),
        ("a*", "bbb", "0-0"), // matches empty at position 0
        (r"a\+", "aaa", "0-3"),
        (r"a\+", "bbb", "nomatch"),
        (r"a\?", "aaa", "0-1"),
        (r"a\?", "bbb", "0-0"),
        (r"a\=", "aaa", "0-1"),
        (".*", "abc", "0-3"),
        ("a.*b", "axbyb", "0-5"),
        ("ab*c*", "abbb", "0-4"),
    ]);
}

/// `\{-}` and its bounded forms take as little as they can.
#[test]
fn a_non_greedy_quantifier_takes_as_little_as_it_can() {
    golden(&[
        (r"a\{-}", "aaa", "0-0"),
        (r"a\{-}b", "aaab", "0-4"),
        (r"a.\{-}b", "axbyb", "0-3"),
        (r"a\{-1,}", "aaa", "0-1"),
        (r"a\{-1,3}", "aaaa", "0-1"),
        (r"a\{-2,3}", "aaaa", "0-2"),
        (r"\v.{-}b", "aaab", "0-4"),
    ]);
}

/// `\{n,m}` in each of its spellings, including the ones `:help /\{` leaves
/// implicit.
#[test]
fn a_counted_quantifier_respects_its_bounds() {
    golden(&[
        (r"a\{2}", "aaaa", "0-2"),
        (r"a\{3}", "aaaa", "0-3"),
        (r"a\{5}", "aaaa", "nomatch"),
        (r"a\{2,3}", "aaaa", "0-3"),
        (r"a\{3,6}", "aaaaaaaa", "0-6"),
        (r"a\{,3}", "aaaa", "0-3"),
        (r"a\{0,}", "aaaa", "0-4"),
        (r"a\{}", "aaaa", "0-4"),
        (r"a\{2,}", "aaaa", "0-4"),
        (r"a\{0}", "aaaa", "0-0"),
        (r"a\{1}", "aaaa", "0-1"),
        // an upper bound below the lower bound is clamped up to the lower one
        // rather than rejected; `:help /\{` leaves the case undefined, so this
        // pins what both engines actually do.
        (r"a\{3,2}", "aaaa", "0-3"),
    ]);
}

/// A quantifier binds to one atom — a character, a class, or a group.
#[test]
fn a_quantifier_applies_to_the_atom_in_front_of_it() {
    golden(&[
        (r"ab\+", "abbb", "0-4"),
        (r"\(ab\)\+", "abab", "0-4 1:2-4"),
        (r"\%(ab\)\{2}", "ababab", "0-4"),
        (r"[ab]\{3}", "abab", "0-3"),
        (r"\w\{2,}", " word ", "1-5"),
    ]);
}

/// A group under a quantifier reports the iteration that matched last, and
/// nine groups is the documented maximum.
#[test]
fn a_capture_records_its_last_iteration() {
    golden(&[
        (r"\(a\)", "a", "0-1 1:0-1"),
        (r"\(a\)\(b\)", "ab", "0-2 1:0-1 2:1-2"),
        (r"\(a\+\)\(b\+\)", "aabbb", "0-5 1:0-2 2:2-5"),
        (r"\(\(a\)\(b\)\)", "ab", "0-2 1:0-2 2:0-1 3:1-2"),
        (r"\(a\)\+", "aaa", "0-3 1:2-3"),
        // nine groups is the documented maximum
        (
            r"\(a\)\(b\)\(c\)\(d\)\(e\)\(f\)\(g\)\(h\)\(i\)",
            "abcdefghi",
            "0-9 1:0-1 2:1-2 3:2-3 4:3-4 5:4-5 6:5-6 7:6-7 8:7-8 9:8-9",
        ),
    ]);
}

/// `\%( ... \)` groups without taking a capture number, so the numbering of
/// the real groups is unaffected.
#[test]
fn a_percent_group_groups_without_capturing() {
    golden(&[
        (r"\%(ab\)\+", "abab", "0-4"),
        (r"\%(a\|b\)\(c\)", "bc", "0-2 1:1-2"),
        (r"\%(\(a\)\)", "a", "0-1 1:0-1"),
    ]);
}

/// Alternation is ordered: the first branch that matches wins, even when a
/// later one would match more.
#[test]
fn alternation_prefers_the_leftmost_branch_that_matches() {
    golden(&[
        (r"a\|ab", "ab", "0-1"),
        (r"ab\|a", "ab", "0-2"),
        (r"\(foo\|foobar\)", "foobar", "0-3 1:0-3"),
        (r"x\|y\|z", "zzz", "0-1"),
        (r"\v(a|b|c){3}", "cba", "0-3 1:2-3"),
        // an empty branch matches empty
        (r"\va|", "b", "0-0"),
    ]);
}

/// `\1` to `\9` match the *text* the group matched, not the pattern.
#[test]
fn a_backreference_matches_what_its_group_matched() {
    golden(&[
        (r"\(a\)\1", "aa", "0-2 1:0-1"),
        (r"\(a\)\1", "ab", "nomatch"),
        (r"\(ab\)\1", "abab", "0-4 1:0-2"),
        (r"\(.\)\1", "xaab", "1-3 1:1-2"),
        (r"\(\w\+\) \1", "the the", "0-7 1:0-3"),
        (r"\(a\)\(b\)\2\1", "abba", "0-4 1:0-1 2:1-2"),
        // a backref to a group that matched empty matches empty
        (r"\(x*\)y\1", "y", "0-1 1:0-0"),
    ]);
}

/// `r\%[ead]` matches `r`, `re`, `rea` or `read` — the way command
/// abbreviations are spelled.
#[test]
fn an_optional_sequence_matches_any_prefix_of_itself() {
    golden(&[
        (r"r\%[ead]", "r", "0-1"),
        (r"r\%[ead]", "re", "0-2"),
        (r"r\%[ead]", "rea", "0-3"),
        (r"r\%[ead]", "read", "0-4"),
        (r"r\%[ead]", "reads", "0-4"),
        (r"f\%[oo]x", "fx", "0-2"),
        (r"f\%[oo]x", "foox", "0-4"),
        (r"f\%[oo]x", "fooox", "nomatch"),
    ]);
}

/// One row per `\d \D \w \W \s \S \a \A \l \u \x \X \o \O \h \H`, plus the
/// three that depend on an option (`\k \i \p`).
#[test]
fn the_named_classes_match_their_documented_sets() {
    golden(&[
        (r"\d\+", "ab123cd", "2-5"),
        (r"\D\+", "12ab34", "2-4"),
        (r"\w\+", " _a1! ", "1-4"),
        (r"\W\+", "ab!?cd", "2-4"),
        (r"\s\+", "a \t b", "1-4"),
        (r"\S\+", "  ab  ", "2-4"),
        (r"\a\+", "12ab34", "2-4"),
        (r"\A\+", "ab12cd", "2-4"),
        (r"\l\+", "ABabAB", "2-4"),
        (r"\u\+", "abABab", "2-4"),
        (r"\x\+", "zzdeadbeefzz", "2-10"),
        (r"\X\+", "abzzab", "2-4"),
        (r"\o\+", "89012789", "2-6"),
        (r"\O\+", "01889012", "2-5"),
        (r"\h\+", "12_ab12", "2-5"),
        (r"\H\+", "ab12ab", "2-4"),
        // \i \I \k \K \f \F depend on 'isident'/'iskeyword'/'isfname'
        (r"\k\+", " word ", "1-5"),
        (r"\i\+", " word ", "1-5"),
        (r"\p\+", "ab", "0-2"),
    ]);
}

/// A `[...]` collection, including the four positions where a character is
/// literal rather than syntax (`]` first, `-` first or last, `^` not
/// first) and the unterminated form, which is not an error.
#[test]
fn a_collection_matches_negates_and_ranges() {
    golden(&[
        (r"[abc]\+", "xxabcxx", "2-5"),
        (r"[^abc]\+", "abcxyz", "3-6"),
        (r"[a-c]\+", "xxabcxx", "2-5"),
        (r"[a-cx-z]\+", "defxyzabc", "3-9"),
        (r"[^a-c]\+", "abcdef", "3-6"),
        (r"[0-9]\{2}", "ab12cd", "2-4"),
        // a ] first in the collection is literal
        ("[]]", "a]b", "1-2"),
        (r"[^]]\+", "]]ab]", "2-4"),
        // a - first or last is literal
        (r"[-a]\+", "x-ax", "1-3"),
        (r"[a-]\+", "xa-x", "1-3"),
        // a ^ that is not first is literal
        (r"[a^]\+", "x^ax", "1-3"),
        // backslash escapes inside a collection
        (r"[\]]", "a]b", "1-2"),
        (r"[\\]", r"a\b", "1-2"),
        (r"[\t]", "a\tb", "1-2"),
        (r"[\d65]", "xAy", "1-2"),
        // an empty-looking collection: [] is a literal [ followed by ]
        ("[]", "[]", "0-2"),
        // an unterminated collection is not an error: the [ is literal
        ("[a-", "x[a-y", "1-4"),
        ("[a", "x[ay", "1-3"),
    ]);
}

/// The `[:name:]` classes, alone and mixed with ordinary members.
#[test]
fn posix_classes_work_inside_a_collection() {
    golden(&[
        (r"[[:digit:]]\+", "ab123", "2-5"),
        (r"[[:alpha:]]\+", "12abc34", "2-5"),
        (r"[[:alnum:]]\+", "!!ab12!!", "2-6"),
        (r"[[:lower:]]\+", "ABabAB", "2-4"),
        (r"[[:upper:]]\+", "abABab", "2-4"),
        (r"[[:space:]]\+", "a \t b", "1-4"),
        (r"[[:punct:]]\+", "ab!?cd", "2-4"),
        (r"[[:xdigit:]]\+", "zzbeefzz", "2-6"),
        (r"[[:blank:]]\+", "a \tb", "1-3"),
        (r"[[:cntrl:]]\+", "a\u{1}\u{2}b", "1-3"),
        (r"[[:print:]]\+", "ab", "0-2"),
        (r"[[:graph:]]\+", " ab ", "1-3"),
        // combined with ordinary members
        (r"[[:digit:]x]\+", "ab1x2ab", "2-5"),
        (r"[^[:digit:]]\+", "12ab34", "2-4"),
    ]);
}

/// `\%d`, `\%x`, `\%o`, `\%u` and `\%U` name one character by its code
/// point, in decimal, hex, octal and two widths of hex.
#[test]
fn a_character_can_be_named_by_its_code() {
    golden(&[
        (r"\%d65", "xAy", "1-2"),
        (r"\%x41", "xAy", "1-2"),
        (r"\%o101", "xAy", "1-2"),
        (r"\%u0041", "xAy", "1-2"),
        (r"\%U00000041", "xAy", "1-2"),
        (r"\%d233", "xéy", "1-3"), // é, two bytes in utf-8
        (r"\%u00e9", "xéy", "1-3"),
        (r"\%d65\%d66", "xABy", "1-3"),
        (r"\%d65\+", "xAAAy", "1-4"),
    ]);
}

/// `\_x` is `x` plus the newline. With `vim_regexec` (not `_nl`) the subject
/// has no newline to match, so these assert the class still matches
/// everything it did before.
#[test]
fn an_underscore_class_still_matches_what_it_did() {
    golden(&[
        // With `vim_regexec` (not `_nl`) the string has no newline to match, so
        // these assert the class still matches everything it did before.
        (r"\_s\+", "a  b", "1-3"),
        (r"\_d\+", "ab12", "2-4"),
        (r"\_a\+", "12ab", "2-4"),
        (r"\_.\+", "abc", "0-3"),
        (r"\_[a-c]\+", "xxabc", "2-5"),
        (r"\_^abc", "abc", "0-3"),
        (r"abc\_$", "abc", "0-3"),
    ]);
}

/// `\t`, `\e`, `\r`, `\b` and the escapes that name a syntax character as
/// itself.
#[test]
fn an_escape_sequence_names_a_control_character() {
    golden(&[
        (r"\t", "a\tb", "1-2"),
        (r"\e", "a\u{1b}b", "1-2"),
        (r"\r", "a\u{d}b", "1-2"),
        (r"\b", "a\u{8}b", "1-2"),
        ("\\\\", r"a\b", "1-2"),
        (r"\/", "a/b", "1-2"),
        (r"\.", "a.b", "1-2"),
        (r"\*", "a*b", "1-2"),
        (r"\[", "a[b", "1-2"),
    ]);
}

/// Folding is not an ASCII-range table.
#[test]
fn case_folding_reaches_non_ascii_characters() {
    golden(&[
        (r"\cé", "É", "0-2"),
        (r"\cÉ", "é", "0-2"),
        (r"\cabcé", "ABCÉ", "0-5"),
        (r"\Cé", "É", "nomatch"),
    ]);
}

/// `.` and the quantifiers count characters; the offsets reported back are
/// byte offsets. A pattern that confuses the two mis-reports the span.
#[test]
fn characters_are_counted_while_offsets_are_bytes() {
    golden(&[
        (".", "é", "0-2"),
        (r".\{2}", "éé", "0-4"),
        (r"\v.{3}", "aéb", "0-4"),
        (r".\+", "héllo", "0-6"),
        (r"é\+", "xééy", "1-5"),
        (r"\v(é)+", "éé", "0-4 1:2-4"),
    ]);
}

/// A collection's members, ranges and negation are over characters.
#[test]
fn a_collection_can_hold_multibyte_members() {
    golden(&[
        ("[é]", "xéy", "1-3"),
        (r"[éè]\+", "xéèy", "1-5"),
        (r"[^é]\+", "ééab", "4-6"),
        (r"[a-é]\+", "xyz", "0-3"),
        (r"\w\+", "aéb", "0-1"), // é is not a word character by default
    ]);
}

/// `\zs` and `\ze` move where the match is *reported* to start and end
/// without changing what has to match.
#[test]
fn zs_and_ze_move_the_reported_match_only() {
    golden(&[
        (r"foo\zsbar", "foobar", "3-6"),
        (r"foo\zebar", "foobar", "0-3"),
        (r"foo\zsbar\zebaz", "foobarbaz", "3-6"),
        (r"\zsfoo", "foo", "0-3"),
        (r"foo\ze", "foo", "0-3"),
        // \zs inside a group still applies to the whole match
        (r"\(foo\zs\)bar", "foobar", "3-6 1:0-3"),
        // with a quantifier in front, \zs follows the last iteration
        (r"a\+\zsb", "aaab", "3-4"),
        // \ze before the start yields an empty match
        (r"foo\zebar", "xfoobar", "1-4"),
    ]);
}

/// `\@=` and `\@!` assert about what comes next without taking it.
#[test]
fn lookahead_asserts_without_consuming() {
    golden(&[
        (r"foo\(bar\)\@=", "foobar", "0-3 1:3-6"),
        (r"foo\(bar\)\@=", "foobaz", "nomatch"),
        (r"foo\(bar\)\@!", "foobaz", "0-3"),
        (r"foo\(bar\)\@!", "foobar", "nomatch"),
        (r"\v(a)@=..", "ab", "0-2 1:0-1"),
        (r"\v\d+(px)@!", "12em", "0-2"),
    ]);
}

/// `\@<=` and `\@<!` assert about what came before, optionally bounded by
/// how far back to look.
#[test]
fn lookbehind_asserts_over_what_came_before() {
    golden(&[
        (r"\(foo\)\@<=bar", "foobar", "3-6 1:0-3"),
        (r"\(foo\)\@<=bar", "bazbar", "nomatch"),
        (r"\(foo\)\@<!bar", "bazbar", "3-6"),
        (r"\(foo\)\@<!bar", "foobar", "nomatch"),
        (r"\v(a)@<=b", "ab", "1-2 1:0-1"),
        // a bounded look-behind: \@123<= limits how far back to look
        (r"\(a\)\@1<=b", "ab", "1-2 1:0-1"),
    ]);
}

/// `\@>` matches its group once and never backtracks into it.
#[test]
fn an_atomic_group_gives_nothing_back() {
    golden(&[
        (r"\(a*\)\@>a", "aaa", "nomatch"),
        (r"\(a*\)\@>b", "aaab", "0-4 1:0-3"),
        (r"\v(a+)@>b", "aab", "0-3 1:0-2"),
    ]);
}

/// `\c` and `\C` apply to the whole pattern wherever they appear, and
/// override the ignore-case flag the caller passed.
#[test]
fn an_explicit_case_flag_overrides_the_caller() {
    golden_ic(&[
        (r"\cabc", "ABC", "0-3", false),
        (r"abc\c", "ABC", "0-3", false),
        (r"a\cbc", "ABC", "0-3", false),
        (r"\CABC", "abc", "nomatch", false),
        (r"\Cabc", "abc", "0-3", false),
        // \c wins over the rm_ic the caller passed, and \C over both
        (r"\cabc", "ABC", "0-3", false),
        (r"\CABC", "abc", "nomatch", true),
    ]);
}

/// With no `\c` or `\C` in the pattern, the caller's `rm_ic` decides — and
/// it reaches collections and backreferences too.
#[test]
fn the_callers_ignore_case_flag_folds_case() {
    golden_ic(&[
        ("abc", "ABC", "0-3", true),
        ("abc", "ABC", "nomatch", false),
        ("ABC", "abc", "0-3", true),
        (r"[a-c]\+", "ABC", "0-3", true),
        (r"\(a\)\1", "aA", "0-2 1:0-1", true),
    ]);
}

// ---------------------------------------------------------------------------
// Compiling
// ---------------------------------------------------------------------------

/// Every documented construct compiles, on both engines. A pattern one
/// accepts and the other rejects is a divergence.
#[test]
fn every_documented_construct_compiles_on_both_engines() {
    let _sandbox = Sandbox::globals();
    for pat in [
        "abc",
        "a.c",
        r"a\+",
        r"\(a\)\1",
        r"\%(a\)\+",
        r"\va+",
        r"\v(a|b)*",
        r"\Ma\*",
        r"\Va.c",
        r"\v%(ab){2,3}",
        r"[a-z]\{2,}",
        r"\%d65",
        r"\%^abc\%$",
        r"a\@<=b",
        r"x\zsy\zez",
    ] {
        for (name, engine) in ENGINES {
            assert_ne!(
                run(engine, pat, "", false),
                "compile-error",
                "{name} rejected /{pat}/"
            );
        }
    }
}

/// Both engines must agree that these are errors: a rejection that turns
/// into a silent mis-compile is how crashes get in.
#[test]
fn both_engines_reject_the_same_malformed_patterns() {
    let _sandbox = Sandbox::globals();
    for pat in [
        r"\(",  // E54: unmatched \(
        r"\)",  // E55: unmatched \)
        r"\%(", // unmatched \%(
        r"a\{", // E554: unmatched \{
        r"\v(", // unmatched ( at very magic
        r"\v)",
        r"\z(a\)",        // \z( is only valid in syntax patterns
        r"\v(a){1,2}{3}", // multi directly after multi
        "a**",
        r"\+", // multi with nothing to repeat
        r"\v+",
        r"\%d", // \%d wants a number
        r"\%[", // unmatched \%[
        r"\@=", // lookaround with nothing in front
    ] {
        for (name, engine) in ENGINES {
            assert_eq!(
                run(engine, pat, "", false),
                "compile-error",
                "{name} accepted /{pat}/"
            );
        }
    }
}

/// The compile/free path on its own: a leak or a double free here is
/// otherwise masked by a successful match.
#[test]
fn a_program_can_be_freed_without_ever_matching() {
    let _sandbox = Sandbox::globals();
    let pat = CString::new(r"\(a\+\)\(b\|c\)\{2,5}").expect("a pattern holds no NUL");
    for _ in 0..100 {
        // SAFETY: a NUL-terminated pattern that outlives the call.
        let prog = unsafe { vim_regcomp(pat.as_ptr(), RE_MAGIC) };
        assert!(!prog.is_null(), "compile failed");
        // SAFETY: the program this loop just compiled, freed once.
        unsafe { vim_regfree(prog) };
    }
}

/// The atoms that mean something only against a buffer still have to
/// compile and terminate here. Their semantics live in the functional spec.
#[test]
fn the_buffer_position_atoms_compile_and_terminate() {
    let _sandbox = Sandbox::globals();
    for pat in [r"\%V.", r"\%23l.", r"\%>3c.", r"\%<9c.", r"\%5v.", r"\%#."] {
        for (name, engine) in ENGINES {
            // Reaching this line at all is the assertion: `run` returns.
            let answer = run(engine, pat, "abcdef", false);
            assert!(!answer.is_empty(), "{name} answered nothing for /{pat}/");
        }
    }
}

// ---------------------------------------------------------------------------
// The cases that need a deadline
// ---------------------------------------------------------------------------

/// The classic way to hang a backtracker: an inner atom with a zero-width
/// match under an outer `*`.
///
/// `\(a*\)*` belongs here too, but the engines disagree about what the group
/// captured — see [`a_star_over_an_empty_group_captures_differently`].
#[test]
fn a_quantified_group_that_matches_empty_still_terminates() {
    let _sandbox = Sandbox::globals();
    deadline("a quantified group that matches empty", 10, || {
        golden_locked(&[
            (r"\(a*\)\+b", "aaab", "0-4 1:3-3"),
            (r"\v(a|)+", "aa", "0-2 1:2-2"),
            (r"\v(){0,10}", "abc", "0-0 1:0-0"),
            (r"\(\)*x", "x", "0-1 1:0-0"),
        ]);
    });
}

/// Patterns and inputs are byte strings; the engine must cope with bytes
/// that are not valid UTF-8 rather than reading past them. No expectation is
/// written down — the two engines answering the same thing is the assertion.
#[test]
fn an_invalid_byte_sequence_does_not_derail_matching() {
    let _sandbox = Sandbox::globals();
    deadline("invalid byte sequences", 10, || {
        for line in [
            b"\xc0" as &[u8],
            b"a\xc0b",
            b"\xff\xff",
            b"a\xe0\xa0",
            b"\xed\xa0\x80",
        ] {
            // `\192` is not an escape a pattern knows: it reads as the
            // backreference `\1` followed by `92`, which is why both
            // engines refuse it. That is the spec's row, kept as it was.
            for pat in [".", r".\+", r"\w\+", r"[a-z]\+", r"\v.{1,3}", r"a\|\192"] {
                assert_eq!(
                    run(BT, pat, line, false),
                    run(NFA, pat, line, false),
                    "engines disagree on {}",
                    label(pat, line)
                );
            }
        }
    });
}

/// Nesting a capture inside a lookaround is a known divergence; see
/// [`a_capture_inside_a_lookaround_is_unset_on_the_nfa_engine`].
#[test]
fn nested_lookaround_terminates() {
    let _sandbox = Sandbox::globals();
    deadline("nested lookaround", 10, || {
        golden_locked(&[(r"\(a\(b\)\@!\)\+", "aac", "0-2 1:1-2")]);
    });
}

// ---------------------------------------------------------------------------
// Known engine divergences
// ---------------------------------------------------------------------------
//
// Cases where the backtracking and NFA engines disagree today. They are
// excluded from the differential corpus so the oracle stays meaningful, and
// pinned here instead: both sides are asserted, so a refactor that moves
// either engine still fails, and one that makes them agree fails loudly
// enough to come delete this block.
//
// All of them concern which span a capture reports, never whether or where
// the overall match lands.

/// Assert both sides of a divergence, and that it is still a divergence.
fn diverges(pat: &str, line: &str, bt: &str, nfa: &str) {
    let ctx = label(pat, line);
    assert_eq!(run(BT, pat, line, false), bt, "bt {ctx}");
    assert_eq!(run(NFA, pat, line, false), nfa, "nfa {ctx}");
    assert_ne!(
        bt, nfa,
        "divergence is gone; fold {ctx} back into the corpus"
    );
}

/// The backtracking engine reports the span the group consumed on its last
/// non-empty iteration; the NFA engine reports the trailing empty one. Both
/// agree on the overall match.
#[test]
fn a_star_over_an_empty_group_captures_differently() {
    let _sandbox = Sandbox::globals();
    diverges(r"\(a*\)*", "a", "0-1 1:0-1", "0-1 1:1-1");
    diverges(r"\(a*\)*", "aaa", "0-3 1:0-3", "0-3 1:3-3");
}

/// A capture that only a lookaround entered is left unset by the NFA engine.
#[test]
fn a_capture_inside_a_lookaround_is_unset_on_the_nfa_engine() {
    let _sandbox = Sandbox::globals();
    diverges(r"\(\(a\)\@=a\)\@=a", "aaa", "0-1 1:0-1 2:0-1", "0-1 2:0-1");
    diverges(r"\v((a)@<=b)@<=c", "abc", "2-3 1:1-2 2:0-1", "2-3 2:0-1");
}

// ---------------------------------------------------------------------------
// The differential corpus
// ---------------------------------------------------------------------------
//
// A cross product of patterns and inputs, run on both engines. No
// expectations to maintain — any disagreement is a bug in one of them, and
// after a refactor it is almost always a fresh one.
//
// Patterns that diverge today are deliberately absent; they live above with
// both behaviours pinned. Adding one back here is how you find out it has
// been fixed.

/// The spec ran these in four `itp`s so that a crash in one forked child
/// left the others reporting. One process, one case.
const PATTERNS: [&str; 81] = [
    // literals and dots
    "abc",
    "a.c",
    ".",
    ".*",
    r".\+",
    r".\{2,4}",
    // anchors
    "^abc",
    "abc$",
    "^.*$",
    r"^\(.\)\1",
    r"\<\w\+\>",
    r"\<a",
    r"a\>",
    // quantifiers
    "a*",
    r"a\+",
    r"a\?",
    r"a\{2,3}",
    r"a\{-}",
    r"a\{-1,}",
    "ab*c",
    r"\(ab\)*",
    r"\(ab\)\{2,}",
    r"\(a*\)\+b",
    r"a\{-}b",
    r"a.\{-}b",
    // alternation and grouping
    r"a\|b",
    r"ab\|ba",
    r"\(a\|b\)\+",
    r"\%(a\|b\)\{2}",
    r"\(a\)\(b\)\(c\)",
    r"\(\(a\)b\)c",
    r"foo\|foobar\|f",
    // backreferences
    r"\(a\)\1",
    r"\(.\)\1\+",
    r"\(\w\)\(\w\)\2\1",
    r"\(x*\)y\1",
    // classes
    r"\d\+",
    r"\w\+",
    r"\s\+",
    r"\a\+",
    r"\u\+",
    r"\x\+",
    r"\h\w*",
    r"[abc]\+",
    r"[^abc]\+",
    r"[a-z0-9]\+",
    r"[[:alpha:]]\+",
    r"[[:digit:][:space:]]\+",
    r"[]a-]\+",
    r"\_s\+",
    r"\_.\{2}",
    // \zs \ze
    r"a\zsb",
    r"a\zeb",
    r"\w\+\zs\d\+",
    r"\zs.*\ze",
    // lookaround
    r"a\(b\)\@=",
    r"a\(b\)\@!",
    r"\(a\)\@<=b",
    r"\(a\)\@<!b",
    r"\(a*\)\@>b",
    r"\v(\d+)@<=px",
    // optional sequence
    r"r\%[ead]",
    r"f\%[oo]\d",
    // magic levels
    r"\v(a|b)+c?",
    r"\v\d{2,}",
    r"\v(.)\1",
    r"\Ma\*b",
    r"\Va.c",
    r"\M\(a\)\1",
    // case
    r"\cabc",
    r"\CABC",
    r"\ca\+",
    // character codes
    r"\%d97\+",
    r"\%x61\%x62",
    r"\%u0061",
    // text anchors
    r"\%^a",
    r"c\%$",
    // empty and near-empty
    "",
    r"\(\)",
    r"\(\)*",
    r"a\{0}",
];

/// The subjects every pattern is run over. All valid UTF-8; the byte string
/// that is not lives in [`INVALID_UTF8_INPUT`], because a Rust `str` cannot
/// hold it and the corpus is the poorer without it.
const INPUTS: [&str; 27] = [
    "",
    "a",
    "b",
    "ab",
    "abc",
    "aaa",
    "aaab",
    "abab",
    "abcabc",
    "ABC",
    "aAbBcC",
    "  ab  ",
    "a\tb",
    "123",
    "ab123cd",
    "x_y1",
    "read",
    "reads",
    "foo",
    "foobar",
    "the the",
    "aabbaa",
    "[a-]",
    "a]b",
    "héllo",
    "ééé", // ...and three built rather than written out
    // (`('a'):rep(64)`, `('ab'):rep(32)`) plus the one below.
    "aaaaaaaaaaaaaaaaaaaaX",
];

/// `a`, a lead byte announcing a two-byte sequence that is not there, `b`.
const INVALID_UTF8_INPUT: &[u8] = b"a\xc0b";

/// Every subject, as bytes, including the two long ones the spec built with
/// `rep` and the one that is not valid UTF-8.
fn inputs() -> Vec<Vec<u8>> {
    let mut all: Vec<Vec<u8>> = INPUTS.iter().map(|s| s.as_bytes().to_vec()).collect();
    all.push(b"a".repeat(64));
    all.push(b"ab".repeat(32));
    all.push(INVALID_UTF8_INPUT.to_vec());
    all
}

/// Any disagreement between the two engines is a bug in one of them.
#[test]
fn the_two_engines_agree_over_the_whole_corpus() {
    let _sandbox = Sandbox::globals();
    deadline("the differential corpus", 240, || {
        let inputs = inputs();
        for pat in PATTERNS {
            for line in &inputs {
                for ic in [false, true] {
                    assert_eq!(
                        run(BT, pat, line, ic),
                        run(NFA, pat, line, ic),
                        "engines disagree (ic={ic}) on {}",
                        label(pat, line)
                    );
                }
            }
        }
    });
}

/// `\%#=0` lets `vim_regcomp` choose, and fall back to the backtracking
/// engine when the NFA one refuses the pattern. The result must still be one
/// of the two, never a third answer.
///
/// Which one it picks is deliberately not asserted, and cannot be: moving
/// `AUTO_MAX_REPEAT` from 500 to 50 — which changes the engine a whole class
/// of patterns is compiled by — leaves every expectation in this file
/// standing, because the two engines agree on all of them. That is the
/// guard's contract (it is a cost heuristic, not a behaviour switch) and
/// this case is what states it.
#[test]
fn the_automatic_engine_answers_as_one_of_the_two() {
    let _sandbox = Sandbox::globals();
    deadline("the automatic engine", 240, || {
        let inputs = inputs();
        for pat in PATTERNS {
            for line in &inputs {
                let (auto, bt, nfa) = (
                    run(AUTO, pat, line, false),
                    run(BT, pat, line, false),
                    run(NFA, pat, line, false),
                );
                assert!(
                    auto == bt || auto == nfa,
                    "auto={auto} bt={bt} nfa={nfa} for {}",
                    label(pat, line)
                );
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Pathological patterns
// ---------------------------------------------------------------------------
//
// Everything here is about liveness, not results: the engine may match or
// fail, but it must return, and it must not take the process with it.

/// Shapes with an exponential number of ways to split the input: nested
/// quantifiers over an atom that can match the same text more than one way.
const BOMBS: [&str; 10] = [
    r"\(a\+\)\+b",
    r"\(a*\)*b",
    r"\(\(a\)\+\)\+b",
    r"\([a-z]\+\)*x",
    r"\v(a|a)+b",
    r"\v(a|aa)+b",
    r"\v(.*){0,20}x",
    ".*.*.*.*.*x",
    r"\v(a{1,10}){1,10}b",
    r"\v(a+)+(b+)+c",
];

/// Inputs that force the full search: no `b`/`c`/`x` to match, so every
/// split has to be tried before the engine can report failure.
fn bomb_inputs(n: usize) -> Vec<Vec<u8>> {
    vec![
        b"a".repeat(n),
        [b"a".repeat(n), b"b".to_vec()].concat(),
        [b"a".repeat(n), b"c".to_vec()].concat(),
        b"ab".repeat(n / 2),
    ]
}

/// This is the property that keeps the editor usable: the automatic engine
/// tries the NFA one first, so anything it handles in stride never reaches
/// the backtracker. The input is long enough that an exponential search
/// could not possibly finish inside the deadline.
#[test]
fn the_nfa_engine_stays_linear_on_backtracking_bombs() {
    let _sandbox = Sandbox::globals();
    deadline("the NFA engine on bombs", 60, || survives(NFA, 40));
}

/// The automatic engine inherits it, which is what a user actually gets.
#[test]
fn the_automatic_engine_stays_linear_on_backtracking_bombs() {
    let _sandbox = Sandbox::globals();
    deadline("the automatic engine on bombs", 60, || survives(AUTO, 40));
}

/// The backtracking engine really is exponential in the input length — that
/// is what the NFA engine exists to avoid, not a defect to assert away.
/// `\%#=1` with a longer input than this does not come back, and a user who
/// types it gets what they asked for. The input is short so the code path is
/// exercised without the test becoming the hang.
#[test]
fn the_backtracking_engine_terminates_on_bombs_it_can_afford() {
    let _sandbox = Sandbox::globals();
    deadline("the backtracking engine on bombs", 60, || survives(BT, 10));
}

/// Every bomb over every input of length `n`, on one engine. Returning at
/// all is the assertion.
fn survives(engine: &str, n: usize) {
    for pat in BOMBS {
        for line in bomb_inputs(n) {
            let answer = run(engine, pat, &line, false);
            assert!(!answer.is_empty(), "no result for {}", label(pat, &line));
        }
    }
}

/// Deep nesting and long repetition, on both engines.
#[test]
fn deeply_nested_and_deeply_repeated_patterns_terminate() {
    let _sandbox = Sandbox::globals();
    let deep: Vec<String> = vec![
        format!("{}a{}", r"\%(".repeat(40), r"\)".repeat(40)),
        format!(r"\v{}a{}", "%(".repeat(40), ")".repeat(40)),
        // nine is the group limit
        format!("{}a{}", r"\(".repeat(9), r"\)".repeat(9)),
        format!(r"\v{}a{}", "(".repeat(9), ")".repeat(9)),
        format!("{}{}", r"a\?".repeat(40), "a".repeat(40)),
        format!(r"\v{}{}", "a?".repeat(40), "a".repeat(40)),
        r"[a-z]\{1,1000}".to_string(),
        // see `a_large_counted_bound_...` below for why not larger
        r"a\{1,1000}".to_string(),
        r"\|a".repeat(200)[2..].to_string(),
        ".".repeat(200),
        format!(r"\v{}\1\2\3\4\5\6\7\8\9", "(a)".repeat(9)),
    ];
    deadline("deep patterns", 120, || {
        for pat in &deep {
            for line in [
                "".to_string(),
                "a".to_string(),
                "a".repeat(60),
                "ab".repeat(30),
            ] {
                for (name, engine) in ENGINES {
                    let answer = run(engine, pat, &line, false);
                    assert!(
                        !answer.is_empty(),
                        "{name}: no result for {}",
                        label(pat, &line)
                    );
                }
            }
        }
    });
}

/// Truncated and unbalanced constructs: the engine may reject them, but it
/// must not read past the end of the pattern to decide.
#[test]
fn a_truncated_pattern_is_rejected_rather_than_mis_compiled() {
    let _sandbox = Sandbox::globals();
    let mut broken: Vec<String> = vec![
        "\\".to_string(),
        r"\v".to_string(),
        r"\M".to_string(),
        r"\V".to_string(),
        r"\%".to_string(),
        r"\%#".to_string(),
        r"\%#=".to_string(),
        r"\%#=9".to_string(),
        r"\%d".to_string(),
        r"\%x".to_string(),
        r"\%u".to_string(),
        r"\%[".to_string(),
        r"\%[abc".to_string(),
        r"\(".to_string(),
        r"\)".to_string(),
        r"\%(".to_string(),
        r"\z(".to_string(),
        r"\z1".to_string(),
        "[".to_string(),
        "[a".to_string(),
        "[a-".to_string(),
        "[[:".to_string(),
        "[[:foo:]]".to_string(),
        r"\{".to_string(),
        r"a\{".to_string(),
        r"a\{1".to_string(),
        r"a\{1,".to_string(),
        r"\@".to_string(),
        r"a\@".to_string(),
        r"a\@<".to_string(),
        r"\v(".to_string(),
        r"\v)".to_string(),
        r"\v[".to_string(),
        r"\v{".to_string(),
        r"\v%(".to_string(),
        r"\v%[".to_string(),
        r"\v\C[\zs".to_string(),
        "\\%#=1\\v(a+)+b\\".to_string(),
    ];
    broken.push(r"\(".repeat(20));
    broken.push("(".repeat(100));
    broken.push("[".repeat(50));
    deadline("broken patterns", 120, || {
        for pat in &broken {
            for (name, engine) in ENGINES {
                let answer = run(engine, pat, "aaabbb", false);
                assert!(!answer.is_empty(), "{name}: no result for /{pat}/");
            }
        }
    });
}

/// `nfa_regpiece` compiles `\{n,m}` by emitting the atom `m` times, so the
/// bound is a compile-time cost. The guard against a silly one
/// (`nfa/parse.rs`, `AUTO_MAX_REPEAT`) is conditional on `RE_AUTO`, which
/// means it only fires for the automatic engine; forcing `\%#=2` walks
/// straight past it. These are the bounds that are safe today; the unsafe
/// ones are the ignored case below.
#[test]
fn a_large_counted_bound_is_expanded_up_to_where_the_guard_ends() {
    let _sandbox = Sandbox::globals();
    deadline("large counted bounds", 60, || {
        for n in [10, 100, 255, 256, 500, 1000] {
            let pat = format!(r"a\{{1,{n}}}");
            for engine in [BT, NFA, AUTO] {
                assert_eq!(run(engine, &pat, "a", false), "0-1", "{}", label(&pat, "a"));
            }
        }
        // The automatic engine is safe at any bound: it declines the NFA
        // compile and falls back to the backtracking engine, which expands
        // `\{n,m}` iteratively.
        assert_eq!(run(AUTO, r"a\{1,50000}", "a", false), "0-1");
        assert_eq!(run(BT, r"a\{1,50000}", "a", false), "0-1");
    });
}

/// A bound that big on a forced NFA engine overflows the stack of the thread
/// it runs on.
///
/// The recursion is `addstate` (`nfa/list.rs`), which calls itself once per
/// state it follows and stops at [a depth of 5,000][ADDSTATE_MAX_DEPTH] —
/// upstream's `regexp.c` has the identical guard, at the identical depth.
/// 5,000 frames fit the editor's 8 MiB main stack, where the pattern simply
/// answers "no match"; they do not fit the 2 MiB a `cargo test` thread gets,
/// where it is a `SIGABRT` that takes the whole binary down. The bound where
/// it turns over here is between 3,000 and 5,000.
///
/// The spec carried this as a `pending` blaming recursion in `post2nfa`.
/// That was the wrong function — `post2nfa` walks the postfix with an
/// explicit stack, here and upstream — and the case is kept ignored for the
/// real reason. Restore it to a plain `#[test]` if `addstate` ever stops
/// recursing; a depth guard alone is not enough, because the depth that is
/// affordable depends on the stack the caller happens to have.
///
/// [ADDSTATE_MAX_DEPTH]: https://github.com/neovim/neovim/blob/v0.12.4/src/nvim/regexp.c#L13058
#[test]
#[ignore = "5,000 recursive addstate frames do not fit a test thread's stack"]
fn a_large_counted_bound_on_a_forced_nfa_engine_overflows_the_stack() {
    let _sandbox = Sandbox::globals();
    deadline("large counted bounds on the NFA engine", 60, || {
        for n in [5000, 20000, 100_000] {
            let pat = format!(r"a\{{1,{n}}}");
            let answer = run(NFA, &pat, "a", false);
            assert!(!answer.is_empty(), "{}", label(&pat, "a"));
        }
    });
}

/// A long subject and a long pattern, on both engines.
#[test]
fn long_inputs_and_long_patterns_terminate() {
    let _sandbox = Sandbox::globals();
    let long = "abcdefghij".repeat(500);
    let mut pats: Vec<String> = [
        ".*",
        r".\+x",
        r"\w\+",
        r"\(abc\)\+",
        r"[a-j]\{100,}",
        r"j\zsa",
        r"\(a\)\@<=b",
    ]
    .iter()
    .map(|p| (*p).to_string())
    .collect();
    pats.push("abcdefghij".repeat(50));
    deadline("long inputs", 120, || {
        for pat in &pats {
            for (name, engine) in ENGINES {
                let answer = run(engine, pat, &long, false);
                assert!(
                    !answer.is_empty(),
                    "{name}: no result for /{pat}/ on a {} byte line",
                    long.len()
                );
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Fuzzing
// ---------------------------------------------------------------------------
//
// Randomly assembled patterns, seeded so a failure reproduces. Most are
// syntactically invalid, which is the point: rejection paths get far less
// hand-written coverage than matching ones, and that is where the reads past
// the end of the pattern live.

/// The pieces a random pattern is assembled from.
const ATOMS: [&[u8]; 41] = [
    b"a",
    b"b",
    b"1",
    b".",
    b"*",
    br"\+",
    br"\?",
    br"\{",
    br"\{2,3}",
    br"\{-}",
    b"[",
    b"]",
    b"[a-z]",
    br"\(",
    br"\)",
    br"\%(",
    br"\|",
    b"^",
    b"$",
    br"\<",
    br"\>",
    br"\zs",
    br"\ze",
    br"\_",
    b"\\",
    br"\d",
    br"\w",
    br"\s",
    br"\1",
    br"\@=",
    br"\@!",
    br"\@<=",
    br"\@>",
    br"\%[",
    br"\%d",
    br"\%^",
    br"\%$",
    br"\c",
    br"\C",
    b"\xc3\xa9",
    b"\xc0",
];

/// The magic level a random pattern opens with.
const PREFIXES: [&str; 5] = ["", r"\v", r"\m", r"\M", r"\V"];

/// The bytes a random subject is assembled from — bytes, not characters, so
/// half of the `é` is reachable and the engine gets fed a broken sequence.
const INPUT_CHARS: &[u8] = b"abc123 \t.[](){}|*+?\\^$\xc3\xa9A";

/// xorshift64*, seeded. Not Lua's generator and not meant to be: the
/// property the corpus needs is that a failing run reproduces, which any
/// deterministic sequence gives.
struct Rand(u64);

impl Rand {
    fn new(seed: u64) -> Rand {
        Rand(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
    }

    /// A number in `0..n`.
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 33) as usize % n
    }
}

fn random_pattern(rand: &mut Rand) -> Vec<u8> {
    let mut pat = PREFIXES[rand.below(PREFIXES.len())].as_bytes().to_vec();
    for _ in 0..=rand.below(12) {
        pat.extend_from_slice(ATOMS[rand.below(ATOMS.len())]);
    }
    pat
}

fn random_input(rand: &mut Rand) -> Vec<u8> {
    (0..rand.below(20))
        .map(|_| INPUT_CHARS[rand.below(INPUT_CHARS.len())])
        .collect()
}

/// Four seeds, each 2,000 patterns on both engines. A failure names the seed
/// and the iteration, which is enough to reproduce it on its own.
#[test]
fn random_patterns_are_rejected_or_matched_but_never_hang() {
    let _sandbox = Sandbox::globals();
    deadline("the fuzz corpus", 300, || {
        for seed in [1, 42, 1337, 20_260_727] {
            let mut rand = Rand::new(seed);
            for i in 0..2000 {
                let pat = random_pattern(&mut rand);
                let line = random_input(&mut rand);
                for (name, engine) in ENGINES {
                    let answer = run(engine, &pat, &line, false);
                    assert!(
                        !answer.is_empty(),
                        "{name} seed {seed} iteration {i}: no result for {}",
                        label(&pat, &line)
                    );
                }
            }
        }
    });
}
