//! Looking a word up in a language's word tree.
//!
//! This is the innermost loop of spell checking: [`find_word`] runs once
//! per candidate word per language, and again for every prefix and every
//! compound continuation, so it is by some margin the hottest code in the
//! subsystem.
//!
//! # The tree
//!
//! A `.spl` file stores words as a trie flattened into two parallel arrays.
//! `byts` holds, for each node, a count followed by that many child bytes
//! in sorted order; `idxs` holds the matching child node indices. Walking
//! is therefore: read the count, binary-search the bytes for the next
//! character of the word, follow `idxs` to the child.
//!
//! A child byte of zero does not mean a character — it means "a word ends
//! here", and the `idxs` entry beside it holds that word's `WF_*` flags and
//! region mask instead of a node index. There can be several such zero
//! entries in a row, one per flag/region combination the same spelling has.
//!
//! Three trees exist per language, and `mode` says which one to walk:
//! the case-folded tree (`FIND_FOLDWORD`), the keep-case tree for words
//! whose capitalisation cannot be described by a flag (`FIND_KEEPWORD`),
//! and the prefix tree ([`find_prefix`], which walks it and then calls
//! [`find_word`] with `FIND_PREFIX` for the remainder).
//!
//! # Endings
//!
//! The walk goes as deep as it can and *then* works back out, because the
//! longest match wins. Every "word ends here" node passed on the way down
//! is pushed onto [`MatchInf::ends`], and the second half of [`find_word`]
//! pops them longest-first, checking case, region, prefix conditions and
//! compounding rules until one is accepted.
//!
//! # Compounding
//!
//! When a word may be followed by another to form a compound, [`find_word`]
//! recurses on the remainder with `FIND_COMPOUND`. The flags of the parts
//! collected so far live in `mi_compflags`, and [`can_compound`] /
//! [`match_compoundrule`] decide whether that sequence is one the language
//! allows.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::cstr::byte_at;
use crate::mbyte::Codepoint;
use crate::os::cshim::gettext;
use crate::spell::WordFlags;
use core::ffi::c_int;
use core::ops::ControlFlow;

use crate::mbyte::{c_cluster_len, char_count, encode_char, head_off};
use crate::message::emsg;
use crate::regexp::vim_regexec_prog;
use crate::spell::SpellLang;

use super::Tree;
use super::chartab::{
    byte_in_str, captype, casefold_len, nofold_len, spell_casefold, spell_iswordp,
    spell_iswordp_nmw, step, tail,
};
use super::{
    E_FORMAT, FIND_COMPOUND, FIND_FOLDWORD, FIND_KEEPCOMPOUND, FIND_KEEPWORD, FIND_PREFIX, MAXWLEN,
    MatchInf, SP_BAD, SP_BANNED, SP_LOCAL, SP_OK, SP_RARE, SpellResult, WordEnd, c_text,
    count_syllables, spell_valid_case,
};

/// Step `at` past the character it starts in `text`, as the pointer walk's
/// `mb_ptr_adv` did: a NUL, or the end, is stepped over as one byte.
#[inline]
fn advance(text: &[u8], at: usize) -> usize {
    at + step(text, at)
}

/// Translate a length in `from` to the same number of characters in `to`,
/// for the two forms of one word, folded and as written: `from[..len]`'s
/// characters are counted off in `to`, which stops at its NUL as the
/// pointer walk did.
fn translate_len(from: &[u8], len: usize, to: &[u8]) -> usize {
    let (mut s, mut p) = (0, 0);
    while s < len {
        s = advance(from, s);
        p += c_cluster_len(tail(to, p));
    }
    p
}

/// The text a walk in `keep` mode reads -- the word as written -- or else
/// the case-folded word.
macro_rules! text {
    ($mip:expr, $keep:expr) => {
        if $keep { $mip.word } else { &$mip.fword[..] }
    };
}

/// Check whether the word at `mip.word` is in one of `mip`'s language's
/// trees, updating `result` and `end` for a match.
///
/// `mode` picks the tree and says where in the word to start:
///
/// * `FIND_FOLDWORD` — the case-folded tree, from the start.
/// * `FIND_KEEPWORD` — the keep-case tree, from the start.
/// * `FIND_PREFIX` — the case-folded tree, after a prefix [`find_prefix`]
///   already matched.
/// * `FIND_COMPOUND` / `FIND_KEEPCOMPOUND` — either tree, after the
///   compound parts found so far.
pub(super) fn find_word(mip: &mut MatchInf<'_>, mode: c_int) {
    let Some(slang) = mip.slang else {
        return;
    };

    let keep = mode == FIND_KEEPWORD || mode == FIND_KEEPCOMPOUND;
    // How many folded bytes are left ahead of `wlen`: signed, because the
    // walk may step past what has been folded before it folds more.
    let mut flen: isize;
    let tree;
    let mut wlen: usize = 0;
    if keep {
        // The keep-case tree is matched against the word as written, so
        // no folding is needed and there are always enough bytes.
        flen = 9999;
        tree = slang.sl_keep_tree.view();

        if mode == FIND_KEEPCOMPOUND {
            wlen += mip.compoff;
        }
    } else {
        flen = mip.fwordlen.cast_signed();
        tree = slang.sl_fold_tree.view();

        if mode == FIND_PREFIX {
            wlen = mip.prefixlen;
            flen -= mip.prefixlen.cast_signed();
        } else if mode == FIND_COMPOUND {
            wlen = mip.compoff;
            flen -= mip.compoff.cast_signed();
        }
    }

    if tree.is_empty() {
        return; // this language has no such tree
    }

    // Where each "a word could end here" node was, and how far into the
    // word it sat: pushed above whatever the calls this one is nested in
    // left on the stack, and popped back to `base` before returning.
    let base = mip.ends.len();
    if !push_word_ends(mip, tree, keep, wlen, flen) {
        return;
    }

    // Now try the endings, longest first.
    let win = mip.win;
    while mip.ends.len() > base {
        let WordEnd { len: end_len, node } = mip.ends.pop().expect("above base");
        let mut arridx = node;
        wlen = end_len;

        if head_off(text!(mip, keep), wlen) > 0 {
            continue; // not on a character boundary
        }

        // A word character right after the match means this is not the
        // end of a word — unless the language compounds or does not
        // break, in which case what follows may continue it.
        let word_ends = if spell_iswordp(tail(text!(mip, keep), wlen), win) {
            if slang.sl_compprog.borrow().is_none() && !slang.sl_nobreak {
                continue;
            }
            false
        } else {
            true
        };

        // The prefix flag comes before the compound flags; once a valid
        // prefix has been found the rest are tried as compound flags.
        let mut prefix_found = false;

        if mode != FIND_KEEPWORD {
            // Translate the length back into the unfolded word, since
            // folding can change how many bytes a character takes. The
            // comparison is a shortcut for the common case where it did
            // not change anything.
            let ptr = text!(mip, keep);
            if !cstr::ncmp_eq(ptr, mip.word, wlen) {
                wlen = translate_len(ptr, wlen, mip.word);
            }
        }

        // Try each flag/region variant recorded for this spelling, until
        // one settles this ending.
        let ending = Ending {
            mode,
            keep,
            wlen,
            at: end_len,
            word_ends,
        };
        let mut len = tree.node_len(arridx - 1);
        while len > 0 && tree.ends_word(arridx) {
            let flags = WordFlags::from_bits(tree.idx(arridx));
            if try_variant(mip, slang, &ending, flags, &mut prefix_found).is_break() {
                break;
            }
            len -= 1;
            arridx += 1;
        }

        if mip.result == SP_OK {
            mip.ends.truncate(base);
            break;
        }
    }
}

/// Descend `tree` along the word from `wlen` (with `flen` folded bytes
/// ahead of it) until a byte does not match, the tree runs out, or the
/// word does, pushing every "a word could end here" node passed onto
/// [`MatchInf::ends`].
///
/// Answers `false` when the tree nests deeper than any word, which only a
/// corrupted spell file can do: the error is reported and the stack is
/// back where it was.
fn push_word_ends(
    mip: &mut MatchInf<'_>,
    tree: Tree<'_>,
    keep: bool,
    mut wlen: usize,
    mut flen: isize,
) -> bool {
    let base = mip.ends.len();
    let mut arridx: usize = 0;
    // `ptr` is re-read whenever folding has added to it.
    let mut ptr = text!(mip, keep);
    loop {
        if flen <= 0 && byte_at(mip.word, mip.fend) != 0 {
            flen = fold_more(mip).cast_signed();
            ptr = text!(mip, keep);
        }

        // The whole node at once: one bounds check for every read below.
        let (children, links) = tree.node(arridx);
        let mut first = 0;

        // A leading zero byte means a word ends here. Remember the spot
        // and carry on; the longest match is preferred, so the endings
        // are only examined once the descent is done.
        if children.first() == Some(&0) {
            if mip.ends.len() - base == MAXWLEN {
                // Only a corrupted spell file can nest this deep.
                emsg(gettext(E_FORMAT));
                mip.ends.truncate(base);
                return false;
            }
            mip.ends.push(WordEnd {
                len: wlen,
                node: arridx + 1,
            });

            // Step over every zero: one per flag/region variant.
            first = Tree::word_ends_in(children);
            if first == children.len() {
                break; // no children, the word must end here
            }
        }

        let mut c = byte_at(ptr, wlen);
        if c == 0 {
            break; // end of the line
        }
        if c == b'\t' {
            c = b' '; // a tab counts as a space
        }

        let Some(at) = Tree::child_in(children, first, c) else {
            break; // no matching byte
        };

        arridx = usize::try_from(links[at]).unwrap_or(0);
        wlen += 1;
        flen -= 1;

        // One space in the dictionary word may stand for a run of
        // spaces and tabs in the text.
        if c == b' ' {
            loop {
                if flen <= 0 && byte_at(mip.word, mip.fend) != 0 {
                    flen = fold_more(mip).cast_signed();
                    ptr = text!(mip, keep);
                }
                let ch = byte_at(ptr, wlen);
                if ch != b' ' && ch != b'\t' {
                    break;
                }
                wlen += 1;
                flen -= 1;
            }
        }
    }
    true
}

/// One ending [`find_word`] is trying: the walk's `mode` and text, the
/// match's length in the word as written (`wlen`) and in the text the walk
/// read (`at`), and whether a word character follows it.
struct Ending {
    mode: c_int,
    keep: bool,
    wlen: usize,
    at: usize,
    word_ends: bool,
}

/// Try one flag/region entry, `flags`, of an ending: check case, prefix
/// and compound conditions, look for the word that must follow, and keep
/// the result if it beats `mip`'s. Breaks when the ending is settled (no
/// later entry can do better), continues on to the next entry otherwise.
fn try_variant(
    mip: &mut MatchInf<'_>,
    slang: &SpellLang,
    ending: &Ending,
    mut flags: WordFlags,
    prefix_found: &mut bool,
) -> ControlFlow<()> {
    let &Ending {
        mode,
        keep,
        wlen,
        at: end_len,
        word_ends,
    } = ending;
    if mode == FIND_FOLDWORD {
        // The fold-case tree records what case the word must be written
        // in; the keep-case tree is right by construction, and prefixes
        // are not worth checking.
        if mip.cend != Some(wlen) {
            mip.cend = Some(wlen);
            mip.capflags = captype(mip.word, wlen);
        }

        if mip.capflags == WordFlags::KEEPCAP || !spell_valid_case(mip.capflags, flags) {
            return ControlFlow::Continue(());
        }
    } else if mode == FIND_PREFIX && !*prefix_found {
        // The word has to accept one of the prefixes find_prefix() left
        // at prefarridx.
        let (cnt, arridx) = (mip.prefcnt, mip.prefarridx);
        let after = tail(mip.word, mip.cprefixlen);
        let c = valid_word_prefix(cnt, arridx, flags, after, slang, false);
        if c == 0 {
            return ControlFlow::Continue(());
        }
        if WordFlags::from_bits(c).has(WordFlags::RAREPFX) {
            flags |= WordFlags::RARE;
        }
        *prefix_found = true;
    }

    if slang.sl_nobreak {
        if (mode == FIND_COMPOUND || mode == FIND_KEEPCOMPOUND) && !flags.has(WordFlags::BANNED) {
            // NOBREAK: a valid word follows, which is all the caller
            // wanted to know.
            mip.result = SP_OK;
            return ControlFlow::Break(());
        }
    } else if mode == FIND_COMPOUND || mode == FIND_KEEPCOMPOUND || !word_ends {
        if !compound_part_allowed(mip, keep, wlen, flags, word_ends, mode, end_len) {
            return ControlFlow::Continue(());
        }
    } else if flags.has(WordFlags::NEEDCOMP) {
        // Only valid as part of a compound.
        return ControlFlow::Continue(());
    }

    let mut nobreak_result = SP_OK;

    if !word_ends {
        let following = find_following(mip, slang, mode, wlen, end_len, flags);
        if slang.sl_nobreak {
            nobreak_result = following;
        } else if mip.result == SP_OK {
            return ControlFlow::Break(());
        } else {
            return ControlFlow::Continue(());
        }
    }

    let res = if flags.has(WordFlags::BANNED) {
        SP_BANNED
    } else if flags.has(WordFlags::REGION) {
        let region = flags.bits().cast_unsigned() >> 16;
        if mip.lp.region.cast_unsigned() & region != 0 {
            SP_OK
        } else {
            SP_LOCAL
        }
    } else if flags.has(WordFlags::RARE) {
        SP_RARE
    } else {
        SP_OK
    };

    // Keep the longest match with the best result. NOBREAK keeps the
    // longest match *without* a following good word separately, as a
    // fall-back.
    let end = wlen;
    if nobreak_result == SP_BAD {
        if mip.result2 > res {
            mip.result2 = res;
            mip.end2 = end;
        } else if mip.result2 == res && mip.end2 < end {
            mip.end2 = end;
        }
    } else if mip.result > res {
        mip.result = res;
        mip.end = end;
    } else if mip.result == res && mip.end < end {
        mip.end = end;
    }

    if mip.result == SP_OK {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}

/// Look for the word that has to follow a match that is not the end of a
/// word: the next compound part, or for NOBREAK any good word, starting at
/// `end_len` in the walk's text (`wlen` as written).
///
/// When compounding, the recursion sets `mip`'s result itself, and that
/// is the answer. For NOBREAK only whether a word follows matters: `mip`'s
/// result and end are put back and what the recursion found is answered.
fn find_following(
    mip: &mut MatchInf<'_>,
    slang: &SpellLang,
    mode: c_int,
    wlen: usize,
    end_len: usize,
    flags: WordFlags,
) -> SpellResult {
    let save_result = mip.result;
    let save_end = mip.end;
    let save_lp = (mip.lp, mip.slang);

    if slang.sl_nobreak {
        mip.result = SP_BAD;
    }

    mip.compoff = end_len;
    if mode == FIND_KEEPWORD {
        // Translate the keep-case length into a case-folded one, again
        // short-cutting when folding changed nothing.
        if !cstr::ncmp_eq(mip.word, &mip.fword, wlen) {
            let to = &mip.fword[..];
            mip.compoff = translate_len(mip.word, wlen, to);
        }
    }
    mip.complen += 1;
    if flags.has(WordFlags::COMPROOT) {
        mip.compextra += 1;
    }

    // For NOBREAK every language has to be tried, if only to reach the
    // ".add" files.
    let (langs, langp) = (mip.langs, mip.langp);
    for lp in langp {
        if slang.sl_nobreak {
            mip.lp = *lp;
            mip.slang = langs.get(lp.slang);
            if !mip
                .slang
                .is_some_and(|next| !next.sl_fold_tree.is_empty() && next.sl_nobreak)
            {
                continue;
            }
        }

        find_word(mip, FIND_COMPOUND);

        // Under NOBREAK any match will do; otherwise the longest one is
        // wanted, so the keep-case tree is tried as well.
        if !slang.sl_nobreak || mip.result == SP_BAD {
            mip.compoff = wlen;
            find_word(mip, FIND_KEEPCOMPOUND);
        }

        if !slang.sl_nobreak {
            break;
        }
    }
    mip.complen -= 1;
    if flags.has(WordFlags::COMPROOT) {
        mip.compextra -= 1;
    }
    (mip.lp, mip.slang) = save_lp;

    let found = mip.result;
    if slang.sl_nobreak {
        mip.result = save_result;
        mip.end = save_end;
    }
    found
}

/// Whether this word may be one part of a compound, given the flags the
/// tree recorded for it and the parts collected in `compflags` so far.
///
/// Split out of [`find_word`] only to keep that function readable; it is
/// the body of its `FIND_COMPOUND` arm and updates `compflags` on the way
/// through. `keep` says which text the walk reads: the word as written, or
/// the folded word.
#[inline]
fn compound_part_allowed(
    mip: &mut MatchInf<'_>,
    keep: bool,
    wlen: usize,
    flags: WordFlags,
    word_ends: bool,
    mode: c_int,
    endlen: usize,
) -> bool {
    let Some(slang) = mip.slang else {
        return false;
    };
    // No compound flag, or shorter than COMPOUNDMIN, rejects quickly.
    // (Myspell compatibility requires accepting a compound flag on a
    // word that is too short to use it.)
    let compflag = compound_flag(flags);
    // A part that ends before it starts is as short as one can be.
    let minlen = usize::try_from(slang.sl_compminlen).unwrap_or(0);
    let Some(part_len) = wlen.checked_sub(mip.compoff) else {
        return false;
    };
    if compflag == 0 || part_len < minlen {
        return false;
    }
    // COMPOUNDMIN counts characters, not bytes.
    if minlen > 0 {
        let part = tail(mip.word, mip.compoff);
        let part = &part[..part.len().min(part_len)];
        if char_count(cstr::until_nul(part)) < minlen {
            return false;
        }
    }

    // COMPOUNDWORDMAX caps the number of parts, unless a syllable
    // maximum was given instead.
    if !word_ends
        && usize::try_from(slang.sl_compmax).is_ok_and(|max| mip.complen + mip.compextra + 2 > max)
        && usize::try_from(slang.sl_compsylmax) == Ok(MAXWLEN)
    {
        return false;
    }

    // Compounding is not allowed on a side where an affix was added,
    // unless COMPOUNDPERMITFLAG said so.
    if mip.complen > 0 && flags.has(WordFlags::NOCOMPBEF) {
        return false;
    }
    if !word_ends && flags.has(WordFlags::NOCOMPAFT) {
        return false;
    }

    // Is this flag usable in this position at all?
    let usable = if mip.complen == 0 {
        &slang.sl_compstartflags
    } else {
        &slang.sl_compallflags
    };
    if !byte_in_str(usable, c_int::from(compflag)) {
        return false;
    }

    if match_checkcompoundpattern(text!(mip, keep), wlen, &slang.sl_comppat) {
        return false;
    }

    if mode == FIND_COMPOUND {
        // Check the capitalisation of the part being appended.
        let word = text!(mip, keep);
        let compoff = mip.compoff;
        let mut p = if !cstr::ncmp_eq(word, mip.word, compoff) {
            // Folding changed the length.
            translate_len(word, compoff, mip.word)
        } else {
            compoff
        };
        let capflags = captype(tail(mip.word, p), wlen - p.min(wlen));
        if capflags == WordFlags::KEEPCAP
            || (capflags == WordFlags::ALLCAP && flags.has(WordFlags::FIXCAP))
        {
            return false;
        }

        if capflags != WordFlags::ALLCAP {
            // A Onecap part is not accepted after a word character. A
            // no-caps part is accepted even where the dictionary word
            // says ONECAP.
            if p > 0 {
                p -= head_off(mip.word, p - 1) + 1;
            }
            let reject = if spell_iswordp_nmw(tail(mip.word, p), mip.win) {
                capflags == WordFlags::ONECAP
            } else {
                flags.has(WordFlags::ONECAP) && capflags != WordFlags::ONECAP
            };
            if reject {
                return false;
            }
        }
    }

    // Record this part's flag, then check the sequence: against
    // COMPOUNDRULE for a complete word, or against the rules' prefixes
    // for a word still being built.
    let complen = mip.complen;
    mip.compflags[complen] = compflag;
    if let Some(after) = mip.compflags.get_mut(complen + 1) {
        *after = 0;
    }
    if word_ends {
        let mut fword = [0u8; MAXWLEN];

        if counts_syllables(slang) {
            // Only syllable counting needs the word itself.
            if keep {
                let _ = spell_casefold(mip.win, mip.word, wlen, &mut fword);
            } else {
                let n = endlen.min(MAXWLEN - 1);
                fword[..n].copy_from_slice(&mip.fword[..n]);
                fword[n] = 0;
            }
        }
        if !can_compound(slang, &fword, &mip.compflags) {
            return false;
        }
    } else if slang.sl_comprules.is_some() && !match_compoundrule(slang, &mip.compflags) {
        return false;
    }

    true
}

/// The compound flag a word's flags carry, in their top byte; 0 for none.
pub(crate) fn compound_flag(flags: WordFlags) -> u8 {
    flags.bits().cast_unsigned().to_be_bytes()[0]
}

/// Whether the language gave a COMPOUNDSYLMAX: a syllable maximum below
/// `MAXWLEN`, the default that means none.
fn counts_syllables(slang: &SpellLang) -> bool {
    usize::try_from(slang.sl_compsylmax).map_or(true, |max| max < MAXWLEN)
}

/// Whether joining another word onto `word[..wlen]` would hit a
/// CHECKCOMPOUNDPATTERN rule, which forbids the join.
///
/// A rule is a pair: the first part has to match at the end of the word so
/// far, the second at the start of what follows. `word` runs on past `wlen`
/// to the end of its string.
pub fn match_checkcompoundpattern(word: &[u8], wlen: usize, pats: &[Box<[u8]>]) -> bool {
    for pair in pats.as_chunks::<2>().0 {
        let (first, second) = (&pair[0], &pair[1]);
        let len = first.len();
        // The length test is what keeps the first compare inside the word
        // so far.
        if len <= wlen
            && starts_with_c(tail(word, wlen), second)
            && cstr::ncmp_eq(tail(word, wlen - len), first, len)
        {
            return true;
        }
    }
    false
}

/// Whether the C string `text` starts with `prefix`.
fn starts_with_c(text: &[u8], prefix: &[u8]) -> bool {
    prefix
        .iter()
        .enumerate()
        .all(|(i, &b)| byte_at(text, i) == b)
}

/// Whether `flags` is a sequence of compound flags the language allows and
/// `word` does not have too many syllables.
///
/// The COMPOUNDRULE patterns are compiled into one regexp at load time, so
/// the check is: widen the one-byte flags to characters and match. Syllable
/// counting is much slower, so it is left until last, and a word over
/// COMPOUNDSYLMAX is still accepted while it has fewer parts than
/// COMPOUNDWORDMAX. `word` and `flags` end at a NUL or at their end.
pub fn can_compound(slang: &SpellLang, word: &[u8], flags: &[u8]) -> bool {
    let mut compprog = slang.sl_compprog.borrow_mut();
    if compprog.is_none() {
        return false;
    }

    let flags = cstr::until_nul(flags);
    let mut uflags = [0u8; MAXWLEN * 2];
    let mut p = 0;
    for &flag in flags {
        p += encode_char(Codepoint::from(flag), &mut uflags[p..]);
    }
    uflags[p] = 0;

    if !vim_regexec_prog(&mut compprog, false, cstr::in_bytes(&uflags), 0) {
        return false;
    }
    drop(compprog);

    if counts_syllables(slang) && count_syllables(slang, word) > slang.sl_compsylmax {
        return usize::try_from(slang.sl_compmax).is_ok_and(|max| flags.len() < max);
    }
    true
}

/// Whether the compound flags collected so far are a prefix of any
/// COMPOUNDRULE, so that it is still worth extending the compound.
///
/// A rule is a sequence of flags, `[abc]` standing for any one of them,
/// with `/` separating rules. A language without the rules has none to
/// match. `compflags` ends at a NUL.
pub fn match_compoundrule(slang: &SpellLang, compflags: &[u8]) -> bool {
    let Some(rules) = &slang.sl_comprules else {
        return false;
    };
    let rule = |at: usize| byte_at(rules, at);
    // Loop over all the COMPOUNDRULE entries.
    let mut p = 0;
    while rule(p) != 0 {
        // Loop over the flags in the compound word made so far, matching
        // them against the current rule entry.
        let mut i = 0;
        loop {
            let c = byte_at(compflags, i);
            if c == 0 {
                // Every flag so far matched, and the rule has more to
                // give: this compound can still work out.
                return true;
            }
            if rule(p) == b'/' || rule(p) == 0 {
                break; // the rule ran out first
            }
            if rule(p) == b'[' {
                let mut matched = false;
                p += 1;
                while rule(p) != b']' && rule(p) != 0 {
                    if rule(p) == c {
                        matched = true;
                    }
                    p += 1;
                }
                if !matched {
                    break;
                }
            } else if rule(p) != c {
                break;
            }
            p += 1;
            i += 1;
        }

        // On to the next rule, past its `/`.
        match rules
            .get(p..)
            .and_then(|rest| rest.iter().position(|&b| b == b'/'))
        {
            Some(slash) => p += slash + 1,
            None => break,
        }
    }
    false
}

/// Whether one of the `totprefcnt` prefixes listed at `arridx` in the
/// prefix tree may be used with the word `word` carrying `flags`.
///
/// Returns the prefix's own flags (non-zero) on a match, including
/// `WordFlags::RAREPFX` when the prefix is rare, or zero when none applies.
///
/// A prefix entry packs its ID in the low byte and the index of its
/// condition regexp in the two bytes above. `word` ends at a NUL or at its
/// end.
pub fn valid_word_prefix(
    totprefcnt: usize,
    arridx: usize,
    flags: WordFlags,
    word: &[u8],
    slang: &SpellLang,
    cond_req: bool,
) -> c_int {
    let tree = slang.sl_prefix_tree.view();
    // The prefix ID shares the top byte with the compound flag.
    let prefid = c_int::from(compound_flag(flags));
    for prefcnt in (0..totprefcnt).rev() {
        let pidx = tree.idx(arridx + prefcnt);

        if prefid != pidx & 0xff {
            continue;
        }

        // A non-combining prefix cannot go on a word that already has a
        // suffix.
        if flags.has(WordFlags::HAS_AFF) && WordFlags::from_bits(pidx).has(WordFlags::PFX_NC) {
            continue;
        }

        let prefprog_idx = ((pidx.cast_unsigned() >> 8) & 0xffff) as usize;
        // An index past the language's conditions (a damaged file) reads as
        // no condition.
        let condition = slang.sl_prefprog.get(prefprog_idx);
        if let Some(rp) = condition.filter(|rp| rp.borrow().is_some()) {
            if !vim_regexec_prog(&mut rp.borrow_mut(), false, &c_text(word), 0) {
                continue;
            }
        } else if cond_req {
            continue;
        }

        return pidx;
    }
    0
}

/// Check whether the word at `mip.word` starts with a known prefix, and if
/// so look the remainder up with [`find_word`].
///
/// `FIND_COMPOUND` does the same after the compound parts found so far.
pub(super) fn find_prefix(mip: &mut MatchInf<'_>, mode: c_int) {
    let Some(slang) = mip.slang else {
        return;
    };
    let tree = slang.sl_prefix_tree.view();
    if tree.is_empty() {
        return; // this language has no prefixes
    }

    // Prefixes are always stored case-folded.
    let mut start = 0;
    // Signed, as in `find_word`.
    let mut flen = mip.fwordlen.cast_signed();
    if mode == FIND_COMPOUND {
        start = mip.compoff;
        flen -= mip.compoff.cast_signed();
    }

    let mut arridx: usize = 0;
    let mut wlen = 0;
    loop {
        if flen == 0 && byte_at(mip.word, mip.fend) != 0 {
            flen = fold_more(mip).cast_signed();
        }

        let mut len = tree.node_len(arridx);
        arridx += 1;

        // A leading zero byte means a prefix ends here. Several prefixes
        // can share a spelling with different conditions, and which one
        // gives the longest match is not known yet, so find_word() gets
        // the whole list to try.
        if tree.ends_word(arridx) {
            mip.prefarridx = arridx;
            let ends = tree.word_ends(arridx, len);
            arridx += ends;
            len -= ends;
            mip.prefcnt = ends;

            mip.prefixlen = wlen;
            if mode == FIND_COMPOUND {
                mip.prefixlen += mip.compoff;
            }

            // The folded length may differ from the original.
            mip.cprefixlen = nofold_len(&mip.fword, mip.prefixlen, mip.word);
            find_word(mip, FIND_PREFIX);

            if len == 0 {
                break; // no children, the prefix must end here
            }
        }

        let c = byte_at(&mip.fword, start + wlen);
        if c == 0 {
            break; // end of the line
        }

        let Some(at) = tree.child(arridx, len, c) else {
            break;
        };

        arridx = tree.child_node(at);
        wlen += 1;
        flen -= 1;
    }
}

/// Fold one more character of the word being checked into `fword`, and
/// return how many bytes that added.
///
/// Folding runs to the next non-word character rather than one character at
/// a time, and includes that character, so that the caller can see where
/// the word ends.
fn fold_more(mip: &mut MatchInf<'_>) -> usize {
    let p = mip.fend;
    let win = mip.win;
    let word = mip.word;
    loop {
        mip.fend = advance(word, mip.fend);
        if byte_at(word, mip.fend) == 0 || !spell_iswordp(tail(word, mip.fend), win) {
            break;
        }
    }

    // Include the non-word character, so the word end is visible.
    if byte_at(word, mip.fend) != 0 {
        mip.fend = advance(word, mip.fend);
    }

    let fwordlen = mip.fwordlen;
    let taken = mip.fend - p;
    let folded = casefold_len(win, tail(word, p), taken, &mut mip.fword[fwordlen..MAXWLEN]);
    let flen = folded.unwrap_or_else(|written| written);
    mip.fwordlen += flen;
    flen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::XString;

    fn rules(text: &[u8]) -> Box<SpellLang> {
        let mut slang = SpellLang::new(b"", XString::new());
        slang.sl_comprules = Some(text.to_vec());
        slang
    }

    #[test]
    fn a_compound_is_a_prefix_of_some_rule() {
        let slang = rules(b"ab/cd");
        assert!(match_compoundrule(&slang, b"a\0"));
        assert!(match_compoundrule(&slang, b"c\0"));
        assert!(!match_compoundrule(&slang, b"x\0"));
        let sets = rules(b"[ab]c");
        assert!(match_compoundrule(&sets, b"bc\0"));
        assert!(!match_compoundrule(&sets, b"cc\0"));
    }

    #[test]
    fn a_compound_longer_than_every_rule_is_no_prefix() {
        // The first rule runs out on its `/`; the scan has to step past it
        // to the next rule rather than stop on it for good.
        let slang = rules(b"ab/cd");
        assert!(!match_compoundrule(&slang, b"abc\0"));
        assert!(!match_compoundrule(&slang, b"cdx\0"));
    }

    #[test]
    fn a_language_without_rules_matches_nothing() {
        let mut slang = rules(b"");
        slang.sl_comprules = None;
        assert!(!match_compoundrule(&slang, b"a\0"));
    }
}
