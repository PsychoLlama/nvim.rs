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
//! - [`sps`] — the `'spellsuggest'` option itself: parsing its items, and
//!   the two outside sources (`expr:`, `file:`) they can name.

#![deny(unsafe_op_in_unsafe_fn)]

mod collect;
mod prompt;
mod score;
mod soundalike;
mod sps;
mod walk;

use crate::winlayer::{Live, Win};
pub(crate) use prompt::spell_suggest;
pub(crate) use sps::spell_check_sps;
use sps::{spell_suggest_expr, spell_suggest_file};

use crate::charset::{skiptowhite, skipwhite};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::getchar::vgetc;
use crate::global_cell::GlobalCell;
use crate::hashtab::{hash_clear_all, hash_init};
use crate::main::{curbuf, curwin, got_int, p_sps};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memory::{xfree, xmalloc, xmemcpyz, xstrdup};
use crate::option::copy_option_part;
use crate::os::cshim::strncmp;
use crate::os::input::os_breakcheck;
use crate::spell::{captype, make_case_word, spell_casefold, spell_check, spell_soundfold};
use crate::spellfile::suggest_load_files;
use crate::spellsuggest::collect::{
    add_banned, add_suggestion, check_suggestions, clean_count, cleanup_suggestions,
    rescore_suggestions, score_combine, score_comp_sal, suggestions,
};
use crate::spellsuggest::score::spell_isupper;
use crate::spellsuggest::soundalike::{
    suggest_try_soundalike, suggest_try_soundalike_finish, suggest_try_soundalike_prep,
};
use crate::spellsuggest::walk::suggest_trie_walk;
use crate::types::{MAXPATHL, NUL, garray_T, hashtab_T, hlf_T, langp_T, slang_T};
use ::libc::{atoi, strcpy, strlen};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::offset_of;
use core::{mem, ptr};

use crate::highlight_group::HLF_COUNT;
/// The longest word the spell code handles, and so the size of every word
/// buffer in this module tree.
pub(crate) use crate::spell::MAXWLEN;

/// A tab, which sound folding treats as a space.
pub(crate) const TAB: c_int = '\t' as c_int;

// Word flags. These live beside each word in the tree, except for
// `WF_MIXCAP`, which only ever appears in `su_badflags`.
/// The word is only valid in some regions.
pub(crate) const WF_REGION: c_int = 0x01;
/// The word starts with a capital.
pub(crate) const WF_ONECAP: c_int = 0x02;
/// The word is all capitals.
pub(crate) const WF_ALLCAP: c_int = 0x04;
/// The word is rare.
pub(crate) const WF_RARE: c_int = 0x08;
/// The word must never be suggested.
pub(crate) const WF_BANNED: c_int = 0x10;
/// A mix of upper and lower case, "macaRONI". Only used for
/// `su_badflags`.
pub(crate) const WF_MIXCAP: c_int = 0x20;
/// The word's case is exactly as spelled and cannot be reconstructed from
/// the case-folded tree.
pub(crate) const WF_KEEPCAP: c_int = 0x80;
/// Every case bit together: `ONECAP | ALLCAP | FIXCAP | KEEPCAP`.
pub(crate) const WF_CAPMASK: c_int = 0xc6;
/// The word only counts as part of a compound.
pub(crate) const WF_NEEDCOMP: c_int = 0x200;
/// The word is never offered as a suggestion.
pub(crate) const WF_NOSUGGEST: c_int = 0x400;
/// The prefix makes the word rare.
pub(crate) const WF_RAREPFX: c_int = 0x1000000;

// What each kind of change costs. A suggestion's score is the sum over
// the changes that reach it, and lower is offered first.
/// Split the bad word in two.
pub(crate) const SCORE_SPLIT: c_int = 149;
/// Split it where the language says not to (`NOSPLITSUGS`).
pub(crate) const SCORE_SPLIT_NO: c_int = 249;
/// Only the case differs.
pub(crate) const SCORE_ICASE: c_int = 52;
/// The word belongs to another region.
pub(crate) const SCORE_REGION: c_int = 200;
/// The word is marked rare.
pub(crate) const SCORE_RARE: c_int = 180;
/// Swap two characters.
pub(crate) const SCORE_SWAP: c_int = 75;
/// Swap two characters that have a third between them.
pub(crate) const SCORE_SWAP3: c_int = 110;
/// Apply one `REP` item from the `.aff` file.
pub(crate) const SCORE_REP: c_int = 65;
/// Substitute a character.
pub(crate) const SCORE_SUBST: c_int = 93;
/// Substitute a character the language's `MAP` lines call similar.
pub(crate) const SCORE_SIMILAR: c_int = 33;
/// Substitute a composing character.
pub(crate) const SCORE_SUBCOMP: c_int = 33;
/// Delete a character.
pub(crate) const SCORE_DEL: c_int = 94;
/// Delete one of two identical characters.
pub(crate) const SCORE_DELDUP: c_int = 66;
/// Delete a composing character.
pub(crate) const SCORE_DELCOMP: c_int = 28;
/// Insert a character.
pub(crate) const SCORE_INS: c_int = 96;
/// Insert a character that duplicates its neighbour.
pub(crate) const SCORE_INSDUP: c_int = 67;
/// Insert a composing character.
pub(crate) const SCORE_INSCOMP: c_int = 30;
/// Turn a non-word character into a word character.
pub(crate) const SCORE_NONWORD: c_int = 103;

/// A suggestion that came out of a `file:` item.
pub(crate) const SCORE_FILE: c_int = 30;
/// The score ceiling a run starts with. Higher means slower; this allows
/// about three changes.
pub(crate) const SCORE_MAXINIT: c_int = 350;

// Discounts for words the dictionary has seen before, and the word counts
// that earn them.
pub(crate) const SCORE_COMMON1: c_int = 30;
pub(crate) const SCORE_COMMON2: c_int = 40;
pub(crate) const SCORE_COMMON3: c_int = 50;
pub(crate) const SCORE_THRES2: c_int = 10;
pub(crate) const SCORE_THRES3: c_int = 100;

// Trying changed sound-folded words gets slow past two changes, and
// stopping at one misses a few good suggestions, so the sound-a-like pass
// runs up to three times with a rising ceiling.
pub(crate) const SCORE_SFMAX1: c_int = 200;
pub(crate) const SCORE_SFMAX2: c_int = 300;
pub(crate) const SCORE_SFMAX3: c_int = 400;

/// Any score at all; used where a score could not be computed.
pub(crate) const SCORE_MAXMAX: c_int = 999999;
/// Past this, `spell_edit_score_limit`'s depth-first search costs more
/// than the full table would.
pub(crate) const SCORE_LIMITMAX: c_int = 350;

// Values for `sps_flags`, one per `'spellsuggest'` method.
/// Weigh the sound-a-like score into the final order.
pub(crate) const SPS_BEST: c_int = 1;
/// Skip the sound-a-like search entirely.
pub(crate) const SPS_FAST: c_int = 2;
/// Score the two searches separately and interleave the results.
pub(crate) const SPS_DOUBLE: c_int = 4;

/// What is known while looking for suggestions.
/// A live [`suginfo_T`]: the suggestion search's whole state, which every
/// step of the search is handed by pointer.
pub(super) type Sug = Live<suginfo_T>;

impl Sug {
    /// The address of one of the `suginfo_T`'s own arrays or tables, for the
    /// `garray`/`hashtab` calls that take a pointer to it.
    ///
    /// [`Live::field_ptr`]: a field's address is the object's plus a
    /// constant, so this reads nothing and hands out no borrow -- which is
    /// what `su.su_ga()` was spelling out at every call.
    pub(super) fn su_ga(self) -> *mut garray_T {
        self.field_ptr(offset_of!(suginfo_T, su_ga))
    }

    /// [`Self::su_ga`], for the "soundalike" list.
    pub(super) fn su_sga(self) -> *mut garray_T {
        self.field_ptr(offset_of!(suginfo_T, su_sga))
    }

    /// [`Self::su_ga`], for the table of words already rejected.
    pub(super) fn su_banned(self) -> *mut hashtab_T {
        self.field_ptr(offset_of!(suginfo_T, su_banned))
    }

    /// The bad word as typed, as a NUL-terminated string.
    pub(super) fn su_badword(self) -> *mut c_char {
        self.field_ptr(offset_of!(suginfo_T, su_badword))
    }

    /// [`Self::su_badword`], case-folded.
    pub(super) fn su_fbadword(self) -> *mut c_char {
        self.field_ptr(offset_of!(suginfo_T, su_fbadword))
    }

    /// [`Self::su_badword`], sound-folded.
    pub(super) fn su_sal_badword(self) -> *mut c_char {
        self.field_ptr(offset_of!(suginfo_T, su_sal_badword))
    }
}

pub(crate) struct suginfo_T {
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
pub(crate) struct suggest_T {
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
pub(crate) unsafe fn window_langs<'a>() -> &'a mut [langp_T] {
    // SAFETY: the caller guarantees the window's spell state; an empty
    // garray has a null data pointer, which `from_raw_parts_mut` rejects
    // even at length zero.
    let gap = unsafe { &raw const (*cur_win().w_s).b_langp };
    if unsafe { (*gap).ga_data.is_null() } || unsafe { (*gap).ga_len } <= 0 {
        &mut []
    } else {
        let data = unsafe { (*gap).ga_data } as *mut langp_T;
        let len = unsafe { (*gap).ga_len } as usize;
        unsafe { ::core::slice::from_raw_parts_mut(data, len) }
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
    let mut flags = unsafe { captype(word, end) };
    if flags & WF_KEEPCAP == 0 {
        return flags;
    }

    // Count the upper and lower case letters.
    let (mut lower, mut upper) = (0, 0);
    let mut first = false;
    let mut p = word;
    while p < end {
        if spell_isupper(unsafe { utf_ptr2char(p) }) {
            upper += 1;
            if p == word {
                first = true;
            }
        } else {
            lower += 1;
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
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

/// Find suggestions for `word` and return them in `gap` as a list of
/// allocated strings.
///
/// This is what the `spellsuggest()` Vimscript function is built on.
///
/// # Safety
///
/// `gap` must be an uninitialised garray, `word` NUL-terminated, and the
/// current window must have its languages loaded.
pub(crate) unsafe fn spell_suggest_list(
    gap: *mut garray_T,
    word: *mut c_char,
    maxcount: c_int,
    need_cap: bool,
    interactive: bool,
) {
    // SAFETY: the caller guarantees the pointers; each string built below
    // is sized from the two pieces copied into it.
    let mut sug: suginfo_T = unsafe { mem::zeroed() };
    // SAFETY: `sug` is this frame's own, live for the whole call.
    let su = unsafe { Sug::new(&raw mut sug) };
    unsafe { spell_find_suggest(word, 0, su, maxcount, false, need_cap, interactive) };

    unsafe { ga_init(gap, size_of::<*mut c_char>() as c_int, sug.su_ga.ga_len + 1) };
    unsafe { ga_grow(gap, sug.su_ga.ga_len) };
    for stp in unsafe { suggestions(&raw mut sug.su_ga) } {
        // A suggestion may replace only part of `word`; what it does
        // not replace goes on the end.
        let tail = unsafe { sug.su_badptr.offset(stp.st_orglen as isize) };
        let wcopy =
            unsafe { xmalloc(stp.st_wordlen as usize + strlen(tail) as usize + 1) } as *mut c_char;
        unsafe { strcpy(wcopy, stp.st_word) };
        unsafe { strcpy(wcopy.offset(stp.st_wordlen as isize), tail) };
        unsafe { *((*gap).ga_data as *mut *mut c_char).offset((*gap).ga_len as isize) = wcopy };
        unsafe { (*gap).ga_len += 1 };
    }

    unsafe { spell_find_cleanup(su) };
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
    mut su: Sug,
    maxcount: c_int,
    banbadword: bool,
    need_cap: bool,
    interactive: bool,
) {
    // SAFETY: the caller guarantees the pointers; every copy into `su`'s
    // buffers is bounded by `MAXWLEN` and `su_badlen` is clamped below it.
    // A `expr:` item may itself call `spellsuggest()`, which lands
    // back here; the inner call must not evaluate the expression
    // again.
    static EXPR_BUSY: GlobalCell<bool> = GlobalCell::new(false);

    let mut attr: hlf_T = HLF_COUNT;
    let mut buf = [0 as c_char; MAXPATHL as usize];
    let bufp = buf.as_mut_ptr();

    unsafe { ptr::write_bytes(su.raw(), 0, 1) };
    unsafe { ga_init(su.su_ga(), size_of::<suggest_T>() as c_int, 10) };
    unsafe { ga_init(su.su_sga(), size_of::<suggest_T>() as c_int, 10) };
    if unsafe { *badptr } as c_int == NUL {
        return;
    }
    unsafe { hash_init(su.su_banned()) };

    su.su_badptr = badptr;
    su.su_badlen = if badlen != 0 {
        badlen
    } else {
        (unsafe { spell_check(curwin.get(), badptr, &raw mut attr, ptr::null_mut(), false) })
            as c_int
    };
    su.su_maxcount = maxcount;
    su.su_maxscore = SCORE_MAXINIT;
    su.su_badlen = unsafe { (*su.raw()).su_badlen.min(MAXWLEN as c_int - 1) }; // just in case
    // SAFETY: `su_badlen` was just clamped under `MAXWLEN`, which is the
    // size of both `su_badword` and `su_fbadword`, and `badptr` really has
    // that many bytes: it is the bad word in the line it came from.
    let badword = su.su_badword() as *mut c_char as *mut c_void;
    unsafe { xmemcpyz(badword, badptr as *const c_void, su.su_badlen as usize) };
    let fbadword = su.su_fbadword() as *mut c_char;
    let win = curwin.get();
    let _ = unsafe { spell_casefold(win, badptr, su.su_badlen, fbadword, MAXWLEN as c_int) };
    // Upstream note: this breaks if the case-folded text comes out
    // longer than the original, because an illegal byte then throws
    // the pointer arithmetic off.
    let badlen = su.su_badlen as usize;
    su.su_fbadword[badlen] = NUL as c_char;

    su.su_badflags = unsafe { badword_captype(badptr, badptr.offset(su.su_badlen as isize)) };
    if need_cap {
        su.su_badflags |= WF_ONECAP;
    }

    // Sound-fold with the first language in 'spelllang' that can. That
    // is right for several files of one language and not too bad for a
    // mixture like "pl,en". Note this is the buffer's list of
    // languages rather than the window's.
    let langp = unsafe { &raw const (*curbuf.get()).b_s.b_langp };
    for i in 0..unsafe { (*langp).ga_len } {
        let lp = unsafe { ((*langp).ga_data as *mut langp_T).offset(i as isize) };
        if !unsafe { (*lp).lp_sallang.is_null() } {
            su.su_sallang = unsafe { (*lp).lp_sallang };
            break;
        }
    }
    if !su.su_sallang.is_null() {
        // Once here, rather than once per candidate.
        // SAFETY: `su_fbadword` is NUL-terminated by the fold above and
        // `su_sal_badword` is `MAXWLEN`, which is a soundfold's bound.
        let folded = su.su_fbadword() as *mut c_char;
        let sounded = su.su_sal_badword() as *mut c_char;
        unsafe { spell_soundfold(su.su_sallang, folded, true, sounded) };
    }

    // A word the spell checker is happy with, spelled lower case, may
    // simply be missing its capital.
    if !spell_isupper(unsafe { utf_ptr2char(badptr) }) && attr == HLF_COUNT {
        unsafe { make_case_word(su.su_badword() as *mut c_char, bufp, WF_ONECAP) };
        let sug = su.raw();
        let ga = su.su_ga();
        let badlen = su.su_badlen;
        let lang = su.su_sallang;
        // SAFETY: `su` is live, so `ga` is its own list of `suggest_T`, and
        // `bufp` is the NUL-terminated word just capitalised into `buf`.
        unsafe { add_suggestion(sug, ga, bufp, badlen, SCORE_ICASE, 0, true, lang, false) };
    }

    // Ban the bad word itself; it may be valid in another region.
    if banbadword {
        unsafe { add_banned(su.raw(), su.su_badword() as *mut c_char) };
    }

    // An expression may change 'spellsuggest' while it runs.
    let sps_copy = unsafe { xstrdup(p_sps.get()) };
    let mut do_combine = false;
    let mut did_intern = false;
    let mut p = sps_copy;
    while unsafe { *p } as c_int != NUL {
        // SAFETY: `p` walks the copy of the option's NUL-terminated value
        // and `buf` is `MAXPATHL`, which is the bound handed over.
        let sep = c",".as_ptr().cast_mut();
        unsafe { copy_option_part(&raw mut p, bufp, MAXPATHL as usize, sep) };
        let part = unsafe { CStr::from_ptr(bufp) }.to_bytes();

        if part.starts_with(b"expr:") {
            if !EXPR_BUSY.get() {
                EXPR_BUSY.set(true);
                unsafe { spell_suggest_expr(su, bufp.add(5)) };
                EXPR_BUSY.set(false);
            }
        } else if part.starts_with(b"file:") {
            unsafe { spell_suggest_file(su, bufp.add(5)) };
        } else if part.starts_with(b"timeout:") {
            spell_suggest_timeout.set(unsafe { atoi(bufp.add(8)) });
        } else if !did_intern {
            // The internal method runs at most once.
            unsafe { spell_suggest_intern(su, interactive) };
            do_combine = sps_flags.get() & SPS_DOUBLE != 0;
            did_intern = true;
        }
    }
    unsafe { xfree(sps_copy as *mut c_void) };

    if do_combine {
        // Last, because sorting would undo the interleaving.
        unsafe { score_combine(su.raw()) };
    }
}

/// Run the internal search, in whichever form [`sps_flags`] asks for.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
unsafe fn spell_suggest_intern(mut su: Sug, interactive: bool) {
    // SAFETY: the caller guarantees `su` and the window's spell state.
    // Load whichever `.sug` files are available and not loaded yet.
    unsafe { suggest_load_files() };

    // 1. Special cases, such as a repeated word: "the the" -> "the".
    unsafe { suggest_try_special(su) };

    // 2. Inserting, deleting, swapping and changing letters, `REP`
    //    items from the `.aff` file, and splitting the word in two.
    unsafe { suggest_try_change(su) };

    // Give the top scorers a sound-a-like score to interleave on.
    if sps_flags.get() & SPS_DOUBLE != 0 {
        unsafe { score_comp_sal(su.raw()) };
    }

    // 3. Words that sound alike.
    if sps_flags.get() & SPS_FAST == 0 {
        if sps_flags.get() & SPS_BEST != 0 {
            unsafe { rescore_suggestions(su.raw()) };
        }

        // Through the sound-fold tree `su_maxscore` bounds the changes
        // tried to the sound-folded word, while `su_sfmaxscore` bounds
        // the rescored result. Small edit distances come first because
        // they are much faster and usually enough; only if too little
        // turns up is a wider search worth its time. `sl_sounddone`
        // keeps the passes from redoing each other's work.
        unsafe { suggest_try_soundalike_prep() };
        su.su_maxscore = SCORE_SFMAX1;
        su.su_sfmaxscore = SCORE_MAXINIT * 3;
        unsafe { suggest_try_soundalike(su.raw()) };
        for ceiling in [SCORE_SFMAX2, SCORE_SFMAX3] {
            if su.su_ga.ga_len >= clean_count(&su) {
                break;
            }
            su.su_maxscore = ceiling;
            unsafe { suggest_try_soundalike(su.raw()) };
        }
        su.su_maxscore = su.su_sfmaxscore;
        unsafe { suggest_try_soundalike_finish() };
    }

    // Interrupted searches still show what they found. `got_int` is
    // only cleared for a command, not for `spellsuggest()`.
    os_breakcheck();
    if interactive && got_int.get() {
        vgetc();
        got_int.set(false);
    }

    if sps_flags.get() & SPS_DOUBLE == 0 && su.su_ga.ga_len != 0 {
        if sps_flags.get() & SPS_BEST != 0 {
            unsafe { rescore_suggestions(su.raw()) };
        }
        unsafe { check_suggestions(su.raw(), su.su_ga()) };
        unsafe { cleanup_suggestions(su.su_ga(), su.su_maxscore, su.su_maxcount) };
    }
}

/// Release everything [`spell_find_suggest`] put in `su`.
///
/// # Safety
///
/// `su` must have been filled by [`spell_find_suggest`].
unsafe fn spell_find_cleanup(mut su: Sug) {
    // SAFETY: the caller guarantees `su`; each suggestion owns its word
    // and the banned table owns its keys.
    for gap in [su.su_ga(), su.su_sga()] {
        for stp in unsafe { suggestions(gap) } {
            unsafe { xfree(stp.st_word as *mut c_void) };
        }
        unsafe { ga_clear(gap) };
    }
    unsafe { hash_clear_all(su.su_banned(), 0) };
}

/// Find suggestions by recognising specific situations.
///
/// There is only one: a word typed twice, "the the", whose suggestion is
/// the word once.
///
/// # Safety
///
/// `su` must be valid.
unsafe fn suggest_try_special(mut su: Sug) {
    // SAFETY: the caller guarantees `su`; the terminator planted in
    // `su_fbadword` is inside the buffer and is put back straight away.
    let fbadword = su.su_fbadword() as *mut c_char;
    let mut p = unsafe { skiptowhite(fbadword) };
    let len = unsafe { p.offset_from(fbadword) } as usize;
    p = unsafe { skipwhite(p) };
    if unsafe { strlen(p) } as usize != len || unsafe { strncmp(fbadword, p, len) } != 0 {
        return;
    }

    // Take the bad word's case with it, so that "The the" -> "The".
    let mut word = [0 as c_char; MAXWLEN];
    let wordp = word.as_mut_ptr();
    let saved = su.su_fbadword[len];
    su.su_fbadword[len] = NUL as c_char;
    unsafe { make_case_word(fbadword, wordp, su.su_badflags) };
    su.su_fbadword[len] = saved;

    // Score it as one deletion, with a sound-a-like score of zero.
    let sug = su.raw();
    let ga = su.su_ga();
    let badlen = su.su_badlen;
    let lang = su.su_sallang;
    // SAFETY: `su` is live by the contract above, so `ga` is its own list
    // of `suggest_T`, and `wordp` is the NUL-terminated word built above.
    let score = 3 * SCORE_REP / 4;
    unsafe { add_suggestion(sug, ga, wordp, badlen, score, 0, true, lang, false) };
}

/// Find suggestions by adding, removing and swapping letters, in every
/// language the window has loaded.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
unsafe fn suggest_try_change(mut su: Sug) {
    // SAFETY: the caller guarantees `su` and the window's spell state;
    // `fword` is `MAXWLEN` and every write into it is told so.
    // The walk rewrites the case-folded bad word in place (for `REP`
    // items especially), so it gets a copy. What follows the bad word
    // is appended: changing characters after it may help.
    let mut fword = [0 as c_char; MAXWLEN];
    let fwordp = fword.as_mut_ptr();
    unsafe { strcpy(fwordp, su.su_fbadword() as *const c_char) };
    let n = unsafe { strlen(fwordp) } as c_int;
    let tail = unsafe { su.su_badptr.offset(su.su_badlen as isize) };
    // SAFETY: `tail` is what follows the bad word in its line, so it is
    // NUL-terminated; `n` bytes of `fword` are used, and the fold is told
    // it may fill the `MAXWLEN - n` that are left.
    let taillen = unsafe { strlen(tail) } as c_int;
    let dest = unsafe { fwordp.offset(n as isize) };
    let _ = unsafe { spell_casefold(curwin.get(), tail, taillen, dest, MAXWLEN as c_int - n) };

    // Keep the result no longer than the original text.
    let n = unsafe { strlen(su.su_badptr) } as usize;
    if n < MAXWLEN {
        fword[n] = NUL as c_char;
    }

    for lp in unsafe { window_langs() } {
        // A spell file that failed to reload is still in the list, but
        // everything in it has been cleared.
        if !unsafe { (*lp.lp_slang).sl_fbyts.is_null() } {
            unsafe { suggest_trie_walk(su.raw(), lp, fwordp, false) };
        }
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
