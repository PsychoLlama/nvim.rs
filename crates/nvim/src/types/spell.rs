#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::spell::{SpellBuf, SylItem, WordTree};

/// `int_wordlist`'s compiled name, as a `vim_snprintf` template.
pub const SPL_FNAME_TMPL: &::core::ffi::CStr = c"%s.%s.spl";

pub type SpellAddType = ::core::ffi::c_uint;
/// One `REP`/`REPSAL` item: text to look for, and what to put in its
/// place. Both ends of the format hold it — the loaded language's tables
/// and `:mkspell`'s, whose copies the writer sorts.
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
#[derive(Copy, Clone)]
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
pub struct SalItem {
    /// The characters this rule matches, terminated by a `NUL`.
    pub sm_lead_w: Box<[::core::ffi::c_int]>,
    /// How many of them there are.
    pub sm_leadlen: ::core::ffi::c_int,
    /// The optional `(abc)` set: any one of these may follow the lead.
    pub sm_oneof_w: Option<Box<[::core::ffi::c_int]>>,
    /// The flag characters after the lead, terminated by a `NUL`.
    pub sm_rules: Box<[u8]>,
    /// What the match is replaced by.
    pub sm_to_w: Option<Box<[::core::ffi::c_int]>>,
}
pub struct SpellLang {
    /// The language name, as `'spelllang'` names it; empty for a file
    /// loaded without one.
    pub sl_name: Vec<u8>,
    /// The `.spl` file it was read from.
    pub sl_fname: crate::memory::XString,
    pub sl_add: bool,
    /// Case-folded words.
    pub sl_fold_tree: WordTree,
    /// Words whose capitalisation no flag can describe.
    pub sl_keep_tree: WordTree,
    /// Postponed prefixes.
    pub sl_prefix_tree: WordTree,
    /// `SN_INFO`'s text, when the file has one.
    pub sl_info: Option<crate::memory::XString>,
    /// Two letters per region, NUL terminated.
    pub sl_regions: [u8; 17],
    /// `MIDWORD`'s characters.
    pub sl_midword: Vec<u8>,
    /// `COMMON` word counts.
    pub sl_wordcount: crate::spell::WordCounts,
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
    pub sl_sounddone: HashTab,
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
#[derive(Copy, Clone)]
pub struct SpellTab {
    pub st_isw: [bool; 256],
    pub st_isu: [bool; 256],
    pub st_fold: [uint8_t; 256],
    pub st_upper: [uint8_t; 256],
}
