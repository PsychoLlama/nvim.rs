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
//! * [`check`] — [`Win::spell_check`], the per-word entry point.
//! * [`navigate`] — [`Win::spell_move_to`], scanning the buffer for the next
//!   bad word.
//! * [`slang`] — the lifetime of one loaded language.
//! * [`lang`] — turning `'spelllang'` into a list of loaded languages.
//! * [`dump`] — walking a whole tree, for `:spelldump` and completion.
//!
//! The parent keeps only the shared constants and globals, and
//! [`ex_spellrepall`], which belongs to neither half: it is the `z=`
//! replacement applied to the rest of the buffer.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::memline::{Lines, ml_replace_owned};
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::narrow::{const_len_as_int, len_as_int};
use crate::os::libc::{int16_t, time_t, uint8_t};
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::CStr;
use core::ffi::c_int;
use std::borrow::Cow;

use crate::change::inserted_bytes;
use crate::ex_cmds::do_sub_msg;
use crate::ex_cmds::{SUB_NLINES, SUB_NSUBS};
use crate::getchar::state::GOT_INT;
use crate::global_cell::GlobalCell;
use crate::message::emsg;
use crate::option::vars::{P_WS, p_ws};
use crate::os::cshim::gettext;
use crate::search::{SEARCH_KEEP, do_search};
use crate::types::{ExArg, LineNr, Pos};
use crate::undo::u_save_cursor;

mod chartab;
mod check;
mod dump;
mod lang;
mod lookup;
mod navigate;
mod slang;
mod soundfold;
mod wordtree;

pub use chartab::{
    allcap_copy, ascii_spell_chartab, byte_in_str, captype, init_spell_chartab, make_case_word,
    nofold_len, onecap_copy, spell_casefold, spell_iswordp, spell_iswordp_nmw,
};
pub(crate) use chartab::{casefold_len, spelltab_fold, spelltab_isu, spelltab_isw, spelltab_upper};
pub use check::{
    check_need_cap, expand_spelling, no_spell_checking, spell_check_window, spell_expand_check_cap,
    spell_to_word_end, spell_valid_case, spell_word_start,
};
pub use dump::{ex_spelldump, ex_spellinfo, spell_dump_compl};
pub use lang::{
    compile_cap_prog, did_set_spell_option, parse_spelllang, spell_delete_wordlist, spell_enc,
    spell_free_all, spell_reload, valid_spellfile, valid_spelllang,
};
pub(crate) use lookup::compound_flag;
pub use lookup::{can_compound, match_checkcompoundpattern, match_compoundrule, valid_word_prefix};
pub use navigate::spell_cat_line;
use slang::count_syllables;
pub(crate) use slang::{LOADED_LANGS, LoadedLangs, init_syl_tab, languages, put_lang, with_lang};
pub use slang::{
    SpellBuf, WordCounts, WordHasher, count_common_word, open_spellbuf, register_lang, slang_clear,
    slang_clear_sug, take_lang,
};
pub use soundfold::{eval_soundfold, spell_soundfold};
pub(crate) use wordtree::Tree;
pub use wordtree::WordTree;

pub(crate) static SPELL_REDRAW_LNUM: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);

/// The longest word, in bytes, that any of this can handle. Every word
/// buffer in the subsystem is this size.
pub const MAXWLEN: usize = 254;
/// [`MAXWLEN`] as the C `int` the spell file's limits are counted in.
pub(crate) const MAXWLEN_INT: c_int = const_len_as_int(MAXWLEN);

crate::flag_set! {
    /// Flags stored with a word in the tree, in the low bits of its
    /// `sl_fidxs` entry. The region mask sits at bit 16 and the affix ID at
    /// bit 24, which is why the two prefix flags are so far up.
    pub struct WordFlags;

    /// The word is limited to the regions in bits 16 and up.
    const REGION = 0x1;
    /// The word starts with a capital.
    const ONECAP = 0x2;
    /// The word is all capitals.
    const ALLCAP = 0x4;
    /// The word is real but unusual.
    const RARE = 0x8;
    /// The word is explicitly wrong.
    const BANNED = 0x10;
    /// An affix may be applied to this word.
    const AFX = 0x20;
    /// A mix of upper and lower case: "macaRONI". **The same bit as
    /// [`Self::AFX`]**, and upstream means it: this one appears only in
    /// `SugInfo::su_badflags`, which never holds a tree flag, and that one
    /// only in the tree.
    const MIXCAP = 0x20;
    /// Do not accept the word spelled in all capitals.
    const FIXCAP = 0x40;
    /// Capitalisation cannot be described by a flag; the word is in the
    /// keep-case tree as written.
    const KEEPCAP = 0x80;
    /// An affix was applied to reach this word.
    const HAS_AFF = 0x100;
    /// The word is only valid inside a compound.
    const NEEDCOMP = 0x200;
    /// The word is never offered as a suggestion.
    const NOSUGGEST = 0x400;
    /// COMPOUNDROOT: the word counts as a root, not a part.
    const COMPROOT = 0x800;
    /// No compounding before this word.
    const NOCOMPBEF = 0x1000;
    /// No compounding after this word.
    const NOCOMPAFT = 0x2000;
    /// A prefix that makes the word rare.
    const RAREPFX = 0x1000000;
    /// A prefix that does not combine with a suffix.
    const PFX_NC = 0x2000000;

    /// Every capitalisation flag, for masking them off together. A word's
    /// case is *one* of these, or none, so this is a sub-field rather than a
    /// set of independent bits.
    const CAPMASK = Self::ONECAP.bits()
        | Self::ALLCAP.bits()
        | Self::FIXCAP.bits()
        | Self::KEEPCAP.bits();
}

/// A `.spl` file is malformed.
/// A word with no region restriction, and the "no such region" answer.
pub const REGION_ALL: c_int = 255;
/// The saturation point of a `COMMON` word count.
pub const MAXWORDCOUNT: u16 = 65535;

pub const SMT_RARE: SpellMoveType = 2;
pub const SMT_BAD: SpellMoveType = 1;
pub const SMT_ALL: SpellMoveType = 0;

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

/// The longest `SYLLABLE` item.
pub const SY_MAXLEN: c_int = 30;
/// A "a word could end here" node [`find_word`](lookup::find_word) passed
/// on its way down the tree: one past the node's index, and how far into
/// the word it sat.
#[derive(Debug)]
pub(crate) struct WordEnd {
    pub len: usize,
    pub node: usize,
}

/// State threaded through one word's lookup, so that the tree walk and the
/// compound recursion can pass it around in one piece rather than a dozen
/// arguments.
///
/// Positions in the word are offsets into [`MatchInf::word`], the text from
/// the word's first character to the end of its string.
#[derive(Debug)]
pub(crate) struct MatchInf<'a> {
    /// The language table the window's entries name.
    pub langs: &'a LoadedLangs,
    /// The window's languages.
    pub langp: &'a [LangP],
    /// The entry being tried,
    pub lp: LangP,
    /// and its language, when its slot holds one.
    pub slang: Option<&'a SpellLang>,
    /// The word as written, from its first character on.
    pub word: &'a [u8],
    /// One past the last character accepted so far.
    pub end: usize,
    /// How far the case folding into `fword` has reached.
    pub fend: usize,
    /// The word length `capflags` was computed for.
    pub cend: Option<usize>,
    /// The case-folded word.
    pub fword: [u8; MAXWLEN + 1],
    pub fwordlen: usize,
    /// Where in the language's prefix tree the candidate prefixes start,
    pub prefarridx: usize,
    /// and how many there are.
    pub prefcnt: usize,
    /// The prefix's length, folded and as written.
    pub prefixlen: usize,
    pub cprefixlen: usize,
    /// Where the next compound part begins.
    pub compoff: usize,
    /// The flag of each compound part used so far.
    pub compflags: [uint8_t; MAXWLEN],
    pub complen: usize,
    /// How many of those parts were COMPOUNDROOT.
    pub compextra: usize,
    /// The best result so far, and the capitalisation it assumed.
    pub result: SpellResult,
    pub capflags: WordFlags,
    /// The window the lookup is for.
    pub win: Win,
    /// For NOBREAK: the best result reached *without* a good word
    /// following, kept as a fall-back.
    pub result2: SpellResult,
    pub end2: usize,
    /// The endings stack every [`find_word`](lookup::find_word) call of
    /// this lookup shares, nested calls above their callers': a scratch
    /// [`LoadedLangs`] lends out, so no call zeroes a `MAXWLEN` array.
    pub ends: &'a mut Vec<WordEnd>,
}

impl<'a> MatchInf<'a> {
    /// The state for checking `word` in `win`, whose languages are `langp`
    /// in `langs`, with the walk at the word's first character and nothing
    /// accepted yet. The language being tried is set per language.
    pub(crate) fn new(
        word: &'a [u8],
        win: Win,
        langs: &'a LoadedLangs,
        langp: &'a [LangP],
        ends: &'a mut Vec<WordEnd>,
    ) -> Self {
        MatchInf {
            langs,
            langp,
            lp: LangP {
                slang: LangId(u32::MAX),
                sallang: None,
                replang: None,
                region: 0,
            },
            slang: None,
            word,
            end: 0,
            fend: 0,
            cend: None,
            fword: [0; MAXWLEN + 1],
            fwordlen: 0,
            prefarridx: 0,
            prefcnt: 0,
            prefixlen: 0,
            cprefixlen: 0,
            compoff: 0,
            compflags: [0; MAXWLEN],
            complen: 0,
            compextra: 0,
            result: 0,
            capflags: WordFlags::NONE,
            win,
            result2: 0,
            end2: 0,
            ends,
        }
    }
}

/// Whether the C strings `a` and `b` agree in their first `n` bytes:
/// `strncmp(a, b, n) == 0`, with the end of a slice reading as its NUL.
pub(crate) fn strn_eq(a: &[u8], b: &[u8], n: usize) -> bool {
    for i in 0..n {
        let (x, y) = (crate::cstr::byte_at(a, i), crate::cstr::byte_at(b, i));
        if x != y {
            return false;
        }
        if x == 0 {
            break;
        }
    }
    true
}

/// `text` as a C string for the regexp engine: up to its first NUL, copied
/// only when it has none.
pub(crate) fn c_text(text: &[u8]) -> Cow<'_, CStr> {
    match CStr::from_bytes_until_nul(text) {
        Ok(terminated) => Cow::Borrowed(terminated),
        Err(_) => Cow::Owned(crate::cstr::owned(text)),
    }
}

/// One `SYLLABLE` item: a short sequence of characters counting as one
/// syllable.
#[derive(Debug)]
pub struct SylItem {
    pub sy_chars: Box<[u8]>,
}

/// The cookie `do_in_runtimepath` carries while loading a language.
#[derive(Debug)]
pub struct SpellLoad {
    /// The language name, cleared when an error is found.
    pub lang: Vec<u8>,
    /// The last file loaded.
    pub last: Option<LangId>,
    /// Whether any file so far declared NOBREAK.
    pub nobreak: bool,
}

/// The word list `zg` appends to when `'spellfile'` is empty.
pub static INT_WORDLIST: GlobalCell<Option<XString>> = GlobalCell::new(None);

/// The character table currently in force.
pub static SPELLTAB: GlobalCell<SpellTab> = GlobalCell::new(SpellTab {
    st_isw: [false; 256],
    st_isu: [false; 256],
    st_fold: [0; 256],
    st_upper: [0; 256],
});
/// Whether a `.spl` file replaced [`SPELLTAB`], rather than the encoding.
pub static DID_SET_SPELLTAB: GlobalCell<bool> = GlobalCell::new(false);

pub const E_FORMAT: &CStr = c"E759: Format error in spell file";

/// What `z=` last replaced, and with what, for [`ex_spellrepall`].
#[derive(Clone, Debug)]
pub(crate) struct SpellReplacement {
    pub(crate) from: XString,
    pub(crate) to: XString,
}

/// The replacement [`ex_spellrepall`] repeats; `None` until `z=` makes one.
pub(crate) static LAST_REPLACEMENT: GlobalCell<Option<SpellReplacement>> = GlobalCell::new(None);

/// `:spellrepall` — repeat the last `z=` replacement everywhere else in
/// the buffer.
pub fn ex_spellrepall(_excmd: &mut ExArg) {
    let pos: Pos = Win::current().w_cursor;
    // Round-tripped through a bool, as in C: any non-zero 'wrapscan'
    // comes back as 1.
    let save_ws = p_ws();
    let mut prev_lnum: LineNr = LineNr::ZERO;

    // A copy: the searches and line changes below can run autocommands.
    let Some(SpellReplacement {
        from: repl_from,
        to: repl_to,
    }) = LAST_REPLACEMENT.with(Clone::clone)
    else {
        emsg(gettext(c"E752: No previous spell replacement"));
        return;
    };
    let repl_from_len = repl_from.len();
    let repl_to_len = repl_to.len();
    let grows = repl_to_len > repl_from_len;

    // The word to replace, anchored as a very-nomagic whole word.
    let mut frompat = XString::from_bytes(b"\\V\\<");
    frompat.push_bytes(&repl_from);
    frompat.push_bytes(b"\\>");
    P_WS.set(false);

    SUB_NSUBS.set(0);
    SUB_NLINES.set(0);
    Win::current().w_cursor.lnum = LineNr::ZERO;
    while !GOT_INT.get() {
        let slash = c_int::from(b'/');
        let found = do_search(None, slash, slash, &frompat, 1, SEARCH_KEEP, None);
        if found.is_none() || u_save_cursor().is_err() {
            break;
        }

        // Only replace where the replacement is not already there. That
        // happens when changing "etc" to "etc.".
        let col = Win::current().w_cursor.col;
        let replaced = {
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum);
            let at = col.as_usize();
            let present = chartab::tail(line, at).starts_with(&repl_to);
            (!grows || !present).then(|| {
                let mut rebuilt = XString::from_bytes(&line[..at.min(line.len())]);
                rebuilt.push_bytes(&repl_to);
                rebuilt.push_bytes(chartab::tail(line, at + repl_from_len));
                rebuilt
            })
        };
        if let Some(replaced) = replaced {
            let _ = ml_replace_owned(Win::current().w_cursor.lnum, replaced);
            let lnum = Win::current().w_cursor.lnum;
            let (was, now) = (len_as_int(repl_from_len), len_as_int(repl_to_len));
            inserted_bytes(lnum, col, was, now);

            if Win::current().w_cursor.lnum != prev_lnum {
                SUB_NLINES.set(SUB_NLINES.get() + 1);
                prev_lnum = Win::current().w_cursor.lnum;
            }
            SUB_NSUBS.set(SUB_NSUBS.get() + 1);
        }
        Win::current().w_cursor.col += len_as_int(repl_to_len);
    }

    P_WS.set(save_ws);
    Win::current().w_cursor = pos;

    if SUB_NSUBS.get() == 0 {
        let arg0 = msg_bytes(&repl_from);
        semsg!("E753: Not found: {arg0}");
    } else {
        do_sub_msg(false);
    }
}

pub type SpellAddType = ::core::ffi::c_uint;
pub(crate) const SPELL_ADD_BAD: SpellAddType = 1;
pub(crate) const SPELL_ADD_GOOD: SpellAddType = 0;
/// One `REP`/`REPSAL` item: text to look for, and what to put in its
/// place. Both ends of the format hold it — the loaded language's tables
/// and `:mkspell`'s, whose copies the writer sorts.
#[derive(Debug)]
pub struct RepItem {
    pub from: Box<[u8]>,
    pub to: Box<[u8]>,
}
pub type SpellIdx = ::core::ffi::c_int;

/// A loaded language's slot in the language table. The slot, not the
/// language's address, is its identity: reloading a spell file refills the
/// slot in place, so every window that names it keeps naming it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LangId(pub(crate) u32);

/// One entry of a window's `'spelllang'`: a language, the region mask
/// chosen for it, and where its sound folding and REP items come from.
#[derive(Copy, Clone, Debug)]
pub struct LangP {
    pub slang: LangId,
    /// The language to sound-fold with: this one, or a similarly named one
    /// that has a table when this has none.
    pub sallang: Option<LangId>,
    /// The language whose REP items to use, chosen the same way.
    pub replang: Option<LangId>,
    pub region: ::core::ffi::c_int,
}
pub type SalFirst = ::core::ffi::c_int;
/// One `SAL` sound-folding rule.
///
/// Only the wide forms are kept: the narrow `sm_lead`/`sm_oneof`/`sm_to`
/// the file stores existed to be widened, and nothing but the reader ever
/// looked at them. `sm_rules` stays narrow because it is a run of ASCII
/// flag characters.
#[derive(Debug)]
pub struct SalItem {
    /// The characters this rule matches, terminated by a `NUL`.
    pub sm_lead_w: Box<[::core::ffi::c_int]>,
    /// How many of them there are.
    pub sm_leadlen: usize,
    /// The optional `(abc)` set: any one of these may follow the lead.
    pub sm_oneof_w: Option<Box<[::core::ffi::c_int]>>,
    /// The flag characters after the lead, terminated by a `NUL`.
    pub sm_rules: Box<[u8]>,
    /// What the match is replaced by.
    pub sm_to_w: Option<Box<[::core::ffi::c_int]>>,
}
#[derive(Debug)]
pub struct SpellLang {
    /// The language name, as `'spelllang'` names it; empty for a file
    /// loaded without one.
    pub sl_name: Vec<u8>,
    /// The `.spl` file it was read from.
    pub sl_fname: XString,
    pub sl_add: bool,
    /// Case-folded words.
    pub sl_fold_tree: WordTree,
    /// Words whose capitalisation no flag can describe.
    pub sl_keep_tree: WordTree,
    /// Postponed prefixes.
    pub sl_prefix_tree: WordTree,
    /// `SN_INFO`'s text, when the file has one.
    pub sl_info: Option<XString>,
    /// Two letters per region, NUL terminated.
    pub sl_regions: [u8; 17],
    /// `MIDWORD`'s characters.
    pub sl_midword: Vec<u8>,
    /// `COMMON` word counts.
    pub sl_wordcount: WordCounts,
    pub sl_compmax: ::core::ffi::c_int,
    pub sl_compminlen: ::core::ffi::c_int,
    pub sl_compsylmax: ::core::ffi::c_int,
    pub sl_compoptions: ::core::ffi::c_int,
    /// `CHECKCOMPOUNDPATTERN`'s pairs, in order.
    pub sl_comppat: Vec<Box<[u8]>>,
    /// COMPOUNDRULE compiled. A cell, because a match may recompile the
    /// program while the language is only borrowed for a lookup.
    pub sl_compprog: core::cell::RefCell<Option<crate::regexp::OwnedProg>>,
    /// The COMPOUNDRULE flags in order, while they form a plain sequence
    /// (no `?`, `+` or `*`).
    pub sl_comprules: Option<Vec<u8>>,
    /// The compound flags that may start a compound.
    pub sl_compstartflags: Vec<u8>,
    /// Every compound flag.
    pub sl_compallflags: Vec<u8>,
    pub sl_nobreak: bool,
    /// `SYLLABLE`'s single syllable characters, when there is the item.
    pub sl_syllable: Option<Vec<u8>>,
    pub sl_syl_items: Vec<SylItem>,
    pub sl_prefixcnt: ::core::ffi::c_int,
    /// The prefix conditions compiled, `sl_prefixcnt` of them; cells for
    /// the reason [`SpellLang::sl_compprog`] is one.
    pub sl_prefprog: Vec<core::cell::RefCell<Option<crate::regexp::OwnedProg>>>,
    pub sl_rep: Vec<RepItem>,
    pub sl_rep_first: [int16_t; 256],
    /// The `SAL` rules, grouped by the low byte of their first character.
    pub sl_sal: Vec<SalItem>,
    /// `SOFOFROM`/`SOFOTO`'s table for characters at or above 256: one
    /// list of `from, to` pairs per low byte, each ending in a zero. 256
    /// entries while the scheme is in force, none otherwise.
    pub sl_sofo_map: Vec<Box<[::core::ffi::c_int]>>,
    /// For `SAL`, the first rule for each low byte, or -1. For `SOFO`, the
    /// direct mapping of every character below 256.
    pub sl_sal_first: [SalFirst; 256],
    pub sl_followup: bool,
    pub sl_collapse: bool,
    pub sl_rem_accents: bool,
    pub sl_sofo: bool,
    pub sl_repsal: Vec<RepItem>,
    pub sl_repsal_first: [int16_t; 256],
    pub sl_nosplitsugs: bool,
    pub sl_nocompoundsugs: bool,
    pub sl_sugtime: time_t,
    /// Sound-folded forms, from the `.sug` file.
    pub sl_sound_tree: WordTree,
    /// The `.sug` file's word-number lines, one per sound-folded word end.
    pub sl_sugbuf: Option<SpellBuf>,
    pub sl_sugloaded: bool,
    pub sl_has_map: bool,
    /// `MAP` groups above 255: each character to its group's head.
    pub sl_map_hash: std::collections::HashMap<::core::ffi::c_int, ::core::ffi::c_int>,
    pub sl_map_array: [::core::ffi::c_int; 256],
}

impl SpellLang {
    /// Whether this language can sound-fold at all, under either scheme.
    pub(crate) fn has_soundfold(&self) -> bool {
        if self.sl_sofo {
            !self.sl_sofo_map.is_empty()
        } else {
            !self.sl_sal.is_empty()
        }
    }
}
pub type SpellMoveType = ::core::ffi::c_uint;
#[derive(Copy, Clone, Debug)]
pub struct SpellTab {
    pub st_isw: [bool; 256],
    pub st_isu: [bool; 256],
    pub st_fold: [uint8_t; 256],
    pub st_upper: [uint8_t; 256],
}
