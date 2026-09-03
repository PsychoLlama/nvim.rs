//! Turning a word into how it sounds.
//!
//! Sound-folding maps every word that is pronounced alike onto one common
//! string, so "nation" and "nashun" both fold to something like "NXN".
//! The suggestion machinery folds the bad word once and then compares that
//! against the folds of dictionary words; the `.sug` writer folds every
//! word in the dictionary to build its second tree.
//!
//! A language picks one of two schemes in its `.aff` file, and the `.spl`
//! reader records which in [`slang_T::sl_sofo`]:
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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int};

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::main::curwin;
use crate::mbyte::{mb_cptr2char_adv, utf_char2bytes, utf_class};
use crate::memory::xstrdup;
use crate::types::{MB_MAXBYTES, NUL, langp_T, slang_T};

use super::MAXWLEN;
use super::chartab::{spell_casefold, spell_iswordp_nmw, spell_iswordp_w};

/// `soundfold()`: the sound-fold of `word` in the first of the window's
/// languages that has a sound-folding table, or a copy of `word` itself
/// when spell checking is off or no language defines one.
pub unsafe fn eval_soundfold(word: *const c_char) -> *mut c_char {
    let win = curwin.get();
    if unsafe { (*win).w_onebuf_opt.wo_spell } != 0 && unsafe { *(*(*win).w_s).b_p_spl } != 0 {
        // SAFETY: `win` is the current window; its syntax block is live.
        let langp = unsafe { &(*(*win).w_s).b_langp };
        for lpi in 0..langp.ga_len {
            let lp = unsafe { (langp.ga_data as *mut langp_T).offset(lpi as isize) };
            if unsafe { (*(*lp).lp_slang).has_soundfold() } {
                let mut sound = [0 as c_char; MAXWLEN];
                let slang = unsafe { (*lp).lp_slang };
                let out = sound.as_mut_ptr();
                unsafe { spell_soundfold(slang, word as *mut c_char, false, out) };
                return unsafe { xstrdup(sound.as_ptr()) };
            }
        }
    }
    unsafe { xstrdup(word) }
}

/// Sound-fold `inword` into `res` using `slang`'s scheme.
///
/// `folded` says whether `inword` has already been case-folded; the SAL
/// rules are written against folded text, so an unfolded word is folded
/// here first. The SOFO scheme folds case as part of its mapping and does
/// not care.
pub unsafe fn spell_soundfold(
    slang: *mut slang_T,
    inword: *mut c_char,
    folded: bool,
    res: *mut c_char,
) {
    if unsafe { (*slang).sl_sofo } {
        unsafe { spell_soundfold_sofo(slang, inword, res) };
    } else if folded {
        unsafe { spell_soundfold_wsal(slang, inword, res) };
    } else {
        let mut fword = [0 as c_char; MAXWLEN];
        let (win, out) = (curwin.get(), fword.as_mut_ptr());
        let len = unsafe { cstr::bytes_at(inword) }.len() as c_int;
        let _ = unsafe { spell_casefold(win, inword, len, out, MAXWLEN as c_int) };
        unsafe { spell_soundfold_wsal(slang, fword.as_ptr(), res) };
    }
}

/// The SOFOFROM/SOFOTO scheme: replace every character by its counterpart,
/// drop the ones that map to nothing, and collapse runs of the same
/// character.
///
/// Characters below 256 are looked up in the flat `sl_sal_first` table.
/// Wider ones select a list by their low byte, where the reader left a
/// zero-terminated run of from/to pairs to scan.
unsafe fn spell_soundfold_sofo(slang: *mut slang_T, inword: *const c_char, res: *mut c_char) {
    let mut ri = 0;
    let mut prevc = 0;
    let mut s = inword;
    while unsafe { *s } != 0 {
        let mut c = unsafe { mb_cptr2char_adv(&raw mut s) };
        if unsafe { utf_class(c) } == 0 {
            c = ' ' as c_int;
        } else if c < 256 {
            c = unsafe { (*slang).sl_sal_first[c as usize] };
        } else {
            // SAFETY: the caller's language; the scheme's table has one
            // entry per low byte, and each list ends in a zero.
            let list = &unsafe { &(*slang).sl_sofo_map }[(c & 0xff) as usize];
            c = list
                .as_chunks::<2>()
                .0
                .iter()
                .take_while(|pair| pair[0] != 0)
                .find(|pair| pair[0] == c)
                .map_or(NUL, |pair| pair[1]);
        }

        if c != NUL && c != prevc {
            ri += unsafe { utf_char2bytes(c, res.offset(ri as isize)) };
            if ri + MB_MAXBYTES as c_int > MAXWLEN as c_int {
                break;
            }
            prevc = c;
        }
    }
    unsafe { *res.offset(ri as isize) = NUL as c_char };
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
unsafe fn spell_soundfold_wsal(slang: *mut slang_T, inword: *const c_char, res: *mut c_char) {
    // Widen the word, dropping what the language does not consider part
    // of a word when it asked for accents to be removed.
    let mut word = [0 as c_int; MAXWLEN];
    let mut wordlen = 0usize;
    let mut did_white = false;
    let mut s = inword;
    while unsafe { *s } != 0 {
        let t = s;
        let mut c = unsafe { mb_cptr2char_adv(&raw mut s) };
        if unsafe { (*slang).sl_rem_accents } {
            if unsafe { utf_class(c) } == 0 {
                if did_white {
                    continue;
                }
                c = ' ' as c_int;
                did_white = true;
            } else {
                did_white = false;
                if !unsafe { spell_iswordp_nmw(t, curwin.get()) } {
                    continue;
                }
            }
        }
        word[wordlen] = c;
        wordlen += 1;
    }
    word[wordlen] = NUL;

    // SAFETY: the caller's language, which stays loaded for the fold.
    let smp = unsafe { &(*slang).sl_sal };
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
        let mut n = unsafe { (*slang).sl_sal_first[(c & 0xff) as usize] };
        let mut z0 = 0;

        if n >= 0 {
            'rules: loop {
                let ws = &smp[n as usize].sm_lead_w;
                if !(ws[0] & 0xff == c & 0xff && ws[0] != NUL) {
                    break 'rules;
                }
                'next_rule: {
                    // Most leads are one or two characters; check the
                    // cheap cases before the loop.
                    if c != ws[0] {
                        break 'next_rule;
                    }
                    k = smp[n as usize].sm_leadlen as usize;
                    if k > 1 {
                        if word[i + 1] != ws[1] {
                            break 'next_rule;
                        }
                        if k > 2 && word[i + 2..i + k] != ws[2..k] {
                            break 'next_rule;
                        }
                    }

                    // The character after the lead must be one of
                    // "sm_oneof", and counts towards the match.
                    if let Some(oneof) = &smp[n as usize].sm_oneof_w {
                        if !oneof
                            .iter()
                            .take_while(|&&ch| ch != NUL)
                            .any(|&ch| ch == word[i + k])
                        {
                            break 'next_rule;
                        }
                        k += 1;
                    }

                    let mut rules = &smp[n as usize].sm_rules[..];
                    let mut pri = 5;

                    p0 = c_int::from(rules[0]);
                    let mut k0 = k;
                    while rules[0] == b'-' && k > 1 {
                        k -= 1;
                        rules = &rules[1..];
                    }
                    if rules[0] == b'<' {
                        rules = &rules[1..];
                    }
                    if ascii_isdigit(c_int::from(rules[0])) {
                        pri = c_int::from(rules[0]) - '0' as c_int;
                        rules = &rules[1..];
                    }
                    if rules[0] == b'^' && rules[1] == b'^' {
                        rules = &rules[1..];
                    }

                    let at_word_start = rules[0] == b'^'
                        && (i == 0
                            || !(word[i - 1] == ' ' as c_int
                                || unsafe { spell_iswordp_w(&word[i - 1..], curwin.get()) }))
                        && (rules[1] != b'$'
                            || !unsafe { spell_iswordp_w(&word[i + k0..], curwin.get()) });
                    let at_word_end = rules[0] == b'$'
                        && i > 0
                        && unsafe { spell_iswordp_w(&word[i - 1..], curwin.get()) }
                        && !unsafe { spell_iswordp_w(&word[i + k0..], curwin.get()) };
                    if !(rules[0] == NUL as u8 || at_word_start || at_word_end) {
                        break 'next_rule;
                    }

                    // A longer rule starting at this match's last
                    // character wins if its priority is at least as high.
                    let c0 = word[i + k - 1];
                    let mut n0 = unsafe { (*slang).sl_sal_first[(c0 & 0xff) as usize] };
                    if unsafe { (*slang).sl_followup }
                        && k > 1
                        && n0 >= 0
                        && p0 != '-' as c_int
                        && word[i + k] != NUL
                    {
                        'followups: loop {
                            // The outer loop stops on the sentinel rule,
                            // whose lead is NUL; this one only compares low
                            // bytes, so a `c0` whose low byte is zero --
                            // U+0100, U+0200, ... -- matches the sentinel
                            // too and walks off the end. The C read past
                            // the array there; this stops.
                            let Some(followup) = smp.get(n0 as usize) else {
                                break 'followups;
                            };
                            let ws = &followup.sm_lead_w;
                            if ws[0] & 0xff != c0 & 0xff {
                                break 'followups;
                            }
                            'next_followup: {
                                if c0 != ws[0] {
                                    break 'next_followup;
                                }
                                k0 = followup.sm_leadlen as usize;
                                if k0 > 1 {
                                    if word[i + k] != ws[1] {
                                        break 'next_followup;
                                    }
                                    if k0 > 2 && word[i + k + 1..i + k + k0 - 1] != ws[2..k0] {
                                        break 'next_followup;
                                    }
                                }
                                k0 += k - 1;

                                if let Some(oneof) = &followup.sm_oneof_w {
                                    if !oneof
                                        .iter()
                                        .take_while(|&&ch| ch != NUL)
                                        .any(|&ch| ch == word[i + k0])
                                    {
                                        break 'next_followup;
                                    }
                                    k0 += 1;
                                }

                                p0 = 5;
                                let mut frules = &followup.sm_rules[..];
                                // "k0" is deliberately not reduced here:
                                // the "k0 == k" test below depends on it.
                                while frules[0] == b'-' {
                                    frules = &frules[1..];
                                }
                                if frules[0] == b'<' {
                                    frules = &frules[1..];
                                }
                                if ascii_isdigit(c_int::from(frules[0])) {
                                    p0 = c_int::from(frules[0]) - '0' as c_int;
                                    frules = &frules[1..];
                                }

                                // A '^' rule never cuts the current match.
                                if frules[0] == NUL as u8
                                    || (frules[0] == b'$'
                                        && !unsafe {
                                            spell_iswordp_w(&word[i + k0..], curwin.get())
                                        })
                                {
                                    // Same length means the follow-up is
                                    // only a piece of this match.
                                    if k0 != k && p0 >= pri {
                                        break 'followups;
                                    }
                                }
                            }
                            n0 += 1;
                        }

                        if p0 >= pri
                            && smp
                                .get(n0 as usize)
                                .is_some_and(|f| f.sm_lead_w[0] & 0xff == c0 & 0xff)
                        {
                            break 'next_rule;
                        }
                    }

                    // The rule applies: substitute "sm_to_w".
                    let to = smp[n as usize].sm_to_w.as_deref();
                    let rules = &smp[n as usize].sm_rules[..];
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
                        k0 = 0;
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
                    break 'rules;
                }
                n += 1;
            }
        } else if ascii_iswhite(c) {
            c = ' ' as c_int;
            k = 1;
        }

        if z0 == 0 {
            if k != 0
                && p0 == 0
                && reslen < MAXWLEN
                && c != NUL
                && (!unsafe { (*slang).sl_collapse } || reslen == 0 || wres[reslen - 1] != c)
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
        l += unsafe { utf_char2bytes(wc, res.offset(l as isize)) };
        if l + MB_MAXBYTES as c_int > MAXWLEN as c_int {
            break;
        }
    }
    unsafe { *res.offset(l as isize) = NUL as c_char };
}
