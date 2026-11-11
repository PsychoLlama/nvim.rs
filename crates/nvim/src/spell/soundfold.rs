//! Turning a word into how it sounds.
//!
//! Sound-folding maps every word that is pronounced alike onto one common
//! string, so "nation" and "nashun" both fold to something like "NXN".
//! The suggestion machinery folds the bad word once and then compares that
//! against the folds of dictionary words; the `.sug` writer folds every
//! word in the dictionary to build its second tree.
//!
//! A language picks one of two schemes in its `.aff` file, and the `.spl`
//! reader records which in [`SpellLang::sl_sofo`]:
//!
//! * `SOFOFROM`/`SOFOTO` — a plain character-for-character mapping, done
//!   by [`spell_soundfold_sofo`]. Fast, and all that most languages need.
//! * `SAL` — a rule table taken from Aspell's `phonet.cpp`, done by
//!   [`spell_soundfold_wsal`]. Each rule matches a run of characters,
//!   optionally constrained by what follows, what precedes, and whether it
//!   sits at a word boundary, and replaces it with another run.
//!
//! Everything here works on words of at most [`MAXWLEN`] bytes and writes
//! into a caller-owned `res` buffer of that size.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mbyte::Codepoint;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::ascii::NUL;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::mbyte::MB_MAXBYTES;
use crate::mbyte::{char_at, char_len, encode_char, utf_class};
use crate::memory::XString;
use crate::regexp::low_byte;
use crate::spell::{SalItem, SpellLang};

use super::chartab::{spell_casefold, spell_iswordp_nmw, spell_iswordp_w};
use super::{LOADED_LANGS, MAXWLEN};
use crate::optionstr::OptString;

/// `soundfold()`: the sound-fold of `word` in the first of the window's
/// languages that has a sound-folding table, or a copy of `word` itself
/// when spell checking is off or no language defines one.
pub fn eval_soundfold(word: &CStr) -> XString {
    let win = Win::current();
    if win.w_onebuf_opt.wo_spell != 0 && win.syn_block().b_p_spl.first_byte() != 0 {
        let folded = LOADED_LANGS.with(|langs| {
            win.syn_block().b_langp.iter().find_map(|lp| {
                let slang = langs.get(lp.slang)?;
                slang.has_soundfold().then(|| {
                    let mut sound = [0u8; MAXWLEN];
                    spell_soundfold(slang, word.to_bytes(), false, &mut sound);
                    XString::from_bytes(crate::cstr::in_bytes(&sound).to_bytes())
                })
            })
        });
        if let Some(folded) = folded {
            return folded;
        }
    }
    XString::from_bytes(word.to_bytes())
}

/// Sound-fold `inword` into `res` using `slang`'s scheme, NUL terminated.
/// At most [`MAXWLEN`] bytes of `res` are used.
///
/// `folded` says whether `inword` has already been case-folded; the SAL
/// rules are written against folded text, so an unfolded word is folded
/// here first. The SOFO scheme folds case as part of its mapping and does
/// not care. `inword` ends at its end or at a NUL, whichever comes first.
pub fn spell_soundfold(slang: &SpellLang, inword: &[u8], folded: bool, res: &mut [u8]) {
    if slang.sl_sofo {
        spell_soundfold_sofo(slang, inword, res);
    } else if folded {
        spell_soundfold_wsal(slang, inword, res);
    } else {
        let mut fword = [0u8; MAXWLEN];
        let len = crate::cstr::until_nul(inword).len();
        let _ = spell_casefold(Win::current(), inword, len, &mut fword);
        spell_soundfold_wsal(slang, &fword, res);
    }
}

/// The SOFOFROM/SOFOTO scheme: replace every character by its counterpart,
/// drop the ones that map to nothing, and collapse runs of the same
/// character.
///
/// Characters below 256 are looked up in the flat `sl_sal_first` table.
/// Wider ones select a list by their low byte, where the reader left a
/// zero-terminated run of from/to pairs to scan.
fn spell_soundfold_sofo(slang: &SpellLang, inword: &[u8], res: &mut [u8]) {
    let mut ri = 0;
    let mut prevc = Codepoint::NUL;
    let mut s = 0;
    while s < inword.len() && inword[s] != 0 {
        let rest = &inword[s..];
        let mut c = char_at(rest);
        s += char_len(rest);
        if utf_class(c) == 0 {
            c = Codepoint::byte(b' ');
        } else if let Ok(byte) = u8::try_from(c) {
            c = Codepoint::new(slang.sl_sal_first[usize::from(byte)]);
        } else {
            // The scheme's table has one entry per low byte, and each list
            // ends in a zero.
            let list = &slang.sl_sofo_map[usize::from(low_byte(c.get()))];
            c = Codepoint::new(
                list.as_chunks::<2>()
                    .0
                    .iter()
                    .take_while(|pair| pair[0] != 0)
                    .find(|pair| pair[0] == c.get())
                    .map_or(NUL, |pair| pair[1]),
            );
        }

        if c != Codepoint::NUL && c != prevc {
            ri += encode_char(c, &mut res[ri..]);
            if ri + MB_MAXBYTES > MAXWLEN {
                break;
            }
            prevc = c;
        }
    }
    res[ri] = 0;
}

/// The SAL scheme, ported from Aspell's `phonet.cpp` by way of Vim.
///
/// The word is first widened to characters, dropping non-word characters
/// (collapsing each run of them to one space) when the language asks for
/// accents to be removed. Then rules are applied left to right.
///
/// Rules live in `sl_sal` sorted by the low byte of their first character,
/// with `sl_sal_first` giving the index of the first rule for each byte, so
/// the candidates for a position are a contiguous run. A candidate matches
/// when its `sm_lead_w` is a prefix of the word at this position, and, if it
/// has one, its `sm_oneof_w` set contains the character just after that.
///
/// `sm_rules` then carries the flags that decide whether the match really
/// applies and what it is worth:
///
/// * leading `-` characters shorten the match by one each;
/// * `<` means the replacement should be re-examined rather than emitted;
/// * a digit is the rule's priority (default 5);
/// * `^` requires the match to start a word, `$` requires it to end one;
/// * `^^` additionally restarts the scan from the replacement.
///
/// When the language sets `sl_followup`, a matched rule can still be
/// rejected in favour of a longer rule starting at its last character, if
/// that one's priority is at least as high.
///
/// # Bounds
///
/// `word` holds at most [`MAXWLEN`] characters because `inword` is at most
/// that many *bytes*. Every index derived from a rule match names a
/// character the match already compared against the word and found
/// non-NUL, or the NUL that terminates it, so no index passes `wordlen`.
///
fn spell_soundfold_wsal(slang: &SpellLang, inword: &[u8], res: &mut [u8]) {
    // `spell_iswordp*` answer for a window's `'iskeyword'`, and this whole
    // body runs in one: read the current one once.
    let win = Win::current();
    let mut word = [0 as c_int; MAXWLEN];
    let wordlen = widen_word(slang, inword, win, &mut word);

    let smp = &slang.sl_sal;
    let mut wres = [0 as c_int; MAXWLEN];
    let mut reslen = 0usize;

    // "k" is the length of the match in characters, "p0" the first byte
    // of the winning rule's flags, and "z" whether the previous position
    // was rewritten by a '<' rule. All three outlive one iteration.
    let mut k = 0usize;
    let mut p0 = -333;
    let mut z = 0;

    let mut i = 0usize;
    loop {
        let mut c = word[i];
        if c == NUL {
            break;
        }
        // Rules are grouped by the low byte of their first character. For
        // a character like 0x300 that byte is NUL, so the run's end has
        // to be recognised by the sentinel entry as well.
        let first = first_rule(slang, c);
        let mut z0 = 0;

        if let Some(mut n) = first {
            loop {
                let ws = &smp[n].sm_lead_w;
                if !(ws[0] & 0xff == c & 0xff && ws[0] != NUL) {
                    break;
                }
                if !rule_applies(slang, win, &word, i, n, &mut k, &mut p0) {
                    n += 1;
                    continue;
                }

                // The rule applies: substitute "sm_to_w".
                let to = smp[n].sm_to_w.as_deref();
                let rules = &smp[n].sm_rules[..];
                p0 = c_int::from(rules.contains(&b'<'));
                if p0 == 1 && z == 0 {
                    // A '<' rule writes back into the word so the
                    // replacement is matched again.
                    let first = to.map_or(NUL, |t| t[0]);
                    if reslen > 0
                        && first != NUL
                        && (wres[reslen - 1] == c || wres[reslen - 1] == first)
                    {
                        reslen -= 1;
                    }
                    z0 = 1;
                    z = 1;
                    let mut k0 = 0;
                    if let Some(to) = to {
                        while to[k0] != NUL && word[i + k0] != NUL {
                            word[i + k0] = to[k0];
                            k0 += 1;
                        }
                    }
                    if k > k0 {
                        word.copy_within(i + k..=wordlen, i + k0);
                    }
                    c = word[i];
                } else {
                    i += k - 1;
                    z = 0;
                    // The last character of the replacement is left
                    // for the tail of the loop to emit, so that the
                    // collapse test sees it.
                    let mut at = 0;
                    if let Some(to) = to {
                        while to[at] != NUL && to[at + 1] != NUL && reslen < MAXWLEN {
                            if reslen == 0 || wres[reslen - 1] != to[at] {
                                wres[reslen] = to[at];
                                reslen += 1;
                            }
                            at += 1;
                        }
                    }
                    c = to.map_or(NUL, |t| t[at]);
                    if rules.windows(2).any(|w| w == b"^^") {
                        if c != NUL && reslen < MAXWLEN {
                            wres[reslen] = c;
                            reslen += 1;
                        }
                        word.copy_within(i + 1..=wordlen, 0);
                        i = 0;
                        z0 = 1;
                    }
                }
                break;
            }
        } else if ascii_iswhite(c) {
            c = c_int::from(b' ');
            k = 1;
        }

        if z0 == 0 {
            if k != 0
                && p0 == 0
                && reslen < MAXWLEN
                && c != NUL
                && (!slang.sl_collapse || reslen == 0 || wres[reslen - 1] != c)
            {
                wres[reslen] = c;
                reslen += 1;
            }
            i += 1;
            z = 0;
            k = 0;
        }
    }

    let mut l = 0;
    for &wc in &wres[..reslen] {
        l += encode_char(Codepoint::new(wc), &mut res[l..]);
        if l + MB_MAXBYTES > MAXWLEN {
            break;
        }
    }
    res[l] = 0;
}

/// Widen `inword` into `word`, NUL-terminated, and answer its length in
/// characters. When the language asks for accents to be removed, what it
/// does not consider part of a word is dropped, and each run of
/// non-word characters (by `utf_class`) becomes one space.
fn widen_word(slang: &SpellLang, inword: &[u8], win: Win, word: &mut [c_int; MAXWLEN]) -> usize {
    let mut wordlen = 0usize;
    let mut did_white = false;
    let mut s = 0;
    while s < inword.len() && inword[s] != 0 {
        let t = &inword[s..];
        let mut c = char_at(t);
        s += char_len(t);
        if slang.sl_rem_accents {
            if utf_class(c) == 0 {
                if did_white {
                    continue;
                }
                c = Codepoint::byte(b' ');
                did_white = true;
            } else {
                did_white = false;
                if !spell_iswordp_nmw(t, win) {
                    continue;
                }
            }
        }
        word[wordlen] = c.get();
        wordlen += 1;
    }
    word[wordlen] = NUL;
    wordlen
}

/// Whether SAL rule `n` applies to `word` at character `i`: its lead and
/// "oneof" set match there, its `^`/`$` flags hold, and no follow-up rule
/// overrides it.
///
/// Leaves the match length in `k` and the rule's first flag byte (or a
/// follow-up's priority) in `p0` whether or not the rule applies: when no
/// rule does, the caller reads what the last candidate left there.
fn rule_applies(
    slang: &SpellLang,
    win: Win,
    word: &[c_int; MAXWLEN],
    i: usize,
    n: usize,
    k: &mut usize,
    p0: &mut c_int,
) -> bool {
    let rule = &slang.sl_sal[n];
    let ws = &rule.sm_lead_w;
    // Most leads are one or two characters; check the cheap cases before
    // the loop.
    if word[i] != ws[0] {
        return false;
    }
    *k = rule.sm_leadlen;
    if *k > 1 {
        if word[i + 1] != ws[1] {
            return false;
        }
        if *k > 2 && word[i + 2..i + *k] != ws[2..*k] {
            return false;
        }
    }

    // The character after the lead must be one of "sm_oneof", and counts
    // towards the match.
    if let Some(oneof) = &rule.sm_oneof_w {
        if !oneof
            .iter()
            .take_while(|&&ch| ch != NUL)
            .any(|&ch| ch == word[i + *k])
        {
            return false;
        }
        *k += 1;
    }

    let mut rules = &rule.sm_rules[..];
    let mut pri = 5;

    *p0 = c_int::from(rules[0]);
    let k0 = *k;
    while rules[0] == b'-' && *k > 1 {
        *k -= 1;
        rules = &rules[1..];
    }
    if rules[0] == b'<' {
        rules = &rules[1..];
    }
    if ascii_isdigit(c_int::from(rules[0])) {
        pri = c_int::from(rules[0]) - c_int::from(b'0');
        rules = &rules[1..];
    }
    if rules[0] == b'^' && rules[1] == b'^' {
        rules = &rules[1..];
    }

    let at_word_start = rules[0] == b'^'
        && (i == 0 || !(word[i - 1] == c_int::from(b' ') || spell_iswordp_w(&word[i - 1..], win)))
        && (rules[1] != b'$' || !spell_iswordp_w(&word[i + k0..], win));
    let at_word_end = rules[0] == b'$'
        && i > 0
        && spell_iswordp_w(&word[i - 1..], win)
        && !spell_iswordp_w(&word[i + k0..], win);
    if !(rules[0] == 0 || at_word_start || at_word_end) {
        return false;
    }

    !followup_overrides(slang, win, word, i, *k, pri, p0)
}

/// Whether a longer rule starting at the last character of a match of
/// length `k` at `i`, with priority `pri`, overrides that match: its
/// priority is at least as high. Only languages that set `sl_followup`
/// look. Leaves the last follow-up's priority in `p0`.
fn followup_overrides(
    slang: &SpellLang,
    win: Win,
    word: &[c_int; MAXWLEN],
    i: usize,
    k: usize,
    pri: c_int,
    p0: &mut c_int,
) -> bool {
    let smp = &slang.sl_sal;
    let c0 = word[i + k - 1];
    let first0 = first_rule(slang, c0);
    let Some(mut n0) = first0 else {
        return false;
    };
    if !(slang.sl_followup && k > 1 && *p0 != c_int::from(b'-') && word[i + k] != NUL) {
        return false;
    }
    // The outer loop stops on the sentinel rule, whose lead is NUL; this
    // one only compares low bytes, so a `c0` whose low byte is zero --
    // U+0100, U+0200, ... -- matches the sentinel too and walks off the end.
    // The C read past the array there; this stops.
    while let Some(followup) = smp.get(n0) {
        if followup.sm_lead_w[0] & 0xff != c0 & 0xff {
            break;
        }
        if followup_cuts(followup, win, word, i, k, pri, p0) {
            break;
        }
        n0 += 1;
    }

    *p0 >= pri
        && smp
            .get(n0)
            .is_some_and(|f| f.sm_lead_w[0] & 0xff == c0 & 0xff)
}

/// Whether `followup`, a rule for the match's last character, ends the
/// follow-up scan: it matches there, is longer than the match, and its
/// priority (left in `p0`) is at least `pri`. A `^` rule never cuts the
/// current match.
fn followup_cuts(
    followup: &SalItem,
    win: Win,
    word: &[c_int; MAXWLEN],
    i: usize,
    k: usize,
    pri: c_int,
    p0: &mut c_int,
) -> bool {
    let ws = &followup.sm_lead_w;
    if word[i + k - 1] != ws[0] {
        return false;
    }
    let mut k0 = followup.sm_leadlen;
    if k0 > 1 {
        if word[i + k] != ws[1] {
            return false;
        }
        if k0 > 2 && word[i + k + 1..i + k + k0 - 1] != ws[2..k0] {
            return false;
        }
    }
    k0 += k - 1;

    if let Some(oneof) = &followup.sm_oneof_w {
        if !oneof
            .iter()
            .take_while(|&&ch| ch != NUL)
            .any(|&ch| ch == word[i + k0])
        {
            return false;
        }
        k0 += 1;
    }

    *p0 = 5;
    let mut frules = &followup.sm_rules[..];
    // "k0" is deliberately not reduced here: the "k0 == k" test below
    // depends on it.
    while frules[0] == b'-' {
        frules = &frules[1..];
    }
    if frules[0] == b'<' {
        frules = &frules[1..];
    }
    if ascii_isdigit(c_int::from(frules[0])) {
        *p0 = c_int::from(frules[0]) - c_int::from(b'0');
        frules = &frules[1..];
    }

    // Same length means the follow-up is only a piece of this match.
    (frules[0] == 0 || (frules[0] == b'$' && !spell_iswordp_w(&word[i + k0..], win)))
        && k0 != k
        && *p0 >= pri
}

/// The first SAL rule for characters whose low byte is `c`'s, if any.
/// Rules are grouped by the low byte of their first character.
fn first_rule(slang: &SpellLang, c: c_int) -> Option<usize> {
    usize::try_from(slang.sl_sal_first[usize::from(low_byte(c))]).ok()
}
