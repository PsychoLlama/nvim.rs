//! Spell checking.
//!
//! The subsystem is three files' worth of work, split across three
//! directories:
//!
//! * `spellfile` reads and writes the on-disk formats — `.aff`/`.dic`
//!   sources, the compiled `.spl` word trees, and the `.sug` soundalike
//!   companion.
//! * `spell` (here) uses them: deciding whether a word is spelled right,
//!   finding the next word that is not, and dumping what a language knows.
//! * `spellsuggest` proposes replacements for a word that is not.
//!
//! What this module owns, in dependency order:
//!
//! * [`chartab`] — which characters are letters and what case they are in,
//!   from the table a `.spl` file installs.
//! * [`soundfold`] — turning a word into how it sounds. Shared with both
//!   sibling directories.
//! * [`lookup`] — walking a language's word tree, the innermost loop.
//! * [`check`] — [`spell_check`], the per-word entry point.
//! * [`navigate`] — [`spell_move_to`], scanning the buffer for the next
//!   bad word.
//! * [`slang`] — the lifetime of one loaded language.
//! * [`lang`] — turning `'spelllang'` into a list of loaded languages.
//! * [`dump`] — walking a whole tree, for `:spelldump` and completion.
//!
//! The parent keeps only the shared constants and globals, and
//! [`ex_spellrepall`], which belongs to neither half: it is the `z=`
//! replacement applied to the rest of the buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg;
use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::change::inserted_bytes;
use crate::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::ex_cmds::do_sub_msg;
use crate::global_cell::GlobalCell;
use crate::main::{curwin, got_int, p_ws, sub_nlines, sub_nsubs};
use crate::memline::ml_replace;
use crate::memory::{xfree, xmalloc};
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, memmove, snprintf, strncmp};
use crate::search::{SEARCH_KEEP, do_search};
use crate::types::{
    FAIL, colnr_T, exarg_T, file_comparison, langp_T, linenr_T, oparg_T, pos_T, searchit_arg_T,
    size_t, slang_T, smt_T, spelltab_T, uint8_t, win_T,
};
use crate::undo::u_save_cursor;
use ::libc::{strcat, strcpy, strlen};

mod chartab;
mod check;
mod dump;
mod lang;
mod lookup;
mod navigate;
mod slang;
mod soundfold;

pub use chartab::{
    allcap_copy, ascii_spell_chartab, byte_in_str, captype, init_spell_chartab, make_case_word,
    nofold_len, onecap_copy, spell_casefold, spell_iswordp, spell_iswordp_nmw,
};
pub(crate) use chartab::{spelltab_fold, spelltab_isu, spelltab_isw, spelltab_upper};
pub use check::{
    check_need_cap, expand_spelling, no_spell_checking, spell_check, spell_check_window,
    spell_expand_check_cap, spell_to_word_end, spell_valid_case, spell_word_start,
};
pub use dump::{ex_spelldump, ex_spellinfo, spell_dump_compl};
pub use lang::{
    compile_cap_prog, did_set_spell_option, parse_spelllang, spell_delete_wordlist, spell_enc,
    spell_free_all, spell_reload, valid_spellfile, valid_spelllang,
};
pub use lookup::{can_compound, match_checkcompoundpattern, match_compoundrule, valid_word_prefix};
pub use navigate::{spell_cat_line, spell_move_to};
use slang::count_syllables;
pub use slang::{
    close_spellbuf, count_common_word, init_syl_tab, open_spellbuf, slang_alloc, slang_clear,
    slang_clear_sug, slang_free,
};
pub use soundfold::{eval_soundfold, spell_soundfold};

pub const kOptValTypeBoolean: crate::types::OptValType = 0;
pub const kEqualFiles: file_comparison = 1;

/// The longest word, in bytes, that any of this can handle. Every word
/// buffer in the subsystem is this size.
pub const MAXWLEN: usize = 254;

/// Flags stored with a word in the tree, in the low bits of its `sl_fidxs`
/// entry. The region mask sits at bit 16 and the affix ID at bit 24.
pub type WordFlags = c_uint;
/// Every capitalisation flag, for masking them off together.
pub const WF_CAPMASK: WordFlags = 198;
/// Capitalisation cannot be described by a flag; the word is in the
/// keep-case tree as written.
pub const WF_KEEPCAP: WordFlags = 128;
/// Do not accept the word spelled in all capitals.
pub const WF_FIXCAP: WordFlags = 64;
/// The word is explicitly wrong.
pub const WF_BANNED: WordFlags = 16;
/// The word is real but unusual.
pub const WF_RARE: WordFlags = 8;
pub const WF_ALLCAP: WordFlags = 4;
pub const WF_ONECAP: WordFlags = 2;
/// The word is limited to the regions in bits 16 and up.
pub const WF_REGION: WordFlags = 1;
/// No compounding after this word.
pub const WF_NOCOMPAFT: WordFlags = 8192;
/// No compounding before this word.
pub const WF_NOCOMPBEF: WordFlags = 4096;
/// COMPOUNDROOT: the word counts as a root, not a part.
pub const WF_COMPROOT: WordFlags = 2048;
/// The word is only valid inside a compound.
pub const WF_NEEDCOMP: WordFlags = 512;
/// An affix was applied to reach this word.
pub const WF_HAS_AFF: WordFlags = 256;
/// A prefix that does not combine with a suffix.
pub const WF_PFX_NC: WordFlags = 33554432;
/// A prefix that makes the word rare.
pub const WF_RAREPFX: WordFlags = 16777216;

/// A `.spl` file is malformed.
pub const SP_FORMERROR: c_int = -2;
/// A word with no region restriction, and the "no such region" answer.
pub const REGION_ALL: c_int = 255;
/// The saturation point of a `COMMON` word count.
pub const MAXWORDCOUNT: c_uint = 65535;

pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;

/// How badly, or not, a word is spelled. Lower is worse, so that a lookup
/// can keep the best answer by comparison.
pub type SpellResult = c_int;
/// Explicitly wrong.
pub const SP_BANNED: SpellResult = -1;
/// Real but unusual.
pub const SP_RARE: SpellResult = 0;
/// Fine.
pub const SP_OK: SpellResult = 1;
/// Real, but not in the regions in use.
pub const SP_LOCAL: SpellResult = 2;
/// Not a word.
pub const SP_BAD: SpellResult = 3;

/// Which tree [`lookup::find_word`] should walk, and where in the word to
/// start.
pub type FindMode = c_int;
/// The case-folded tree, from the start of the word.
pub const FIND_FOLDWORD: FindMode = 0;
/// The keep-case tree, from the start of the word.
pub const FIND_KEEPWORD: FindMode = 1;
/// The case-folded tree, after a prefix that already matched.
pub const FIND_PREFIX: FindMode = 2;
/// The case-folded tree, after the compound parts found so far.
pub const FIND_COMPOUND: FindMode = 3;
/// The keep-case tree, after the compound parts found so far.
pub const FIND_KEEPCOMPOUND: FindMode = 4;

/// What kind of character a camel-case split looks at.
pub type CharType = c_int;
pub const CHAR_OTHER: CharType = 0;
pub const CHAR_UPPER: CharType = 1;
pub const CHAR_DIGIT: CharType = 2;

pub const TAB: c_int = '\t' as c_int;

/// The longest `SYLLABLE` item.
pub const SY_MAXLEN: c_int = 30;
/// A `wordcount_T`'s key starts this far into it, so a hash item's key
/// pointer can be walked back to the struct. Derived from the type rather
/// than spelled out: the word-count table stores the record and hashes on
/// the inline `wc_word`, so a wrong value here is a wild pointer.
pub const WC_KEY_OFF: usize = ::core::mem::offset_of!(crate::types::wordcount_T, wc_word);

/// State threaded through one word's lookup, so that the tree walk and the
/// compound recursion can pass it around in one piece rather than a dozen
/// arguments.
#[derive(Copy, Clone)]
pub struct matchinf_T {
    /// The language being tried.
    pub mi_lp: *mut langp_T,
    /// The word as written, at its first character.
    pub mi_word: *mut c_char,
    /// One past the last character accepted so far.
    pub mi_end: *mut c_char,
    /// How far the case folding into `mi_fword` has reached.
    pub mi_fend: *mut c_char,
    /// The word length `mi_capflags` was computed for.
    pub mi_cend: *mut c_char,
    /// The case-folded word.
    pub mi_fword: [c_char; MAXWLEN + 1],
    pub mi_fwordlen: c_int,
    /// Where in `sl_pidxs` the candidate prefixes start,
    pub mi_prefarridx: c_int,
    /// and how many there are.
    pub mi_prefcnt: c_int,
    /// The prefix's length, folded and as written.
    pub mi_prefixlen: c_int,
    pub mi_cprefixlen: c_int,
    /// Where the next compound part begins.
    pub mi_compoff: c_int,
    /// The flag of each compound part used so far.
    pub mi_compflags: [uint8_t; MAXWLEN],
    pub mi_complen: c_int,
    /// How many of those parts were COMPOUNDROOT.
    pub mi_compextra: c_int,
    /// The best result so far, and the capitalisation it assumed.
    pub mi_result: SpellResult,
    pub mi_capflags: c_int,
    pub mi_win: *mut win_T,
    /// For NOBREAK: the best result reached *without* a good word
    /// following, kept as a fall-back.
    pub mi_result2: SpellResult,
    pub mi_end2: *mut c_char,
}

/// One `SYLLABLE` item: a short sequence of characters counting as one
/// syllable.
#[derive(Copy, Clone)]
pub struct syl_item_T {
    pub sy_chars: [c_char; SY_MAXLEN as usize],
    pub sy_len: c_int,
}

/// The cookie `do_in_runtimepath` carries while loading a language.
#[derive(Copy, Clone)]
pub struct spelload_T {
    /// The language name, truncated when an error is found.
    pub sl_lang: [c_char; MAXWLEN + 1],
    /// The last file loaded.
    pub sl_slang: *mut slang_T,
    /// Whether any file so far declared NOBREAK.
    pub sl_nobreak: c_int,
}

/// Every language loaded, chained on `sl_next`.
pub static first_lang: GlobalCell<*mut slang_T> = GlobalCell::new(::core::ptr::null_mut());
/// The word list `zg` appends to when `'spellfile'` is empty.
pub static int_wordlist: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut());

/// The character table currently in force.
pub static spelltab: GlobalCell<spelltab_T> = GlobalCell::new(spelltab_T {
    st_isw: [false; 256],
    st_isu: [false; 256],
    st_fold: [0; 256],
    st_upper: [0; 256],
});
/// Whether a `.spl` file replaced [`spelltab`], rather than the encoding.
pub static did_set_spelltab: GlobalCell<bool> = GlobalCell::new(false);

pub static e_format: GlobalCell<*mut c_char> =
    GlobalCell::new(c"E759: Format error in spell file".as_ptr() as *mut c_char);

/// What `z=` last replaced, and with what, for [`ex_spellrepall`].
pub static repl_from: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut());
pub static repl_to: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut());

/// `:spellrepall` — repeat the last `z=` replacement everywhere else in
/// the buffer.
pub unsafe fn ex_spellrepall(_eap: *mut exarg_T) {
    let pos: pos_T = unsafe { (*curwin.get()).w_cursor };
    // Round-tripped through a bool, as in C: any non-zero 'wrapscan'
    // comes back as 1.
    let save_ws = p_ws.get() != 0;
    let mut prev_lnum: linenr_T = 0;

    if repl_from.get().is_null() || repl_to.get().is_null() {
        emsg(gettext(c"E752: No previous spell replacement"));
        return;
    }
    let repl_from_len = unsafe { strlen(repl_from.get()) };
    let repl_to_len = unsafe { strlen(repl_to.get()) };
    let addlen = repl_to_len as i64 - repl_from_len as i64;

    let frompatsize = repl_from_len + 7;
    let frompat = unsafe { xmalloc(frompatsize) } as *mut c_char;
    let fmt = c"\\V\\<%s\\>".as_ptr();
    let from = repl_from.get();
    let frompatlen = unsafe { snprintf(frompat, frompatsize, fmt, from) } as size_t;
    p_ws.set(0);

    sub_nsubs.set(0);
    sub_nlines.set(0);
    unsafe { (*curwin.get()).w_cursor.lnum = 0 };
    while !got_int.get() {
        let slash = '/' as c_int;
        let no_oap = ::core::ptr::null_mut::<oparg_T>();
        let no_arg = ::core::ptr::null_mut::<searchit_arg_T>();
        let found = unsafe {
            do_search(
                no_oap,
                slash,
                slash,
                frompat,
                frompatlen,
                1,
                SEARCH_KEEP,
                no_arg,
            )
        };
        if found == 0 || u_save_cursor() == FAIL {
            break;
        }

        // Only replace where the replacement is not already there. That
        // happens when changing "etc" to "etc.".
        let line = get_cursor_line_ptr();
        let col = unsafe { (*curwin.get()).w_cursor.col };
        if addlen <= 0
            || unsafe { strncmp(line.offset(col as isize), repl_to.get(), repl_to_len) } != 0
        {
            let p = unsafe { xmalloc((get_cursor_line_len() as i64 + addlen) as size_t + 1) }
                as *mut c_char;
            unsafe { memmove(p as *mut c_void, line as *const c_void, col as size_t) };
            unsafe { strcpy(p.offset(col as isize), repl_to.get()) };
            unsafe { strcat(p, line.offset(col as isize).add(repl_from_len)) };
            unsafe { ml_replace((*curwin.get()).w_cursor.lnum, p, false) };
            let lnum = unsafe { (*curwin.get()).w_cursor.lnum };
            let (was, now) = (repl_from_len as c_int, repl_to_len as c_int);
            unsafe { inserted_bytes(lnum, col, was, now) };

            if unsafe { (*curwin.get()).w_cursor.lnum } != prev_lnum {
                sub_nlines.set(sub_nlines.get() + 1);
                prev_lnum = unsafe { (*curwin.get()).w_cursor.lnum };
            }
            sub_nsubs.set(sub_nsubs.get() + 1);
        }
        unsafe { (*curwin.get()).w_cursor.col += repl_to_len as colnr_T };
    }

    p_ws.set(save_ws as c_int);
    unsafe { (*curwin.get()).w_cursor = pos };
    unsafe { xfree(frompat as *mut c_void) };

    if sub_nsubs.get() == 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe { c_str(repl_from.get()) };
        semsg!("E753: Not found: {arg0}");
    } else {
        unsafe { do_sub_msg(false) };
    }
}
