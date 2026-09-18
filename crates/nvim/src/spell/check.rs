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

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::spell::WordFlags;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};
use core::mem;

use crate::charset::{skip, skipbin, skipdigits, skiphex};
use crate::cursor::get_cursor_line_ptr;
use crate::global_cell::GlobalCell;
use crate::mbyte::{head_off, mb_isupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::message::e_no_spell;
use crate::message::emsg;
use crate::options::kOptSpoFlagCamel;
use crate::os::cshim::gettext;
use crate::regexp::vim_regexec;
use crate::spellsuggest::spell_suggest_list;
use crate::types::{ColNr, GArray, Hlf, LangP, LineNr, RegMatch, size_t, uint8_t};

use super::chartab::{spell_iswordp, spell_iswordp_nmw};
use super::lookup::{find_prefix, find_word};
use super::{
    CHAR_DIGIT, CHAR_OTHER, CHAR_UPPER, FIND_COMPOUND, FIND_FOLDWORD, FIND_KEEPWORD, MAXWLEN,
    MatchInf, SP_BAD, SP_BANNED, SP_OK, SP_RARE, count_common_word, spelltab_isu,
};
use crate::highlight_group::{HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::optionstr::LocalOptStr;

/// Whether `c` is upper case, by the spell table below 128 and the general
/// rules above it.
fn is_upper(c: c_int) -> bool {
    if c >= 128 {
        mb_isupper(c)
    } else {
        spelltab_isu(c as usize)
    }
}

/// Check the word starting at `text` in window `window`.
///
/// `attrp` is set to the highlight for a badly spelled word and left alone
/// otherwise. `capcol`, when not null, is the column at which to check for
/// a capital after a sentence end; it is set to the next such column, or
/// -1 when no sentence end was found. `docount` asks for the word to be
/// counted towards the `COMMON` statistics.
///
/// Must only be called with a non-empty `'spelllang'`.
///
/// Returns the length of the word in bytes, good or bad, so the caller can
/// skip over it.
///
/// # Safety
///
/// `text` must point at a NUL-terminated string, unaliased for the call.
/// `attrp` must point at a live `Hlf`, unaliased for the call. `capcol` must
/// point at a writable `int` the caller owns.
pub unsafe fn spell_check(
    mut window: Win,
    text: *mut c_char,
    attrp: *mut Hlf,
    capcol: *mut c_int,
    docount: bool,
) -> size_t {
    // A word never starts at a space or a control character.
    if unsafe { *text } as uint8_t as c_int <= ' ' as c_int {
        return 1;
    }
    // Loading the language files failed.
    if unsafe { (*window.w_s).b_langp.ga_len } <= 0 {
        return 1;
    }

    let mut nrlen: size_t = 0; // a number came first
    let mut wrongcaplen: size_t = 0;
    let mut count_word = docount;
    let use_camel_case = unsafe { (*window.w_s).b_p_spo_flags } & kOptSpoFlagCamel != 0;
    let mut is_camel_case = false;

    // Nearly everything lives in "mi" so that it can be handed to the
    // lookup functions in one go.
    // SAFETY: the caller's promise -- `text` is a NUL-terminated line of
    // `window`'s buffer, which nothing writes while the lookup runs.
    let mut mi = unsafe { MatchInf::new(text, window) };

    // A number is always fine, including hex and binary literals. The
    // word is still checked, so that "3GPP" and "11 julifeest" are
    // caught.
    if unsafe { *text } >= b'0' as c_char && unsafe { *text } <= b'9' as c_char {
        mi.mi_end = if unsafe { *text } == b'0' as c_char
            && (unsafe { *text.offset(1) } == b'b' as c_char
                || unsafe { *text.offset(1) } == b'B' as c_char)
        {
            unsafe { skipbin(text.offset(2)) as *mut c_char }
        } else if unsafe { *text } == b'0' as c_char
            && (unsafe { *text.offset(1) } == b'x' as c_char
                || unsafe { *text.offset(1) } == b'X' as c_char)
        {
            unsafe { skiphex(text.offset(2)) }
        } else {
            unsafe { skipdigits(text) }
        };
        nrlen = unsafe { mi.mi_end.offset_from(text) } as size_t;
    }

    // Find the end of the word: the next non-word character.
    if unsafe { spell_iswordp(mi.mi_fend, window) } {
        if use_camel_case {
            mi.mi_fend = unsafe { advance_camelcase_word(text, window, &mut is_camel_case) };
        } else {
            loop {
                mi.mi_fend = unsafe { mi.mi_fend.offset(utfc_ptr2len(mi.mi_fend) as isize) };
                if unsafe { *mi.mi_fend } == 0 || !unsafe { spell_iswordp(mi.mi_fend, window) } {
                    break;
                }
            }
        }

        if !capcol.is_null()
            && unsafe { *capcol } == 0
            && !unsafe { (*window.w_s).b_cap_prog }.is_null()
        {
            // This word should have started with a capital.
            if !is_upper(unsafe { utf_ptr2char(text) }) {
                wrongcaplen = unsafe { mi.mi_fend.offset_from(text) } as size_t;
            }
        }
    }
    if !capcol.is_null() {
        unsafe { *capcol = -1 };
    }

    // Characters up to the next non-word character are consumed even
    // for a bad word.
    mi.mi_end = mi.mi_fend;

    // The caps type is worked out later, on demand.
    mi.mi_capflags = WordFlags::NONE;
    mi.mi_cend = core::ptr::null_mut();

    // Fold one character past the word, so the lookup can see where the
    // word ends.
    if unsafe { *mi.mi_fend } != 0 {
        mi.mi_fend = unsafe { mi.mi_fend.offset(utfc_ptr2len(mi.mi_fend) as isize) };
    }
    let fword = &raw mut mi.mi_fword as *mut c_char;
    let taken = unsafe { mi.mi_fend.offset_from(text) } as c_int;
    let room = MAXWLEN as c_int + 1;
    let _ = unsafe { super::chartab::spell_casefold(window, text, taken, fword, room) };
    mi.mi_fwordlen = unsafe { cstr::bytes_at(fword) }.len() as c_int;

    if is_camel_case && mi.mi_fwordlen > 0 {
        // Put a fake word end into the folded word.
        mi.mi_fword[(mi.mi_fwordlen - 1) as usize] = b' ' as c_char;
    }

    // Bad until recognised.
    mi.mi_result = SP_BAD;
    mi.mi_result2 = SP_BAD;

    // Every language is tried, because a later one may match longer.
    let langp_data = unsafe { (*window.w_s).b_langp.ga_data } as *mut LangP;
    let langp_len = unsafe { (*window.w_s).b_langp.ga_len };
    for lpi in 0..langp_len {
        mi.mi_lp = unsafe { langp_data.offset(lpi as isize) };

        // A language whose reload failed stays in the list with
        // everything cleared out.
        if unsafe { (*(*mi.mi_lp).lp_slang).sl_fold_tree.is_empty() } {
            continue;
        }

        unsafe { find_word(&mut mi, FIND_FOLDWORD) };
        unsafe { find_word(&mut mi, FIND_KEEPWORD) };
        unsafe { find_prefix(&mut mi, FIND_FOLDWORD) };

        // A NOBREAK language may fall back on a word with nothing valid
        // after it.
        if unsafe { (*(*mi.mi_lp).lp_slang).sl_nobreak }
            && mi.mi_result == SP_BAD
            && mi.mi_result2 != SP_BAD
        {
            mi.mi_result = mi.mi_result2;
            mi.mi_end = mi.mi_end2;
        }

        // Count the word in the first language that accepts it.
        if count_word && mi.mi_result == SP_OK {
            let slang = unsafe { (*mi.mi_lp).lp_slang };
            let len = unsafe { mi.mi_end.offset_from(text) } as c_int;
            unsafe { count_common_word(slang, text, len, 1) };
            count_word = false;
        }
    }

    if mi.mi_result != SP_OK {
        if nrlen > 0 {
            // Skip over a leading number, so that "42nd" works. Rare and
            // local words are still flagged, for "3GPP".
            if mi.mi_result == SP_BAD || mi.mi_result == SP_BANNED {
                return nrlen;
            }
        } else if !unsafe { spell_iswordp_nmw(text, window) } {
            // Sitting on a non-word character is not an error; step over
            // it and look for a word after it.
            if !capcol.is_null() && !unsafe { (*window.w_s).b_cap_prog }.is_null() {
                // Did a sentence end here?
                let mut regmatch: RegMatch = unsafe { mem::zeroed() };
                regmatch.regprog = unsafe { (*window.w_s).b_cap_prog };
                regmatch.rm_ic = false;
                // SAFETY: the caller's NUL-terminated text.
                let r = vim_regexec(&mut regmatch, unsafe { cstr::at(text) }, 0);
                unsafe { (*window.w_s).b_cap_prog = regmatch.regprog };
                if let Some(end) = regmatch.ends[0]
                    && r
                {
                    unsafe { *capcol = end as c_int };
                }
            }

            return unsafe { utfc_ptr2len(text) } as size_t;
        } else if mi.mi_end == text {
            // Always consume at least one character, in case 'midword'
            // left the word empty.
            mi.mi_end = unsafe { mi.mi_end.offset(utfc_ptr2len(mi.mi_end) as isize) };
        } else if mi.mi_result == SP_BAD && unsafe { (*(*langp_data).lp_slang).sl_nobreak } {
            // The first language is NOBREAK: find the first position at
            // which any word would be valid.
            let save_result = mi.mi_result;
            mi.mi_lp = langp_data;
            if !unsafe { (*(*mi.mi_lp).lp_slang).sl_fold_tree.is_empty() } {
                let mut p = mi.mi_word;
                let mut fp = fword;
                loop {
                    p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
                    fp = unsafe { fp.offset(utfc_ptr2len(fp) as isize) };
                    if p >= mi.mi_end {
                        break;
                    }
                    mi.mi_compoff = unsafe { fp.offset_from(fword) } as c_int;
                    unsafe { find_word(&mut mi, FIND_COMPOUND) };
                    if mi.mi_result != SP_BAD {
                        mi.mi_end = p;
                        break;
                    }
                }
                mi.mi_result = save_result;
            }
        }

        let attr = if mi.mi_result == SP_BAD || mi.mi_result == SP_BANNED {
            HLF_SPB
        } else if mi.mi_result == SP_RARE {
            HLF_SPR
        } else {
            HLF_SPL
        };
        unsafe { *attrp = attr };
    }

    if wrongcaplen > 0 && (mi.mi_result == SP_OK || mi.mi_result == SP_RARE) {
        // SpellCap is only reported when the word itself is fine.
        unsafe { *attrp = HLF_SPC };
        return wrongcaplen;
    }

    unsafe { mi.mi_end.offset_from(text) as size_t }
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

/// The end of the word starting at `str`, splitting camel-case words into
/// their parts.
///
/// A split happens where the character type changes in a way that only a
/// word boundary explains: `fooBar`, `fooA1`, `1a`, and — looking two
/// characters back — `HTTPServer`, which splits before the `S` rather than
/// after it.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string, unaliased for the call.
unsafe fn advance_camelcase_word(
    str: *mut c_char,
    window: Win,
    is_camel_case: &mut bool,
) -> *mut c_char {
    *is_camel_case = false;
    if unsafe { *str } == 0 {
        return str;
    }

    let mut end = str;
    let c = unsafe { utf_ptr2char(end) };
    end = unsafe { end.offset(utfc_ptr2len(end) as isize) };

    // Only the last two characters' types are ever needed.
    let mut last_last_type = -1;
    let mut last_type = get_char_type(c);

    while unsafe { *end } != 0 && unsafe { spell_iswordp(end, window) } {
        let this_type = get_char_type(unsafe { utf_ptr2char(end) });

        if last_last_type == CHAR_UPPER && last_type == CHAR_UPPER && this_type == CHAR_OTHER {
            // UpperUpperLower: the word ends one character back.
            *is_camel_case = true;
            end = unsafe { end.offset(-(utf_head_off(str, end.offset(-1)) as isize + 1)) };
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

        end = unsafe { end.offset(utfc_ptr2len(end) as isize) };
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
    let on =
        unsafe { window.w_onebuf_opt.wo_spell != 0 && (*window.w_s).b_p_spl.first_byte() != 0 };
    on && unsafe { (*window.w_s).b_langp.ga_len } > 0
        && !unsafe { *((*window.w_s).b_langp.ga_data as *mut *mut c_char) }.is_null()
}

/// Whether spell checking is *off* for `window`, giving an error if so.
pub fn no_spell_checking(window: Win) -> bool {
    if window.w_onebuf_opt.wo_spell == 0
        || unsafe { (*window.w_s).b_p_spl.first_byte() } == 0
        || unsafe { (*window.w_s).b_langp.ga_len } <= 0
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
    if unsafe { (*window.w_s).b_cap_prog }.is_null() {
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
    let CapCheck::Scan { mut text, endcol } = context else {
        return true;
    };

    // Does a sentence end before the word?
    let mut need_cap = false;
    let mut regmatch: RegMatch = unsafe { mem::zeroed() };
    regmatch.regprog = unsafe { (*window.w_s).b_cap_prog };
    regmatch.rm_ic = false;
    // `text` is this frame's own NUL-terminated buffer, and `at` stays
    // inside it: the walk starts at `endcol` and only ever moves back.
    let base = text.as_mut_ptr().cast::<c_char>();
    let mut at = endcol;
    loop {
        at -= head_off(&text[..endcol], at - 1) + 1;
        // SAFETY: `base + at` is inside `text`, which is NUL-terminated.
        if at == 0 || unsafe { spell_iswordp_nmw(base.wrapping_add(at), window) } {
            break;
        }
        // The match has to end exactly where the word begins, which is
        // `endcol` bytes into `text` and so that many past `at`.
        let word = cstr::in_bytes(&text[at..]);
        if vim_regexec(&mut regmatch, word, 0) && regmatch.ends[0] == Some(endcol - at) {
            need_cap = true;
            break;
        }
    }
    unsafe { (*window.w_s).b_cap_prog = regmatch.regprog };
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

/// The end of the word starting at `start`, by the spell word characters.
///
/// # Safety
///
/// `start` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn spell_to_word_end(start: *mut c_char, win: Win) -> *mut c_char {
    let mut p = start;
    while unsafe { *p } != 0 && unsafe { spell_iswordp(p, win) } {
        p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
    }
    p
}

/// For Insert-mode completion `CTRL-X s`: the column where the word in
/// front of `startcol` begins.
///
/// Whether it is misspelled is not checked — completion can only replace
/// the word before the cursor anyway.
pub fn spell_word_start(startcol: c_int) -> c_int {
    if no_spell_checking(Win::current()) {
        return startcol;
    }

    let line = get_cursor_line_ptr();

    // Back up to a word character.
    let mut p = unsafe { line.offset(startcol as isize) };
    while p > line {
        p = unsafe { p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1)) };
        if unsafe { spell_iswordp_nmw(p, Win::current()) } {
            break;
        }
    }

    // Then back to the start of that word.
    let mut col = 0;
    while p > line {
        col = unsafe { p.offset_from(line) } as c_int;
        p = unsafe { p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1)) };
        if !unsafe { spell_iswordp(p, Win::current()) } {
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

/// Insert-mode completion `CTRL-X ?`: fill `matchp` with suggestions for
/// `pat` and return how many there are.
///
/// # Safety
///
/// `pat` must point at a NUL-terminated string, unaliased for the call.
/// `matchp` must point at a writable `*mut *mut c_char` slot the caller owns
/// for the call.
pub unsafe fn expand_spelling(
    _lnum: LineNr,
    pat: *mut c_char,
    matchp: *mut *mut *mut c_char,
) -> c_int {
    let mut ga: GArray = unsafe { mem::zeroed() };
    unsafe { spell_suggest_list(&raw mut ga, pat, 100, spell_expand_need_cap.get(), true) };
    unsafe { *matchp = ga.ga_data as *mut *mut c_char };
    ga.ga_len
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
