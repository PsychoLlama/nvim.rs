//! Compiling a shell-style file pattern into a regexp: `fileio/names.rs`.
//!
//! This is how `'wildignore'`, `'backupskip'` and every autocommand pattern
//! become something the regexp engine can run, so its escaping rules are
//! user-visible in three places at once.
//!
//! Ported from `test/unit/fileio_spec.lua`, whose sixteen cases all called
//! it with no explicit end and no `allow_dirs` and only ever looked at the
//! returned string. The second answer, the explicit end and the two brace
//! errors are covered here for the first time.

use neovim::fileio::file_pat_to_reg_pat;

/// The regexp for the first `end` bytes of `pat`, and whether it says a path
/// separator has to match; `None` when it does not compile.
fn compile(pat: &str, end: usize) -> Option<(String, bool)> {
    file_pat_to_reg_pat(pat.as_bytes(), end)
        .map(|(reg, dirs)| (String::from_utf8_lossy(&reg).into_owned(), dirs))
}

/// The regexp for `pat`, the whole of it.
#[track_caller]
fn reg_pat(pat: &str) -> String {
    reg_pat_dirs(pat).0
}

/// The regexp for `pat`, plus whether it says a path separator has to match.
#[track_caller]
fn reg_pat_dirs(pat: &str) -> (String, bool) {
    compile(pat, pat.len()).unwrap_or_else(|| panic!("{pat:?} did not compile"))
}

#[test]
fn a_literal_path_is_anchored_at_both_ends() {
    assert_eq!(reg_pat("path"), "^path$");
}

#[test]
fn an_empty_pattern_matches_only_the_empty_name() {
    assert_eq!(reg_pat(""), "^$");
}

#[test]
fn a_leading_or_trailing_star_drops_the_anchor_it_makes_pointless() {
    assert_eq!(reg_pat("*path"), "path$");
    assert_eq!(reg_pat("path*"), "^path");
    assert_eq!(reg_pat("*path*"), "path");
}

#[test]
fn a_star_in_the_middle_becomes_dot_star() {
    assert_eq!(reg_pat("foo*bar"), "^foo.*bar$");
}

#[test]
fn a_question_mark_becomes_the_regexp_any_character() {
    assert_eq!(reg_pat("foo?bar"), "^foo.bar$");
}

#[test]
fn a_run_of_stars_collapses_to_one() {
    assert_eq!(reg_pat("foo*******bar"), "^foo.*bar$");
    assert_eq!(reg_pat("********foobar"), "foobar$");
    assert_eq!(reg_pat("foobar********"), "^foobar");
    // A pattern that is nothing but stars loses both anchors and keeps one
    // `.*`; upstream's leading-star loop stops one short of the end so the
    // trailing-star loop still has something to walk.
    assert_eq!(reg_pat("****"), ".*");
}

/// `ExpandEscape` escapes these on the way in, so the compiler has to take
/// the backslash back off — the regexp meaning of each is the literal one.
#[test]
fn the_escapes_the_expander_added_are_undone() {
    assert_eq!(reg_pat(r"foo\?bar"), "^foo?bar$");
    assert_eq!(reg_pat(r"foo\%bar"), "^foo%bar$");
    assert_eq!(reg_pat(r"foo\,bar"), "^foo,bar$");
    assert_eq!(reg_pat(r"foo\ bar"), "^foo bar$");
    assert_eq!(reg_pat(r"foo\#bar"), "^foo#bar$");
    assert_eq!(reg_pat("foo\\\tbar"), "^foo\tbar$");
}

/// A backslash in front of anything else stays a backslash: those characters
/// mean something to the regexp engine too, so unescaping them would change
/// what the pattern matches.
#[test]
fn an_escape_of_a_regexp_special_is_left_alone() {
    assert_eq!(reg_pat(r"foo\*bar"), r"^foo\*bar$");
    assert_eq!(reg_pat(r"foo\.bar"), r"^foo\.bar$");
    // A trailing backslash has nothing to escape and ends the walk, which
    // is why the `$` is still added but the backslash is not emitted.
    assert_eq!(reg_pat(r"foo\"), "^foo$");
}

/// Magic, not very magic: `\{n,m}` is spelled `\\\{n,m\}` in a file pattern
/// and the `\{` has to survive as `\{`.
#[test]
fn an_escaped_brace_reaches_the_engine_as_a_multi() {
    assert_eq!(reg_pat(r"a\\\{2,3\}"), r"^a\{2,3}$");
    assert_eq!(reg_pat(r"a\{b"), "^a{b$");
    assert_eq!(reg_pat(r"a\}b"), "^a}b$");
}

#[test]
fn a_dot_and_a_tilde_are_escaped() {
    assert_eq!(reg_pat("foo.bar"), r"^foo\.bar$");
    assert_eq!(reg_pat("foo~bar"), r"^foo\~bar$");
}

#[test]
fn brace_expansion_becomes_a_group_of_alternatives() {
    assert_eq!(reg_pat("foo{bar,baz}"), r"^foo\(bar\|baz\)$");
    assert_eq!(reg_pat("{a,b,c}"), r"^\(a\|b\|c\)$");
    // Nested braces keep their own alternation depth.
    assert_eq!(reg_pat("{a,{b,c}}"), r"^\(a\|\(b\|c\)\)$");
}

/// A comma is only an alternative *inside* braces; on its own it is a
/// literal, which is what makes a comma-separated 'wildignore' work at all.
#[test]
fn a_comma_outside_braces_is_a_literal() {
    assert_eq!(reg_pat("foo,bar"), "^foo,bar$");
}

/// `^` and `$` are regexp anchors and are deliberately not escaped, so a
/// pattern carrying one gets a regexp with two.
#[test]
fn the_anchors_are_not_escaped() {
    assert_eq!(reg_pat("^blah"), "^^blah$");
    assert_eq!(reg_pat("foo^bar"), "^foo^bar$");
    assert_eq!(reg_pat("blah$"), "^blah$$");
    assert_eq!(reg_pat("foo$bar"), "^foo$bar$");
}

/// The out-parameter tells `match_file_pat` whether to try the pattern
/// against the full path as well as the tail. Only a `/` sets it, and an
/// escaped one counts.
#[test]
fn allow_dirs_is_set_by_a_path_separator_and_nothing_else() {
    assert_eq!(reg_pat_dirs("foo"), ("^foo$".to_string(), false));
    assert_eq!(reg_pat_dirs("*.c"), (r"\.c$".to_string(), false));
    assert_eq!(reg_pat_dirs("foo/bar"), ("^foo/bar$".to_string(), true));
    assert_eq!(reg_pat_dirs("*/foo"), ("/foo$".to_string(), true));
    assert_eq!(reg_pat_dirs(r"foo\/bar"), (r"^foo\/bar$".to_string(), true));
    // A `/` the brace expansion carries counts too.
    assert_eq!(reg_pat_dirs("{a,b/c}"), (r"^\(a\|b/c\)$".to_string(), true));
}

/// `pat_end` stops the walk short of the NUL, which is how `match_file_list`
/// feeds one comma-separated entry at a time without copying it out.
#[test]
fn an_explicit_end_stops_before_the_nul() {
    assert_eq!(
        compile("foo/bar", 3),
        Some(("^foo$".into(), false)),
        "the `/` is past the end"
    );
    // A zero-length span is the empty pattern, not the whole string.
    assert_eq!(compile("foo/bar", 0), Some(("^$".into(), false)));
}

/// Unbalanced braces are the one way this fails: it answers NULL and reports
/// which side is missing.
#[cfg(not(miri))]
#[test]
fn unbalanced_braces_are_refused_with_a_message() {
    use crate::support::{check_emsg, editor_lock};

    let editor = editor_lock();
    for (pat, msg) in [
        ("foo{bar", "E220: Missing }."),
        ("foo}bar", "E219: Missing {."),
        ("{{a}", "E220: Missing }."),
    ] {
        let compiled = check_emsg(&editor, || compile(pat, pat.len()), Some(msg));
        assert!(compiled.is_none(), "{pat:?} should not have compiled");
    }
}
