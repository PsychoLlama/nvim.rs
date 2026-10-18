//! Deciding whether one word is spelled correctly.
//!
//! [`spell_check`] is the entry point everything else funnels through: the
//! screen drawing code calls it for every word on every redrawn line, `]s`
//! calls it walking the buffer, and `spellbadword()` calls it once. It
//! takes a pointer at what might be the start of a word, works out where
//! that word ends, and hands the result to [`find_word`] once per language
//! in `'spelllang'` — all of them, because a longer match in a later
//! language wins.
//!
//! The return value is always the number of bytes to skip, whether the
//! word was good or bad, so a caller can walk a line by repeatedly adding
//! it. Badness comes back through `attrp` as the highlight to use.
//!
//! Two things ride along with the check because they need the same word
//! boundaries:
//!
//! * `'spelloptions'` `camel` splits `camelCaseWords` into their parts,
//!   which [`advance_camelcase_word`] does by watching for a change in
//!   character type.
//! * `'spellcapcheck'` wants the first word of a sentence capitalised;
//!   `capcol` carries the column where the next such check is due, and
//!   [`check_need_cap`] answers the question for a given position.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr::byte_at;
use crate::memline::Lines;
use crate::memory::XString;
use crate::spell::WordFlags;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::charset::skip;
use crate::global_cell::GlobalCell;
use crate::mbyte::{c_cluster_len, char_at, head_off, mb_isupper};
use crate::message::e_no_spell;
use crate::message::emsg;
use crate::options::kOptSpoFlagCamel;
use crate::os::cshim::gettext;
use crate::regexp::vim_regexec;
use crate::spellsuggest::spell_suggest_list;
use crate::types::{ColNr, Hlf, LangId, LineNr, RegMatch};

use super::chartab::{spell_casefold, spell_iswordp, spell_iswordp_nmw, tail};
use super::lookup::{find_prefix, find_word};
use super::slang::LoadedLangs;
use super::{
    CHAR_DIGIT, CHAR_OTHER, CHAR_UPPER, FIND_COMPOUND, FIND_FOLDWORD, FIND_KEEPWORD, MAXWLEN,
    MatchInf, SP_BAD, SP_BANNED, SP_OK, SP_RARE, c_text, count_common_word, loaded_langs,
    spelltab_isu,
};
use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::optionstr::OptString;

/// Whether `c` is upper case, by the spell table below 128 and the general
/// rules above it.
fn is_upper(c: c_int) -> bool {
    if c >= 128 {
        mb_isupper(c)
    } else {
        spelltab_isu(c as usize)
    }
}

/// Check the word `text` starts with in window `window`.
///
/// `text` runs from the word to the end of its string; its end, or a NUL
/// in it, is the end of the text.
///
/// Answers the length of the word in bytes, good or bad, so the caller can
/// skip over it, and the highlight for a badly spelled word -- `HLF_COUNT`
/// for one spelled right. `capcol`, when given, is the column at which to
/// check for a capital after a sentence end; it is set to the next such
/// column, or -1 when no sentence end was found. `docount` asks for the
/// word to be counted towards the `COMMON` statistics.
///
/// Must only be called with a non-empty `'spelllang'`.
pub fn spell_check(
    window: Win,
    text: &[u8],
    mut capcol: Option<&mut c_int>,
    docount: bool,
) -> (usize, Hlf) {
    // A word never starts at a space or a control character.
    if byte_at(text, 0) <= b' ' {
        return (1, HLF_COUNT);
    }
    // Loading the language files failed.
    if window.syn_block().b_langp.is_empty() {
        return (1, HLF_COUNT);
    }

    // The languages are resolved once for the whole lookup.
    let mut counted = None;
    let checked = loaded_langs.with(|langs| {
        look_up(
            window,
            text,
            langs,
            capcol.as_deref_mut(),
            docount,
            &mut counted,
        )
    });
    if let Some((id, len)) = counted {
        // Counted once the lookup is over, which only reads the language.
        loaded_langs.with_mut(|langs| {
            if let Some(slang) = langs.get_mut(id) {
                count_common_word(slang, &text[..len.min(text.len())], 1);
            }
        });
    }
    if let Some(checked) = checked {
        return checked;
    }

    // Sitting on a non-word character is not an error; step over it and
    // look for a word after it.
    if let Some(capcol) = capcol
        && window.syn_block().b_cap_prog.is_some()
    {
        // Did a sentence end here?
        let mut block = window;
        let mut regmatch = RegMatch::new(block.syn_block_mut().b_cap_prog.take(), false);
        let r = vim_regexec(&mut regmatch, &c_text(text), 0);
        block.syn_block_mut().b_cap_prog = regmatch.regprog.take();
        if let Some(end) = regmatch.ends[0]
            && r
        {
            *capcol = end as c_int;
        }
    }
    (c_cluster_len(text), HLF_COUNT)
}

/// The lookup half of [`spell_check`], over the language table `langs`:
/// the word's length and highlight, or `None` when `text` starts with a
/// character that is not part of a word. A word to count towards
/// `COMMON` is left in `counted`, as the language and the word's length.
fn look_up(
    window: Win,
    text: &[u8],
    langs: &LoadedLangs,
    capcol: Option<&mut c_int>,
    docount: bool,
    counted: &mut Option<(LangId, usize)>,
) -> Option<(usize, Hlf)> {
    let syn = window.syn_block();
    let langp = &syn.b_langp[..];
    let mut nrlen = 0usize; // a number came first
    let mut wrongcaplen = 0usize;
    let mut count_word = docount;
    let use_camel_case = syn.b_p_spo_flags & kOptSpoFlagCamel != 0;
    let mut is_camel_case = false;

    // Nearly everything lives in "mi" so that it can be handed to the
    // lookup functions in one go.
    let mut mi = MatchInf::new(text, window, langs, langp);

    // A number is always fine, including hex and binary literals. The
    // word is still checked, so that "3GPP" and "11 julifeest" are
    // caught.
    let first = text[0];
    if first.is_ascii_digit() {
        let second = byte_at(text, 1);
        mi.end = if first == b'0' && matches!(second, b'b' | b'B') {
            2 + skip::bin(tail(text, 2))
        } else if first == b'0' && matches!(second, b'x' | b'X') {
            2 + skip::hex(tail(text, 2))
        } else {
            skip::digits(text)
        };
        nrlen = mi.end;
    }

    // Find the end of the word: the next non-word character.
    if spell_iswordp(tail(text, mi.fend), window) {
        if use_camel_case {
            mi.fend = advance_camelcase_word(text, window, &mut is_camel_case);
        } else {
            loop {
                mi.fend += c_cluster_len(tail(text, mi.fend)).max(1);
                if byte_at(text, mi.fend) == 0 || !spell_iswordp(tail(text, mi.fend), window) {
                    break;
                }
            }
        }

        if capcol.as_deref() == Some(&0) && syn.b_cap_prog.is_some() {
            // This word should have started with a capital.
            if !is_upper(char_at(text)) {
                wrongcaplen = mi.fend;
            }
        }
    }
    if let Some(capcol) = capcol {
        *capcol = -1;
    }

    // Characters up to the next non-word character are consumed even
    // for a bad word.
    mi.end = mi.fend;

    // The caps type is worked out later, on demand.
    mi.capflags = WordFlags::NONE;
    mi.cend = None;

    // Fold one character past the word, so the lookup can see where the
    // word ends.
    if byte_at(text, mi.fend) != 0 {
        mi.fend += c_cluster_len(tail(text, mi.fend));
    }
    let _ = spell_casefold(window, text, mi.fend, &mut mi.fword[..MAXWLEN + 1]);
    mi.fwordlen = crate::cstr::until_nul(&mi.fword).len() as c_int;

    if is_camel_case && mi.fwordlen > 0 {
        // Put a fake word end into the folded word.
        mi.fword[(mi.fwordlen - 1) as usize] = b' ';
    }

    // Bad until recognised.
    mi.result = SP_BAD;
    mi.result2 = SP_BAD;

    // Every language is tried, because a later one may match longer.
    for lp in langp {
        mi.lp = *lp;
        mi.slang = langs.get(lp.slang);

        // A language whose reload failed stays in the list with
        // everything cleared out.
        let Some(slang) = mi.slang.filter(|slang| !slang.sl_fold_tree.is_empty()) else {
            continue;
        };

        find_word(&mut mi, FIND_FOLDWORD);
        find_word(&mut mi, FIND_KEEPWORD);
        find_prefix(&mut mi, FIND_FOLDWORD);

        // A NOBREAK language may fall back on a word with nothing valid
        // after it.
        if slang.sl_nobreak && mi.result == SP_BAD && mi.result2 != SP_BAD {
            mi.result = mi.result2;
            mi.end = mi.end2;
        }

        // Count the word in the first language that accepts it.
        if count_word && mi.result == SP_OK {
            *counted = Some((lp.slang, mi.end));
            count_word = false;
        }
    }

    let mut attr = HLF_COUNT;
    if mi.result != SP_OK {
        let first_lang = langs.get(langp[0].slang);
        if nrlen > 0 {
            // Skip over a leading number, so that "42nd" works. Rare and
            // local words are still flagged, for "3GPP".
            if mi.result == SP_BAD || mi.result == SP_BANNED {
                return Some((nrlen, HLF_COUNT));
            }
        } else if !spell_iswordp_nmw(text, window) {
            return None;
        } else if mi.end == 0 {
            // Always consume at least one character, in case 'midword'
            // left the word empty.
            mi.end = c_cluster_len(text);
        } else if mi.result == SP_BAD && first_lang.is_some_and(|slang| slang.sl_nobreak) {
            // The first language is NOBREAK: find the first position at
            // which any word would be valid.
            let save_result = mi.result;
            mi.lp = langp[0];
            mi.slang = first_lang;
            if first_lang.is_some_and(|slang| !slang.sl_fold_tree.is_empty()) {
                let mut p = 0;
                let mut fp = 0;
                loop {
                    p += c_cluster_len(tail(text, p));
                    fp += c_cluster_len(tail(&mi.fword, fp));
                    if p >= mi.end {
                        break;
                    }
                    mi.compoff = fp as c_int;
                    find_word(&mut mi, FIND_COMPOUND);
                    if mi.result != SP_BAD {
                        mi.end = p;
                        break;
                    }
                }
                mi.result = save_result;
            }
        }

        attr = if mi.result == SP_BAD || mi.result == SP_BANNED {
            HLF_SPB
        } else if mi.result == SP_RARE {
            HLF_SPR
        } else {
            HLF_SPL
        };
    }

    if wrongcaplen > 0 && (mi.result == SP_OK || mi.result == SP_RARE) {
        // SpellCap is only reported when the word itself is fine.
        return Some((wrongcaplen, HLF_SPC));
    }

    Some((mi.end, attr))
}

/// Classify `c` for the camel-case split.
fn get_char_type(c: c_int) -> c_int {
    if crate::ascii::ascii_isdigit(c) {
        return CHAR_DIGIT;
    }
    if is_upper(c) {
        return CHAR_UPPER;
    }
    CHAR_OTHER
}

/// The end of the word `text` starts with, splitting camel-case words into
/// their parts, as an offset into `text`.
///
/// A split happens where the character type changes in a way that only a
/// word boundary explains: `fooBar`, `fooA1`, `1a`, and — looking two
/// characters back — `HTTPServer`, which splits before the `S` rather than
/// after it.
fn advance_camelcase_word(text: &[u8], window: Win, is_camel_case: &mut bool) -> usize {
    *is_camel_case = false;
    if byte_at(text, 0) == 0 {
        return 0;
    }

    let c = char_at(text);
    let mut end = c_cluster_len(text);

    // Only the last two characters' types are ever needed.
    let mut last_last_type = -1;
    let mut last_type = get_char_type(c);

    while byte_at(text, end) != 0 && spell_iswordp(tail(text, end), window) {
        let this_type = get_char_type(char_at(tail(text, end)));

        if last_last_type == CHAR_UPPER && last_type == CHAR_UPPER && this_type == CHAR_OTHER {
            // UpperUpperLower: the word ends one character back.
            *is_camel_case = true;
            end -= head_off(text, end - 1) + 1;
            break;
        } else if (this_type == CHAR_UPPER && last_type == CHAR_OTHER)
            || (this_type != last_type && (this_type == CHAR_DIGIT || last_type == CHAR_DIGIT))
        {
            // LowerUpper LowerDigit UpperDigit DigitUpper DigitLower
            *is_camel_case = true;
            break;
        }

        last_last_type = last_type;
        last_type = this_type;

        end += c_cluster_len(tail(text, end)).max(1);
    }

    end
}

/// Whether a word written with `wordflags` capitalisation satisfies a tree
/// entry recorded with `treeflags`.
pub fn spell_valid_case(wordflags: WordFlags, treeflags: WordFlags) -> bool {
    (wordflags == WordFlags::ALLCAP && !treeflags.has(WordFlags::FIXCAP))
        || (!treeflags.has(WordFlags::ALLCAP | WordFlags::KEEPCAP)
            && (!treeflags.has(WordFlags::ONECAP) || wordflags.has(WordFlags::ONECAP)))
}

/// Whether spell checking is on for `window` and a language is actually loaded.
pub fn spell_check_window(window: Win) -> bool {
    let syn = window.syn_block();
    window.w_onebuf_opt.wo_spell != 0 && syn.b_p_spl.first_byte() != 0 && !syn.b_langp.is_empty()
}

/// Whether spell checking is *off* for `window`, giving an error if so.
pub fn no_spell_checking(window: Win) -> bool {
    if window.w_onebuf_opt.wo_spell == 0
        || window.syn_block().b_p_spl.first_byte() == 0
        || window.syn_block().b_langp.is_empty()
    {
        emsg(gettext(e_no_spell));
        return true;
    }
    false
}

/// Whether the word at line `lnum` column `col` has to start with a
/// capital, according to the buffer's `'spellcapcheck'`.
///
/// The question is whether a sentence ends just before it. At the start of
/// a line that means looking at the previous line, with a space standing
/// in for the line break.
pub fn check_need_cap(mut window: Win, lnum: LineNr, col: ColNr) -> bool {
    if window.syn_block().b_cap_prog.is_none() {
        return false;
    }

    // Which text the pattern runs over. Every read of the buffer happens
    // here, one line at a time, and what comes out is owned -- the pattern
    // walk below hands the regexp engine a pointer, and that must not be
    // into the cache.
    let context = {
        let mut lines = window.buffer().lines();
        let at_line_start = col == 0 || skip::white(lines.line(lnum)) as ColNr >= col;
        if !at_line_start {
            scan_before(lines.line(lnum), col as usize)
        } else if lnum == 1 {
            CapCheck::Needed
        } else {
            sentence_before(lines.line(lnum - 1))
        }
    };
    let CapCheck::Scan { text, endcol } = context else {
        return true;
    };

    // Does a sentence end before the word?
    let mut need_cap = false;
    let mut regmatch = RegMatch::new(window.syn_block_mut().b_cap_prog.take(), false);
    // The walk starts at `endcol` and only ever moves back.
    let mut at = endcol;
    loop {
        at -= head_off(&text[..endcol], at - 1) + 1;
        if at == 0 || spell_iswordp_nmw(&text[at..], window) {
            break;
        }
        // The match has to end exactly where the word begins, which is
        // `endcol` bytes into `text` and so that many past `at`.
        let word = crate::cstr::in_bytes(&text[at..]);
        if vim_regexec(&mut regmatch, word, 0) && regmatch.ends[0] == Some(endcol - at) {
            need_cap = true;
            break;
        }
    }
    window.syn_block_mut().b_cap_prog = regmatch.regprog.take();
    need_cap
}

/// What [`check_need_cap`] has to look at.
enum CapCheck {
    /// A capital is wanted outright: the word is at the start of the file,
    /// or the line before it is blank.
    Needed,
    /// Run 'spellcapcheck' over `text` -- NUL-terminated, because the
    /// regexp engine walks it as a C string -- and see whether a match ends
    /// exactly at `endcol`.
    Scan { text: Vec<u8>, endcol: usize },
}

/// The word is not at the start of its line, so the sentence that has to end
/// before it ends inside the line itself: run the pattern over the line, up
/// to the word.
fn scan_before(line: &[u8], col: usize) -> CapCheck {
    let mut text = line.to_vec();
    text.push(0);
    CapCheck::Scan { text, endcol: col }
}

/// The word is at the start of its line, so what has to end a sentence is
/// the line before -- with a space standing in for the line break, which is
/// what lets a pattern ending in `\s` match at all.
///
/// A blank line before it ends a paragraph, which needs no pattern.
fn sentence_before(prev: &[u8]) -> CapCheck {
    if skip::white(prev) == prev.len() {
        return CapCheck::Needed;
    }
    let mut text = Vec::with_capacity(prev.len() + 2);
    text.extend_from_slice(prev);
    text.push(b' ');
    let endcol = text.len();
    text.push(0);
    CapCheck::Scan { text, endcol }
}

/// The end of the word `text` starts with, by the spell word characters, as
/// an offset into it.
pub fn spell_to_word_end(text: &[u8], win: Win) -> usize {
    let mut p = 0;
    while byte_at(text, p) != 0 && spell_iswordp(tail(text, p), win) {
        p += c_cluster_len(tail(text, p)).max(1);
    }
    p
}

/// For Insert-mode completion `CTRL-X s`: the column where the word in
/// front of `startcol` begins.
///
/// Whether it is misspelled is not checked — completion can only replace
/// the word before the cursor anyway.
pub fn spell_word_start(startcol: c_int) -> c_int {
    let win = Win::current();
    if no_spell_checking(win) {
        return startcol;
    }

    let mut lines = Lines::current();
    let line = lines.line(win.w_cursor.lnum);

    // Back up to a word character.
    let mut p = usize::try_from(startcol).unwrap_or(0).min(line.len());
    while p > 0 {
        p -= head_off(line, p - 1) + 1;
        if spell_iswordp_nmw(&line[p..], win) {
            break;
        }
    }

    // Then back to the start of that word.
    let mut col = 0;
    while p > 0 {
        col = p as c_int;
        p -= head_off(line, p - 1) + 1;
        if !spell_iswordp(&line[p..], win) {
            break;
        }
        col = 0;
    }

    col
}

/// Whether the word [`expand_spelling`] is about to suggest for needs a
/// capital.
///
/// The word is deleted from the buffer before [`expand_spelling`] runs, so
/// the answer has to be taken beforehand and parked here.
static spell_expand_need_cap: GlobalCell<bool> = GlobalCell::new(false);

/// Record, before the word is removed, whether its replacement will need a
/// capital.
pub fn spell_expand_check_cap(col: ColNr) {
    spell_expand_need_cap.set(check_need_cap(
        Win::current(),
        Win::current().w_cursor.lnum,
        col,
    ));
}

/// Insert-mode completion `CTRL-X ?`: the suggestions for `pat`.
pub fn expand_spelling(_lnum: LineNr, pat: &CStr) -> Vec<XString> {
    spell_suggest_list(pat, 100, spell_expand_need_cap.get(), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The text `check_need_cap` would run 'spellcapcheck' over, and where
    /// the word starts in it -- `None` when no pattern is needed because a
    /// capital is wanted outright.
    fn scan(check: CapCheck) -> Option<(Vec<u8>, usize)> {
        match check {
            CapCheck::Needed => None,
            CapCheck::Scan { text, endcol } => Some((text, endcol)),
        }
    }

    #[test]
    fn a_word_inside_its_line_is_matched_against_the_line_itself() {
        // "one. two", asking about the `t`: the pattern runs over the whole
        // line and has to end exactly at column 5.
        let (text, endcol) = scan(scan_before(b"one. two", 5)).expect("a scan");
        assert_eq!(text, b"one. two\0");
        assert_eq!(endcol, 5);
    }

    #[test]
    fn the_line_break_before_a_word_is_a_space() {
        // The previous line's text with a space on the end, because a
        // 'spellcapcheck' of `[.?!]\_[\])'"\t ]\+` has to match the break.
        let (text, endcol) = scan(sentence_before(b"one.")).expect("a scan");
        assert_eq!(text, b"one. \0");
        assert_eq!(endcol, 5);
    }

    #[test]
    fn a_blank_line_before_a_word_needs_no_pattern() {
        assert!(scan(sentence_before(b"")).is_none());
        assert!(scan(sentence_before(b"   ")).is_none());
        assert!(scan(sentence_before(b"\t \t")).is_none());
        // One non-blank byte is enough to make it a scan.
        assert!(scan(sentence_before(b"  x")).is_some());
    }

    #[test]
    fn the_text_handed_to_the_regexp_engine_is_terminated() {
        // Both shapes: the engine walks a C string, and the text it is
        // given is never the cache's own line.
        assert_eq!(scan_before_bytes(b"abc", 3).last(), Some(&0));
        assert_eq!(
            scan(sentence_before(b"abc")).expect("a scan").0.last(),
            Some(&0)
        );
    }

    fn scan_before_bytes(line: &[u8], col: usize) -> Vec<u8> {
        scan(scan_before(line, col)).expect("a scan").0
    }

    /// `spell_valid_case` decides whether a word as written may use a tree
    /// entry. Two arms: an all-capitals word matches an entry that is not
    /// `FIXCAP`, and *any* word matches an entry that demands neither
    /// all-capitals nor keep-case and whose `ONECAP` the word answers.
    #[test]
    fn an_all_capitals_word_takes_the_first_arm() {
        assert!(spell_valid_case(WordFlags::ALLCAP, WordFlags::KEEPCAP));
        assert!(spell_valid_case(WordFlags::ALLCAP, WordFlags::ALLCAP));
        assert!(spell_valid_case(WordFlags::ALLCAP, WordFlags::ONECAP));
        // `FIXCAP` closes that arm -- but the second one is still open,
        // because a bare `FIXCAP` entry demands no capitals of its own.
        assert!(spell_valid_case(WordFlags::ALLCAP, WordFlags::FIXCAP));
        // Both arms closed: `FIXCAP` bars the first and `KEEPCAP` the
        // second. This is the pair `FIXCAP` exists for.
        let fixed_keep = WordFlags::FIXCAP.or(WordFlags::KEEPCAP);
        assert!(!spell_valid_case(WordFlags::ALLCAP, fixed_keep));
    }

    #[test]
    fn a_lower_case_word_does_not_match_an_entry_that_demands_capitals() {
        assert!(spell_valid_case(WordFlags::NONE, WordFlags::NONE));
        assert!(!spell_valid_case(WordFlags::NONE, WordFlags::ALLCAP));
        assert!(!spell_valid_case(WordFlags::NONE, WordFlags::KEEPCAP));
        assert!(!spell_valid_case(WordFlags::NONE, WordFlags::ONECAP));
        assert!(spell_valid_case(WordFlags::ONECAP, WordFlags::ONECAP));
    }
}
