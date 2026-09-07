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

use crate::charset::{getwhitecols, skipbin, skipdigits, skiphex, skipwhite};
use crate::cursor::get_cursor_line_ptr;
use crate::global_cell::GlobalCell;
use crate::mbyte::{mb_isupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::xfree;
use crate::message::e_no_spell;
use crate::message::emsg;
use crate::options::kOptSpoFlagCamel;
use crate::os::cshim::gettext;
use crate::regexp::vim_regexec;
use crate::spellsuggest::spell_suggest_list;
use crate::strings::concat_str;
use crate::types::{ColNr, GArray, Hlf, LangP, LineNr, RegMatch, size_t, uint8_t};

use super::chartab::{spell_iswordp, spell_iswordp_nmw};
use super::lookup::{find_prefix, find_word};
use super::{
    CHAR_DIGIT, CHAR_OTHER, CHAR_UPPER, FIND_COMPOUND, FIND_FOLDWORD, FIND_KEEPWORD, MAXWLEN,
    MatchInf, SP_BAD, SP_BANNED, SP_OK, SP_RARE, count_common_word, spelltab_isu,
};
use crate::highlight_group::{HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};

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
    let mut mi: MatchInf = unsafe { mem::zeroed() };

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
    mi.mi_word = text;
    mi.mi_fend = text;
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
    mi.mi_win = window.raw();

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
                let r = unsafe { vim_regexec(&raw mut regmatch, text, 0) };
                unsafe { (*window.w_s).b_cap_prog = regmatch.regprog };
                if r {
                    unsafe { *capcol = regmatch.endp[0].offset_from(text) as c_int };
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
    let on = unsafe { window.w_onebuf_opt.wo_spell != 0 && *(*window.w_s).b_p_spl != 0 };
    on && unsafe { (*window.w_s).b_langp.ga_len } > 0
        && !unsafe { *((*window.w_s).b_langp.ga_data as *mut *mut c_char) }.is_null()
}

/// Whether spell checking is *off* for `window`, giving an error if so.
pub fn no_spell_checking(window: Win) -> bool {
    if window.w_onebuf_opt.wo_spell == 0
        || unsafe { *(*window.w_s).b_p_spl } == 0
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

    let mut need_cap = false;
    let mut line = if col != 0 {
        unsafe { ml_get_buf(window.buffer(), lnum) }
    } else {
        core::ptr::null_mut()
    };
    let mut line_copy: *mut c_char = core::ptr::null_mut();
    let mut endcol: ColNr = 0;

    if col == 0 || unsafe { getwhitecols(line) } >= col as isize {
        // At the start of the line: the previous line has to be empty,
        // or end a sentence.
        if lnum == 1 {
            need_cap = true;
        } else {
            line = unsafe { ml_get_buf(window.buffer(), lnum - 1) };
            if unsafe { *skipwhite(line) } == 0 {
                need_cap = true;
            } else {
                // A space stands in for the line break.
                line_copy = unsafe { concat_str(line, c" ".as_ptr()) };
                line = line_copy;
                endcol = unsafe { cstr::bytes_at(line) }.len() as ColNr;
            }
        }
    } else {
        endcol = col;
    }

    if endcol > 0 {
        // Does a sentence end before the word?
        let mut regmatch: RegMatch = unsafe { mem::zeroed() };
        regmatch.regprog = unsafe { (*window.w_s).b_cap_prog };
        regmatch.rm_ic = false;
        let end = unsafe { line.offset(endcol as isize) };
        let mut p = end;
        loop {
            p = unsafe { p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1)) };
            if p == line || unsafe { spell_iswordp_nmw(p, window) } {
                break;
            }
            if unsafe { vim_regexec(&raw mut regmatch, p, 0) } && regmatch.endp[0] == end {
                need_cap = true;
                break;
            }
        }
        unsafe { (*window.w_s).b_cap_prog = regmatch.regprog };
    }

    unsafe { xfree(line_copy as *mut core::ffi::c_void) };
    need_cap
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
