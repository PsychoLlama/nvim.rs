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

use std::ffi::{CStr, c_char, c_int};
use std::ptr;

use neovim::main::{cmdmod, curwin};
use neovim::regexp::vim_regfree;
use neovim::search::{get_search_pat, pat_has_uppercase, search_regcomp};
use neovim::types::{CmdModFlags, regmmatch_T};

use crate::support::{Sandbox, cstr};

/// Whether `pat` counts as containing an uppercase character.
fn has_uppercase(pat: &str) -> bool {
    let _sandbox = Sandbox::globals();
    let pat = cstr(pat);
    // SAFETY: `pat` is NUL-terminated and outlives the call, and the editor
    // lock is held for the character tables `pat_has_uppercase` reads.
    unsafe { pat_has_uppercase(pat.as_ptr().cast_mut()) }
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
    rlc: *mut c_char,
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
                rlc: (*win).w_onebuf_opt.wo_rlc,
                flags: cmdmod.with(|c| c.cmod_flags),
            };
            (*win).w_onebuf_opt.wo_rl = 1;
            (*win).w_onebuf_opt.wo_rlc = rlc.as_ptr().cast_mut();
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
            (*win).w_onebuf_opt.wo_rlc = self.rlc;
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
    let mut regmatch = regmmatch_T::default();
    let pat = cstr(*b"a\xc0");
    // SAFETY: the pattern outlives the call, `used_pat` is optional and
    // null here, and `regmatch` is this stack frame's.
    let rc = unsafe {
        search_regcomp(
            pat.as_ptr().cast_mut(),
            2,
            ptr::null_mut(),
            0,
            0,
            0,
            &raw mut regmatch,
        )
    };
    assert_eq!(rc, Ok(()), "the reversed pattern still compiles");

    // SAFETY: `get_search_pat` answers the buffer `search_regcomp` just
    // filled; it belongs to the search module, so it is only read here.
    let compiled = unsafe { CStr::from_ptr(get_search_pat()) };
    assert_eq!(compiled.to_bytes(), b"\xc0a", "the pattern was reversed");

    // SAFETY: the program came from the `search_regcomp` above and is not
    // referenced anywhere else.
    unsafe { vim_regfree(regmatch.regprog) };
}

#[test]
fn only_a_rightleftcmd_of_s_reverses_the_pattern() {
    // `'rightleft'` alone is not the trigger: `'rightleftcmd'` says which
    // command lines are typed right to left, and only `s` (search) is a
    // value the reversal applies to. The Lua spec never asserted this side,
    // so dropping the `'rightleftcmd'` test entirely went unnoticed.
    let _guard = RightLeft::claim(c"");
    let mut regmatch = regmmatch_T::default();
    let pat = cstr(*b"a\xc0");
    // SAFETY: as in the case above.
    let rc = unsafe {
        search_regcomp(
            pat.as_ptr().cast_mut(),
            2,
            ptr::null_mut(),
            0,
            0,
            0,
            &raw mut regmatch,
        )
    };
    assert_eq!(rc, Ok(()));

    // SAFETY: as in the case above.
    let compiled = unsafe { CStr::from_ptr(get_search_pat()) };
    assert_eq!(
        compiled.to_bytes(),
        b"a\xc0",
        "the pattern was kept as typed"
    );

    // SAFETY: as in the case above.
    unsafe { vim_regfree(regmatch.regprog) };
}
