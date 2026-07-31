//! Spelling suggestions: what to offer instead of a badly spelled word.
//!
//! One run starts with a bad word and ends with a scored, sorted list of
//! replacements. [`spell_find_suggest`] is the top of it: it fills a
//! [`suginfo_T`] with everything the search needs — the bad word as typed,
//! case-folded and sound-folded, its capitalisation flags and the score
//! ceiling — and then walks `'spellsuggest'` to decide which methods run.
//!
//! `'spellsuggest'` is a comma-separated list, and each item is either a
//! source of suggestions or a knob on the internal one:
//!
//! - `best`, `fast` and `double` choose how the internal search scores
//!   what it finds. [`spell_check_sps`] turns them into [`sps_flags`],
//!   which [`spell_suggest_intern`] then reads.
//! - `expr:{expr}` and `file:{fname}` are outside sources, handled by
//!   [`spell_suggest_expr`] and [`spell_suggest_file`].
//! - `timeout:{ms}` bounds how long the trie walk may run for.
//! - A bare number caps how many suggestions `z=` lists.
//!
//! The two entry points are [`spell_suggest`], the interactive `z=`
//! command, and [`spell_suggest_list`], which is what the `spellsuggest()`
//! Vimscript function and `:spellrepall` are built on.
//!
//! # The submodules
//!
//! - [`prompt`] — the `z=` command itself: find the bad word, list the
//!   numbered choices, ask, and make the replacement.
//! - [`walk`] — the edit-distance search, which walks the language's word
//!   tree trying insertions, deletions, swaps and `REP` items.
//! - [`soundalike`] — the search over the sound-folded tree, which finds
//!   words that are spelled differently but sound the same.
//! - [`score`] — how far one word is from another, by letters and by
//!   sound.
//! - [`collect`] — keeping, deduplicating, rescoring and sorting the
//!   candidates the two searches produce.

#![deny(unsafe_op_in_unsafe_fn)]

mod collect;
mod prompt;
mod score;
mod soundalike;
mod walk;

pub use prompt::spell_suggest;

use crate::src::nvim::charset::{getdigits_int, skiptowhite, skipwhite};
use crate::src::nvim::eval::typval::tv_list_unref;
use crate::src::nvim::eval::vars::{eval_spell_expr, get_spellword};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::getchar::vgetc;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::{hash_clear_all, hash_init};
use crate::src::nvim::main::{curbuf, curwin, e_notopen, got_int, p_sps};
use crate::src::nvim::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz, xstrdup};
use crate::src::nvim::message::semsg;
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{atoi, fclose, gettext, strcasecmp, strcpy, strlen, strncmp};
use crate::src::nvim::spell::{
    captype, make_case_word, spell_casefold, spell_check, spell_soundfold,
};
use crate::src::nvim::spellfile::suggest_load_files;
use crate::src::nvim::spellsuggest::collect::{
    add_banned, add_suggestion, check_suggestions, clean_count, cleanup_suggestions,
    rescore_suggestions, score_combine, score_comp_sal, suggestions,
};
use crate::src::nvim::spellsuggest::score::spell_isupper;
use crate::src::nvim::spellsuggest::soundalike::{
    suggest_try_soundalike, suggest_try_soundalike_finish, suggest_try_soundalike_prep,
};
use crate::src::nvim::spellsuggest::walk::suggest_trie_walk;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{FILE, VarType, garray_T, hashtab_T, hlf_T, langp_T, slang_T, smt_T};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

/// The longest word the spell code handles, and so the size of every word
/// buffer in this module tree.
pub const MAXWLEN: usize = 254;

/// The string terminator, as an `int` so that it can be compared against a
/// widened byte.
pub const NUL: c_int = 0;
/// A tab, which sound folding treats as a space.
pub const TAB: c_int = '\t' as c_int;

/// The longest path, which is also the size of the scratch buffer one
/// `'spellsuggest'` item is copied into.
const MAXPATHL: usize = 4096;

/// `spell_check_sps`'s two answers, as the option code expects them.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// "No highlight", which `spell_check` leaves in place when it finds
/// nothing wrong with the word.
pub const HLF_COUNT: hlf_T = 76;
/// Move to any kind of spelling mistake, not just bad or rare words.
const SMT_ALL: smt_T = 0;
/// A `v:t_list` typval, which is what a `'spellsuggest'` expression must
/// yield one of per suggestion.
const VAR_LIST: VarType = 4;

// Word flags. These live beside each word in the tree, except for
// `WF_MIXCAP`, which only ever appears in `su_badflags`.
/// The word is only valid in some regions.
pub const WF_REGION: c_int = 0x01;
/// The word starts with a capital.
pub const WF_ONECAP: c_int = 0x02;
/// The word is all capitals.
pub const WF_ALLCAP: c_int = 0x04;
/// The word is rare.
pub const WF_RARE: c_int = 0x08;
/// The word must never be suggested.
pub const WF_BANNED: c_int = 0x10;
/// A mix of upper and lower case, "macaRONI". Only used for
/// `su_badflags`.
pub const WF_MIXCAP: c_int = 0x20;
/// The word's case is exactly as spelled and cannot be reconstructed from
/// the case-folded tree.
pub const WF_KEEPCAP: c_int = 0x80;
/// Every case bit together: `ONECAP | ALLCAP | FIXCAP | KEEPCAP`.
pub const WF_CAPMASK: c_int = 0xc6;
/// The word only counts as part of a compound.
pub const WF_NEEDCOMP: c_int = 0x200;
/// The word is never offered as a suggestion.
pub const WF_NOSUGGEST: c_int = 0x400;
/// The prefix makes the word rare.
pub const WF_RAREPFX: c_int = 0x1000000;

// What each kind of change costs. A suggestion's score is the sum over
// the changes that reach it, and lower is offered first.
/// Split the bad word in two.
pub const SCORE_SPLIT: c_int = 149;
/// Split it where the language says not to (`NOSPLITSUGS`).
pub const SCORE_SPLIT_NO: c_int = 249;
/// Only the case differs.
pub const SCORE_ICASE: c_int = 52;
/// The word belongs to another region.
pub const SCORE_REGION: c_int = 200;
/// The word is marked rare.
pub const SCORE_RARE: c_int = 180;
/// Swap two characters.
pub const SCORE_SWAP: c_int = 75;
/// Swap two characters that have a third between them.
pub const SCORE_SWAP3: c_int = 110;
/// Apply one `REP` item from the `.aff` file.
pub const SCORE_REP: c_int = 65;
/// Substitute a character.
pub const SCORE_SUBST: c_int = 93;
/// Substitute a character the language's `MAP` lines call similar.
pub const SCORE_SIMILAR: c_int = 33;
/// Substitute a composing character.
pub const SCORE_SUBCOMP: c_int = 33;
/// Delete a character.
pub const SCORE_DEL: c_int = 94;
/// Delete one of two identical characters.
pub const SCORE_DELDUP: c_int = 66;
/// Delete a composing character.
pub const SCORE_DELCOMP: c_int = 28;
/// Insert a character.
pub const SCORE_INS: c_int = 96;
/// Insert a character that duplicates its neighbour.
pub const SCORE_INSDUP: c_int = 67;
/// Insert a composing character.
pub const SCORE_INSCOMP: c_int = 30;
/// Turn a non-word character into a word character.
pub const SCORE_NONWORD: c_int = 103;

/// A suggestion that came out of a `file:` item.
pub const SCORE_FILE: c_int = 30;
/// The score ceiling a run starts with. Higher means slower; this allows
/// about three changes.
pub const SCORE_MAXINIT: c_int = 350;

// Discounts for words the dictionary has seen before, and the word counts
// that earn them.
pub const SCORE_COMMON1: c_int = 30;
pub const SCORE_COMMON2: c_int = 40;
pub const SCORE_COMMON3: c_int = 50;
pub const SCORE_THRES2: c_int = 10;
pub const SCORE_THRES3: c_int = 100;

// Trying changed sound-folded words gets slow past two changes, and
// stopping at one misses a few good suggestions, so the sound-a-like pass
// runs up to three times with a rising ceiling.
pub const SCORE_SFMAX1: c_int = 200;
pub const SCORE_SFMAX2: c_int = 300;
pub const SCORE_SFMAX3: c_int = 400;

/// Any score at all; used where a score could not be computed.
pub const SCORE_MAXMAX: c_int = 999999;
/// Past this, `spell_edit_score_limit`'s depth-first search costs more
/// than the full table would.
pub const SCORE_LIMITMAX: c_int = 350;

// Values for `sps_flags`, one per `'spellsuggest'` method.
/// Weigh the sound-a-like score into the final order.
pub const SPS_BEST: c_int = 1;
/// Skip the sound-a-like search entirely.
pub const SPS_FAST: c_int = 2;
/// Score the two searches separately and interleave the results.
pub const SPS_DOUBLE: c_int = 4;

/// What is known while looking for suggestions.
#[repr(C)]
pub struct suginfo_T {
    /// The suggestions found so far, a garray of [`suggest_T`].
    pub su_ga: garray_T,
    /// How many suggestions will be displayed.
    pub su_maxcount: c_int,
    /// The score ceiling for adding to `su_ga`.
    pub su_maxscore: c_int,
    /// The same, while working on sound-folded words.
    pub su_sfmaxscore: c_int,
    /// Like `su_ga`, but scored by sound; only used in "double" mode.
    pub su_sga: garray_T,
    /// Where the bad word starts, in the line it came from.
    pub su_badptr: *mut c_char,
    /// How much of that line the bad word covers.
    pub su_badlen: c_int,
    /// The bad word's capitalisation, as `WF_` bits.
    pub su_badflags: c_int,
    /// The bad word, truncated at `su_badlen`.
    pub su_badword: [c_char; MAXWLEN],
    /// `su_badword`, case-folded.
    pub su_fbadword: [c_char; MAXWLEN],
    /// `su_badword`, sound-folded.
    pub su_sal_badword: [c_char; MAXWLEN],
    /// Words that must never be suggested.
    pub su_banned: hashtab_T,
    /// The language sound folding defaults to.
    pub su_sallang: *mut slang_T,
}

/// One suggestion.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct suggest_T {
    /// The suggested word, an allocated string this entry owns.
    pub st_word: *mut c_char,
    /// `strlen(st_word)`.
    pub st_wordlen: c_int,
    /// How much of the bad word it replaces.
    pub st_orglen: c_int,
    /// Lower is better.
    pub st_score: c_int,
    /// The tie-breaker when `st_score` compares equal.
    pub st_altscore: c_int,
    /// `st_score` is a sound-a-like score.
    pub st_salscore: bool,
    /// The sound-a-like bonus is already in `st_score`.
    pub st_had_bonus: bool,
    /// The language the word was sound-folded with.
    pub st_slang: *mut slang_T,
}

/// How long the trie walk may run for, in milliseconds; `timeout:` in
/// `'spellsuggest'` sets it, and a fresh `z=` resets it.
pub(crate) static spell_suggest_timeout: GlobalCell<c_int> = GlobalCell::new(5000);

/// Which internal method `'spellsuggest'` asks for, as `SPS_` bits.
pub(crate) static sps_flags: GlobalCell<c_int> = GlobalCell::new(SPS_BEST);
/// How many suggestions `z=` may list; the number in `'spellsuggest'`.
static sps_limit: GlobalCell<c_int> = GlobalCell::new(9999);

/// The spell languages the current window has loaded, in the order
/// `'spelllang'` put them in.
///
/// # Safety
///
/// The current window must have its spell state set up, which it has
/// whenever `'spell'` is on.
pub unsafe fn window_langs<'a>() -> &'a mut [langp_T] {
    // SAFETY: the caller guarantees the window's spell state; an empty
    // garray has a null data pointer, which `from_raw_parts_mut` rejects
    // even at length zero.
    unsafe {
        let gap = &raw const (*(*curwin.get()).w_s).b_langp;
        if (*gap).ga_data.is_null() || (*gap).ga_len <= 0 {
            &mut []
        } else {
            ::core::slice::from_raw_parts_mut(
                (*gap).ga_data as *mut langp_T,
                (*gap).ga_len as usize,
            )
        }
    }
}

/// The capitalisation of a bad word, for reproducing it in the
/// replacement.
///
/// Like `captype`, except that a `KEEPCAP` word is looked at more closely
/// so that `make_case_word` can turn "WOrd" into "Word" and "WOrD" into
/// "WORD".
///
/// # Safety
///
/// `word` and `end` must bound one word of a live line.
pub(crate) unsafe fn badword_captype(word: *mut c_char, end: *mut c_char) -> c_int {
    // SAFETY: the caller guarantees the word; the scan steps whole
    // characters and stops at `end`.
    unsafe {
        let mut flags = captype(word, end);
        if flags & WF_KEEPCAP == 0 {
            return flags;
        }

        // Count the upper and lower case letters.
        let (mut lower, mut upper) = (0, 0);
        let mut first = false;
        let mut p = word;
        while p < end {
            if spell_isupper(utf_ptr2char(p)) {
                upper += 1;
                if p == word {
                    first = true;
                }
            } else {
                lower += 1;
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }

        // More upper than lower case suggests an all-capitals word;
        // otherwise a leading capital suggests one capital. "ALl" is most
        // likely "All", hence the three-capital floor.
        if upper > lower && upper > 2 {
            flags |= WF_ALLCAP;
        } else if first {
            flags |= WF_ONECAP;
        }
        if upper >= 2 && lower >= 2 {
            // maCARONI, maCAroni
            flags |= WF_MIXCAP;
        }
        flags
    }
}

/// Does a `timeout:` item name a number? The value may be negative, which
/// switches the timeout off.
fn is_timeout_value(value: &[u8]) -> bool {
    let digits = value.strip_prefix(b"-").unwrap_or(value);
    digits.first().is_some_and(u8::is_ascii_digit)
}

/// Check `'spellsuggest'` and set [`sps_flags`] and [`sps_limit`] from it.
///
/// Returns `FAIL` for a value the option should not take, having put both
/// back to their defaults.
///
/// # Safety
///
/// `'spellsuggest'` must hold a NUL-terminated string.
pub unsafe fn spell_check_sps() -> c_int {
    // SAFETY: the caller guarantees the option; `buf` is `MAXPATHL`, which
    // is what `copy_option_part` is told it may fill.
    unsafe {
        let mut buf = [0 as c_char; MAXPATHL];
        let bufp = buf.as_mut_ptr();

        sps_flags.set(0);
        sps_limit.set(9999);

        let mut p = p_sps.get();
        while *p as c_int != NUL {
            copy_option_part(&raw mut p, bufp, MAXPATHL, c",".as_ptr().cast_mut());
            let part = CStr::from_ptr(bufp).to_bytes();

            // Zero means "this item said nothing about the method", -1
            // means "this item is not a valid one".
            let mut f = 0;
            if part.first().is_some_and(u8::is_ascii_digit) {
                let mut s = bufp;
                sps_limit.set(getdigits_int(&raw mut s, true, 0));
                if *s as c_int != NUL && !(*s as u8).is_ascii_digit() {
                    f = -1;
                }
            // Keep the three names in sync with `opt_sps_values`.
            } else if part == b"best" {
                f = SPS_BEST;
            } else if part == b"fast" {
                f = SPS_FAST;
            } else if part == b"double" {
                f = SPS_DOUBLE;
            } else if !part.starts_with(b"expr:")
                && !part.starts_with(b"file:")
                && !part
                    .strip_prefix(b"timeout:".as_slice())
                    .is_some_and(is_timeout_value)
            {
                f = -1;
            }

            // Only one method may be named.
            if f == -1 || (sps_flags.get() != 0 && f != 0) {
                sps_flags.set(SPS_BEST);
                sps_limit.set(9999);
                return FAIL;
            }
            if f != 0 {
                sps_flags.set(f);
            }
        }

        if sps_flags.get() == 0 {
            sps_flags.set(SPS_BEST);
        }
        OK
    }
}

/// Find suggestions for `word` and return them in `gap` as a list of
/// allocated strings.
///
/// This is what the `spellsuggest()` Vimscript function is built on.
///
/// # Safety
///
/// `gap` must be an uninitialised garray, `word` NUL-terminated, and the
/// current window must have its languages loaded.
pub unsafe fn spell_suggest_list(
    gap: *mut garray_T,
    word: *mut c_char,
    maxcount: c_int,
    need_cap: bool,
    interactive: bool,
) {
    // SAFETY: the caller guarantees the pointers; each string built below
    // is sized from the two pieces copied into it.
    unsafe {
        let mut sug: suginfo_T = mem::zeroed();
        spell_find_suggest(
            word,
            0,
            &raw mut sug,
            maxcount,
            false,
            need_cap,
            interactive,
        );

        ga_init(
            gap,
            mem::size_of::<*mut c_char>() as c_int,
            sug.su_ga.ga_len + 1,
        );
        ga_grow(gap, sug.su_ga.ga_len);
        for stp in suggestions(&raw mut sug.su_ga) {
            // A suggestion may replace only part of `word`; what it does
            // not replace goes on the end.
            let tail = sug.su_badptr.offset(stp.st_orglen as isize);
            let wcopy = xmalloc(stp.st_wordlen as usize + strlen(tail) as usize + 1) as *mut c_char;
            strcpy(wcopy, stp.st_word);
            strcpy(wcopy.offset(stp.st_wordlen as isize), tail);
            *((*gap).ga_data as *mut *mut c_char).offset((*gap).ga_len as isize) = wcopy;
            (*gap).ga_len += 1;
        }

        spell_find_cleanup(&raw mut sug);
    }
}

/// Find suggestions for the word at the start of `badptr` and leave them
/// in `su->su_ga`.
///
/// `badlen` is how much of the line the bad word covers, or 0 to let the
/// spell checker decide. `banbadword` keeps the bad word itself out of the
/// list, and `need_cap` says `'spellcapcheck'` wants a capital.
///
/// The mechanism is Aspell's, reimplemented.
///
/// # Safety
///
/// `badptr` must be NUL-terminated and stay live for the whole call, `su`
/// must be writable, and the current window must have its languages
/// loaded.
#[allow(clippy::too_many_arguments)]
unsafe fn spell_find_suggest(
    badptr: *mut c_char,
    badlen: c_int,
    su: *mut suginfo_T,
    maxcount: c_int,
    banbadword: bool,
    need_cap: bool,
    interactive: bool,
) {
    // SAFETY: the caller guarantees the pointers; every copy into `su`'s
    // buffers is bounded by `MAXWLEN` and `su_badlen` is clamped below it.
    unsafe {
        // A `expr:` item may itself call `spellsuggest()`, which lands
        // back here; the inner call must not evaluate the expression
        // again.
        static EXPR_BUSY: GlobalCell<bool> = GlobalCell::new(false);

        let mut attr: hlf_T = HLF_COUNT;
        let mut buf = [0 as c_char; MAXPATHL];
        let bufp = buf.as_mut_ptr();

        ptr::write_bytes(su, 0, 1);
        ga_init(
            &raw mut (*su).su_ga,
            mem::size_of::<suggest_T>() as c_int,
            10,
        );
        ga_init(
            &raw mut (*su).su_sga,
            mem::size_of::<suggest_T>() as c_int,
            10,
        );
        if *badptr as c_int == NUL {
            return;
        }
        hash_init(&raw mut (*su).su_banned);

        (*su).su_badptr = badptr;
        (*su).su_badlen = if badlen != 0 {
            badlen
        } else {
            spell_check(curwin.get(), badptr, &raw mut attr, ptr::null_mut(), false) as c_int
        };
        (*su).su_maxcount = maxcount;
        (*su).su_maxscore = SCORE_MAXINIT;
        (*su).su_badlen = (*su).su_badlen.min(MAXWLEN as c_int - 1); // just in case
        xmemcpyz(
            &raw mut (*su).su_badword as *mut c_char as *mut c_void,
            badptr as *const c_void,
            (*su).su_badlen as usize,
        );
        spell_casefold(
            curwin.get(),
            badptr,
            (*su).su_badlen,
            &raw mut (*su).su_fbadword as *mut c_char,
            MAXWLEN as c_int,
        );
        // Upstream note: this breaks if the case-folded text comes out
        // longer than the original, because an illegal byte then throws
        // the pointer arithmetic off.
        (*su).su_fbadword[(*su).su_badlen as usize] = NUL as c_char;

        (*su).su_badflags = badword_captype(badptr, badptr.offset((*su).su_badlen as isize));
        if need_cap {
            (*su).su_badflags |= WF_ONECAP;
        }

        // Sound-fold with the first language in 'spelllang' that can. That
        // is right for several files of one language and not too bad for a
        // mixture like "pl,en". Note this is the buffer's list of
        // languages rather than the window's.
        let langp = &raw const (*curbuf.get()).b_s.b_langp;
        for i in 0..(*langp).ga_len {
            let lp = ((*langp).ga_data as *mut langp_T).offset(i as isize);
            if !(*lp).lp_sallang.is_null() {
                (*su).su_sallang = (*lp).lp_sallang;
                break;
            }
        }
        if !(*su).su_sallang.is_null() {
            // Once here, rather than once per candidate.
            spell_soundfold(
                (*su).su_sallang,
                &raw mut (*su).su_fbadword as *mut c_char,
                true,
                &raw mut (*su).su_sal_badword as *mut c_char,
            );
        }

        // A word the spell checker is happy with, spelled lower case, may
        // simply be missing its capital.
        if !spell_isupper(utf_ptr2char(badptr)) && attr == HLF_COUNT {
            make_case_word(&raw mut (*su).su_badword as *mut c_char, bufp, WF_ONECAP);
            add_suggestion(
                su,
                &raw mut (*su).su_ga,
                bufp,
                (*su).su_badlen,
                SCORE_ICASE,
                0,
                true,
                (*su).su_sallang,
                false,
            );
        }

        // Ban the bad word itself; it may be valid in another region.
        if banbadword {
            add_banned(su, &raw mut (*su).su_badword as *mut c_char);
        }

        // An expression may change 'spellsuggest' while it runs.
        let sps_copy = xstrdup(p_sps.get());
        let mut do_combine = false;
        let mut did_intern = false;
        let mut p = sps_copy;
        while *p as c_int != NUL {
            copy_option_part(&raw mut p, bufp, MAXPATHL, c",".as_ptr().cast_mut());
            let part = CStr::from_ptr(bufp).to_bytes();

            if part.starts_with(b"expr:") {
                if !EXPR_BUSY.get() {
                    EXPR_BUSY.set(true);
                    spell_suggest_expr(su, bufp.add(5));
                    EXPR_BUSY.set(false);
                }
            } else if part.starts_with(b"file:") {
                spell_suggest_file(su, bufp.add(5));
            } else if part.starts_with(b"timeout:") {
                spell_suggest_timeout.set(atoi(bufp.add(8)));
            } else if !did_intern {
                // The internal method runs at most once.
                spell_suggest_intern(su, interactive);
                do_combine = sps_flags.get() & SPS_DOUBLE != 0;
                did_intern = true;
            }
        }
        xfree(sps_copy as *mut c_void);

        if do_combine {
            // Last, because sorting would undo the interleaving.
            score_combine(su);
        }
    }
}

/// Find suggestions by evaluating `expr`, the `expr:` item of
/// `'spellsuggest'`.
///
/// # Safety
///
/// `su` must be valid and `expr` NUL-terminated.
unsafe fn spell_suggest_expr(su: *mut suginfo_T, expr: *mut c_char) {
    // SAFETY: the caller guarantees the pointers; the list the expression
    // returns is owned here until it is unreferenced.
    unsafe {
        // The work is split up so that `suginfo_T` need not be exported to
        // the evaluator.
        let list = eval_spell_expr(&raw mut (*su).su_badword as *mut c_char, expr);
        if !list.is_null() {
            let mut li = (*list).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type == VAR_LIST {
                    // Each item is a [word, score] pair.
                    let mut word: *const c_char = ptr::null();
                    let score = get_spellword((*li).li_tv.vval.v_list, &raw mut word);
                    if score >= 0 && score <= (*su).su_maxscore {
                        add_suggestion(
                            su,
                            &raw mut (*su).su_ga,
                            word,
                            (*su).su_badlen,
                            score,
                            0,
                            true,
                            (*su).su_sallang,
                            false,
                        );
                    }
                }
                li = (*li).li_next;
            }
            tv_list_unref(list);
        }

        check_suggestions(su, &raw mut (*su).su_ga);
        cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
    }
}

/// Find suggestions in `fname`, the `file:` item of `'spellsuggest'`.
///
/// Every line of the file is `badword/goodword`.
///
/// # Safety
///
/// `su` must be valid and `fname` NUL-terminated.
unsafe fn spell_suggest_file(su: *mut suginfo_T, fname: *mut c_char) {
    // SAFETY: the caller guarantees the pointers; `line` is what
    // `vim_fgets` is told its size is, and the good word is terminated
    // inside it before it is used.
    unsafe {
        let fd: *mut FILE = os_fopen(fname, c"r".as_ptr());
        if fd.is_null() {
            semsg(gettext(&raw const e_notopen as *const c_char), fname);
            return;
        }

        let mut line = [0 as c_char; MAXWLEN * 2];
        let mut cword = [0 as c_char; MAXWLEN];
        let linep = line.as_mut_ptr();
        let cwordp = cword.as_mut_ptr();
        while !vim_fgets(linep, (MAXWLEN * 2) as c_int, fd) && !got_int.get() {
            line_breakcheck();

            let mut p = vim_strchr(linep, '/' as c_int);
            if p.is_null() {
                continue; // no separator, so not an entry
            }
            *p = NUL as c_char;
            p = p.add(1);
            if strcasecmp(&raw const (*su).su_badword as *const c_char, linep) != 0 {
                continue;
            }

            // A match: the good word runs to the CR or NL.
            let mut len = 0isize;
            while *p.offset(len) as u8 >= b' ' {
                len += 1;
            }
            *p.offset(len) = NUL as c_char;

            // A suggestion with no case of its own takes the bad word's.
            if captype(p, ptr::null()) == 0 {
                make_case_word(p, cwordp, (*su).su_badflags);
                p = cwordp;
            }

            add_suggestion(
                su,
                &raw mut (*su).su_ga,
                p,
                (*su).su_badlen,
                SCORE_FILE,
                0,
                true,
                (*su).su_sallang,
                false,
            );
        }
        fclose(fd);

        check_suggestions(su, &raw mut (*su).su_ga);
        cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
    }
}

/// Run the internal search, in whichever form [`sps_flags`] asks for.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
unsafe fn spell_suggest_intern(su: *mut suginfo_T, interactive: bool) {
    // SAFETY: the caller guarantees `su` and the window's spell state.
    unsafe {
        // Load whichever `.sug` files are available and not loaded yet.
        suggest_load_files();

        // 1. Special cases, such as a repeated word: "the the" -> "the".
        suggest_try_special(su);

        // 2. Inserting, deleting, swapping and changing letters, `REP`
        //    items from the `.aff` file, and splitting the word in two.
        suggest_try_change(su);

        // Give the top scorers a sound-a-like score to interleave on.
        if sps_flags.get() & SPS_DOUBLE != 0 {
            score_comp_sal(su);
        }

        // 3. Words that sound alike.
        if sps_flags.get() & SPS_FAST == 0 {
            if sps_flags.get() & SPS_BEST != 0 {
                rescore_suggestions(su);
            }

            // Through the sound-fold tree `su_maxscore` bounds the changes
            // tried to the sound-folded word, while `su_sfmaxscore` bounds
            // the rescored result. Small edit distances come first because
            // they are much faster and usually enough; only if too little
            // turns up is a wider search worth its time. `sl_sounddone`
            // keeps the passes from redoing each other's work.
            suggest_try_soundalike_prep();
            (*su).su_maxscore = SCORE_SFMAX1;
            (*su).su_sfmaxscore = SCORE_MAXINIT * 3;
            suggest_try_soundalike(su);
            for ceiling in [SCORE_SFMAX2, SCORE_SFMAX3] {
                if (*su).su_ga.ga_len >= clean_count(&*su) {
                    break;
                }
                (*su).su_maxscore = ceiling;
                suggest_try_soundalike(su);
            }
            (*su).su_maxscore = (*su).su_sfmaxscore;
            suggest_try_soundalike_finish();
        }

        // Interrupted searches still show what they found. `got_int` is
        // only cleared for a command, not for `spellsuggest()`.
        os_breakcheck();
        if interactive && got_int.get() {
            vgetc();
            got_int.set(false);
        }

        if sps_flags.get() & SPS_DOUBLE == 0 && (*su).su_ga.ga_len != 0 {
            if sps_flags.get() & SPS_BEST != 0 {
                rescore_suggestions(su);
            }
            check_suggestions(su, &raw mut (*su).su_ga);
            cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
        }
    }
}

/// Release everything [`spell_find_suggest`] put in `su`.
///
/// # Safety
///
/// `su` must have been filled by [`spell_find_suggest`].
unsafe fn spell_find_cleanup(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees `su`; each suggestion owns its word
    // and the banned table owns its keys.
    unsafe {
        for gap in [&raw mut (*su).su_ga, &raw mut (*su).su_sga] {
            for stp in suggestions(gap) {
                xfree(stp.st_word as *mut c_void);
            }
            ga_clear(gap);
        }
        hash_clear_all(&raw mut (*su).su_banned, 0);
    }
}

/// Find suggestions by recognising specific situations.
///
/// There is only one: a word typed twice, "the the", whose suggestion is
/// the word once.
///
/// # Safety
///
/// `su` must be valid.
unsafe fn suggest_try_special(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees `su`; the terminator planted in
    // `su_fbadword` is inside the buffer and is put back straight away.
    unsafe {
        let fbadword = &raw mut (*su).su_fbadword as *mut c_char;
        let mut p = skiptowhite(fbadword);
        let len = p.offset_from(fbadword) as usize;
        p = skipwhite(p);
        if strlen(p) as usize != len || strncmp(fbadword, p, len) != 0 {
            return;
        }

        // Take the bad word's case with it, so that "The the" -> "The".
        let mut word = [0 as c_char; MAXWLEN];
        let wordp = word.as_mut_ptr();
        let saved = (*su).su_fbadword[len];
        (*su).su_fbadword[len] = NUL as c_char;
        make_case_word(fbadword, wordp, (*su).su_badflags);
        (*su).su_fbadword[len] = saved;

        // Score it as one deletion, with a sound-a-like score of zero.
        add_suggestion(
            su,
            &raw mut (*su).su_ga,
            wordp,
            (*su).su_badlen,
            3 * SCORE_REP / 4,
            0,
            true,
            (*su).su_sallang,
            false,
        );
    }
}

/// Find suggestions by adding, removing and swapping letters, in every
/// language the window has loaded.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
unsafe fn suggest_try_change(su: *mut suginfo_T) {
    // SAFETY: the caller guarantees `su` and the window's spell state;
    // `fword` is `MAXWLEN` and every write into it is told so.
    unsafe {
        // The walk rewrites the case-folded bad word in place (for `REP`
        // items especially), so it gets a copy. What follows the bad word
        // is appended: changing characters after it may help.
        let mut fword = [0 as c_char; MAXWLEN];
        let fwordp = fword.as_mut_ptr();
        strcpy(fwordp, &raw const (*su).su_fbadword as *const c_char);
        let n = strlen(fwordp) as c_int;
        let tail = (*su).su_badptr.offset((*su).su_badlen as isize);
        spell_casefold(
            curwin.get(),
            tail,
            strlen(tail) as c_int,
            fwordp.offset(n as isize),
            MAXWLEN as c_int - n,
        );

        // Keep the result no longer than the original text.
        let n = strlen((*su).su_badptr) as usize;
        if n < MAXWLEN {
            fword[n] = NUL as c_char;
        }

        for lp in window_langs() {
            // A spell file that failed to reload is still in the list, but
            // everything in it has been cleared.
            if !(*lp.lp_slang).sl_fbyts.is_null() {
                suggest_trie_walk(su, lp, fwordp, false);
            }
        }
    }
}
