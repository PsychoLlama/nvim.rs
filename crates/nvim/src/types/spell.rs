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
use crate::spell::{WordTree, syl_item_T};

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
pub type idx_T = ::core::ffi::c_int;
pub struct langp_T {
    pub lp_slang: *mut slang_T,
    pub lp_sallang: *mut slang_T,
    pub lp_replang: *mut slang_T,
    pub lp_region: ::core::ffi::c_int,
}
pub type salfirst_T = ::core::ffi::c_int;
/// One `SAL` sound-folding rule.
///
/// Only the wide forms are kept: the narrow `sm_lead`/`sm_oneof`/`sm_to`
/// the file stores existed to be widened, and nothing but the reader ever
/// looked at them. `sm_rules` stays narrow because it is a run of ASCII
/// flag characters.
pub struct salitem_T {
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
pub struct slang_S {
    pub sl_next: *mut slang_T,
    pub sl_name: *mut ::core::ffi::c_char,
    pub sl_fname: *mut ::core::ffi::c_char,
    pub sl_add: bool,
    /// Case-folded words.
    pub sl_fold_tree: WordTree,
    /// Words whose capitalisation no flag can describe.
    pub sl_keep_tree: WordTree,
    /// Postponed prefixes.
    pub sl_prefix_tree: WordTree,
    pub sl_info: *mut ::core::ffi::c_char,
    pub sl_regions: [::core::ffi::c_char; 17],
    pub sl_midword: *mut ::core::ffi::c_char,
    pub sl_wordcount: hashtab_T,
    pub sl_compmax: ::core::ffi::c_int,
    pub sl_compminlen: ::core::ffi::c_int,
    pub sl_compsylmax: ::core::ffi::c_int,
    pub sl_compoptions: ::core::ffi::c_int,
    /// `CHECKCOMPOUNDPATTERN`'s pairs, in order.
    pub sl_comppat: Vec<Box<[u8]>>,
    pub sl_compprog: *mut regprog_T,
    pub sl_comprules: *mut uint8_t,
    pub sl_compstartflags: *mut uint8_t,
    pub sl_compallflags: *mut uint8_t,
    pub sl_nobreak: bool,
    pub sl_syllable: *mut ::core::ffi::c_char,
    pub sl_syl_items: Vec<syl_item_T>,
    pub sl_prefixcnt: ::core::ffi::c_int,
    pub sl_prefprog: *mut *mut regprog_T,
    pub sl_rep: Vec<RepItem>,
    pub sl_rep_first: [int16_t; 256],
    /// The `SAL` rules, grouped by the low byte of their first character.
    pub sl_sal: Vec<salitem_T>,
    /// `SOFOFROM`/`SOFOTO`'s table for characters at or above 256: one
    /// list of `from, to` pairs per low byte, each ending in a zero. 256
    /// entries while the scheme is in force, none otherwise.
    pub sl_sofo_map: Vec<Box<[::core::ffi::c_int]>>,
    /// For `SAL`, the first rule for each low byte, or -1. For `SOFO`, the
    /// direct mapping of every character below 256.
    pub sl_sal_first: [salfirst_T; 256],
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
    pub sl_sugbuf: *mut buf_T,
    pub sl_sugloaded: bool,
    pub sl_has_map: bool,
    pub sl_map_hash: hashtab_T,
    pub sl_map_array: [::core::ffi::c_int; 256],
    pub sl_sounddone: hashtab_T,
}
pub type slang_T = slang_S;

impl slang_S {
    /// Whether this language can sound-fold at all, under either scheme.
    pub(crate) fn has_soundfold(&self) -> bool {
        if self.sl_sofo {
            !self.sl_sofo_map.is_empty()
        } else {
            !self.sl_sal.is_empty()
        }
    }
}
pub type smt_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct spelltab_T {
    pub st_isw: [bool; 256],
    pub st_isu: [bool; 256],
    pub st_fold: [uint8_t; 256],
    pub st_upper: [uint8_t; 256],
}
/// `#[repr(C)]`: `wc_word` is a flexible array member. A record is
/// `xmalloc(WC_KEY_OFF + len + 1)` and the hash table keys on the inline
/// word, stepping back by `WC_KEY_OFF` to recover the record -- which only
/// describes the allocation while `wc_word` is last.
#[repr(C)]
pub struct wordcount_T {
    pub wc_count: uint16_t,
    pub wc_word: [::core::ffi::c_char; 0],
}
