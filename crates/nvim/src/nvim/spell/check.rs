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

use core::ffi::{c_char, c_int};
use core::mem;

use crate::src::nvim::charset::{getwhitecols, skipbin, skipdigits, skiphex, skipwhite};
use crate::src::nvim::cursor::get_cursor_line_ptr;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curwin, e_no_spell};
use crate::src::nvim::mbyte::{mb_isupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::emsg;
use crate::src::nvim::options::kOptSpoFlagCamel;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::regexp::vim_regexec;
use crate::src::nvim::spellsuggest::spell_suggest_list;
use crate::src::nvim::strings::concat_str;
use crate::src::nvim::types::{
    colnr_T, garray_T, hlf_T, langp_T, linenr_T, regmatch_T, size_t, uint8_t, win_T,
};

use super::chartab::{spell_iswordp, spell_iswordp_nmw};
use super::lookup::{find_prefix, find_word};
use super::{
    CHAR_DIGIT, CHAR_OTHER, CHAR_UPPER, FIND_COMPOUND, FIND_FOLDWORD, FIND_KEEPWORD, MAXWLEN,
    SP_BAD, SP_BANNED, SP_OK, SP_RARE, WF_ALLCAP, WF_FIXCAP, WF_KEEPCAP, WF_ONECAP,
    count_common_word, matchinf_T, spelltab,
};
use crate::src::nvim::highlight_group::{HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};

/// Whether `c` is upper case, by the spell table below 128 and the general
/// rules above it.
fn is_upper(c: c_int) -> bool {
    unsafe {
        if c >= 128 {
            mb_isupper(c)
        } else {
            (*spelltab.ptr()).st_isu[c as usize]
        }
    }
}

/// Check the word starting at `ptr` in window `wp`.
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
pub unsafe fn spell_check(
    wp: *mut win_T,
    ptr: *mut c_char,
    attrp: *mut hlf_T,
    capcol: *mut c_int,
    docount: bool,
) -> size_t {
    unsafe {
        // A word never starts at a space or a control character.
        if *ptr as uint8_t as c_int <= ' ' as c_int {
            return 1;
        }
        // Loading the language files failed.
        if (*(*wp).w_s).b_langp.ga_len <= 0 {
            return 1;
        }

        let mut nrlen: size_t = 0; // a number came first
        let mut wrongcaplen: size_t = 0;
        let mut count_word = docount;
        let use_camel_case = (*(*wp).w_s).b_p_spo_flags & kOptSpoFlagCamel != 0;
        let mut is_camel_case = false;

        // Nearly everything lives in "mi" so that it can be handed to the
        // lookup functions in one go.
        let mut mi: matchinf_T = mem::zeroed();

        // A number is always fine, including hex and binary literals. The
        // word is still checked, so that "3GPP" and "11 julifeest" are
        // caught.
        if *ptr >= b'0' as c_char && *ptr <= b'9' as c_char {
            mi.mi_end = if *ptr == b'0' as c_char
                && (*ptr.offset(1) == b'b' as c_char || *ptr.offset(1) == b'B' as c_char)
            {
                skipbin(ptr.offset(2)) as *mut c_char
            } else if *ptr == b'0' as c_char
                && (*ptr.offset(1) == b'x' as c_char || *ptr.offset(1) == b'X' as c_char)
            {
                skiphex(ptr.offset(2))
            } else {
                skipdigits(ptr)
            };
            nrlen = mi.mi_end.offset_from(ptr) as size_t;
        }

        // Find the end of the word: the next non-word character.
        mi.mi_word = ptr;
        mi.mi_fend = ptr;
        if spell_iswordp(mi.mi_fend, wp) {
            if use_camel_case {
                mi.mi_fend = advance_camelcase_word(ptr, wp, &mut is_camel_case);
            } else {
                loop {
                    mi.mi_fend = mi.mi_fend.offset(utfc_ptr2len(mi.mi_fend) as isize);
                    if *mi.mi_fend == 0 || !spell_iswordp(mi.mi_fend, wp) {
                        break;
                    }
                }
            }

            if !capcol.is_null() && *capcol == 0 && !(*(*wp).w_s).b_cap_prog.is_null() {
                // This word should have started with a capital.
                if !is_upper(utf_ptr2char(ptr)) {
                    wrongcaplen = mi.mi_fend.offset_from(ptr) as size_t;
                }
            }
        }
        if !capcol.is_null() {
            *capcol = -1;
        }

        // Characters up to the next non-word character are consumed even
        // for a bad word.
        mi.mi_end = mi.mi_fend;

        // The caps type is worked out later, on demand.
        mi.mi_capflags = 0;
        mi.mi_cend = core::ptr::null_mut();
        mi.mi_win = wp;

        // Fold one character past the word, so the lookup can see where the
        // word ends.
        if *mi.mi_fend != 0 {
            mi.mi_fend = mi.mi_fend.offset(utfc_ptr2len(mi.mi_fend) as isize);
        }
        let fword = &raw mut mi.mi_fword as *mut c_char;
        super::chartab::spell_casefold(
            wp,
            ptr,
            mi.mi_fend.offset_from(ptr) as c_int,
            fword,
            MAXWLEN as c_int + 1,
        );
        mi.mi_fwordlen = strlen(fword) as c_int;

        if is_camel_case && mi.mi_fwordlen > 0 {
            // Put a fake word end into the folded word.
            mi.mi_fword[(mi.mi_fwordlen - 1) as usize] = b' ' as c_char;
        }

        // Bad until recognised.
        mi.mi_result = SP_BAD;
        mi.mi_result2 = SP_BAD;

        // Every language is tried, because a later one may match longer.
        let langp_data = (*(*wp).w_s).b_langp.ga_data as *mut langp_T;
        let langp_len = (*(*wp).w_s).b_langp.ga_len;
        for lpi in 0..langp_len {
            mi.mi_lp = langp_data.offset(lpi as isize);

            // A language whose reload failed stays in the list with
            // everything cleared out.
            if (*(*mi.mi_lp).lp_slang).sl_fidxs.is_null() {
                continue;
            }

            find_word(&raw mut mi, FIND_FOLDWORD);
            find_word(&raw mut mi, FIND_KEEPWORD);
            find_prefix(&raw mut mi, FIND_FOLDWORD);

            // A NOBREAK language may fall back on a word with nothing valid
            // after it.
            if (*(*mi.mi_lp).lp_slang).sl_nobreak
                && mi.mi_result == SP_BAD
                && mi.mi_result2 != SP_BAD
            {
                mi.mi_result = mi.mi_result2;
                mi.mi_end = mi.mi_end2;
            }

            // Count the word in the first language that accepts it.
            if count_word && mi.mi_result == SP_OK {
                count_common_word(
                    (*mi.mi_lp).lp_slang,
                    ptr,
                    mi.mi_end.offset_from(ptr) as c_int,
                    1,
                );
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
            } else if !spell_iswordp_nmw(ptr, wp) {
                // Sitting on a non-word character is not an error; step over
                // it and look for a word after it.
                if !capcol.is_null() && !(*(*wp).w_s).b_cap_prog.is_null() {
                    // Did a sentence end here?
                    let mut regmatch: regmatch_T = mem::zeroed();
                    regmatch.regprog = (*(*wp).w_s).b_cap_prog;
                    regmatch.rm_ic = false;
                    let r = vim_regexec(&raw mut regmatch, ptr, 0);
                    (*(*wp).w_s).b_cap_prog = regmatch.regprog;
                    if r {
                        *capcol = regmatch.endp[0].offset_from(ptr) as c_int;
                    }
                }

                return utfc_ptr2len(ptr) as size_t;
            } else if mi.mi_end == ptr {
                // Always consume at least one character, in case 'midword'
                // left the word empty.
                mi.mi_end = mi.mi_end.offset(utfc_ptr2len(mi.mi_end) as isize);
            } else if mi.mi_result == SP_BAD && (*(*langp_data).lp_slang).sl_nobreak {
                // The first language is NOBREAK: find the first position at
                // which any word would be valid.
                let save_result = mi.mi_result;
                mi.mi_lp = langp_data;
                if !(*(*mi.mi_lp).lp_slang).sl_fidxs.is_null() {
                    let mut p = mi.mi_word;
                    let mut fp = fword;
                    loop {
                        p = p.offset(utfc_ptr2len(p) as isize);
                        fp = fp.offset(utfc_ptr2len(fp) as isize);
                        if p >= mi.mi_end {
                            break;
                        }
                        mi.mi_compoff = fp.offset_from(fword) as c_int;
                        find_word(&raw mut mi, FIND_COMPOUND);
                        if mi.mi_result != SP_BAD {
                            mi.mi_end = p;
                            break;
                        }
                    }
                    mi.mi_result = save_result;
                }
            }

            *attrp = if mi.mi_result == SP_BAD || mi.mi_result == SP_BANNED {
                HLF_SPB
            } else if mi.mi_result == SP_RARE {
                HLF_SPR
            } else {
                HLF_SPL
            };
        }

        if wrongcaplen > 0 && (mi.mi_result == SP_OK || mi.mi_result == SP_RARE) {
            // SpellCap is only reported when the word itself is fine.
            *attrp = HLF_SPC;
            return wrongcaplen;
        }

        mi.mi_end.offset_from(ptr) as size_t
    }
}

/// Classify `c` for the camel-case split.
fn get_char_type(c: c_int) -> c_int {
    if crate::src::nvim::ascii::ascii_isdigit(c) {
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
unsafe fn advance_camelcase_word(
    str: *mut c_char,
    wp: *mut win_T,
    is_camel_case: &mut bool,
) -> *mut c_char {
    unsafe {
        *is_camel_case = false;
        if *str == 0 {
            return str;
        }

        let mut end = str;
        let c = utf_ptr2char(end);
        end = end.offset(utfc_ptr2len(end) as isize);

        // Only the last two characters' types are ever needed.
        let mut last_last_type = -1;
        let mut last_type = get_char_type(c);

        while *end != 0 && spell_iswordp(end, wp) {
            let this_type = get_char_type(utf_ptr2char(end));

            if last_last_type == CHAR_UPPER && last_type == CHAR_UPPER && this_type == CHAR_OTHER {
                // UpperUpperLower: the word ends one character back.
                *is_camel_case = true;
                end = end.offset(-(utf_head_off(str, end.offset(-1)) as isize + 1));
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

            end = end.offset(utfc_ptr2len(end) as isize);
        }

        end
    }
}

/// Whether a word written with `wordflags` capitalisation satisfies a tree
/// entry recorded with `treeflags`.
pub fn spell_valid_case(wordflags: c_int, treeflags: c_int) -> bool {
    (wordflags == WF_ALLCAP as c_int && treeflags & WF_FIXCAP as c_int == 0)
        || (treeflags & (WF_ALLCAP | WF_KEEPCAP) as c_int == 0
            && (treeflags & WF_ONECAP as c_int == 0 || wordflags & WF_ONECAP as c_int != 0))
}

/// Whether spell checking is on for `wp` and a language is actually loaded.
pub unsafe fn spell_check_window(wp: *mut win_T) -> bool {
    unsafe {
        (*wp).w_onebuf_opt.wo_spell != 0
            && *(*(*wp).w_s).b_p_spl != 0
            && (*(*wp).w_s).b_langp.ga_len > 0
            && !(*((*(*wp).w_s).b_langp.ga_data as *mut *mut c_char)).is_null()
    }
}

/// Whether spell checking is *off* for `wp`, giving an error if so.
pub unsafe fn no_spell_checking(wp: *mut win_T) -> bool {
    unsafe {
        if (*wp).w_onebuf_opt.wo_spell == 0
            || *(*(*wp).w_s).b_p_spl == 0
            || (*(*wp).w_s).b_langp.ga_len <= 0
        {
            emsg(gettext(e_no_spell.as_ptr()));
            return true;
        }
        false
    }
}

/// Whether the word at line `lnum` column `col` has to start with a
/// capital, according to the buffer's `'spellcapcheck'`.
///
/// The question is whether a sentence ends just before it. At the start of
/// a line that means looking at the previous line, with a space standing
/// in for the line break.
pub unsafe fn check_need_cap(wp: *mut win_T, lnum: linenr_T, col: colnr_T) -> bool {
    unsafe {
        if (*(*wp).w_s).b_cap_prog.is_null() {
            return false;
        }

        let mut need_cap = false;
        let mut line = if col != 0 {
            ml_get_buf((*wp).w_buffer, lnum)
        } else {
            core::ptr::null_mut()
        };
        let mut line_copy: *mut c_char = core::ptr::null_mut();
        let mut endcol: colnr_T = 0;

        if col == 0 || getwhitecols(line) >= col as isize {
            // At the start of the line: the previous line has to be empty,
            // or end a sentence.
            if lnum == 1 {
                need_cap = true;
            } else {
                line = ml_get_buf((*wp).w_buffer, lnum - 1);
                if *skipwhite(line) == 0 {
                    need_cap = true;
                } else {
                    // A space stands in for the line break.
                    line_copy = concat_str(line, c" ".as_ptr());
                    line = line_copy;
                    endcol = strlen(line) as colnr_T;
                }
            }
        } else {
            endcol = col;
        }

        if endcol > 0 {
            // Does a sentence end before the word?
            let mut regmatch: regmatch_T = mem::zeroed();
            regmatch.regprog = (*(*wp).w_s).b_cap_prog;
            regmatch.rm_ic = false;
            let end = line.offset(endcol as isize);
            let mut p = end;
            loop {
                p = p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1));
                if p == line || spell_iswordp_nmw(p, wp) {
                    break;
                }
                if vim_regexec(&raw mut regmatch, p, 0) && regmatch.endp[0] == end {
                    need_cap = true;
                    break;
                }
            }
            (*(*wp).w_s).b_cap_prog = regmatch.regprog;
        }

        xfree(line_copy as *mut core::ffi::c_void);
        need_cap
    }
}

/// The end of the word starting at `start`, by the spell word characters.
pub unsafe fn spell_to_word_end(start: *mut c_char, win: *mut win_T) -> *mut c_char {
    unsafe {
        let mut p = start;
        while *p != 0 && spell_iswordp(p, win) {
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        p
    }
}

/// For Insert-mode completion `CTRL-X s`: the column where the word in
/// front of `startcol` begins.
///
/// Whether it is misspelled is not checked — completion can only replace
/// the word before the cursor anyway.
pub unsafe fn spell_word_start(startcol: c_int) -> c_int {
    unsafe {
        if no_spell_checking(curwin.get()) {
            return startcol;
        }

        let line = get_cursor_line_ptr();

        // Back up to a word character.
        let mut p = line.offset(startcol as isize);
        while p > line {
            p = p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1));
            if spell_iswordp_nmw(p, curwin.get()) {
                break;
            }
        }

        // Then back to the start of that word.
        let mut col = 0;
        while p > line {
            col = p.offset_from(line) as c_int;
            p = p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1));
            if !spell_iswordp(p, curwin.get()) {
                break;
            }
            col = 0;
        }

        col
    }
}

/// Whether the word [`expand_spelling`] is about to suggest for needs a
/// capital.
///
/// The word is deleted from the buffer before [`expand_spelling`] runs, so
/// the answer has to be taken beforehand and parked here.
static spell_expand_need_cap: GlobalCell<bool> = GlobalCell::new(false);

/// Record, before the word is removed, whether its replacement will need a
/// capital.
pub unsafe fn spell_expand_check_cap(col: colnr_T) {
    unsafe {
        spell_expand_need_cap.set(check_need_cap(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            col,
        ));
    }
}

/// Insert-mode completion `CTRL-X ?`: fill `matchp` with suggestions for
/// `pat` and return how many there are.
pub unsafe fn expand_spelling(
    _lnum: linenr_T,
    pat: *mut c_char,
    matchp: *mut *mut *mut c_char,
) -> c_int {
    unsafe {
        let mut ga: garray_T = mem::zeroed();
        spell_suggest_list(&raw mut ga, pat, 100, spell_expand_need_cap.get(), true);
        *matchp = ga.ga_data as *mut *mut c_char;
        ga.ga_len
    }
}
