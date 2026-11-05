//! The two search entry points that only an outside caller can reach.
//!
//! A port of `test/unit/search_spec.lua`. `pat_has_uppercase` is the
//! `'smartcase'` decision — "does this pattern contain an uppercase
//! character", answered over a pattern whose escapes depend on its own
//! magicness. `search_regcomp` is the compile-and-remember step every
//! search command goes through; the one case here pins its
//! `'rightleft'`/`'rightleftcmd'` branch, which reverses the pattern before
//! compiling it and has to do so bytewise, because a reversed pattern is
//! not necessarily valid UTF-8.
//!
//! Both need the editor: the first reads the character tables and
//! `'magic'`, the second reads `curwin`, `curbuf` and the command
//! modifiers. The LuaJIT harness forked a child per case and could write
//! those freely; here every write is saved and put back.

#![cfg(not(miri))]

use std::ffi::{CStr, c_int};

use neovim::ex_docmd::state::cmdmod;
use neovim::memory::XString;
use neovim::search::{get_search_pat, pat_has_uppercase, search_regcomp};
use neovim::types::{CmdModFlags, RegMMatch};
use neovim::winlayer::graph::curwin;

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

/// `curwin`'s `'rightleft'`/`'rightleftcmd'` pair and the command
/// modifiers, restored however the case leaves.
struct RightLeft {
    _sandbox: Sandbox,
    rl: c_int,
    rlc: Option<XString>,
    flags: CmdModFlags,
}

impl RightLeft {
    /// Turn on `'rightleft'` with `'rightleftcmd'` set to `s`, which is
    /// what makes `search_regcomp` reverse the pattern, and add
    /// `:keeppatterns` so the compile does not disturb the remembered
    /// search patterns of whatever runs next.
    fn claim(rlc: &'static CStr) -> RightLeft {
        let sandbox = Sandbox::globals();
        // SAFETY: `curwin` is the first window `early_init` allocated, and
        // the sandbox holds the editor lock for as long as this guard.
        unsafe {
            let win = curwin.get();
            let saved = RightLeft {
                _sandbox: sandbox,
                rl: (*win).w_onebuf_opt.wo_rl,
                rlc: (*win).w_onebuf_opt.wo_rlc.take(),
                flags: cmdmod.with(|c| c.cmod_flags),
            };
            (*win).w_onebuf_opt.wo_rl = 1;
            (*win).w_onebuf_opt.wo_rlc = Some(XString::from_cstr(rlc));
            cmdmod.with_mut(|c| c.cmod_flags |= CmdModFlags::KEEPPATTERNS);
            saved
        }
    }
}

impl Drop for RightLeft {
    fn drop(&mut self) {
        // SAFETY: the sandbox in this guard still holds the editor lock.
        unsafe {
            let win = curwin.get();
            (*win).w_onebuf_opt.wo_rl = self.rl;
            (*win).w_onebuf_opt.wo_rlc = self.rlc.take();
            cmdmod.with_mut(|c| c.cmod_flags = self.flags);
        }
    }
}

#[test]
fn a_right_to_left_pattern_is_reversed_bytewise() {
    // `a\xc0` is deliberately not valid UTF-8: `\xc0` announces a two-byte
    // sequence that is not there. Reversing it has to move bytes, not
    // characters, or the reversal reads past the end of the pattern.
    let _guard = RightLeft::claim(c"s");
    let mut regmatch = RegMMatch::default();
    let rc = search_regcomp(b"a\xc0", 0, 0, 0, &mut regmatch);
    assert!(rc.is_ok(), "the reversed pattern still compiles");

    let compiled = get_search_pat();
    assert_eq!(&compiled[..], b"\xc0a", "the pattern was reversed");
}

#[test]
fn only_a_rightleftcmd_of_s_reverses_the_pattern() {
    // `'rightleft'` alone is not the trigger: `'rightleftcmd'` says which
    // command lines are typed right to left, and only `s` (search) is a
    // value the reversal applies to. The Lua spec never asserted this side,
    // so dropping the `'rightleftcmd'` test entirely went unnoticed.
    let _guard = RightLeft::claim(c"");
    let mut regmatch = RegMMatch::default();
    let rc = search_regcomp(b"a\xc0", 0, 0, 0, &mut regmatch);
    assert!(rc.is_ok());

    let compiled = get_search_pat();
    assert_eq!(&compiled[..], b"a\xc0", "the pattern was kept as typed");
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
            "set nows | exe \"norm! G/zzz\\r\" | set ws",
            "E385: Search hit BOTTOM without match for: zzz",
        ),
        (
            "set nows | exe \"norm! gg?zzz\\r\" | set ws",
            "E384: Search hit TOP without match for: zzz",
        ),
        ("/zzz/", "E385: Search hit BOTTOM without match for: zzz"),
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
    fx.run("bwipe!");
    fx.home();
    fx.settle();
}
