//! A port of `test/unit/search_spec.lua`, and the search refusals.
//!
//! `pat_has_uppercase` is the `'smartcase'` decision — "does this pattern
//! contain an uppercase character", answered over a pattern whose escapes
//! depend on its own magicness. It reads the character tables and
//! `'magic'`. The spec's `search_regcomp` cases (its
//! `'rightleft'`/`'rightleftcmd'` reversal) live beside the function, in
//! `search/pattern.rs`.

#![cfg(not(miri))]

use neovim::search::pat_has_uppercase;

use crate::support::Sandbox;

/// Whether `pat` counts as containing an uppercase character.
fn has_uppercase(pat: &str) -> bool {
    let _sandbox = Sandbox::globals();
    pat_has_uppercase(pat.as_bytes())
}

#[test]
fn an_empty_pattern_has_no_uppercase() {
    assert!(!has_uppercase(""));
}

#[test]
fn case_is_judged_over_whole_characters() {
    // A multibyte character is decoded before the case test, so the lead
    // byte of a lowercase `ä` must not be mistaken for anything.
    assert!(!has_uppercase("ä"));
    assert!(has_uppercase("Ä"));
    assert!(has_uppercase("äaÅ"));
}

#[test]
fn a_trailing_backslash_ends_the_pattern() {
    // The escape has nothing to introduce; reading the byte after it would
    // read past the NUL.
    assert!(!has_uppercase("\\"));
    assert!(!has_uppercase("ab$\\"));
}

#[test]
fn an_escaped_character_is_not_the_users_case() {
    // `\A` is the "not alphabetic" class, not an uppercase A.
    assert!(!has_uppercase("\\Ab"));
    // ...but the character after the escape is the user's again.
    assert!(has_uppercase("\\AU"));
}

#[test]
fn an_escaped_underscore_or_percent_pair_is_skipped_whole() {
    // `\_A` and `\%A` are three-byte constructs: the class letter is part
    // of the escape, and only what follows is the user's text.
    assert!(!has_uppercase("\\_Ab"));
    assert!(has_uppercase("\\_AU"));
    assert!(!has_uppercase("aa\\%Ab"));
    assert!(has_uppercase("aab\\%AU"));
}

#[test]
fn a_very_magic_pattern_spells_the_same_pairs_without_a_backslash() {
    // Under `\v` the pattern's own magicness is `MAGIC_ALL`, and then `%A`
    // and `_A` are the escapes -- the backslash branch above never runs.
    // The Lua spec only ever reached the `MAGIC_ON` half.
    assert!(!has_uppercase("\\vaa%Ab"));
    assert!(has_uppercase("\\vaab%AU"));
    assert!(!has_uppercase("\\vaa_Ab"));
    assert!(has_uppercase("\\vaab_AU"));
}

/// Every message a refused or failed search puts out, through each entry
/// point that reports: the `/` address, `search()`, `:s`, the `/` and `?`
/// commands, `;` offsets and the include searches. A compile error the
/// compiler only half-reports (E1281, E867) is followed by E383; one that
/// counts as the pattern's fault is not. A doubled, dropped or reordered
/// message moves a row.
#[test]
fn refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = crate::support::Probe::new(sandbox.editor());
    fx.run("new | only!");
    fx.run("call setline(1, ['one a', 'two b', 'three a'])");
    let e1281 = "E1281: Atom '\\%#=1' must be at the start of the pattern";
    for (cmd, want) in [
        (r"/\%#=1\(/", "E54: Unmatched \\("),
        (r"/\%#=2\(/", "E54: Unmatched \\("),
        (
            r"/\%#=2a\%#=1/",
            &format!("{e1281}\nE383: Invalid search string: \\%#=2a\\%#=1"),
        ),
        (
            r"/\%#=1a\%#=1/",
            &format!("{e1281}\nE383: Invalid search string: \\%#=1a\\%#=1"),
        ),
        (
            r"/\%#=2a\zq/",
            "E867: (NFA) Unknown operator '\\zq'\nE383: Invalid search string: \\%#=2a\\zq",
        ),
        (
            r"/\%#=2\%[\(a\)]x/",
            "E486: Pattern not found: \\%#=2\\%[\\(a\\)]x",
        ),
        (r"call search('\%#=1\(')", "E54: Unmatched \\("),
        (r"call search('\%#=2a\%#=1')", e1281),
        ("call search('zzz')", "none"),
        (r"s/\%#=1\(/x/", "E54: Unmatched \\(\nE476: Invalid command"),
        (
            r"s/\%#=2a\%#=1/x/",
            &format!("{e1281}\nE476: Invalid command"),
        ),
        ("s/zzz/x/", "E486: Pattern not found: zzz"),
        ("exe \"norm! /\\\\%#=1\\\\(\\r\"", "E54: Unmatched \\("),
        (
            "exe \"norm! /\\\\%#=2a\\\\%#=1\\r\"",
            &format!("{e1281}\nE383: Invalid search string: \\%#=2a\\%#=1"),
        ),
        ("exe \"norm! /zzz\\r\"", "E486: Pattern not found: zzz"),
        (
            "set nows | exe \"norm! G/zzz\\r\"",
            "E385: Search hit BOTTOM without match for: zzz",
        ),
        (
            "set nows | exe \"norm! gg?zzz\\r\"",
            "E384: Search hit TOP without match for: zzz",
        ),
        (
            "set nows | /zzz/",
            "E385: Search hit BOTTOM without match for: zzz",
        ),
        (
            "exe \"norm! gg/a/;x\\r\"",
            "E386: Expected '?' or '/'  after ';'",
        ),
        ("ijump zzz", "E389: Couldn't find pattern"),
        ("ilist zzz", "E389: Couldn't find pattern"),
        ("dsearch zzz", "E388: Couldn't find definition"),
        ("exe \"norm! gg[i\"", "E387: Match is on current line"),
    ] {
        fx.check_messages(cmd, want);
    }
    // The failing searches above leave 'wrapscan' off: an error ends the
    // command line before anything after it could turn it back on.
    fx.run("set ws&");
    fx.run("bwipe!");
    fx.home();
    fx.settle();
}
