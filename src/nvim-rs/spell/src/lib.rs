//! Spell checking utilities for Neovim
//!
//! This crate provides Rust implementations of spell checking functions
//! from `src/nvim/spell.c`, `src/nvim/spellfile.c`, and `src/nvim/spellsuggest.c`.
//!
//! ## Architecture
//!
//! The spell crate is organized into the following modules:
//!
//! - [`spellfile`] - Spell file I/O (.spl file format parsing and writing)
//! - [`check`] - Core spell checking logic (word lookup, case validation)
//! - [`suggest`] - Suggestion algorithms (edit distance, scoring)
//! - [`soundfold`] - Phonetic folding for sound-alike suggestions
//! - [`wordtree`] - Word tree traversal for the compact trie structure
//! - [`wordnode`] - Word node helpers for tree construction
//! - [`compress`] - Word tree compression for mkspell
//! - [`commands`] - Support for spell commands (zg, zw, ]s, [s, etc.)
//!
//! ## Data Structures
//!
//! The spell system uses opaque handles to interface with C:
//!
//! - [`SlangHandle`] - Handle to a spell language (slang_T)
//! - [`LangpHandle`] - Handle to a language pointer entry (langp_T)
//! - [`SpelltabHandle`] - Handle to the spell character table (spelltab_T)
//!
//! ## FFI Integration
//!
//! All public FFI functions are prefixed with `rs_` and are safe to call from C.
//! The functions follow the opaque handle pattern to avoid exposing Rust memory
//! layout to C code.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

pub mod check;
pub mod commands;
pub mod compress;
pub mod soundfold;
pub mod spellfile;
pub mod suggest;
pub mod wordnode;
pub mod wordtree;

// Re-export types for cbindgen
pub use check::{CaseType, SpellResult, WordLookupResult};

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// repr(C) Struct Definitions matching C layout exactly
// =============================================================================

/// Growing array structure matching C garray_T layout.
/// sizeof(garray_T) = 24 bytes on 64-bit.
#[repr(C)]
pub struct GArrayRaw {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// Hashtable item matching C hashitem_T layout.
/// sizeof(hashitem_T) = 16 bytes on 64-bit.
#[repr(C)]
pub struct HashitemRaw {
    pub hi_hash: usize,
    pub hi_key: *mut c_char,
}

/// Hashtable structure matching C hashtab_T layout.
/// sizeof(hashtab_T) = 296 bytes on 64-bit.
/// HT_INIT_SIZE = 16 items inline.
#[repr(C)]
pub struct HashtabRaw {
    pub ht_mask: usize,
    pub ht_used: usize,
    pub ht_filled: usize,
    pub ht_changed: c_int,
    pub ht_locked: c_int,
    pub ht_array: *mut HashitemRaw,
    pub ht_smallarray: [HashitemRaw; 16],
}

/// Spell character table matching C spelltab_T layout.
/// sizeof(spelltab_T) = 1024 bytes.
#[repr(C)]
pub struct SpelltabT {
    pub st_isw: [bool; 256],
    pub st_isu: [bool; 256],
    pub st_fold: [u8; 256],
    pub st_upper: [u8; 256],
}

/// Language pointer entry matching C langp_T layout.
#[repr(C)]
pub struct LangpT {
    pub lp_slang: *mut SlangRaw,
    pub lp_sallang: *mut SlangRaw,
    pub lp_replang: *mut SlangRaw,
    pub lp_region: c_int,
}

/// Spell language structure matching C slang_T layout.
/// sizeof(slang_T) = 4344 bytes on 64-bit.
///
/// Fields after sl_nocompoundsugs (sug file info, map hash, sounddone)
/// are not accessed from Rust and are represented as padding.
#[repr(C)]
pub struct SlangRaw {
    pub sl_next: *mut SlangRaw,        // offset 0
    pub sl_name: *mut c_char,          // offset 8
    pub sl_fname: *mut c_char,         // offset 16
    pub sl_add: bool,                  // offset 24
    _pad0: [u8; 7],                    // padding 25..31
    pub sl_fbyts: *mut u8,             // offset 32
    pub sl_fbyts_len: c_int,           // offset 40
    _pad1: [u8; 4],                    // padding 44..47
    pub sl_fidxs: *mut c_int,          // offset 48
    pub sl_kbyts: *mut u8,             // offset 56
    pub sl_kidxs: *mut c_int,          // offset 64
    pub sl_pbyts: *mut u8,             // offset 72
    pub sl_pidxs: *mut c_int,          // offset 80
    pub sl_info: *mut c_char,          // offset 88
    pub sl_regions: [c_char; 17],      // offset 96 (MAXREGIONS*2+1 = 17)
    _pad2: [u8; 7],                    // padding 113..119
    pub sl_midword: *mut c_char,       // offset 120
    pub sl_wordcount: HashtabRaw,      // offset 128 (size 296)
    pub sl_compmax: c_int,             // offset 424
    pub sl_compminlen: c_int,          // offset 428
    pub sl_compsylmax: c_int,          // offset 432
    pub sl_compoptions: c_int,         // offset 436
    pub sl_comppat: GArrayRaw,         // offset 440 (size 24)
    pub sl_compprog: *mut c_void,      // offset 464
    pub sl_comprules: *mut u8,         // offset 472
    pub sl_compstartflags: *mut u8,    // offset 480
    pub sl_compallflags: *mut u8,      // offset 488
    pub sl_nobreak: bool,              // offset 496
    _pad3: [u8; 7],                    // padding 497..503
    pub sl_syllable: *mut c_char,      // offset 504
    pub sl_syl_items: GArrayRaw,       // offset 512 (size 24)
    pub sl_prefixcnt: c_int,           // offset 536
    _pad4: [u8; 4],                    // padding 540..543
    pub sl_prefprog: *mut *mut c_void, // offset 544
    pub sl_rep: GArrayRaw,             // offset 552 (size 24)
    pub sl_rep_first: [i16; 256],      // offset 576 (size 512)
    pub sl_sal: GArrayRaw,             // offset 1088 (size 24)
    pub sl_sal_first: [c_int; 256],    // offset 1112 (size 1024)
    pub sl_followup: bool,             // offset 2136
    pub sl_collapse: bool,             // offset 2137
    pub sl_rem_accents: bool,          // offset 2138
    pub sl_sofo: bool,                 // offset 2139
    _pad5: [u8; 4],                    // padding 2140..2143
    pub sl_repsal: GArrayRaw,          // offset 2144 (size 24)
    pub sl_repsal_first: [i16; 256],   // offset 2168 (size 512)
    pub sl_nosplitsugs: bool,          // offset 2680
    pub sl_nocompoundsugs: bool,       // offset 2681
    // Remaining fields (sug file info, map hash, sounddone): padding only
    _tail: [u8; 1662], // 4344 - 2682 = 1662 bytes
}

// =============================================================================
// Handle Types (pointer-to-struct wrappers)
// =============================================================================

/// Handle to a spell language (slang_T).
/// Transparent wrapper around *mut SlangRaw for FFI compatibility.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SlangHandle(*mut SlangRaw);

impl SlangHandle {
    /// Creates a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Returns true if this handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Handle to a language pointer entry (langp_T).
///
/// Used for entries in the buffer's b_langp array which maps
/// spell languages to slang_T structures.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct LangpHandle(*mut LangpT);

impl LangpHandle {
    /// Creates a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Returns true if this handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Handle to the global spell character table (spelltab_T).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SpelltabHandle(*mut SpelltabT);

impl SpelltabHandle {
    /// Creates a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Returns true if this handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Type for SAL first lookup table entry
pub type SalfirstT = c_int;

// =============================================================================
// C globals accessed directly via link_name
// =============================================================================

extern "C" {
    /// The first loaded spell language (C global: first_lang)
    #[link_name = "first_lang"]
    static first_lang_global: *mut SlangRaw;

    /// The global spell character table (C global: spelltab)
    #[link_name = "spelltab"]
    static spelltab_global: SpelltabT;
}

// =============================================================================
// Safe Wrappers for slang_T fields (direct struct access)
// =============================================================================

impl SlangHandle {
    /// Get case-folded word bytes array
    #[must_use]
    pub fn fbyts(self) -> *mut u8 {
        unsafe { (*self.0).sl_fbyts }
    }

    /// Get case-folded word indexes array
    #[must_use]
    pub fn fidxs(self) -> *mut IdxT {
        unsafe { (*self.0).sl_fidxs }
    }

    /// Get keep-case word bytes array
    #[must_use]
    pub fn kbyts(self) -> *mut u8 {
        unsafe { (*self.0).sl_kbyts }
    }

    /// Get keep-case word indexes array
    #[must_use]
    pub fn kidxs(self) -> *mut IdxT {
        unsafe { (*self.0).sl_kidxs }
    }

    /// Get prefix tree bytes array
    #[must_use]
    pub fn pbyts(self) -> *mut u8 {
        unsafe { (*self.0).sl_pbyts }
    }

    /// Get prefix tree indexes array
    #[must_use]
    pub fn pidxs(self) -> *mut IdxT {
        unsafe { (*self.0).sl_pidxs }
    }

    /// Get maximum compound word count
    #[must_use]
    pub fn compmax(self) -> c_int {
        unsafe { (*self.0).sl_compmax }
    }

    /// Get minimum compound word length
    #[must_use]
    pub fn compminlen(self) -> c_int {
        unsafe { (*self.0).sl_compminlen }
    }

    /// Get maximum compound syllables
    #[must_use]
    pub fn compsylmax(self) -> c_int {
        unsafe { (*self.0).sl_compsylmax }
    }

    /// Get nobreak flag (no spaces between words)
    #[must_use]
    pub fn nobreak(self) -> bool {
        unsafe { (*self.0).sl_nobreak }
    }

    /// Get SOFOFROM/SOFOTO mode flag
    #[must_use]
    pub fn sofo(self) -> bool {
        unsafe { (*self.0).sl_sofo }
    }

    /// Get remove accents flag
    #[must_use]
    pub fn rem_accents(self) -> bool {
        unsafe { (*self.0).sl_rem_accents }
    }

    /// Get SAL followup flag
    #[must_use]
    pub fn followup(self) -> bool {
        unsafe { (*self.0).sl_followup }
    }

    /// Get SAL collapse flag
    #[must_use]
    pub fn collapse(self) -> bool {
        unsafe { (*self.0).sl_collapse }
    }

    /// Get SAL first lookup table
    #[must_use]
    pub fn sal_first(self) -> *mut SalfirstT {
        unsafe { (*self.0).sl_sal_first.as_mut_ptr() }
    }

    /// Get regions string
    #[must_use]
    pub fn regions(self) -> *const c_char {
        unsafe { (*self.0).sl_regions.as_ptr() }
    }

    /// Get language name (e.g., "en", "en.rare", "nl")
    #[must_use]
    pub fn name(self) -> *const c_char {
        unsafe { (*self.0).sl_name }
    }

    /// Get file name of the .spl file
    #[must_use]
    pub fn fname(self) -> *const c_char {
        unsafe { (*self.0).sl_fname }
    }

    /// Get whether this is an .add file
    #[must_use]
    pub fn is_add(self) -> bool {
        unsafe { (*self.0).sl_add }
    }

    /// Get next language in the linked list
    #[must_use]
    pub fn next(self) -> Self {
        Self(unsafe { (*self.0).sl_next })
    }

    /// Get the first loaded spell language
    #[must_use]
    pub fn first() -> Self {
        Self(unsafe { first_lang_global })
    }

    /// Get compound options flags (COMP_* values)
    #[must_use]
    pub fn compoptions(self) -> c_int {
        unsafe { (*self.0).sl_compoptions }
    }

    /// Get compiled compound regex program
    #[must_use]
    pub fn compprog(self) -> *mut c_void {
        unsafe { (*self.0).sl_compprog }
    }

    /// Get prefix condition regex programs array
    #[must_use]
    pub fn prefprog(self) -> *mut *mut c_void {
        unsafe { (*self.0).sl_prefprog }
    }

    /// Get prefix condition count
    #[must_use]
    pub fn prefixcnt(self) -> c_int {
        unsafe { (*self.0).sl_prefixcnt }
    }

    /// Get compound rules (all COMPOUNDRULE concatenated)
    #[must_use]
    pub fn comprules(self) -> *mut u8 {
        unsafe { (*self.0).sl_comprules }
    }

    /// Get compound start flags (flags for first compound word)
    #[must_use]
    pub fn compstartflags(self) -> *mut u8 {
        unsafe { (*self.0).sl_compstartflags }
    }

    /// Get compound all flags (all flags for compound words)
    #[must_use]
    pub fn compallflags(self) -> *mut u8 {
        unsafe { (*self.0).sl_compallflags }
    }

    /// Get REP items array (garray_T*)
    #[must_use]
    pub fn rep(self) -> *mut c_void {
        unsafe { std::ptr::addr_of_mut!((*self.0).sl_rep).cast::<c_void>() }
    }

    /// Get REP first lookup table
    #[must_use]
    pub fn rep_first(self) -> *mut i16 {
        unsafe { (*self.0).sl_rep_first.as_mut_ptr() }
    }

    /// Get REPSAL items array (garray_T*)
    #[must_use]
    pub fn repsal(self) -> *mut c_void {
        unsafe { std::ptr::addr_of_mut!((*self.0).sl_repsal).cast::<c_void>() }
    }

    /// Get REPSAL first lookup table
    #[must_use]
    pub fn repsal_first(self) -> *mut i16 {
        unsafe { (*self.0).sl_repsal_first.as_mut_ptr() }
    }

    /// Get compound patterns array (garray_T*)
    #[must_use]
    pub fn comppat(self) -> *mut c_void {
        unsafe { std::ptr::addr_of_mut!((*self.0).sl_comppat).cast::<c_void>() }
    }

    /// Get syllable pattern string
    #[must_use]
    pub fn syllable(self) -> *mut c_char {
        unsafe { (*self.0).sl_syllable }
    }

    /// Get midword characters string
    #[must_use]
    pub fn midword(self) -> *mut c_char {
        unsafe { (*self.0).sl_midword }
    }
}

// =============================================================================
// Safe Wrappers for langp_T fields (direct struct access)
// =============================================================================

impl LangpHandle {
    /// Get the associated slang_T handle
    #[must_use]
    pub fn slang(self) -> SlangHandle {
        SlangHandle(unsafe { (*self.0).lp_slang })
    }

    /// Get the sound-alike language for this entry
    #[must_use]
    pub fn sallang(self) -> SlangHandle {
        SlangHandle(unsafe { (*self.0).lp_sallang })
    }

    /// Get the REP items language for this entry
    #[must_use]
    pub fn replang(self) -> SlangHandle {
        SlangHandle(unsafe { (*self.0).lp_replang })
    }

    /// Get the region bitmask (or REGION_ALL)
    #[must_use]
    pub fn region(self) -> c_int {
        unsafe { (*self.0).lp_region }
    }
}

// =============================================================================
// Safe Wrappers for spelltab_T fields (direct struct access)
// =============================================================================

impl SpelltabHandle {
    /// Get the global spelltab
    #[must_use]
    pub fn global() -> Self {
        Self(std::ptr::addr_of!(spelltab_global).cast_mut())
    }

    /// Get is-word flags array (256 entries)
    #[must_use]
    pub fn isw(self) -> *mut bool {
        unsafe { (*self.0).st_isw.as_mut_ptr() }
    }

    /// Get is-uppercase flags array (256 entries)
    #[must_use]
    pub fn isu(self) -> *mut bool {
        unsafe { (*self.0).st_isu.as_mut_ptr() }
    }

    /// Get fold (lowercase) mapping array (256 entries)
    #[must_use]
    pub fn fold(self) -> *mut u8 {
        unsafe { (*self.0).st_fold.as_mut_ptr() }
    }

    /// Get uppercase mapping array (256 entries)
    #[must_use]
    pub fn upper(self) -> *mut u8 {
        unsafe { (*self.0).st_upper.as_mut_ptr() }
    }
}

// =============================================================================
// Spell Chartab Functions
// =============================================================================

/// Initialize the chartab used for spelling for ASCII.
///
/// Sets up the spell character tables with default ASCII values:
/// - Digits 0-9 are word characters
/// - Letters A-Z are word characters and uppercase
/// - Letters a-z are word characters
/// - Case folding maps A-Z to a-z
/// - Uppercase mapping maps a-z to A-Z
///
/// # Safety
///
/// `sp` must be a valid `SpelltabHandle` pointing to a valid `spelltab_T`.
fn clear_spell_chartab_impl(sp: SpelltabHandle) {
    unsafe {
        let isw = sp.isw();
        let isu = sp.isu();
        let fold = sp.fold();
        let upper = sp.upper();

        // Init everything to false/identity
        for i in 0..256 {
            *isw.add(i) = false;
            *isu.add(i) = false;
            #[allow(clippy::cast_possible_truncation)]
            {
                *fold.add(i) = i as u8;
                *upper.add(i) = i as u8;
            }
        }

        // Digits are word characters
        for i in b'0'..=b'9' {
            *isw.add(usize::from(i)) = true;
        }

        // Uppercase letters
        for i in b'A'..=b'Z' {
            let idx = usize::from(i);
            *isw.add(idx) = true;
            *isu.add(idx) = true;
            *fold.add(idx) = i + 0x20; // A-Z -> a-z
        }

        // Lowercase letters
        for i in b'a'..=b'z' {
            let idx = usize::from(i);
            *isw.add(idx) = true;
            *upper.add(idx) = i - 0x20; // a-z -> A-Z
        }
    }
}

/// Initialize the chartab used for spelling for ASCII.
#[export_name = "clear_spell_chartab"]
pub extern "C" fn rs_clear_spell_chartab(sp: SpelltabHandle) {
    clear_spell_chartab_impl(sp);
}

// =============================================================================
// Offset Encoding/Decoding Functions
// =============================================================================

/// Convert an offset into a minimal number of bytes.
///
/// Uses base-255 encoding with high bits to indicate length, similar to UTF-8
/// but avoiding NUL bytes. Returns the number of bytes written (1-4).
///
/// Encoding scheme:
/// - 1 byte:  0x01-0x7F (values 0-126)
/// - 2 bytes: 0x80-0xBF as first byte (values 127-16510)
/// - 3 bytes: 0xC0-0xDF as first byte (values 16511-4210942)
/// - 4 bytes: 0xE0-0xFF as first byte (larger values)
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn offset2bytes(nr: i32, buf: &mut [u8]) -> usize {
    // Split the number in parts of base 255, avoiding NUL bytes
    let b1 = (nr % 255 + 1) as u8;
    let rem = nr / 255;
    let b2 = (rem % 255 + 1) as u8;
    let rem = rem / 255;
    let b3 = (rem % 255 + 1) as u8;
    let b4 = (rem / 255 + 1) as u8;

    if b4 > 1 || b3 > 0x1f {
        // 4 bytes
        buf[0] = 0xe0 + b4;
        buf[1] = b3;
        buf[2] = b2;
        buf[3] = b1;
        4
    } else if b3 > 1 || b2 > 0x3f {
        // 3 bytes
        buf[0] = 0xc0 + b3;
        buf[1] = b2;
        buf[2] = b1;
        3
    } else if b2 > 1 || b1 > 0x7f {
        // 2 bytes
        buf[0] = 0x80 + b2;
        buf[1] = b1;
        2
    } else {
        // 1 byte
        buf[0] = b1;
        1
    }
}

/// Decode bytes back into an offset (opposite of `offset2bytes`).
///
/// Returns (offset, bytes_consumed).
/// Returns None if the buffer is too short.
#[inline]
#[must_use]
pub fn bytes2offset(buf: &[u8]) -> Option<(i32, usize)> {
    if buf.is_empty() {
        return None;
    }

    let c = buf[0];
    if (c & 0x80) == 0x00 {
        // 1 byte
        let nr = i32::from(c) - 1;
        Some((nr, 1))
    } else if (c & 0xc0) == 0x80 {
        // 2 bytes
        if buf.len() < 2 {
            return None;
        }
        let mut nr = i32::from(c & 0x3f) - 1;
        nr = nr * 255 + (i32::from(buf[1]) - 1);
        Some((nr, 2))
    } else if (c & 0xe0) == 0xc0 {
        // 3 bytes
        if buf.len() < 3 {
            return None;
        }
        let mut nr = i32::from(c & 0x1f) - 1;
        nr = nr * 255 + (i32::from(buf[1]) - 1);
        nr = nr * 255 + (i32::from(buf[2]) - 1);
        Some((nr, 3))
    } else {
        // 4 bytes
        if buf.len() < 4 {
            return None;
        }
        let mut nr = i32::from(c & 0x0f) - 1;
        nr = nr * 255 + (i32::from(buf[1]) - 1);
        nr = nr * 255 + (i32::from(buf[2]) - 1);
        nr = nr * 255 + (i32::from(buf[3]) - 1);
        Some((nr, 4))
    }
}

/// FFI wrapper for `offset2bytes`.
///
/// Writes the encoded offset to `buf` and returns the number of bytes written.
///
/// # Safety
///
/// `buf` must point to at least 4 bytes of valid memory.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_offset2bytes(nr: c_int, buf: *mut u8) -> c_int {
    if buf.is_null() {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, 4);
    offset2bytes(nr, slice) as c_int
}

/// FFI wrapper for `bytes2offset`.
///
/// Reads the encoded offset from `*pp` and advances `*pp` past the consumed bytes.
/// Returns the decoded offset.
///
/// # Safety
///
/// `pp` must point to a valid pointer to at least 4 bytes of data.
#[no_mangle]
pub unsafe extern "C" fn rs_bytes2offset(pp: *mut *const u8) -> c_int {
    if pp.is_null() || (*pp).is_null() {
        return 0;
    }

    // Read up to 4 bytes
    let p = *pp;
    let slice = std::slice::from_raw_parts(p, 4);

    match bytes2offset(slice) {
        Some((nr, consumed)) => {
            *pp = p.add(consumed);
            nr
        }
        None => 0,
    }
}

// Region constant from spell_defs.h
/// Word is valid in all regions.
pub const REGION_ALL: c_int = 0xff;

// Word flags from spell_defs.h
const WF_ONECAP: c_int = 0x02; // word with one capital (or all capitals)
const WF_ALLCAP: c_int = 0x04; // word must be all capitals
const WF_FIXCAP: c_int = 0x40; // keep-case word, allcap not allowed
const WF_KEEPCAP: c_int = 0x80; // keep-case word

// =============================================================================
// Spell Suggestion Scoring Constants (from spellsuggest.c)
// =============================================================================

/// Score for splitting a word
pub const SCORE_SPLIT: c_int = 149;
/// Score for splitting with NOSPLITSUGS
pub const SCORE_SPLIT_NO: c_int = 249;
/// Score for slightly different case
pub const SCORE_ICASE: c_int = 52;
/// Score for word in different region
pub const SCORE_REGION: c_int = 200;
/// Score for rare word
pub const SCORE_RARE: c_int = 180;
/// Score for swapping two characters
pub const SCORE_SWAP: c_int = 75;
/// Score for swap in three characters
pub const SCORE_SWAP3: c_int = 110;
/// Score for REP replacement
pub const SCORE_REP: c_int = 65;
/// Score for character substitution
pub const SCORE_SUBST: c_int = 93;
/// Score for similar character substitution
pub const SCORE_SIMILAR: c_int = 33;
/// Score for composing character substitution
pub const SCORE_SUBCOMP: c_int = 33;
/// Score for deleting a character
pub const SCORE_DEL: c_int = 94;
/// Score for deleting a duplicate character
pub const SCORE_DELDUP: c_int = 66;
/// Score for deleting a composing character
pub const SCORE_DELCOMP: c_int = 28;
/// Score for inserting a character
pub const SCORE_INS: c_int = 96;
/// Score for inserting a duplicate character
pub const SCORE_INSDUP: c_int = 67;
/// Score for inserting a composing character
pub const SCORE_INSCOMP: c_int = 30;
/// Score for changing non-word to word char
pub const SCORE_NONWORD: c_int = 103;

/// Score for suggestion from a file
pub const SCORE_FILE: c_int = 30;
/// Initial maximum score (higher = slower)
pub const SCORE_MAXINIT: c_int = 350;

/// Score subtracted for words seen before
pub const SCORE_COMMON1: c_int = 30;
/// Score subtracted for words often seen
pub const SCORE_COMMON2: c_int = 40;
/// Score subtracted for words very often seen
pub const SCORE_COMMON3: c_int = 50;
/// Word count threshold for COMMON2
pub const SCORE_THRES2: c_int = 10;
/// Word count threshold for COMMON3
pub const SCORE_THRES3: c_int = 100;

/// Maximum score for first soundfold try
pub const SCORE_SFMAX1: c_int = 200;
/// Maximum score for second soundfold try
pub const SCORE_SFMAX2: c_int = 300;
/// Maximum score for third soundfold try
pub const SCORE_SFMAX3: c_int = 400;

/// Accept any score (very large value)
pub const SCORE_MAXMAX: c_int = 999_999;
/// Maximum for spell_edit_score_limit
pub const SCORE_LIMITMAX: c_int = 350;

/// Minimum edit score (used for optimization)
pub const SCORE_EDIT_MIN: c_int = SCORE_SIMILAR;

/// Big difference score (3x insert)
pub const SCORE_BIG: c_int = SCORE_INS * 3;

// =============================================================================
// Spell File Format Constants (from spellfile.c)
// =============================================================================

/// Magic string at start of Vim spell file
pub const VIMSPELL_MAGIC: &[u8; 8] = b"VIMspell";
/// Length of magic string
pub const VIMSPELL_MAGIC_LEN: usize = 8;
/// Current spell file version
pub const VIMSPELL_VERSION: u8 = 50;

/// Section IDs for spell file format
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSection {
    /// Region section
    Region = 0,
    /// Character flags section
    CharFlags = 1,
    /// Middle word characters
    MidWord = 2,
    /// Prefix conditions
    PrefCond = 3,
    /// Replacements
    Rep = 4,
    /// Soundalike replacements
    RepSal = 5,
    /// Soundalike conversion
    Sal = 6,
    /// Soundfold
    SoFo = 7,
    /// Map of similar characters
    Map = 8,
    /// Compound rules
    Compound = 9,
    /// No break
    NoBreak = 10,
    /// Suggestion file timestamp
    SugFile = 11,
    /// Don't split for suggestions
    NoSplitSugs = 12,
    /// Don't compound for suggestions
    NoCompoundSugs = 13,
    /// Common words
    Words = 14,
    /// Syllable info
    Syllable = 15,
    /// Info text
    Info = 16,
    /// End of sections
    End = 255,
}

/// Section flags
pub const SNF_REQUIRED: u8 = 1; // Section is required for correct spell checking

/// Byte values used in word tree
pub const BY_NOFLAGS: u8 = 0; // End of word without flags
pub const BY_FLAGS: u8 = 1; // End of word, flags follow
pub const BY_FLAGS2: u8 = 2; // End of word, flags and flags2 follow
pub const BY_INDEX: u8 = 3; // Child is shared, index follows

/// Character flags for spell file (charflags section)
pub const CF_WORD: u8 = 0x01; // Word character
pub const CF_UPPER: u8 = 0x02; // Upper-case character

// =============================================================================
// Spell Error Constants (from spell_defs.h)
// =============================================================================

/// Spell file truncated error
pub const SP_TRUNCERROR: c_int = -1;
/// Format error in spell file
pub const SP_FORMERROR: c_int = -2;
/// Other error while reading spell file
pub const SP_OTHERERROR: c_int = -3;

// =============================================================================
// Word Flags (from spell_defs.h)
// =============================================================================

/// Region byte follows
pub const WF_REGION: c_int = 0x01;
/// Word with one capital (or all capitals)
pub const WF_ONECAP_FLAG: c_int = 0x02;
/// Word must be all capitals
pub const WF_ALLCAP_FLAG: c_int = 0x04;
/// Rare word
pub const WF_RARE: c_int = 0x08;
/// Bad/banned word
pub const WF_BANNED: c_int = 0x10;
/// Affix ID follows
pub const WF_AFX: c_int = 0x20;
/// Keep-case word, allcap not allowed
pub const WF_FIXCAP_FLAG: c_int = 0x40;
/// Keep-case word
pub const WF_KEEPCAP_FLAG: c_int = 0x80;

/// Maximum word length in bytes
pub const MAXWLEN: usize = 254;

// =============================================================================
// Binary Format Reading Functions
// =============================================================================

/// Check if a byte buffer starts with the Vim spell magic string.
///
/// Returns true if the first 8 bytes match "VIMspell".
#[inline]
#[must_use]
pub const fn check_spell_magic(buf: &[u8]) -> bool {
    if buf.len() < VIMSPELL_MAGIC_LEN {
        return false;
    }
    buf[0] == VIMSPELL_MAGIC[0]
        && buf[1] == VIMSPELL_MAGIC[1]
        && buf[2] == VIMSPELL_MAGIC[2]
        && buf[3] == VIMSPELL_MAGIC[3]
        && buf[4] == VIMSPELL_MAGIC[4]
        && buf[5] == VIMSPELL_MAGIC[5]
        && buf[6] == VIMSPELL_MAGIC[6]
        && buf[7] == VIMSPELL_MAGIC[7]
}

/// FFI wrapper to check spell magic from C buffer.
///
/// Returns true if the first 8 bytes match "VIMspell".
///
/// # Safety
///
/// `buf` must point to at least 8 bytes of valid memory.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_check_spell_magic(buf: *const u8) -> bool {
    if buf.is_null() {
        return false;
    }
    let slice = std::slice::from_raw_parts(buf, VIMSPELL_MAGIC_LEN);
    check_spell_magic(slice)
}

/// Check if a spell file version is compatible.
///
/// Returns true if the version can be read (not too old or too new).
#[inline]
#[must_use]
pub const fn check_spell_version(version: u8) -> bool {
    version == VIMSPELL_VERSION
}

/// FFI wrapper to check spell version compatibility.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_check_spell_version(version: u8) -> bool {
    check_spell_version(version)
}

/// Check if a spell file version is too old.
#[inline]
#[must_use]
pub const fn spell_version_too_old(version: u8) -> bool {
    version < VIMSPELL_VERSION
}

/// FFI wrapper to check if spell version is too old.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_spell_version_too_old(version: u8) -> bool {
    spell_version_too_old(version)
}

/// Check if a spell file version is too new.
#[inline]
#[must_use]
pub const fn spell_version_too_new(version: u8) -> bool {
    version > VIMSPELL_VERSION
}

/// FFI wrapper to check if spell version is too new.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_spell_version_too_new(version: u8) -> bool {
    spell_version_too_new(version)
}

/// Read a 2-byte big-endian integer from a buffer.
///
/// Returns the value and advances the offset by 2.
/// Returns None if there aren't enough bytes.
#[inline]
#[must_use]
pub const fn read_be_u16(buf: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 > buf.len() {
        return None;
    }
    Some(((buf[offset] as u16) << 8) | (buf[offset + 1] as u16))
}

/// Read a 3-byte big-endian integer from a buffer.
///
/// Returns the value and advances the offset by 3.
/// Returns None if there aren't enough bytes.
#[inline]
#[must_use]
pub const fn read_be_u24(buf: &[u8], offset: usize) -> Option<u32> {
    if offset + 3 > buf.len() {
        return None;
    }
    Some(((buf[offset] as u32) << 16) | ((buf[offset + 1] as u32) << 8) | (buf[offset + 2] as u32))
}

/// Read a 4-byte big-endian integer from a buffer.
///
/// Returns the value and advances the offset by 4.
/// Returns None if there aren't enough bytes.
#[inline]
#[must_use]
pub const fn read_be_u32(buf: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > buf.len() {
        return None;
    }
    Some(
        ((buf[offset] as u32) << 24)
            | ((buf[offset + 1] as u32) << 16)
            | ((buf[offset + 2] as u32) << 8)
            | (buf[offset + 3] as u32),
    )
}

/// FFI wrapper to read 2-byte big-endian integer.
///
/// Returns the value or -1 on error.
///
/// # Safety
///
/// `buf` must point to at least `offset + 2` bytes of valid memory.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_read_be_u16(buf: *const u8, len: usize, offset: usize) -> c_int {
    if buf.is_null() || offset + 2 > len {
        return -1;
    }
    let slice = std::slice::from_raw_parts(buf, len);
    read_be_u16(slice, offset).map_or(-1, c_int::from)
}

/// FFI wrapper to read 3-byte big-endian integer.
///
/// Returns the value or -1 on error.
///
/// # Safety
///
/// `buf` must point to at least `offset + 3` bytes of valid memory.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_read_be_u24(buf: *const u8, len: usize, offset: usize) -> c_int {
    if buf.is_null() || offset + 3 > len {
        return -1;
    }
    let slice = std::slice::from_raw_parts(buf, len);
    read_be_u24(slice, offset).map_or(-1, |v| v as c_int)
}

/// FFI wrapper to read 4-byte big-endian integer.
///
/// Returns the value or -1 on error.
///
/// # Safety
///
/// `buf` must point to at least `offset + 4` bytes of valid memory.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_read_be_u32(buf: *const u8, len: usize, offset: usize) -> c_int {
    if buf.is_null() || offset + 4 > len {
        return -1;
    }
    let slice = std::slice::from_raw_parts(buf, len);
    read_be_u32(slice, offset).map_or(-1, |v| v as c_int)
}

/// Parse a spell section header.
///
/// Returns (section_id, flags, length, bytes_read) or None on error.
/// The section header is: <sectionID:1> <sectionflags:1> <sectionlen:4>
#[inline]
#[must_use]
#[allow(clippy::manual_let_else)] // const fn cannot use let...else
pub const fn parse_section_header(buf: &[u8], offset: usize) -> Option<(u8, u8, u32, usize)> {
    if offset + 6 > buf.len() {
        return None;
    }
    let section_id = buf[offset];
    let flags = buf[offset + 1];
    let len = match read_be_u32(buf, offset + 2) {
        Some(v) => v,
        None => return None,
    };
    Some((section_id, flags, len, 6))
}

// =============================================================================
// Word Flag Functions
// =============================================================================

/// Check if the word flags match the tree flags for valid case handling.
///
/// Returns true if case handling is valid:
/// - word is ALLCAP and tree doesn't require FIXCAP, OR
/// - tree doesn't have ALLCAP/KEEPCAP, and either tree doesn't have ONECAP
///   or word has ONECAP
#[inline]
const fn spell_valid_case_impl(wordflags: c_int, treeflags: c_int) -> bool {
    // (wordflags == WF_ALLCAP && (treeflags & WF_FIXCAP) == 0)
    // || ((treeflags & (WF_ALLCAP | WF_KEEPCAP)) == 0
    //     && ((treeflags & WF_ONECAP) == 0
    //         || (wordflags & WF_ONECAP) != 0))
    (wordflags == WF_ALLCAP && (treeflags & WF_FIXCAP) == 0)
        || ((treeflags & (WF_ALLCAP | WF_KEEPCAP)) == 0
            && ((treeflags & WF_ONECAP) == 0 || (wordflags & WF_ONECAP) != 0))
}

/// Check if the word flags match the tree flags for valid case handling.
#[must_use]
#[export_name = "spell_valid_case"]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_spell_valid_case(wordflags: c_int, treeflags: c_int) -> bool {
    spell_valid_case_impl(wordflags, treeflags)
}

/// Check if byte `n` appears in string `str`.
///
/// Like `strchr()` but independent of locale.
/// Returns true if the byte is found.
#[inline]
#[allow(clippy::cast_sign_loss)] // n is always in range 0-255 for byte values
#[allow(clippy::cast_possible_truncation)] // n is always in range 0-255 for byte values
#[allow(clippy::missing_const_for_fn)] // unsafe blocks prevent const
fn byte_in_str_impl(str: *const u8, n: c_int) -> bool {
    if str.is_null() {
        return false;
    }

    let n = n as u8;
    let mut p = str;

    // SAFETY: We iterate until we hit NUL, which is the contract
    unsafe {
        while *p != 0 {
            if *p == n {
                return true;
            }
            p = p.add(1);
        }
    }
    false
}

/// Check if byte `n` appears in string `str`.
#[must_use]
#[export_name = "byte_in_str"]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_byte_in_str(str: *const u8, n: c_int) -> bool {
    byte_in_str_impl(str, n)
}

/// Allowed characters for 'spelllang' option value.
const SPELLLANG_ALLOWED: &[u8] = b".-_,@";

/// Check if a string is a valid 'spelllang' value.
///
/// Valid spelllang values contain only alphanumeric characters,
/// dots, hyphens, underscores, commas, and @ signs.
///
/// # Safety
///
/// `val` must be a valid null-terminated C string.
#[must_use]
#[export_name = "valid_spelllang"]
pub unsafe extern "C" fn rs_valid_spelllang(val: *const c_char) -> bool {
    if val.is_null() {
        return true;
    }

    // Convert C string to slice
    let mut len = 0usize;
    let mut p = val;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    let slice = std::slice::from_raw_parts(val as *const u8, len);
    nvim_strings::valid_name(slice, SPELLLANG_ALLOWED)
}

/// Check if a string is a valid 'spellfile' value.
///
/// Valid spellfile values are comma-separated file paths where each path:
/// - Has at least 4 characters
/// - Ends with ".add"
/// - Contains only valid filename characters
///
/// # Safety
///
/// `val` must be a valid null-terminated C string.
#[must_use]
#[export_name = "valid_spellfile"]
pub unsafe extern "C" fn rs_valid_spellfile(val: *const c_char) -> bool {
    if val.is_null() {
        return true;
    }

    let val_ptr = val as *const u8;

    // Convert C string to slice
    let mut len = 0usize;
    let mut p = val_ptr;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if len == 0 {
        return true;
    }

    let slice = std::slice::from_raw_parts(val_ptr, len);
    valid_spellfile_impl(slice)
}

/// Check if a character is a valid filename character (simplified check).
///
/// This is a simplified version that checks for printable ASCII characters
/// that are commonly allowed in filenames. The full C version uses the
/// 'isfname' option which is runtime-configurable.
#[inline]
const fn is_fname_char(c: u8) -> bool {
    // Allow alphanumeric, and common path characters
    // This matches the default 'isfname' for most systems
    c.is_ascii_alphanumeric()
        || c == b'_'
        || c == b'-'
        || c == b'.'
        || c == b'/'
        || c == b'\\'
        || c == b':'
        || c == b'~'
        || c == b'@'
        || c == b'!'
        || c == b'#'
        || c == b'$'
        || c == b'%'
        || c == b'&'
        || c == b'('
        || c == b')'
        || c == b'+'
        || c == b'='
        || c == b'{'
        || c == b'}'
        || c == b'['
        || c == b']'
        || c >= 0x80 // Allow high bytes (UTF-8 continuation)
}

/// Implementation of spellfile validation.
///
/// Parses comma-separated file paths, handling backslash escapes.
fn valid_spellfile_impl(val: &[u8]) -> bool {
    let mut pos = 0;

    while pos < val.len() {
        // Skip leading whitespace (like skip_to_option_part does)
        while pos < val.len() && (val[pos] == b' ' || val[pos] == b'\t') {
            pos += 1;
        }

        if pos >= val.len() {
            break;
        }

        // Extract one part (until comma or end)
        let part_start = pos;
        let mut part_len = 0;

        while pos < val.len() && val[pos] != b',' {
            // Handle backslash escape before comma
            if val[pos] == b'\\' && pos + 1 < val.len() && val[pos + 1] == b',' {
                pos += 1; // Skip backslash, include comma as part of path
            }
            part_len += 1;
            pos += 1;
        }

        // Skip the comma separator
        if pos < val.len() && val[pos] == b',' {
            pos += 1;
        }

        // Validate the part
        // Part must be at least 4 characters and end with ".add"
        if part_len < 4 {
            return false;
        }

        // Get the actual part bytes (need to re-extract handling escapes)
        let mut part = Vec::with_capacity(part_len);
        let mut scan = part_start;
        while scan < part_start + part_len + (pos - part_start - part_len) && part.len() < part_len
        {
            if val[scan] == b'\\' && scan + 1 < val.len() && val[scan + 1] == b',' {
                scan += 1; // Skip backslash
            }
            if scan < val.len() && val[scan] != b',' {
                part.push(val[scan]);
            }
            scan += 1;
        }

        // Check suffix ".add"
        if part.len() < 4 || &part[part.len() - 4..] != b".add" {
            return false;
        }

        // Check all characters are valid filename characters
        for &c in &part {
            if !is_fname_char(c) {
                return false;
            }
        }
    }

    true
}

/// Find a region in the region list.
///
/// The region list (from `sl_regions`) stores region names as consecutive
/// pairs of ASCII characters (e.g., "usuk" for "us" and "uk" regions).
///
/// # Arguments
///
/// * `rp` - Pointer to the region list string (NUL-terminated, pairs of chars)
/// * `region` - Pointer to a 2-character region name to find
///
/// # Returns
///
/// The index of the region if found (0 for first region, 1 for second, etc.),
/// or `REGION_ALL` (0xff) if not found.
///
/// # Safety
///
/// Both `rp` and `region` must be valid null-terminated C strings.
/// `region` must point to at least 2 characters.
#[inline]
#[allow(clippy::cast_possible_wrap)] // index is always small and positive
#[allow(clippy::cast_possible_truncation)] // index is always small and positive
#[allow(clippy::missing_const_for_fn)] // requires unsafe const which has limitations
unsafe fn find_region_impl(rp: *const c_char, region: *const c_char) -> c_int {
    if rp.is_null() || region.is_null() {
        return REGION_ALL;
    }

    let r0 = *region;
    let r1 = *region.add(1);

    let mut i: usize = 0;
    loop {
        let c0 = *rp.add(i);
        if c0 == 0 {
            // End of region list, not found
            return REGION_ALL;
        }
        let c1 = *rp.add(i + 1);

        if c0 == r0 && c1 == r1 {
            // Found matching region
            return (i / 2) as c_int;
        }

        i += 2;
    }
}

/// FFI wrapper for `find_region`.
///
/// Find the region `region[0..2]` in the region list `rp`.
/// Returns the index if found (first is 0), REGION_ALL (0xff) if not found.
///
/// # Safety
///
/// Both `rp` and `region` must be valid null-terminated C strings.
/// `region` must point to at least 2 characters.
#[no_mangle]
pub unsafe extern "C" fn rs_find_region(rp: *const c_char, region: *const c_char) -> c_int {
    find_region_impl(rp, region)
}

/// Convert a SAL line argument to boolean.
///
/// Returns true if the string is "1" or "true", false otherwise.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string.
#[inline]
#[allow(clippy::missing_const_for_fn)] // unsafe blocks prevent const
#[allow(clippy::cast_possible_wrap)] // ASCII chars are always valid in both u8 and i8
unsafe fn sal_to_bool_impl(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    // Check for "1"
    if *s == b'1' as c_char && *s.add(1) == 0 {
        return true;
    }

    // Check for "true"
    if *s == b't' as c_char
        && *s.add(1) == b'r' as c_char
        && *s.add(2) == b'u' as c_char
        && *s.add(3) == b'e' as c_char
        && *s.add(4) == 0
    {
        return true;
    }

    false
}

/// FFI wrapper for `sal_to_bool`.
///
/// Converts a boolean argument in a SAL line to true or false.
/// Returns true if the string is "1" or "true", false otherwise.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sal_to_bool(s: *const c_char) -> bool {
    sal_to_bool_impl(s)
}

// =============================================================================
// Word class checking
// =============================================================================

/// Check if a word class indicates a word character.
///
/// Only for characters above 255 (multibyte characters).
/// Unicode subscript and superscript are not considered word characters.
/// See also `utf_class()` in mbyte.c.
///
/// # Arguments
///
/// * `cl` - The character class from `utf_class()` or `mb_get_class()`
/// * `cjk` - True if CJK mode is enabled (East Asian characters not word chars)
///
/// # Returns
///
/// True if the character class indicates a word character.
#[inline]
const fn spell_mb_isword_class_impl(cl: c_int, cjk: bool) -> bool {
    if cjk {
        // East Asian characters are not considered word characters.
        // Only class 2 (word char) and 0x2800 (Braille) are valid.
        cl == 2 || cl == 0x2800
    } else {
        // Normal mode: class >= 2 but not subscript (0x2070), superscript (0x2080), or 3
        cl >= 2 && cl != 0x2070 && cl != 0x2080 && cl != 3
    }
}

/// FFI wrapper for `spell_mb_isword_class`.
///
/// Check if a word class indicates a word character (for multibyte chars).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_spell_mb_isword_class(cl: c_int, cjk: bool) -> bool {
    spell_mb_isword_class_impl(cl, cjk)
}

// =============================================================================
// Phase 3: Case/Word Utility Functions
// =============================================================================

extern "C" {
    // Multibyte character functions
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn mb_get_class(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_isupper(c: c_int) -> bool;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn utf_class(c: c_int) -> c_int;

    // Window synblock accessors (window_shim.c)
    fn nvim_win_get_b_cjk(wp: *const c_void) -> bool;
    fn nvim_win_get_b_spell_ismw(wp: *const c_void) -> *const bool;
    fn nvim_win_get_b_spell_ismw_mb(wp: *const c_void) -> *const c_char;

    // curwin global
    #[link_name = "curwin"]
    static curwin_global: *mut c_void;
}

/// Fold character using spelltab or utf_fold for high chars (SPELL_TOFOLD macro).
#[inline]
#[allow(clippy::cast_sign_loss)] // c is always in 0..128 range in the else branch
unsafe fn spell_tofold(c: c_int) -> c_int {
    if c >= 128 {
        utf_fold(c)
    } else {
        let fold_table = (*std::ptr::addr_of!(spelltab_global)).st_fold.as_ptr();
        c_int::from(*fold_table.add(c as usize))
    }
}

/// Uppercase character using spelltab or mb_toupper for high chars (SPELL_TOUPPER macro).
#[inline]
#[allow(clippy::cast_sign_loss)] // c is always in 0..128 range in the else branch
unsafe fn spell_toupper(c: c_int) -> c_int {
    if c >= 128 {
        mb_toupper(c)
    } else {
        let upper_table = (*std::ptr::addr_of!(spelltab_global)).st_upper.as_ptr();
        c_int::from(*upper_table.add(c as usize))
    }
}

/// Check if character is uppercase using spelltab or mb_isupper (SPELL_ISUPPER macro).
#[inline]
#[allow(clippy::cast_sign_loss)] // c is always in 0..128 range in the else branch
unsafe fn spell_isupper(c: c_int) -> bool {
    if c >= 128 {
        mb_isupper(c)
    } else {
        let isu_table = (*std::ptr::addr_of!(spelltab_global)).st_isu.as_ptr();
        *isu_table.add(c as usize)
    }
}

/// Check if a char class is a word char for high-byte chars.
#[inline]
unsafe fn iswordp_mb(p: *const c_char, wp: *const c_void) -> bool {
    spell_mb_isword_class_impl(mb_get_class(p), nvim_win_get_b_cjk(wp))
}

/// Returns true if "p" points to a word character.
/// Unlike spell_iswordp() this doesn't check for "midword" characters.
///
/// # Safety
/// `p` and `wp` must be valid pointers.
#[must_use]
#[export_name = "spell_iswordp_nmw"]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_sign_loss)] // c is validated > 0 by utf_ptr2char
pub unsafe extern "C" fn rs_spell_iswordp_nmw(p: *const c_char, wp: *const c_void) -> bool {
    let c = utf_ptr2char(p);
    if c > 255 {
        iswordp_mb(p, wp)
    } else {
        let isw = (*std::ptr::addr_of!(spelltab_global)).st_isw.as_ptr();
        // SAFETY: c >= 0 and c <= 255, so cast is safe
        *isw.add(c as u32 as usize)
    }
}

/// Returns true if "p" points to a word character.
/// Checks for "midword" characters (skips them when checking the next char).
///
/// # Safety
/// `p` and `wp` must be valid pointers.
#[must_use]
#[export_name = "spell_iswordp"]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_sign_loss)] // lengths are always positive
pub unsafe extern "C" fn rs_spell_iswordp(p: *const c_char, wp: *const c_void) -> bool {
    let l = utfc_ptr2len(p) as usize;
    let s = if l == 1 {
        // ASCII: check midword
        let b = *(p as *const u8);
        let ismw = nvim_win_get_b_spell_ismw(wp);
        if !ismw.is_null() && *ismw.add(b as usize) {
            p.add(1)
        } else {
            p
        }
    } else {
        let c = utf_ptr2char(p);
        if c < 256 {
            let ismw = nvim_win_get_b_spell_ismw(wp);
            // SAFETY: c >= 0 and c < 256, so cast is safe
            if !ismw.is_null() && *ismw.add(c as u32 as usize) {
                p.add(l)
            } else {
                p
            }
        } else {
            let ismw_mb = nvim_win_get_b_spell_ismw_mb(wp);
            if !ismw_mb.is_null() && !vim_strchr(ismw_mb, c).is_null() {
                p.add(l)
            } else {
                p
            }
        }
    };

    let c = utf_ptr2char(s);
    if c > 255 {
        iswordp_mb(s, wp)
    } else {
        let isw = (*std::ptr::addr_of!(spelltab_global)).st_isw.as_ptr();
        // SAFETY: c >= 0 and c <= 255, so cast is safe
        *isw.add(c as u32 as usize)
    }
}

/// Wide version of spell_iswordp() — checks if wide-char position is a word char.
///
/// # Safety
/// `p` and `wp` must be valid pointers.
#[must_use]
#[export_name = "spell_iswordp_w"]
#[allow(clippy::cast_sign_loss)] // *p and *s are validated >= 0 at check sites
pub unsafe extern "C" fn rs_spell_iswordp_w(p: *const c_int, wp: *const c_void) -> bool {
    let s = if *p < 256 {
        let ismw = nvim_win_get_b_spell_ismw(wp);
        if !ismw.is_null() && *ismw.add(*p as u32 as usize) {
            p.add(1)
        } else {
            p
        }
    } else {
        let ismw_mb = nvim_win_get_b_spell_ismw_mb(wp);
        if !ismw_mb.is_null() && !vim_strchr(ismw_mb, *p).is_null() {
            p.add(1)
        } else {
            p
        }
    };

    if *s > 255 {
        spell_mb_isword_class_impl(utf_class(*s), nvim_win_get_b_cjk(wp))
    } else {
        let isw = (*std::ptr::addr_of!(spelltab_global)).st_isw.as_ptr();
        *isw.add(*s as u32 as usize)
    }
}

/// Determine case type of a word.
///
/// Returns:
/// - 0: all lower (or only non-word chars)
/// - WF_ONECAP: first char upper, rest lower
/// - WF_ALLCAP: all caps
/// - WF_KEEPCAP: mixed upper/lower
///
/// # Safety
/// `word` must be a valid NUL-terminated C string.
#[export_name = "captype"]
#[allow(clippy::cast_sign_loss)] // lengths and char indices are always >= 0
#[allow(clippy::cast_possible_wrap)] // WF_* constants fit in c_int
pub unsafe extern "C" fn rs_captype(word: *const c_char, end: *const c_char) -> c_int {
    let mut p = word;

    // Find first word character
    loop {
        if end.is_null() {
            if *p == 0 {
                return 0; // only non-word chars
            }
        } else if p >= end {
            return 0;
        }
        if rs_spell_iswordp_nmw(p, curwin_global) {
            break;
        }
        // advance pointer (MB_PTR_ADV)
        let l = utfc_ptr2len(p).max(1) as usize;
        p = p.add(l);
    }

    // mb_cptr2char_adv advances p in-place via the pointer-to-pointer
    let c = mb_cptr2char_adv(std::ptr::addr_of_mut!(p));
    let firstcap = spell_isupper(c);
    let mut allcap = firstcap;
    let mut past_second = false;

    loop {
        if end.is_null() {
            if *p == 0 {
                break;
            }
        } else if p >= end {
            break;
        }
        if rs_spell_iswordp_nmw(p, curwin_global) {
            let c2 = utf_ptr2char(p);
            if !spell_isupper(c2) {
                if past_second && allcap {
                    return crate::wordtree::WF_KEEPCAP as c_int;
                }
                allcap = false;
            } else if !allcap {
                return crate::wordtree::WF_KEEPCAP as c_int;
            }
            past_second = true;
        }
        let l = utfc_ptr2len(p).max(1) as usize;
        p = p.add(l);
    }

    if allcap {
        return crate::wordtree::WF_ALLCAP as c_int;
    }
    if firstcap {
        return crate::wordtree::WF_ONECAP as c_int;
    }
    0
}

/// Copy word with first letter case changed.
///
/// If `upper` is true, capitalize the first letter; otherwise fold it.
///
/// # Safety
/// `word` and `wcopy` must be valid pointers. `wcopy` must have at least MAXWLEN bytes.
#[export_name = "onecap_copy"]
#[allow(clippy::cast_sign_loss)] // l is utf_char2bytes result, always >= 0
pub unsafe extern "C" fn rs_onecap_copy(word: *const c_char, wcopy: *mut c_char, upper: bool) {
    let mut p = word;
    let mut c = mb_cptr2char_adv(std::ptr::addr_of_mut!(p));
    c = if upper {
        spell_toupper(c)
    } else {
        spell_tofold(c)
    };
    let byte_len = utf_char2bytes(c, wcopy) as usize;
    xstrlcpy(wcopy.add(byte_len), p, MAXWLEN.saturating_sub(byte_len));
}

/// Copy word with all letters uppercased.
///
/// # Safety
/// `word` and `wcopy` must be valid pointers. `wcopy` must have at least MAXWLEN bytes.
#[export_name = "allcap_copy"]
#[allow(clippy::cast_sign_loss)] // bytes result is always >= 0
#[allow(clippy::cast_possible_wrap)] // MAXWLEN fits in isize
#[allow(clippy::cast_possible_truncation)] // c is always ASCII 'S' (0x53) in the truncation path
pub unsafe extern "C" fn rs_allcap_copy(word: *const c_char, wcopy: *mut c_char) {
    let mut d = wcopy;
    let mut src = word;

    while *(src as *const u8) != 0 {
        let mut c = mb_cptr2char_adv(std::ptr::addr_of_mut!(src));

        if c == 0xdf {
            // German sharp-s: becomes 'S'
            c = i32::from(b'S');
            if d.offset_from(wcopy) >= (MAXWLEN as isize) - 1 {
                break;
            }
            *d = c as c_char;
            d = d.add(1);
        } else {
            c = spell_toupper(c);
        }

        if d.offset_from(wcopy) >= (MAXWLEN as isize) - 4 {
            // MB_MAXBYTES = 4
            break;
        }
        let byte_count = utf_char2bytes(c, d) as usize;
        d = d.add(byte_count);
    }
    *d = 0;
}

/// Count byte length of N chars in original word.
///
/// Case-folding may change the number of bytes: count nr of chars in
/// fword[flen] and return the byte length of that many chars in "word".
///
/// # Safety
/// `fword` and `word` must be valid pointers.
#[export_name = "nofold_len"]
#[must_use]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_sign_loss)] // lengths are always positive
#[allow(clippy::cast_possible_truncation)] // word byte count fits in c_int for any real word
pub unsafe extern "C" fn rs_nofold_len(
    fword: *const c_char,
    flen: c_int,
    word: *const c_char,
) -> c_int {
    // Count number of chars in fword[0..flen]
    let mut fp = fword;
    let fword_end = fword.add(flen as usize);
    let mut char_count: c_int = 0;
    while fp < fword_end {
        let step = utfc_ptr2len(fp).max(1) as usize;
        fp = fp.add(step);
        char_count += 1;
    }

    // Count that many chars in word
    let mut wp = word;
    let mut remaining = char_count;
    while remaining > 0 {
        let step = utfc_ptr2len(wp).max(1) as usize;
        wp = wp.add(step);
        remaining -= 1;
    }
    wp.offset_from(word) as c_int
}

/// Copy "fword" to "cword", fixing case according to "flags".
///
/// # Safety
/// `fword` and `cword` must be valid pointers. `cword` must have MAXWLEN bytes.
#[export_name = "make_case_word"]
#[allow(clippy::cast_possible_wrap)] // WF_* constants fit in c_int
#[allow(clippy::cast_possible_truncation)] // byte values are always 0..=127 for ASCII chars
pub unsafe extern "C" fn rs_make_case_word(fword: *const c_char, cword: *mut c_char, flags: c_int) {
    if flags & (crate::wordtree::WF_ALLCAP as c_int) != 0 {
        rs_allcap_copy(fword, cword);
    } else if flags & (crate::wordtree::WF_ONECAP as c_int) != 0 {
        rs_onecap_copy(fword, cword, true);
    } else {
        // Copy as-is (STRCPY)
        let mut src = fword;
        let mut dst = cword;
        loop {
            let b = *(src as *const u8);
            #[allow(clippy::cast_possible_wrap)] // c_char is i8, bytes 0-127 are always positive
            {
                *dst = b as c_char;
            }
            if b == 0 {
                break;
            }
            src = src.add(1);
            dst = dst.add(1);
        }
    }
}

/// Concatenate spell check line into buffer.
///
/// Strips leading comment characters and whitespace, then copies into buf
/// at an offset so words from consecutive lines are separated.
///
/// # Safety
/// `buf`, `line` must be valid pointers. `buf` must have at least `maxlen` bytes.
#[export_name = "spell_cat_line"]
#[allow(clippy::cast_sign_loss)] // maxlen comes from C as positive value
#[allow(clippy::cast_possible_truncation)] // offset fits in usize for any reasonable string
pub unsafe extern "C" fn rs_spell_cat_line(buf: *mut c_char, line: *const c_char, maxlen: c_int) {
    let mut p = skipwhite(line);
    // Skip comment and list chars: *#/"\t
    let comment_chars: &[u8] = b"*#/\"\t";
    while !p.is_null() {
        let ch = *(p as *const u8);
        if ch == 0 || !comment_chars.contains(&ch) {
            break;
        }
        p = skipwhite(p.add(1));
    }
    if p.is_null() || *(p as *const u8) == 0 {
        return;
    }

    // SAFETY: p >= line since we only advanced forward from line
    let n = p.offset_from(line).max(0) as usize + 1;
    let maxlen = maxlen.max(0) as usize;
    if n < maxlen.saturating_sub(1) {
        // Fill leading bytes with spaces
        std::ptr::write_bytes(buf, b' ', n);
        xstrlcpy(buf.add(n), p, maxlen - n);
    }
}

// =============================================================================
// Word Tree Functions
// =============================================================================

/// Type alias for word tree index (matches C's idx_T = int)
pub type IdxT = c_int;

/// Mask for shared node index
const SHARED_MASK: i32 = 0x0800_0000;

/// Fill in the wordcount fields for a trie.
///
/// This function traverses the word tree and counts the number of words
/// at each node, storing the count in the `idxs` array at the node's position.
///
/// The tree uses two arrays:
/// - `byts`: stores the possible bytes at each node, preceded by count
/// - `idxs`: stores child indexes or word counts
///
/// # Arguments
///
/// * `byts` - Pointer to the bytes array
/// * `idxs` - Pointer to the indexes array (will be modified with word counts)
/// * `len` - Length of both arrays
///
/// # Safety
///
/// Both `byts` and `idxs` must point to valid memory of at least `len` elements.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_tree_count_words(byts: *const u8, idxs: *mut IdxT, len: c_int) {
    if byts.is_null() || idxs.is_null() || len <= 0 {
        return;
    }

    let len = len as usize;

    // Stack arrays for tree traversal (matching C's MAXWLEN)
    let mut arridx = [0i32; MAXWLEN];
    let mut curi = [0i32; MAXWLEN];
    let mut wordcount = [0i32; MAXWLEN];

    arridx[0] = 0;
    curi[0] = 1;
    wordcount[0] = 0;
    let mut depth: i32 = 0;

    while depth >= 0 {
        let d = depth as usize;
        let node_idx = arridx[d] as usize;

        // Safety: bounds check
        if node_idx >= len {
            break;
        }

        let node_len = i32::from(*byts.add(node_idx));

        if curi[d] > node_len {
            // Done all bytes at this node, go up one level.
            // Store the word count at this node's index position.
            *idxs.add(node_idx) = wordcount[d];
            if depth > 0 {
                wordcount[d - 1] += wordcount[d];
            }
            depth -= 1;
        } else {
            // Do one more byte at this node.
            let n = (arridx[d] + curi[d]) as usize;
            curi[d] += 1;

            // Safety: bounds check
            if n >= len {
                break;
            }

            let c = *byts.add(n);
            if c == 0 {
                // End of word, count it.
                wordcount[d] += 1;

                // Skip over any other NUL bytes (same word with different flags).
                let mut check_n = n + 1;
                while check_n < len && *byts.add(check_n) == 0 {
                    curi[d] += 1;
                    check_n += 1;
                }
            } else {
                // Normal char, go one level deeper to count the words.
                let child_idx = *idxs.add(n);
                if child_idx < 0 || child_idx as usize >= len {
                    // Invalid child index, skip
                    continue;
                }

                depth += 1;
                let new_d = depth as usize;
                if new_d >= MAXWLEN {
                    // Stack overflow protection
                    depth -= 1;
                    continue;
                }
                arridx[new_d] = child_idx;
                curi[new_d] = 1;
                wordcount[new_d] = 0;
            }
        }
    }
}

/// Read a word tree node from a buffer.
///
/// This function parses a tree node from a spell file buffer, filling in
/// the `byts` and `idxs` arrays. It handles:
/// - Sibling counts
/// - End-of-word flags
/// - Shared node indexes
/// - Prefix tree entries
///
/// # Arguments
///
/// * `buf` - Pointer to the spell file buffer
/// * `buf_len` - Total length of the buffer
/// * `buf_offset` - Current offset in the buffer (will be updated)
/// * `byts` - Pointer to the bytes array to fill
/// * `idxs` - Pointer to the indexes array to fill
/// * `max_idx` - Maximum valid index in arrays
/// * `start_idx` - Starting index for this node
/// * `prefixtree` - True if reading prefix tree
/// * `max_prefcondnr` - Maximum prefix condition number
///
/// # Returns
///
/// The next index after this node's siblings, or a negative SP_* error code.
///
/// # Safety
///
/// All pointers must be valid and point to sufficient memory.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_lossless)]
pub unsafe extern "C" fn rs_read_tree_node(
    buf: *const u8,
    buf_len: usize,
    buf_offset: *mut usize,
    byts: *mut u8,
    idxs: *mut IdxT,
    max_idx: c_int,
    start_idx: IdxT,
    prefixtree: bool,
    max_prefcondnr: c_int,
) -> IdxT {
    if buf.is_null() || buf_offset.is_null() || byts.is_null() || idxs.is_null() {
        return SP_TRUNCERROR;
    }

    let mut offset = *buf_offset;
    let max_idx_usize = max_idx as usize;

    // Read sibling count
    if offset >= buf_len {
        return SP_TRUNCERROR;
    }
    let sibling_count = i32::from(*buf.add(offset));
    offset += 1;

    if sibling_count <= 0 {
        return SP_TRUNCERROR;
    }

    let start = start_idx as usize;
    if start + (sibling_count as usize) >= max_idx_usize {
        return SP_FORMERROR;
    }

    let mut idx = start_idx;
    *byts.add(idx as usize) = sibling_count as u8;
    idx += 1;

    // Read the byte values, flag/region bytes and shared indexes.
    for _i in 1..=sibling_count {
        if offset >= buf_len {
            *buf_offset = offset;
            return SP_TRUNCERROR;
        }

        let mut c = i32::from(*buf.add(offset));
        offset += 1;

        if c <= i32::from(BY_INDEX) {
            if c == i32::from(BY_NOFLAGS) && !prefixtree {
                // No flags, all regions.
                *idxs.add(idx as usize) = 0;
            } else if c != i32::from(BY_INDEX) {
                if prefixtree {
                    // Read the optional pflags byte, the prefix ID and the condition nr.
                    if c == i32::from(BY_FLAGS) {
                        if offset >= buf_len {
                            *buf_offset = offset;
                            return SP_TRUNCERROR;
                        }
                        c = i32::from(*buf.add(offset)) << 24;
                        offset += 1;
                    } else {
                        c = 0;
                    }

                    // Read affixID
                    if offset >= buf_len {
                        *buf_offset = offset;
                        return SP_TRUNCERROR;
                    }
                    c |= i32::from(*buf.add(offset));
                    offset += 1;

                    // Read prefcondnr (2 bytes, big-endian)
                    if offset + 1 >= buf_len {
                        *buf_offset = offset;
                        return SP_TRUNCERROR;
                    }
                    let n = (i32::from(*buf.add(offset)) << 8) | i32::from(*buf.add(offset + 1));
                    offset += 2;

                    if n >= max_prefcondnr {
                        *buf_offset = offset;
                        return SP_FORMERROR;
                    }
                    c |= n << 8;
                } else {
                    // c is BY_FLAGS or BY_FLAGS2
                    let c2 = c;
                    if offset >= buf_len {
                        *buf_offset = offset;
                        return SP_TRUNCERROR;
                    }
                    c = i32::from(*buf.add(offset));
                    offset += 1;

                    if c2 == i32::from(BY_FLAGS2) {
                        if offset >= buf_len {
                            *buf_offset = offset;
                            return SP_TRUNCERROR;
                        }
                        c += i32::from(*buf.add(offset)) << 8;
                        offset += 1;
                    }

                    if (c & WF_REGION) != 0 {
                        if offset >= buf_len {
                            *buf_offset = offset;
                            return SP_TRUNCERROR;
                        }
                        c += i32::from(*buf.add(offset)) << 16;
                        offset += 1;
                    }

                    if (c & WF_AFX) != 0 {
                        if offset >= buf_len {
                            *buf_offset = offset;
                            return SP_TRUNCERROR;
                        }
                        c += i32::from(*buf.add(offset)) << 24;
                        offset += 1;
                    }
                }

                *idxs.add(idx as usize) = c;
                c = 0;
            } else {
                // c == BY_INDEX: read nodeidx (3 bytes) and xbyte
                if offset + 3 >= buf_len {
                    *buf_offset = offset;
                    return SP_TRUNCERROR;
                }
                let n = (i32::from(*buf.add(offset)) << 16)
                    | (i32::from(*buf.add(offset + 1)) << 8)
                    | i32::from(*buf.add(offset + 2));
                offset += 3;

                if n < 0 || n >= max_idx {
                    *buf_offset = offset;
                    return SP_FORMERROR;
                }
                *idxs.add(idx as usize) = n + SHARED_MASK;

                c = i32::from(*buf.add(offset));
                offset += 1;
            }
        }
        *byts.add(idx as usize) = c as u8;
        idx += 1;
    }

    *buf_offset = offset;

    // Recursively read the children for non-shared siblings.
    for i in 1..=sibling_count {
        let sib_idx = start + i as usize;
        if *byts.add(sib_idx) != 0 {
            let cur_idxs = *idxs.add(sib_idx);
            if (cur_idxs & SHARED_MASK) != 0 {
                // Remove shared mask
                *idxs.add(sib_idx) = cur_idxs & !SHARED_MASK;
            } else {
                // Set the child index and recursively read
                *idxs.add(sib_idx) = idx;
                idx = rs_read_tree_node(
                    buf,
                    buf_len,
                    buf_offset,
                    byts,
                    idxs,
                    max_idx,
                    idx,
                    prefixtree,
                    max_prefcondnr,
                );
                if idx < 0 {
                    return idx;
                }
            }
        }
    }

    idx
}

// =============================================================================
// Suggestion Scoring Utility Functions
// =============================================================================

/// The RESCORE macro: adjust word score based on soundfold score.
///
/// Used after finding suggestions to adjust scores based on how similar
/// the sound-folded versions are. Formula: (3 * word_score + sound_score) / 4
#[inline]
#[must_use]
pub const fn rescore(word_score: c_int, sound_score: c_int) -> c_int {
    (3 * word_score + sound_score) / 4
}

/// FFI wrapper for rescore.
#[no_mangle]
pub extern "C" fn rs_rescore(word_score: c_int, sound_score: c_int) -> c_int {
    rescore(word_score, sound_score)
}

/// The MAXSCORE macro: compute maximum word score from end score and sound score.
///
/// Inverse of RESCORE. Given a maximum end score and known sound score,
/// compute the maximum word score that can be used.
/// Formula: (4 * end_score - sound_score) / 3
#[inline]
#[must_use]
pub const fn maxscore(end_score: c_int, sound_score: c_int) -> c_int {
    (4 * end_score - sound_score) / 3
}

/// FFI wrapper for maxscore.
#[no_mangle]
pub extern "C" fn rs_maxscore(end_score: c_int, sound_score: c_int) -> c_int {
    maxscore(end_score, sound_score)
}

/// Calculate score for a swap operation.
///
/// Returns `SCORE_SWAP` (75) for a simple character swap.
#[no_mangle]
pub extern "C" fn rs_score_swap() -> c_int {
    SCORE_SWAP
}

/// Calculate score for a swap in three characters.
///
/// Returns `SCORE_SWAP3` (110) for swapping two characters over three.
#[no_mangle]
pub extern "C" fn rs_score_swap3() -> c_int {
    SCORE_SWAP3
}

/// Calculate score for a substitution.
///
/// Returns `SCORE_SUBST` (93) for substituting a character.
#[no_mangle]
pub extern "C" fn rs_score_subst() -> c_int {
    SCORE_SUBST
}

/// Calculate score for substituting a similar character.
///
/// Returns `SCORE_SIMILAR` (33) for substituting a similar character.
#[no_mangle]
pub extern "C" fn rs_score_similar() -> c_int {
    SCORE_SIMILAR
}

/// Calculate score for deleting a character.
///
/// Returns `SCORE_DEL` (94) for deleting a character.
#[no_mangle]
pub extern "C" fn rs_score_del() -> c_int {
    SCORE_DEL
}

/// Calculate score for deleting a duplicated character.
///
/// Returns `SCORE_DELDUP` (66) for deleting a duplicate.
#[no_mangle]
pub extern "C" fn rs_score_deldup() -> c_int {
    SCORE_DELDUP
}

/// Calculate score for inserting a character.
///
/// Returns `SCORE_INS` (96) for inserting a character.
#[no_mangle]
pub extern "C" fn rs_score_ins() -> c_int {
    SCORE_INS
}

/// Calculate score for inserting a duplicate character.
///
/// Returns `SCORE_INSDUP` (67) for inserting a duplicate.
#[no_mangle]
pub extern "C" fn rs_score_insdup() -> c_int {
    SCORE_INSDUP
}

/// Calculate score for changing non-word to word character.
///
/// Returns `SCORE_NONWORD` (103).
#[no_mangle]
pub extern "C" fn rs_score_nonword() -> c_int {
    SCORE_NONWORD
}

/// Calculate score for a word split.
///
/// Returns `SCORE_SPLIT` (149).
#[no_mangle]
pub extern "C" fn rs_score_split() -> c_int {
    SCORE_SPLIT
}

/// Calculate score for a word split with NOSPLITSUGS.
///
/// Returns `SCORE_SPLIT_NO` (249).
#[no_mangle]
pub extern "C" fn rs_score_split_no() -> c_int {
    SCORE_SPLIT_NO
}

/// Calculate score for slightly different case.
///
/// Returns `SCORE_ICASE` (52).
#[no_mangle]
pub extern "C" fn rs_score_icase() -> c_int {
    SCORE_ICASE
}

/// Calculate score for word from different region.
///
/// Returns `SCORE_REGION` (200).
#[no_mangle]
pub extern "C" fn rs_score_region() -> c_int {
    SCORE_REGION
}

/// Calculate score for rare word.
///
/// Returns `SCORE_RARE` (180).
#[no_mangle]
pub extern "C" fn rs_score_rare() -> c_int {
    SCORE_RARE
}

/// Calculate score for REP replacement.
///
/// Returns `SCORE_REP` (65).
#[no_mangle]
pub extern "C" fn rs_score_rep() -> c_int {
    SCORE_REP
}

/// Calculate score for suggestion from a file.
///
/// Returns `SCORE_FILE` (30).
#[no_mangle]
pub extern "C" fn rs_score_file() -> c_int {
    SCORE_FILE
}

/// Calculate initial maximum score.
///
/// Returns `SCORE_MAXINIT` (350). Higher values make suggestion search slower.
#[no_mangle]
pub extern "C" fn rs_score_maxinit() -> c_int {
    SCORE_MAXINIT
}

/// Check if a score is acceptable (below maximum).
///
/// A score is acceptable if it's less than `SCORE_MAXMAX` (999999).
#[no_mangle]
pub extern "C" fn rs_score_is_acceptable(score: c_int) -> bool {
    score < SCORE_MAXMAX
}

/// Check if a score represents a complete failure (no match possible).
///
/// Returns true if score equals `SCORE_MAXMAX` (999999).
#[no_mangle]
pub extern "C" fn rs_score_is_failed(score: c_int) -> bool {
    score >= SCORE_MAXMAX
}

/// Calculate combined score for two operations.
///
/// Simple addition of two scores.
#[no_mangle]
pub extern "C" fn rs_score_combine(score1: c_int, score2: c_int) -> c_int {
    score1 + score2
}

/// Calculate score for common word bonus.
///
/// Returns the appropriate bonus based on word count threshold.
/// - count >= SCORE_THRES3 (100): returns SCORE_COMMON3 (50)
/// - count >= SCORE_THRES2 (10): returns SCORE_COMMON2 (40)
/// - count >= 1: returns SCORE_COMMON1 (30)
/// - count == 0: returns 0 (no bonus)
#[no_mangle]
pub extern "C" fn rs_score_common_bonus(word_count: c_int) -> c_int {
    if word_count >= SCORE_THRES3 {
        SCORE_COMMON3
    } else if word_count >= SCORE_THRES2 {
        SCORE_COMMON2
    } else if word_count >= 1 {
        SCORE_COMMON1
    } else {
        0
    }
}

/// Apply common word bonus to a score (subtract bonus).
///
/// The bonus is subtracted from the score to make common words rank higher.
#[no_mangle]
pub extern "C" fn rs_score_apply_common_bonus(score: c_int, word_count: c_int) -> c_int {
    let bonus = rs_score_common_bonus(word_count);
    // Don't go negative
    if score > bonus {
        score - bonus
    } else {
        0
    }
}

/// Get the appropriate soundfold maximum score for a try number.
///
/// - try_nr == 1: returns SCORE_SFMAX1 (200)
/// - try_nr == 2: returns SCORE_SFMAX2 (300)
/// - try_nr >= 3: returns SCORE_SFMAX3 (400)
#[no_mangle]
pub extern "C" fn rs_score_sfmax(try_nr: c_int) -> c_int {
    match try_nr {
        1 => SCORE_SFMAX1,
        2 => SCORE_SFMAX2,
        _ => SCORE_SFMAX3,
    }
}

/// Check if a score is within the soundfold limit for a given try.
#[no_mangle]
pub extern "C" fn rs_score_within_sfmax(score: c_int, try_nr: c_int) -> bool {
    score <= rs_score_sfmax(try_nr)
}

/// Calculate the minimum edit score (for optimization).
///
/// Returns `SCORE_EDIT_MIN` which equals `SCORE_SIMILAR` (33).
#[no_mangle]
pub extern "C" fn rs_score_edit_min() -> c_int {
    SCORE_EDIT_MIN
}

/// Calculate a "big difference" score.
///
/// Returns `SCORE_BIG` which is 3 * SCORE_INS (288).
#[no_mangle]
pub extern "C" fn rs_score_big() -> c_int {
    SCORE_BIG
}

/// Get the maximum score for `spell_edit_score_limit`.
///
/// Returns `SCORE_LIMITMAX` (350).
#[no_mangle]
pub extern "C" fn rs_score_limitmax() -> c_int {
    SCORE_LIMITMAX
}

// =============================================================================
// Spell Word Flag Utilities
// =============================================================================

/// Check if word flags indicate a rare word.
#[no_mangle]
pub extern "C" fn rs_wf_is_rare(flags: c_int) -> bool {
    (flags & WF_RARE) != 0
}

/// Check if word flags indicate a banned word.
#[no_mangle]
pub extern "C" fn rs_wf_is_banned(flags: c_int) -> bool {
    (flags & WF_BANNED) != 0
}

/// Check if word flags indicate an all-caps word.
#[no_mangle]
pub extern "C" fn rs_wf_is_allcap(flags: c_int) -> bool {
    (flags & WF_ALLCAP_FLAG) != 0
}

/// Check if word flags indicate a one-cap word.
#[no_mangle]
pub extern "C" fn rs_wf_is_onecap(flags: c_int) -> bool {
    (flags & WF_ONECAP_FLAG) != 0
}

/// Check if word flags indicate keep-case.
#[no_mangle]
pub extern "C" fn rs_wf_is_keepcap(flags: c_int) -> bool {
    (flags & WF_KEEPCAP_FLAG) != 0
}

/// Check if word flags indicate fix-case.
#[no_mangle]
pub extern "C" fn rs_wf_is_fixcap(flags: c_int) -> bool {
    (flags & WF_FIXCAP_FLAG) != 0
}

/// Check if word flags have region byte.
#[no_mangle]
pub extern "C" fn rs_wf_has_region(flags: c_int) -> bool {
    (flags & WF_REGION) != 0
}

/// Check if word flags have affix ID.
#[no_mangle]
pub extern "C" fn rs_wf_has_afx(flags: c_int) -> bool {
    (flags & WF_AFX) != 0
}

// =============================================================================
// Spell Language List FFI Functions
// =============================================================================

/// Get the first loaded spell language.
///
/// Returns null handle if no languages are loaded.
#[no_mangle]
pub extern "C" fn rs_get_first_lang() -> SlangHandle {
    SlangHandle::first()
}

/// Get the next language in the linked list.
///
/// Returns null handle if this is the last language.
#[no_mangle]
pub extern "C" fn rs_slang_next(slang: SlangHandle) -> SlangHandle {
    if slang.is_null() {
        SlangHandle::null()
    } else {
        slang.next()
    }
}

/// Get the language name from a slang handle.
///
/// Returns null if handle is null.
#[no_mangle]
pub extern "C" fn rs_slang_name(slang: SlangHandle) -> *const c_char {
    if slang.is_null() {
        std::ptr::null()
    } else {
        slang.name()
    }
}

/// Get the file name from a slang handle.
///
/// Returns null if handle is null.
#[no_mangle]
pub extern "C" fn rs_slang_fname(slang: SlangHandle) -> *const c_char {
    if slang.is_null() {
        std::ptr::null()
    } else {
        slang.fname()
    }
}

/// Check if a slang is an .add file.
///
/// Returns false if handle is null.
#[no_mangle]
pub extern "C" fn rs_slang_is_add(slang: SlangHandle) -> bool {
    if slang.is_null() {
        false
    } else {
        slang.is_add()
    }
}

/// Check if a slang handle is null.
#[no_mangle]
pub extern "C" fn rs_slang_is_null(slang: SlangHandle) -> bool {
    slang.is_null()
}

// =============================================================================
// Language Pointer (langp_T) FFI Functions
// =============================================================================

/// Get the slang handle from a langp entry.
#[no_mangle]
pub extern "C" fn rs_langp_slang(langp: LangpHandle) -> SlangHandle {
    if langp.is_null() {
        SlangHandle::null()
    } else {
        langp.slang()
    }
}

/// Get the sound-alike language from a langp entry.
#[no_mangle]
pub extern "C" fn rs_langp_sallang(langp: LangpHandle) -> SlangHandle {
    if langp.is_null() {
        SlangHandle::null()
    } else {
        langp.sallang()
    }
}

/// Get the REP items language from a langp entry.
#[no_mangle]
pub extern "C" fn rs_langp_replang(langp: LangpHandle) -> SlangHandle {
    if langp.is_null() {
        SlangHandle::null()
    } else {
        langp.replang()
    }
}

/// Get the region bitmask from a langp entry.
///
/// Returns REGION_ALL (0xff) if handle is null.
#[no_mangle]
pub extern "C" fn rs_langp_region(langp: LangpHandle) -> c_int {
    if langp.is_null() {
        REGION_ALL
    } else {
        langp.region()
    }
}

/// Check if a langp handle is null.
#[no_mangle]
pub extern "C" fn rs_langp_is_null(langp: LangpHandle) -> bool {
    langp.is_null()
}

// =============================================================================
// Spell UI Integration Types (Phase 5)
// =============================================================================

/// Type of word to add to spell file.
///
/// Used by `spell_add_word()` and `zg`, `zw`, `zG`, `zW` commands.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellAddType {
    /// Add as good word
    Good = 0,
    /// Add as bad/wrong word
    Bad = 1,
    /// Add as rare word
    Rare = 2,
}

impl SpellAddType {
    /// Convert from C integer to SpellAddType.
    #[must_use]
    pub const fn from_c_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Good),
            1 => Some(Self::Bad),
            2 => Some(Self::Rare),
            _ => None,
        }
    }
}

/// FFI wrapper to convert integer to SpellAddType.
///
/// Returns 0 (Good) if invalid.
#[no_mangle]
pub extern "C" fn rs_spell_add_type_from_int(value: c_int) -> c_int {
    SpellAddType::from_c_int(value).map_or(SpellAddType::Good as c_int, |t| t as c_int)
}

/// Type of spell word movement.
///
/// Used by `]s`, `[s`, `]S`, `[S` navigation commands.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellMoveType {
    /// Move to any misspelled word (all types)
    All = 0,
    /// Move to bad (wrong) words only
    Bad = 1,
    /// Move to rare words only
    Rare = 2,
}

impl SpellMoveType {
    /// Convert from C integer to SpellMoveType.
    #[must_use]
    pub const fn from_c_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::All),
            1 => Some(Self::Bad),
            2 => Some(Self::Rare),
            _ => None,
        }
    }
}

/// FFI wrapper to convert integer to SpellMoveType.
///
/// Returns 0 (All) if invalid.
#[no_mangle]
pub extern "C" fn rs_spell_move_type_from_int(value: c_int) -> c_int {
    SpellMoveType::from_c_int(value).map_or(SpellMoveType::All as c_int, |t| t as c_int)
}

// =============================================================================
// Spell Suggest Timeout Constants (Phase 5)
// =============================================================================

/// Default timeout for spell suggestions in milliseconds.
///
/// This is the default value for `spell_suggest_timeout` in spellsuggest.c.
/// The timeout can be changed via the 'timeout:' option in 'spellsuggest'.
pub const SPELL_SUGGEST_TIMEOUT_DEFAULT: c_int = 5000;

/// FFI wrapper to get the default spell suggest timeout.
#[no_mangle]
pub extern "C" fn rs_spell_suggest_timeout_default() -> c_int {
    SPELL_SUGGEST_TIMEOUT_DEFAULT
}

/// Minimum reasonable spell suggest timeout (100ms).
pub const SPELL_SUGGEST_TIMEOUT_MIN: c_int = 100;

/// Maximum reasonable spell suggest timeout (60 seconds).
pub const SPELL_SUGGEST_TIMEOUT_MAX: c_int = 60000;

/// Validate a spell suggest timeout value.
///
/// Returns true if the value is within reasonable bounds.
#[must_use]
pub const fn validate_spell_suggest_timeout(timeout: c_int) -> bool {
    timeout >= SPELL_SUGGEST_TIMEOUT_MIN && timeout <= SPELL_SUGGEST_TIMEOUT_MAX
}

/// FFI wrapper to validate spell suggest timeout.
#[no_mangle]
pub extern "C" fn rs_validate_spell_suggest_timeout(timeout: c_int) -> bool {
    validate_spell_suggest_timeout(timeout)
}

/// Clamp a spell suggest timeout to valid bounds.
///
/// Returns the timeout clamped to [SPELL_SUGGEST_TIMEOUT_MIN, SPELL_SUGGEST_TIMEOUT_MAX].
#[must_use]
pub const fn clamp_spell_suggest_timeout(timeout: c_int) -> c_int {
    if timeout < SPELL_SUGGEST_TIMEOUT_MIN {
        SPELL_SUGGEST_TIMEOUT_MIN
    } else if timeout > SPELL_SUGGEST_TIMEOUT_MAX {
        SPELL_SUGGEST_TIMEOUT_MAX
    } else {
        timeout
    }
}

/// FFI wrapper to clamp spell suggest timeout.
#[no_mangle]
pub extern "C" fn rs_clamp_spell_suggest_timeout(timeout: c_int) -> c_int {
    clamp_spell_suggest_timeout(timeout)
}

// =============================================================================
// Spell Word Validation (Phase 5)
// =============================================================================

/// Check if a word contains only valid spell word characters.
///
/// A valid spell word contains only word characters (letters, digits for some
/// languages) without any control characters or other problematic bytes.
///
/// # Arguments
///
/// * `word` - Pointer to the start of the word
/// * `end` - Pointer to end of the word (exclusive)
///
/// # Safety
///
/// Both pointers must be valid and `word <= end`.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_valid_spell_word(word: *const u8, end: *const u8) -> bool {
    if word.is_null() || end.is_null() {
        return false;
    }

    if word > end {
        return false;
    }

    // Empty words are not valid
    if word == end {
        return false;
    }

    let mut p = word;
    while p < end {
        let c = *p;
        // Control characters (0x00-0x1F, 0x7F) are not allowed
        if c < 0x20 || c == 0x7F {
            return false;
        }
        p = p.add(1);
    }

    true
}

/// Check if a character is valid at the start of a spell word.
///
/// The first character must be a letter (not a digit or punctuation).
#[must_use]
pub const fn is_valid_spell_word_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c >= 0x80 // ASCII letter or high byte (UTF-8)
}

/// FFI wrapper for is_valid_spell_word_start.
#[no_mangle]
pub extern "C" fn rs_is_valid_spell_word_start(c: u8) -> bool {
    is_valid_spell_word_start(c)
}

// =============================================================================
// Undo Support for Spell Commands (Phase 5)
// =============================================================================

/// Flags for spell add operations with undo support.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellAddFlags {
    /// The type of word to add
    pub add_type: c_int,
    /// Index into 'spellfile' option (1-based, 0 for internal list)
    pub spellfile_idx: c_int,
    /// Whether this operation is undoable
    pub undoable: bool,
}

impl SpellAddFlags {
    /// Create new spell add flags.
    #[must_use]
    pub const fn new(add_type: SpellAddType, spellfile_idx: c_int, undoable: bool) -> Self {
        Self {
            add_type: add_type as c_int,
            spellfile_idx,
            undoable,
        }
    }

    /// Create flags for adding a good word (zg command).
    #[must_use]
    pub const fn good(spellfile_idx: c_int) -> Self {
        Self::new(SpellAddType::Good, spellfile_idx, true)
    }

    /// Create flags for adding a bad word (zw command).
    #[must_use]
    pub const fn bad(spellfile_idx: c_int) -> Self {
        Self::new(SpellAddType::Bad, spellfile_idx, true)
    }

    /// Create flags for adding a rare word.
    #[must_use]
    pub const fn rare(spellfile_idx: c_int) -> Self {
        Self::new(SpellAddType::Rare, spellfile_idx, true)
    }

    /// Check if this is an undo operation (removing a word).
    #[must_use]
    pub const fn is_undo(&self) -> bool {
        self.undoable && self.add_type == SpellAddType::Bad as c_int
    }
}

/// FFI wrapper to create SpellAddFlags for a good word.
#[no_mangle]
pub extern "C" fn rs_spell_add_flags_good(spellfile_idx: c_int) -> SpellAddFlags {
    SpellAddFlags::good(spellfile_idx)
}

/// FFI wrapper to create SpellAddFlags for a bad word.
#[no_mangle]
pub extern "C" fn rs_spell_add_flags_bad(spellfile_idx: c_int) -> SpellAddFlags {
    SpellAddFlags::bad(spellfile_idx)
}

/// FFI wrapper to create SpellAddFlags for a rare word.
#[no_mangle]
pub extern "C" fn rs_spell_add_flags_rare(spellfile_idx: c_int) -> SpellAddFlags {
    SpellAddFlags::rare(spellfile_idx)
}

// =============================================================================
// Spell Suggestion Display Constants (Phase 5)
// =============================================================================

/// Maximum number of suggestions to display in z= menu.
pub const SPELL_SUGGEST_DISPLAY_MAX: c_int = 25;

/// FFI wrapper to get max suggestions to display.
#[no_mangle]
pub extern "C" fn rs_spell_suggest_display_max() -> c_int {
    SPELL_SUGGEST_DISPLAY_MAX
}

/// Default number of suggestions to show initially.
pub const SPELL_SUGGEST_DISPLAY_DEFAULT: c_int = 5;

/// FFI wrapper to get default number of suggestions to display.
#[no_mangle]
pub extern "C" fn rs_spell_suggest_display_default() -> c_int {
    SPELL_SUGGEST_DISPLAY_DEFAULT
}

/// Minimum score difference to show another suggestion.
///
/// If the next suggestion's score is more than this much worse
/// than the previous, it may be omitted.
pub const SPELL_SUGGEST_SCORE_THRESHOLD: c_int = 200;

/// FFI wrapper to get suggestion score threshold.
#[no_mangle]
pub extern "C" fn rs_spell_suggest_score_threshold() -> c_int {
    SPELL_SUGGEST_SCORE_THRESHOLD
}

// =============================================================================
// Phase 147: Spell File I/O Deep Migration
// =============================================================================

// Note: slang_T setter functions would require C accessor additions.
// For now, Phase 147 focuses on standalone helpers that don't need C accessors.

/// Spell file section IDs.
pub mod section_ids {
    use std::ffi::c_int;

    /// End of sections marker.
    pub const SN_END: c_int = 0;
    /// Region section.
    pub const SN_REGION: c_int = 1;
    /// Charflags section.
    pub const SN_CHARFLAGS: c_int = 2;
    /// Midword section.
    pub const SN_MIDWORD: c_int = 3;
    /// Prefcond section.
    pub const SN_PREFCOND: c_int = 4;
    /// REP section.
    pub const SN_REP: c_int = 5;
    /// REPSAL section.
    pub const SN_REPSAL: c_int = 6;
    /// SAL section.
    pub const SN_SAL: c_int = 7;
    /// SOFO section.
    pub const SN_SOFO: c_int = 8;
    /// MAP section.
    pub const SN_MAP: c_int = 9;
    /// Compound section.
    pub const SN_COMPOUND: c_int = 10;
    /// Syllable section.
    pub const SN_SYLLABLE: c_int = 11;
    /// NoBreak section.
    pub const SN_NOBREAK: c_int = 12;
    /// Sugfile section.
    pub const SN_SUGFILE: c_int = 13;
    /// NoSplitSugs section.
    pub const SN_NOSPLITSUGS: c_int = 14;
    /// NoCompSugs section.
    pub const SN_NOCOMPOUNDSUGS: c_int = 15;
    /// Words section.
    pub const SN_WORDS: c_int = 16;
    /// Prefix tree section.
    pub const SN_PREFIXTREE: c_int = 17;
    /// Info section.
    pub const SN_INFO: c_int = 18;
}

/// Get section ID for SN_END.
#[no_mangle]
pub extern "C" fn rs_spell_section_end() -> c_int {
    section_ids::SN_END
}

/// Get section ID for SN_REGION.
#[no_mangle]
pub extern "C" fn rs_spell_section_region() -> c_int {
    section_ids::SN_REGION
}

/// Get section ID for SN_CHARFLAGS.
#[no_mangle]
pub extern "C" fn rs_spell_section_charflags() -> c_int {
    section_ids::SN_CHARFLAGS
}

/// Get section ID for SN_MIDWORD.
#[no_mangle]
pub extern "C" fn rs_spell_section_midword() -> c_int {
    section_ids::SN_MIDWORD
}

/// Get section ID for SN_PREFCOND.
#[no_mangle]
pub extern "C" fn rs_spell_section_prefcond() -> c_int {
    section_ids::SN_PREFCOND
}

/// Get section ID for SN_REP.
#[no_mangle]
pub extern "C" fn rs_spell_section_rep() -> c_int {
    section_ids::SN_REP
}

/// Get section ID for SN_SAL.
#[no_mangle]
pub extern "C" fn rs_spell_section_sal() -> c_int {
    section_ids::SN_SAL
}

/// Get section ID for SN_SOFO.
#[no_mangle]
pub extern "C" fn rs_spell_section_sofo() -> c_int {
    section_ids::SN_SOFO
}

/// Get section ID for SN_COMPOUND.
#[no_mangle]
pub extern "C" fn rs_spell_section_compound() -> c_int {
    section_ids::SN_COMPOUND
}

/// Get section ID for SN_PREFIXTREE.
#[no_mangle]
pub extern "C" fn rs_spell_section_prefixtree() -> c_int {
    section_ids::SN_PREFIXTREE
}

/// Get section ID for SN_WORDS.
#[no_mangle]
pub extern "C" fn rs_spell_section_words() -> c_int {
    section_ids::SN_WORDS
}

/// Get section ID for SN_INFO.
#[no_mangle]
pub extern "C" fn rs_spell_section_info() -> c_int {
    section_ids::SN_INFO
}

/// Check if a section ID is valid.
#[no_mangle]
pub extern "C" fn rs_spell_section_is_valid(id: c_int) -> bool {
    (section_ids::SN_END..=section_ids::SN_INFO).contains(&id)
}

/// Check if a section is required (cannot be skipped).
#[no_mangle]
pub extern "C" fn rs_spell_section_is_required(id: c_int) -> bool {
    // Most sections can be skipped if unknown
    // These are typically required for basic spell checking
    matches!(
        id,
        section_ids::SN_REGION | section_ids::SN_CHARFLAGS | section_ids::SN_PREFCOND
    )
}

/// Spell file tree types.
pub mod tree_types {
    use std::ffi::c_int;

    /// Case-folded word tree.
    pub const TREE_FWORD: c_int = 0;
    /// Keep-case word tree.
    pub const TREE_KWORD: c_int = 1;
    /// Prefix tree.
    pub const TREE_PREFIX: c_int = 2;
}

/// Get tree type for case-folded words.
#[no_mangle]
pub extern "C" fn rs_spell_tree_fword() -> c_int {
    tree_types::TREE_FWORD
}

/// Get tree type for keep-case words.
#[no_mangle]
pub extern "C" fn rs_spell_tree_kword() -> c_int {
    tree_types::TREE_KWORD
}

/// Get tree type for prefixes.
#[no_mangle]
pub extern "C" fn rs_spell_tree_prefix() -> c_int {
    tree_types::TREE_PREFIX
}

/// Spell file loading result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SpellFileResult {
    /// Error code (0 for success).
    pub error: c_int,
    /// Number of bytes consumed.
    pub bytes_consumed: c_int,
    /// Number of items parsed.
    pub items_parsed: c_int,
}

impl SpellFileResult {
    /// Create a success result.
    #[must_use]
    pub const fn success(bytes_consumed: c_int, items_parsed: c_int) -> Self {
        Self {
            error: 0,
            bytes_consumed,
            items_parsed,
        }
    }

    /// Create an error result.
    #[must_use]
    pub const fn error(error: c_int) -> Self {
        Self {
            error,
            bytes_consumed: 0,
            items_parsed: 0,
        }
    }

    /// Check if the result is successful.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.error == 0
    }
}

/// Create a success spell file result.
#[no_mangle]
pub extern "C" fn rs_spell_file_result_success(
    bytes_consumed: c_int,
    items_parsed: c_int,
) -> SpellFileResult {
    SpellFileResult::success(bytes_consumed, items_parsed)
}

/// Create an error spell file result.
#[no_mangle]
pub extern "C" fn rs_spell_file_result_error(error: c_int) -> SpellFileResult {
    SpellFileResult::error(error)
}

/// Check if a spell file result is OK.
///
/// # Safety
/// `result` must be a valid pointer to a `SpellFileResult`.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_file_result_is_ok(result: *const SpellFileResult) -> bool {
    if result.is_null() {
        return false;
    }
    (*result).is_ok()
}

/// Get error code from spell file result.
///
/// # Safety
/// `result` must be a valid pointer to a `SpellFileResult`.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_file_result_get_error(result: *const SpellFileResult) -> c_int {
    if result.is_null() {
        return SP_OTHERERROR;
    }
    (*result).error
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Opaque handle tests
    // =========================================================================

    #[test]
    fn test_slang_handle_null() {
        let handle = SlangHandle::null();
        assert!(handle.is_null());
        // rs_slang_is_null just checks the handle directly, no FFI call
        assert!(rs_slang_is_null(handle));
    }

    #[test]
    fn test_langp_handle_null() {
        let handle = LangpHandle::null();
        assert!(handle.is_null());
        // rs_langp_is_null just checks the handle directly, no FFI call
        assert!(rs_langp_is_null(handle));
    }

    #[test]
    fn test_spelltab_handle_null() {
        let handle = SpelltabHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_handle_sizes() {
        // Ensure handles are pointer-sized for FFI compatibility
        use std::mem::size_of;
        assert_eq!(size_of::<SlangHandle>(), size_of::<*mut c_void>());
        assert_eq!(size_of::<LangpHandle>(), size_of::<*mut c_void>());
        assert_eq!(size_of::<SpelltabHandle>(), size_of::<*mut c_void>());
    }

    #[test]
    fn test_valid_spelllang() {
        use std::ffi::CString;

        unsafe {
            // Valid spelllang values
            let en = CString::new("en").unwrap();
            assert!(rs_valid_spelllang(en.as_ptr()));

            let en_us = CString::new("en_US").unwrap();
            assert!(rs_valid_spelllang(en_us.as_ptr()));

            let complex = CString::new("en,de,fr").unwrap();
            assert!(rs_valid_spelllang(complex.as_ptr()));

            let with_at = CString::new("en@spell").unwrap();
            assert!(rs_valid_spelllang(with_at.as_ptr()));

            let with_dot = CString::new("en.utf-8").unwrap();
            assert!(rs_valid_spelllang(with_dot.as_ptr()));

            // Invalid spelllang values
            let with_space = CString::new("en us").unwrap();
            assert!(!rs_valid_spelllang(with_space.as_ptr()));

            let with_special = CString::new("en!us").unwrap();
            assert!(!rs_valid_spelllang(with_special.as_ptr()));

            // Empty is valid
            let empty = CString::new("").unwrap();
            assert!(rs_valid_spelllang(empty.as_ptr()));

            // Null is valid
            assert!(rs_valid_spelllang(std::ptr::null()));
        }
    }

    #[test]
    fn test_spell_valid_case_allcap_word() {
        // ALLCAP word, tree doesn't require FIXCAP -> valid (first branch)
        assert!(spell_valid_case_impl(WF_ALLCAP, 0));
        assert!(spell_valid_case_impl(WF_ALLCAP, WF_ONECAP));
        // ALLCAP word, tree has ALLCAP but no FIXCAP -> valid via first branch
        // (treeflags & WF_FIXCAP) == (0x04 & 0x40) == 0 -> TRUE
        assert!(spell_valid_case_impl(WF_ALLCAP, WF_ALLCAP));

        // ALLCAP word, tree requires FIXCAP only -> valid via second branch
        // First branch: (0x40 & 0x40) != 0 -> FALSE
        // Second branch: (0x40 & (0x04|0x80)) == 0 -> TRUE
        assert!(spell_valid_case_impl(WF_ALLCAP, WF_FIXCAP));

        // ALLCAP word, tree has FIXCAP|ALLCAP -> first branch fails, second branch
        // (treeflags & (ALLCAP|KEEPCAP)) = (0x44 & 0x84) = 0x04 != 0 -> FALSE
        assert!(!spell_valid_case_impl(WF_ALLCAP, WF_FIXCAP | WF_ALLCAP));
    }

    #[test]
    fn test_spell_valid_case_normal_word() {
        // Normal word (no flags), tree has no ALLCAP/KEEPCAP/ONECAP -> valid
        assert!(spell_valid_case_impl(0, 0));

        // Normal word, tree has ONECAP -> invalid (word doesn't have ONECAP)
        assert!(!spell_valid_case_impl(0, WF_ONECAP));

        // ONECAP word, tree has ONECAP -> valid
        assert!(spell_valid_case_impl(WF_ONECAP, WF_ONECAP));

        // Normal word, tree has ALLCAP -> invalid
        assert!(!spell_valid_case_impl(0, WF_ALLCAP));

        // Normal word, tree has KEEPCAP -> invalid
        assert!(!spell_valid_case_impl(0, WF_KEEPCAP));
    }

    #[test]
    fn test_spell_valid_case_onecap_word() {
        // ONECAP word matches ONECAP tree
        assert!(spell_valid_case_impl(WF_ONECAP, WF_ONECAP));

        // ONECAP word, no tree flags -> valid
        assert!(spell_valid_case_impl(WF_ONECAP, 0));
    }

    #[test]
    fn test_ffi_spell_valid_case() {
        assert!(rs_spell_valid_case(WF_ALLCAP, 0));
        assert!(rs_spell_valid_case(WF_ALLCAP, WF_FIXCAP)); // valid via second branch
        assert!(rs_spell_valid_case(WF_ALLCAP, WF_ALLCAP)); // valid via first branch
        assert!(rs_spell_valid_case(0, 0));
        assert!(!rs_spell_valid_case(0, WF_ONECAP));
        assert!(!rs_spell_valid_case(WF_ALLCAP, WF_FIXCAP | WF_ALLCAP)); // both branches fail
    }

    #[test]
    fn test_byte_in_str_found() {
        let s = b"hello\0";
        assert!(byte_in_str_impl(s.as_ptr(), c_int::from(b'h')));
        assert!(byte_in_str_impl(s.as_ptr(), c_int::from(b'e')));
        assert!(byte_in_str_impl(s.as_ptr(), c_int::from(b'l')));
        assert!(byte_in_str_impl(s.as_ptr(), c_int::from(b'o')));
    }

    #[test]
    fn test_byte_in_str_not_found() {
        let s = b"hello\0";
        assert!(!byte_in_str_impl(s.as_ptr(), c_int::from(b'x')));
        assert!(!byte_in_str_impl(s.as_ptr(), c_int::from(b'H'))); // case-sensitive
        assert!(!byte_in_str_impl(s.as_ptr(), 0)); // NUL is terminator, not in string
    }

    #[test]
    fn test_byte_in_str_empty() {
        let s = b"\0";
        assert!(!byte_in_str_impl(s.as_ptr(), c_int::from(b'a')));
    }

    #[test]
    fn test_byte_in_str_null() {
        assert!(!byte_in_str_impl(std::ptr::null(), c_int::from(b'a')));
    }

    #[test]
    fn test_ffi_byte_in_str() {
        let s = b"test\0";
        assert!(rs_byte_in_str(s.as_ptr(), c_int::from(b't')));
        assert!(!rs_byte_in_str(s.as_ptr(), c_int::from(b'x')));
    }

    #[test]
    fn test_word_flag_constants() {
        // Verify word flag constants match C definitions from spell_defs.h
        assert_eq!(WF_ONECAP, 0x02);
        assert_eq!(WF_ALLCAP, 0x04);
        assert_eq!(WF_FIXCAP, 0x40);
        assert_eq!(WF_KEEPCAP, 0x80);
    }

    #[test]
    fn test_spelllang_allowed_chars() {
        // Verify SPELLLANG_ALLOWED contains expected special characters
        assert_eq!(SPELLLANG_ALLOWED, b".-_,@");
    }

    #[test]
    fn test_valid_spellfile_basic() {
        // Valid: ends with .add, >= 4 chars
        assert!(valid_spellfile_impl(b"test.add"));
        assert!(valid_spellfile_impl(b"a.add"));
        assert!(valid_spellfile_impl(b".add")); // exactly 4 chars

        // Invalid: too short
        assert!(!valid_spellfile_impl(b"add"));
        assert!(!valid_spellfile_impl(b".ad"));

        // Invalid: wrong suffix
        assert!(!valid_spellfile_impl(b"test.txt"));
        assert!(!valid_spellfile_impl(b"test.ada"));
    }

    #[test]
    fn test_valid_spellfile_multiple() {
        // Multiple valid paths
        assert!(valid_spellfile_impl(b"foo.add,bar.add"));
        assert!(valid_spellfile_impl(b"path/to/file.add,other.add"));

        // One invalid path fails all
        assert!(!valid_spellfile_impl(b"good.add,bad"));
        assert!(!valid_spellfile_impl(b"bad,good.add"));
    }

    #[test]
    fn test_valid_spellfile_empty() {
        // Empty is valid
        assert!(valid_spellfile_impl(b""));
    }

    #[test]
    fn test_valid_spellfile_ffi() {
        use std::ffi::CString;

        unsafe {
            // Valid
            let valid = CString::new("test.add").unwrap();
            assert!(rs_valid_spellfile(valid.as_ptr()));

            // Invalid suffix
            let invalid = CString::new("test.txt").unwrap();
            assert!(!rs_valid_spellfile(invalid.as_ptr()));

            // Empty is valid
            let empty = CString::new("").unwrap();
            assert!(rs_valid_spellfile(empty.as_ptr()));

            // Null is valid
            assert!(rs_valid_spellfile(std::ptr::null()));
        }
    }

    #[test]
    fn test_find_region_found() {
        // Region list: "us", "uk", "au"
        let regions = b"usukau\0";
        let us = b"us\0";
        let uk = b"uk\0";
        let au = b"au\0";

        unsafe {
            assert_eq!(
                find_region_impl(
                    regions.as_ptr() as *const c_char,
                    us.as_ptr() as *const c_char
                ),
                0
            );
            assert_eq!(
                find_region_impl(
                    regions.as_ptr() as *const c_char,
                    uk.as_ptr() as *const c_char
                ),
                1
            );
            assert_eq!(
                find_region_impl(
                    regions.as_ptr() as *const c_char,
                    au.as_ptr() as *const c_char
                ),
                2
            );
        }
    }

    #[test]
    fn test_find_region_not_found() {
        let regions = b"usuk\0";
        let de = b"de\0";

        unsafe {
            assert_eq!(
                find_region_impl(
                    regions.as_ptr() as *const c_char,
                    de.as_ptr() as *const c_char
                ),
                REGION_ALL
            );
        }
    }

    #[test]
    fn test_find_region_empty() {
        let empty = b"\0";
        let us = b"us\0";

        unsafe {
            assert_eq!(
                find_region_impl(
                    empty.as_ptr() as *const c_char,
                    us.as_ptr() as *const c_char
                ),
                REGION_ALL
            );
        }
    }

    #[test]
    fn test_find_region_null() {
        let us = b"us\0";

        unsafe {
            assert_eq!(
                find_region_impl(std::ptr::null(), us.as_ptr() as *const c_char),
                REGION_ALL
            );
            assert_eq!(
                find_region_impl(us.as_ptr() as *const c_char, std::ptr::null()),
                REGION_ALL
            );
        }
    }

    #[test]
    fn test_find_region_ffi() {
        let regions = b"usukau\0";
        let uk = b"uk\0";
        let de = b"de\0";

        unsafe {
            assert_eq!(
                rs_find_region(
                    regions.as_ptr() as *const c_char,
                    uk.as_ptr() as *const c_char
                ),
                1
            );
            assert_eq!(
                rs_find_region(
                    regions.as_ptr() as *const c_char,
                    de.as_ptr() as *const c_char
                ),
                REGION_ALL
            );
        }
    }

    #[test]
    fn test_region_all_constant() {
        assert_eq!(REGION_ALL, 0xff);
    }

    #[test]
    fn test_sal_to_bool_one() {
        let one = b"1\0";
        unsafe {
            assert!(sal_to_bool_impl(one.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_true() {
        let t = b"true\0";
        unsafe {
            assert!(sal_to_bool_impl(t.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_false() {
        let f = b"false\0";
        unsafe {
            assert!(!sal_to_bool_impl(f.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_zero() {
        let zero = b"0\0";
        unsafe {
            assert!(!sal_to_bool_impl(zero.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_empty() {
        let empty = b"\0";
        unsafe {
            assert!(!sal_to_bool_impl(empty.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_null() {
        unsafe {
            assert!(!sal_to_bool_impl(std::ptr::null()));
        }
    }

    #[test]
    fn test_sal_to_bool_true_uppercase() {
        // "TRUE" should return false (case-sensitive)
        let t = b"TRUE\0";
        unsafe {
            assert!(!sal_to_bool_impl(t.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_partial_matches() {
        // "true1" should return false
        let t1 = b"true1\0";
        unsafe {
            assert!(!sal_to_bool_impl(t1.as_ptr() as *const c_char));
        }

        // "1true" should return false
        let one_true = b"1true\0";
        unsafe {
            assert!(!sal_to_bool_impl(one_true.as_ptr() as *const c_char));
        }

        // "tru" should return false
        let tru = b"tru\0";
        unsafe {
            assert!(!sal_to_bool_impl(tru.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_sal_to_bool_ffi() {
        let one = b"1\0";
        let t = b"true\0";
        let f = b"false\0";
        unsafe {
            assert!(rs_sal_to_bool(one.as_ptr() as *const c_char));
            assert!(rs_sal_to_bool(t.as_ptr() as *const c_char));
            assert!(!rs_sal_to_bool(f.as_ptr() as *const c_char));
            assert!(!rs_sal_to_bool(std::ptr::null()));
        }
    }

    // =========================================================================
    // spell_mb_isword_class tests
    // =========================================================================

    #[test]
    fn test_spell_mb_isword_class_normal_mode() {
        // Class 2 is word character
        assert!(spell_mb_isword_class_impl(2, false));

        // Classes >= 2 are word chars (except specific exclusions)
        assert!(spell_mb_isword_class_impl(4, false));
        assert!(spell_mb_isword_class_impl(10, false));
        assert!(spell_mb_isword_class_impl(100, false));

        // Class 0 and 1 are not word characters
        assert!(!spell_mb_isword_class_impl(0, false));
        assert!(!spell_mb_isword_class_impl(1, false));

        // Class 3 is explicitly excluded
        assert!(!spell_mb_isword_class_impl(3, false));

        // Subscript (0x2070) and superscript (0x2080) are excluded
        assert!(!spell_mb_isword_class_impl(0x2070, false));
        assert!(!spell_mb_isword_class_impl(0x2080, false));

        // Braille (0x2800) is valid in normal mode (>= 2 and not excluded)
        assert!(spell_mb_isword_class_impl(0x2800, false));
    }

    #[test]
    fn test_spell_mb_isword_class_cjk_mode() {
        // In CJK mode, only class 2 and 0x2800 are valid

        // Class 2 is valid
        assert!(spell_mb_isword_class_impl(2, true));

        // Braille (0x2800) is valid
        assert!(spell_mb_isword_class_impl(0x2800, true));

        // Other classes are not valid in CJK mode
        assert!(!spell_mb_isword_class_impl(0, true));
        assert!(!spell_mb_isword_class_impl(1, true));
        assert!(!spell_mb_isword_class_impl(3, true));
        assert!(!spell_mb_isword_class_impl(4, true));
        assert!(!spell_mb_isword_class_impl(10, true));

        // Even classes that are valid in normal mode are not valid in CJK
        assert!(!spell_mb_isword_class_impl(5, true));
        assert!(!spell_mb_isword_class_impl(100, true));
    }

    #[test]
    fn test_spell_mb_isword_class_boundary_values() {
        // Test at class boundary
        assert!(!spell_mb_isword_class_impl(1, false)); // just below 2
        assert!(spell_mb_isword_class_impl(2, false)); // exactly 2
        assert!(!spell_mb_isword_class_impl(3, false)); // excluded
        assert!(spell_mb_isword_class_impl(4, false)); // just above 3

        // Negative values
        assert!(!spell_mb_isword_class_impl(-1, false));
        assert!(!spell_mb_isword_class_impl(-1, true));
    }

    #[test]
    fn test_spell_mb_isword_class_ffi() {
        // Test FFI wrapper
        assert!(rs_spell_mb_isword_class(2, false));
        assert!(rs_spell_mb_isword_class(2, true));
        assert!(rs_spell_mb_isword_class(0x2800, true));
        assert!(!rs_spell_mb_isword_class(1, false));
        assert!(!rs_spell_mb_isword_class(4, true));
    }

    #[test]
    fn test_spell_mb_isword_class_special_classes() {
        // Test the specific exclusion values
        // 0x2070 = Unicode subscript block marker
        // 0x2080 = Unicode superscript block marker
        assert!(!spell_mb_isword_class_impl(0x2070, false));
        assert!(!spell_mb_isword_class_impl(0x2080, false));

        // Values around these should still work (if >= 2)
        assert!(spell_mb_isword_class_impl(0x206F, false)); // just before 0x2070
        assert!(spell_mb_isword_class_impl(0x2071, false)); // just after 0x2070
        assert!(spell_mb_isword_class_impl(0x207F, false)); // just before 0x2080
        assert!(spell_mb_isword_class_impl(0x2081, false)); // just after 0x2080
    }

    // =========================================================================
    // Spell file format tests
    // =========================================================================

    #[test]
    fn test_spell_file_constants() {
        assert_eq!(VIMSPELL_MAGIC, b"VIMspell");
        assert_eq!(VIMSPELL_MAGIC_LEN, 8);
        assert_eq!(VIMSPELL_VERSION, 50);
    }

    #[test]
    fn test_check_spell_magic_valid() {
        let buf = b"VIMspell\x32extra";
        assert!(check_spell_magic(buf));
    }

    #[test]
    fn test_check_spell_magic_invalid() {
        let buf = b"VIMspelX"; // Wrong last char
        assert!(!check_spell_magic(buf));

        let buf = b"vimspell"; // Wrong case
        assert!(!check_spell_magic(buf));
    }

    #[test]
    fn test_check_spell_magic_too_short() {
        let buf = b"VIMspel"; // Only 7 bytes
        assert!(!check_spell_magic(buf));
    }

    #[test]
    fn test_spell_version_checks() {
        assert!(check_spell_version(50));
        assert!(!check_spell_version(49));
        assert!(!check_spell_version(51));

        assert!(spell_version_too_old(49));
        assert!(!spell_version_too_old(50));
        assert!(!spell_version_too_old(51));

        assert!(!spell_version_too_new(49));
        assert!(!spell_version_too_new(50));
        assert!(spell_version_too_new(51));
    }

    #[test]
    fn test_read_be_u16() {
        let buf = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(read_be_u16(&buf, 0), Some(0x1234));
        assert_eq!(read_be_u16(&buf, 1), Some(0x3456));
        assert_eq!(read_be_u16(&buf, 2), Some(0x5678));
        assert_eq!(read_be_u16(&buf, 3), None); // Not enough bytes
    }

    #[test]
    fn test_read_be_u24() {
        let buf = [0x12, 0x34, 0x56, 0x78, 0x9A];
        assert_eq!(read_be_u24(&buf, 0), Some(0x0012_3456));
        assert_eq!(read_be_u24(&buf, 1), Some(0x0034_5678));
        assert_eq!(read_be_u24(&buf, 2), Some(0x0056_789A));
        assert_eq!(read_be_u24(&buf, 3), None); // Not enough bytes
    }

    #[test]
    fn test_read_be_u32() {
        let buf = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        assert_eq!(read_be_u32(&buf, 0), Some(0x1234_5678));
        assert_eq!(read_be_u32(&buf, 1), Some(0x3456_789A));
        assert_eq!(read_be_u32(&buf, 2), Some(0x5678_9ABC));
        assert_eq!(read_be_u32(&buf, 3), None); // Not enough bytes
    }

    #[test]
    fn test_parse_section_header() {
        // Section header: id=5, flags=1, len=0x00001234
        let buf = [0x05, 0x01, 0x00, 0x00, 0x12, 0x34, 0xFF, 0xFF];
        let result = parse_section_header(&buf, 0);
        assert_eq!(result, Some((5, 1, 0x1234, 6)));

        // Not enough bytes
        let short_buf = [0x05, 0x01, 0x00, 0x00, 0x12];
        assert_eq!(parse_section_header(&short_buf, 0), None);
    }

    #[test]
    fn test_spell_section_enum() {
        assert_eq!(SpellSection::Region as u8, 0);
        assert_eq!(SpellSection::CharFlags as u8, 1);
        assert_eq!(SpellSection::End as u8, 255);
    }

    #[test]
    fn test_section_flags() {
        assert_eq!(SNF_REQUIRED, 1);
    }

    #[test]
    fn test_byte_values() {
        assert_eq!(BY_NOFLAGS, 0);
        assert_eq!(BY_FLAGS, 1);
        assert_eq!(BY_FLAGS2, 2);
        assert_eq!(BY_INDEX, 3);
    }

    #[test]
    fn test_char_flags() {
        assert_eq!(CF_WORD, 0x01);
        assert_eq!(CF_UPPER, 0x02);
    }

    #[test]
    fn test_ffi_spell_magic() {
        let valid = b"VIMspell";
        let invalid = b"notmagic";

        unsafe {
            assert!(rs_check_spell_magic(valid.as_ptr()));
            assert!(!rs_check_spell_magic(invalid.as_ptr()));
            assert!(!rs_check_spell_magic(std::ptr::null()));
        }
    }

    #[test]
    fn test_ffi_spell_version() {
        assert!(rs_check_spell_version(50));
        assert!(!rs_check_spell_version(49));
        assert!(rs_spell_version_too_old(49));
        assert!(rs_spell_version_too_new(51));
    }

    #[test]
    fn test_ffi_read_be() {
        let buf = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];

        unsafe {
            assert_eq!(rs_read_be_u16(buf.as_ptr(), 6, 0), 0x1234);
            assert_eq!(rs_read_be_u24(buf.as_ptr(), 6, 0), 0x0012_3456);
            assert_eq!(rs_read_be_u32(buf.as_ptr(), 6, 0), 0x1234_5678);

            // Out of bounds
            assert_eq!(rs_read_be_u16(buf.as_ptr(), 6, 5), -1);
            assert_eq!(rs_read_be_u24(buf.as_ptr(), 6, 4), -1);
            assert_eq!(rs_read_be_u32(buf.as_ptr(), 6, 3), -1);

            // Null pointer
            assert_eq!(rs_read_be_u16(std::ptr::null(), 6, 0), -1);
        }
    }

    // =========================================================================
    // Spell error constants tests
    // =========================================================================

    #[test]
    fn test_sp_error_constants() {
        assert_eq!(SP_TRUNCERROR, -1);
        assert_eq!(SP_FORMERROR, -2);
        assert_eq!(SP_OTHERERROR, -3);
    }

    #[test]
    fn test_wf_flag_constants() {
        assert_eq!(WF_REGION, 0x01);
        assert_eq!(WF_ONECAP_FLAG, 0x02);
        assert_eq!(WF_ALLCAP_FLAG, 0x04);
        assert_eq!(WF_RARE, 0x08);
        assert_eq!(WF_BANNED, 0x10);
        assert_eq!(WF_AFX, 0x20);
        assert_eq!(WF_FIXCAP_FLAG, 0x40);
        assert_eq!(WF_KEEPCAP_FLAG, 0x80);
    }

    #[test]
    fn test_maxwlen() {
        assert_eq!(MAXWLEN, 254);
    }

    // =========================================================================
    // Tree count words tests
    // =========================================================================

    #[test]
    fn test_tree_count_words_null() {
        unsafe {
            // Should not crash with null pointers
            rs_tree_count_words(std::ptr::null(), std::ptr::null_mut(), 0);
            rs_tree_count_words(std::ptr::null(), std::ptr::null_mut(), 10);
        }
    }

    #[test]
    fn test_tree_count_words_simple() {
        // Tree with 2 NUL siblings = 1 word (adjacent NULs are same word with different flags)
        // byts[0] = 2 (sibling count)
        // byts[1] = 0 (word end)
        // byts[2] = 0 (same word, different flags - skipped)
        let byts: [u8; 3] = [2, 0, 0];
        let mut idxs: [IdxT; 3] = [0, 0, 0];

        unsafe {
            rs_tree_count_words(byts.as_ptr(), idxs.as_mut_ptr(), 3);
        }

        // Adjacent NULs are treated as same word with different flags, so count = 1
        assert_eq!(idxs[0], 1);
    }

    #[test]
    fn test_tree_count_words_two_words() {
        // Tree with a character 'a' followed by word end, then another char 'b' with word end
        // Root: [2, 'a', 'b']
        //         ^   ^    ^
        //         |   |    +-- character 'b', child at index 5
        //         |   +------- character 'a', child at index 3
        //         +----------- sibling count = 2
        // Child for 'a' at idx 3: [1, 0] = 1 sibling, word end
        // Child for 'b' at idx 5: [1, 0] = 1 sibling, word end
        let byts: [u8; 7] = [
            2,    // root: 2 siblings
            b'a', // first sibling
            b'b', // second sibling
            1,    // child of 'a': 1 sibling
            0,    // word end
            1,    // child of 'b': 1 sibling
            0,    // word end
        ];
        let mut idxs: [IdxT; 7] = [
            0, // root word count (will be set)
            3, // 'a' points to child at idx 3
            5, // 'b' points to child at idx 5
            0, // child 'a' word count (will be set)
            0, // flags for word end
            0, // child 'b' word count (will be set)
            0, // flags for word end
        ];

        unsafe {
            rs_tree_count_words(byts.as_ptr(), idxs.as_mut_ptr(), 7);
        }

        // Each child should count 1 word
        assert_eq!(idxs[3], 1); // child of 'a' has 1 word
        assert_eq!(idxs[5], 1); // child of 'b' has 1 word
                                // Root should have 2 words total
        assert_eq!(idxs[0], 2);
    }

    #[test]
    fn test_tree_count_words_empty() {
        let byts: [u8; 1] = [0]; // Empty tree (0 siblings)
        let mut idxs: [IdxT; 1] = [0];

        unsafe {
            rs_tree_count_words(byts.as_ptr(), idxs.as_mut_ptr(), 1);
        }

        // Should handle gracefully
        assert_eq!(idxs[0], 0);
    }

    // =========================================================================
    // Read tree node tests
    // =========================================================================

    #[test]
    fn test_read_tree_node_null() {
        unsafe {
            // Should return error with null pointers
            let result = rs_read_tree_node(
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                0,
                false,
                0,
            );
            assert_eq!(result, SP_TRUNCERROR);
        }
    }

    #[test]
    fn test_read_tree_node_truncated() {
        let buf: [u8; 0] = [];
        let mut offset: usize = 0;
        let mut byts: [u8; 10] = [0; 10];
        let mut idxs: [IdxT; 10] = [0; 10];

        unsafe {
            let result = rs_read_tree_node(
                buf.as_ptr(),
                0,
                &raw mut offset,
                byts.as_mut_ptr(),
                idxs.as_mut_ptr(),
                10,
                0,
                false,
                0,
            );
            assert_eq!(result, SP_TRUNCERROR);
        }
    }

    #[test]
    fn test_read_tree_node_simple() {
        // Simple tree node: 2 siblings, both are word ends (BY_NOFLAGS)
        let buf: [u8; 3] = [
            2,          // sibling count
            BY_NOFLAGS, // first sibling: word end, no flags
            BY_NOFLAGS, // second sibling: word end, no flags
        ];
        let mut offset: usize = 0;
        let mut byts: [u8; 10] = [0; 10];
        let mut idxs: [IdxT; 10] = [0; 10];

        unsafe {
            let result = rs_read_tree_node(
                buf.as_ptr(),
                buf.len(),
                &raw mut offset,
                byts.as_mut_ptr(),
                idxs.as_mut_ptr(),
                10,
                0,
                false,
                0,
            );

            // Should return next index (3: start=0 + 1 for count + 2 siblings)
            assert_eq!(result, 3);
            assert_eq!(offset, 3);
            assert_eq!(byts[0], 2); // sibling count
            assert_eq!(byts[1], 0); // BY_NOFLAGS -> 0
            assert_eq!(byts[2], 0); // BY_NOFLAGS -> 0
            assert_eq!(idxs[1], 0); // flags for first word
            assert_eq!(idxs[2], 0); // flags for second word
        }
    }

    #[test]
    fn test_read_tree_node_with_flags() {
        // Tree node with flags: 1 sibling with BY_FLAGS
        let buf: [u8; 3] = [
            1,        // sibling count
            BY_FLAGS, // BY_FLAGS indicates flags follow
            0x08,     // flags byte (WF_RARE)
        ];
        let mut offset: usize = 0;
        let mut byts: [u8; 10] = [0; 10];
        let mut idxs: [IdxT; 10] = [0; 10];

        unsafe {
            let result = rs_read_tree_node(
                buf.as_ptr(),
                buf.len(),
                &raw mut offset,
                byts.as_mut_ptr(),
                idxs.as_mut_ptr(),
                10,
                0,
                false,
                0,
            );

            assert_eq!(result, 2); // start=0 + 1 count + 1 sibling
            assert_eq!(byts[0], 1); // sibling count
            assert_eq!(byts[1], 0); // word end
            assert_eq!(idxs[1], 0x08); // flags = WF_RARE
        }
    }

    #[test]
    fn test_read_tree_node_format_error() {
        // Tree node that would overflow the array
        let buf: [u8; 2] = [
            100, // sibling count (way too many for array size)
            BY_NOFLAGS,
        ];
        let mut offset: usize = 0;
        let mut byts: [u8; 10] = [0; 10];
        let mut idxs: [IdxT; 10] = [0; 10];

        unsafe {
            let result = rs_read_tree_node(
                buf.as_ptr(),
                buf.len(),
                &raw mut offset,
                byts.as_mut_ptr(),
                idxs.as_mut_ptr(),
                10,
                0,
                false,
                0,
            );

            assert_eq!(result, SP_FORMERROR);
        }
    }

    // =========================================================================
    // Offset encoding/decoding tests
    // =========================================================================

    #[test]
    fn test_offset2bytes_one_byte() {
        // Values 0-126 use 1 byte
        let mut buf = [0u8; 4];

        assert_eq!(offset2bytes(0, &mut buf), 1);
        assert_eq!(buf[0], 0x01); // 0 + 1

        assert_eq!(offset2bytes(1, &mut buf), 1);
        assert_eq!(buf[0], 0x02); // 1 + 1

        assert_eq!(offset2bytes(126, &mut buf), 1);
        assert_eq!(buf[0], 0x7f); // 126 + 1
    }

    #[test]
    fn test_offset2bytes_two_bytes() {
        // Values requiring 2 bytes
        let mut buf = [0u8; 4];

        assert_eq!(offset2bytes(127, &mut buf), 2);
        // b1 = 127 % 255 + 1 = 128 (0x80), b2 = 127 / 255 + 1 = 1
        assert_eq!(buf[0], 0x81); // 0x80 + 1
        assert_eq!(buf[1], 0x80); // 128
    }

    #[test]
    fn test_offset2bytes_roundtrip() {
        // Test that encode/decode is reversible
        let test_values = [0, 1, 126, 127, 254, 255, 1000, 10_000, 100_000, 1_000_000];

        for &val in &test_values {
            let mut buf = [0u8; 4];
            let len = offset2bytes(val, &mut buf);

            let result = bytes2offset(&buf[..len]);
            assert!(result.is_some(), "Failed to decode value {val}");
            let (decoded, consumed) = result.unwrap();
            assert_eq!(decoded, val, "Value {val} decoded to {decoded}");
            assert_eq!(consumed, len, "Consumed bytes mismatch for value {val}");
        }
    }

    #[test]
    fn test_bytes2offset_one_byte() {
        // 1-byte encoding: 0x01-0x7F
        assert_eq!(bytes2offset(&[0x01]), Some((0, 1)));
        assert_eq!(bytes2offset(&[0x02]), Some((1, 1)));
        assert_eq!(bytes2offset(&[0x7f]), Some((126, 1)));
    }

    #[test]
    fn test_bytes2offset_two_bytes() {
        // 2-byte encoding: first byte 0x80-0xBF
        // Value 127: b1 = 128 (0x80), b2 = 1
        // Encoding: 0x81, 0x80
        let result = bytes2offset(&[0x81, 0x80]);
        assert!(result.is_some());
        let (val, len) = result.unwrap();
        assert_eq!(len, 2);
        assert_eq!(val, 127);
    }

    #[test]
    fn test_bytes2offset_empty() {
        assert_eq!(bytes2offset(&[]), None);
    }

    #[test]
    fn test_bytes2offset_truncated_two() {
        // 2-byte marker but only 1 byte
        assert_eq!(bytes2offset(&[0x80]), None);
    }

    #[test]
    fn test_bytes2offset_truncated_three() {
        // 3-byte marker but only 2 bytes
        assert_eq!(bytes2offset(&[0xc1, 0x01]), None);
    }

    #[test]
    fn test_bytes2offset_truncated_four() {
        // 4-byte marker but only 3 bytes
        assert_eq!(bytes2offset(&[0xe1, 0x01, 0x01]), None);
    }

    #[test]
    fn test_offset_encoding_boundary_values() {
        // Test boundary values between byte widths
        let mut buf = [0u8; 4];

        // Just fits in 1 byte
        assert_eq!(offset2bytes(126, &mut buf), 1);

        // First value needing 2 bytes
        assert_eq!(offset2bytes(127, &mut buf), 2);
    }

    // =============================================================================
    // Suggestion Scoring Tests
    // =============================================================================

    #[test]
    fn test_rescore() {
        // Formula: (3 * word_score + sound_score) / 4
        assert_eq!(rescore(100, 100), 100);
        assert_eq!(rescore(100, 0), 75);
        assert_eq!(rescore(0, 100), 25);
        assert_eq!(rescore(200, 100), 175);
    }

    #[test]
    fn test_maxscore() {
        // Formula: (4 * end_score - sound_score) / 3
        // Should be inverse of rescore
        assert_eq!(maxscore(100, 100), 100);
        assert_eq!(maxscore(75, 0), 100);
    }

    #[test]
    fn test_score_constants() {
        assert_eq!(rs_score_swap(), 75);
        assert_eq!(rs_score_swap3(), 110);
        assert_eq!(rs_score_subst(), 93);
        assert_eq!(rs_score_similar(), 33);
        assert_eq!(rs_score_del(), 94);
        assert_eq!(rs_score_deldup(), 66);
        assert_eq!(rs_score_ins(), 96);
        assert_eq!(rs_score_insdup(), 67);
        assert_eq!(rs_score_nonword(), 103);
        assert_eq!(rs_score_split(), 149);
        assert_eq!(rs_score_split_no(), 249);
        assert_eq!(rs_score_icase(), 52);
        assert_eq!(rs_score_region(), 200);
        assert_eq!(rs_score_rare(), 180);
        assert_eq!(rs_score_rep(), 65);
        assert_eq!(rs_score_file(), 30);
        assert_eq!(rs_score_maxinit(), 350);
    }

    #[test]
    fn test_score_acceptability() {
        assert!(rs_score_is_acceptable(0));
        assert!(rs_score_is_acceptable(100));
        assert!(rs_score_is_acceptable(SCORE_MAXMAX - 1));
        assert!(!rs_score_is_acceptable(SCORE_MAXMAX));
    }

    #[test]
    fn test_score_failed() {
        assert!(!rs_score_is_failed(0));
        assert!(!rs_score_is_failed(SCORE_MAXMAX - 1));
        assert!(rs_score_is_failed(SCORE_MAXMAX));
        assert!(rs_score_is_failed(SCORE_MAXMAX + 1));
    }

    #[test]
    fn test_score_combine() {
        assert_eq!(rs_score_combine(50, 50), 100);
        assert_eq!(rs_score_combine(0, 100), 100);
        assert_eq!(
            rs_score_combine(SCORE_SWAP, SCORE_DEL),
            SCORE_SWAP + SCORE_DEL
        );
    }

    #[test]
    fn test_score_common_bonus() {
        // No bonus for count 0
        assert_eq!(rs_score_common_bonus(0), 0);
        // COMMON1 for count >= 1
        assert_eq!(rs_score_common_bonus(1), SCORE_COMMON1);
        assert_eq!(rs_score_common_bonus(9), SCORE_COMMON1);
        // COMMON2 for count >= 10
        assert_eq!(rs_score_common_bonus(10), SCORE_COMMON2);
        assert_eq!(rs_score_common_bonus(99), SCORE_COMMON2);
        // COMMON3 for count >= 100
        assert_eq!(rs_score_common_bonus(100), SCORE_COMMON3);
        assert_eq!(rs_score_common_bonus(1000), SCORE_COMMON3);
    }

    #[test]
    fn test_score_apply_common_bonus() {
        // Apply COMMON1 bonus
        assert_eq!(rs_score_apply_common_bonus(100, 5), 100 - SCORE_COMMON1);
        // Score less than bonus should return 0
        assert_eq!(rs_score_apply_common_bonus(20, 5), 0);
        // No bonus for count 0
        assert_eq!(rs_score_apply_common_bonus(100, 0), 100);
    }

    #[test]
    fn test_score_sfmax() {
        assert_eq!(rs_score_sfmax(1), SCORE_SFMAX1);
        assert_eq!(rs_score_sfmax(2), SCORE_SFMAX2);
        assert_eq!(rs_score_sfmax(3), SCORE_SFMAX3);
        assert_eq!(rs_score_sfmax(4), SCORE_SFMAX3);
    }

    #[test]
    fn test_score_within_sfmax() {
        assert!(rs_score_within_sfmax(100, 1));
        assert!(rs_score_within_sfmax(200, 1));
        assert!(!rs_score_within_sfmax(201, 1));
        assert!(rs_score_within_sfmax(300, 2));
        assert!(!rs_score_within_sfmax(301, 2));
    }

    #[test]
    fn test_score_edit_min() {
        assert_eq!(rs_score_edit_min(), SCORE_SIMILAR);
    }

    #[test]
    fn test_score_big() {
        assert_eq!(rs_score_big(), 3 * SCORE_INS);
    }

    #[test]
    fn test_score_limitmax() {
        assert_eq!(rs_score_limitmax(), SCORE_LIMITMAX);
    }

    // =============================================================================
    // Word Flag Tests
    // =============================================================================

    #[test]
    fn test_wf_is_rare() {
        assert!(rs_wf_is_rare(WF_RARE));
        assert!(!rs_wf_is_rare(0));
        assert!(!rs_wf_is_rare(WF_BANNED));
    }

    #[test]
    fn test_wf_is_banned() {
        assert!(rs_wf_is_banned(WF_BANNED));
        assert!(!rs_wf_is_banned(0));
        assert!(!rs_wf_is_banned(WF_RARE));
    }

    #[test]
    fn test_wf_is_allcap() {
        assert!(rs_wf_is_allcap(WF_ALLCAP_FLAG));
        assert!(!rs_wf_is_allcap(0));
    }

    #[test]
    fn test_wf_is_onecap() {
        assert!(rs_wf_is_onecap(WF_ONECAP_FLAG));
        assert!(!rs_wf_is_onecap(0));
    }

    #[test]
    fn test_wf_is_keepcap() {
        assert!(rs_wf_is_keepcap(WF_KEEPCAP_FLAG));
        assert!(!rs_wf_is_keepcap(0));
    }

    #[test]
    fn test_wf_is_fixcap() {
        assert!(rs_wf_is_fixcap(WF_FIXCAP_FLAG));
        assert!(!rs_wf_is_fixcap(0));
    }

    #[test]
    fn test_wf_has_region() {
        assert!(rs_wf_has_region(WF_REGION));
        assert!(!rs_wf_has_region(0));
    }

    #[test]
    fn test_wf_has_afx() {
        assert!(rs_wf_has_afx(WF_AFX));
        assert!(!rs_wf_has_afx(0));
    }

    // =========================================================================
    // Phase 5: Spell UI Integration Tests
    // =========================================================================

    #[test]
    fn test_spell_add_type() {
        assert_eq!(SpellAddType::Good as c_int, 0);
        assert_eq!(SpellAddType::Bad as c_int, 1);
        assert_eq!(SpellAddType::Rare as c_int, 2);

        assert_eq!(SpellAddType::from_c_int(0), Some(SpellAddType::Good));
        assert_eq!(SpellAddType::from_c_int(1), Some(SpellAddType::Bad));
        assert_eq!(SpellAddType::from_c_int(2), Some(SpellAddType::Rare));
        assert_eq!(SpellAddType::from_c_int(3), None);
        assert_eq!(SpellAddType::from_c_int(-1), None);
    }

    #[test]
    fn test_spell_move_type() {
        assert_eq!(SpellMoveType::All as c_int, 0);
        assert_eq!(SpellMoveType::Bad as c_int, 1);
        assert_eq!(SpellMoveType::Rare as c_int, 2);

        assert_eq!(SpellMoveType::from_c_int(0), Some(SpellMoveType::All));
        assert_eq!(SpellMoveType::from_c_int(1), Some(SpellMoveType::Bad));
        assert_eq!(SpellMoveType::from_c_int(2), Some(SpellMoveType::Rare));
        assert_eq!(SpellMoveType::from_c_int(3), None);
    }

    #[test]
    fn test_spell_add_type_ffi() {
        assert_eq!(rs_spell_add_type_from_int(0), 0);
        assert_eq!(rs_spell_add_type_from_int(1), 1);
        assert_eq!(rs_spell_add_type_from_int(2), 2);
        // Invalid values return Good (0)
        assert_eq!(rs_spell_add_type_from_int(3), 0);
        assert_eq!(rs_spell_add_type_from_int(-1), 0);
    }

    #[test]
    fn test_spell_move_type_ffi() {
        assert_eq!(rs_spell_move_type_from_int(0), 0);
        assert_eq!(rs_spell_move_type_from_int(1), 1);
        assert_eq!(rs_spell_move_type_from_int(2), 2);
        // Invalid values return All (0)
        assert_eq!(rs_spell_move_type_from_int(3), 0);
    }

    #[test]
    fn test_spell_suggest_timeout_constants() {
        assert_eq!(rs_spell_suggest_timeout_default(), 5000);
        assert_eq!(SPELL_SUGGEST_TIMEOUT_MIN, 100);
        assert_eq!(SPELL_SUGGEST_TIMEOUT_MAX, 60000);
    }

    #[test]
    fn test_validate_spell_suggest_timeout() {
        assert!(validate_spell_suggest_timeout(5000));
        assert!(validate_spell_suggest_timeout(100));
        assert!(validate_spell_suggest_timeout(60000));
        assert!(!validate_spell_suggest_timeout(99));
        assert!(!validate_spell_suggest_timeout(60001));
        assert!(!validate_spell_suggest_timeout(0));
        assert!(!validate_spell_suggest_timeout(-1));
    }

    #[test]
    fn test_clamp_spell_suggest_timeout() {
        assert_eq!(clamp_spell_suggest_timeout(5000), 5000);
        assert_eq!(clamp_spell_suggest_timeout(50), SPELL_SUGGEST_TIMEOUT_MIN);
        assert_eq!(
            clamp_spell_suggest_timeout(100_000),
            SPELL_SUGGEST_TIMEOUT_MAX
        );
        assert_eq!(clamp_spell_suggest_timeout(0), SPELL_SUGGEST_TIMEOUT_MIN);
    }

    #[test]
    fn test_valid_spell_word_start() {
        // ASCII letters
        assert!(is_valid_spell_word_start(b'a'));
        assert!(is_valid_spell_word_start(b'z'));
        assert!(is_valid_spell_word_start(b'A'));
        assert!(is_valid_spell_word_start(b'Z'));
        // High bytes (UTF-8 continuation)
        assert!(is_valid_spell_word_start(0x80));
        assert!(is_valid_spell_word_start(0xFF));
        // Digits are not valid word starts
        assert!(!is_valid_spell_word_start(b'0'));
        assert!(!is_valid_spell_word_start(b'9'));
        // Punctuation
        assert!(!is_valid_spell_word_start(b'.'));
        assert!(!is_valid_spell_word_start(b' '));
    }

    #[test]
    fn test_spell_add_flags() {
        let good = SpellAddFlags::good(1);
        assert_eq!(good.add_type, SpellAddType::Good as c_int);
        assert_eq!(good.spellfile_idx, 1);
        assert!(good.undoable);

        let bad = SpellAddFlags::bad(2);
        assert_eq!(bad.add_type, SpellAddType::Bad as c_int);
        assert_eq!(bad.spellfile_idx, 2);
        assert!(bad.undoable);
        assert!(bad.is_undo());

        let rare = SpellAddFlags::rare(0);
        assert_eq!(rare.add_type, SpellAddType::Rare as c_int);
        assert_eq!(rare.spellfile_idx, 0);
        assert!(rare.undoable);
        assert!(!rare.is_undo());
    }

    #[test]
    fn test_spell_suggest_display_constants() {
        assert_eq!(rs_spell_suggest_display_max(), 25);
        assert_eq!(rs_spell_suggest_display_default(), 5);
        assert_eq!(rs_spell_suggest_score_threshold(), 200);
    }

    #[test]
    fn test_valid_spell_word() {
        // Valid words
        let word = b"hello";
        unsafe {
            assert!(rs_valid_spell_word(word.as_ptr(), word.as_ptr().add(5)));
        }

        // Empty word
        let empty = b"";
        unsafe {
            assert!(!rs_valid_spell_word(empty.as_ptr(), empty.as_ptr()));
        }

        // Word with control char
        let with_ctrl = b"hel\x01lo";
        unsafe {
            assert!(!rs_valid_spell_word(
                with_ctrl.as_ptr(),
                with_ctrl.as_ptr().add(6)
            ));
        }

        // Word with NUL
        let with_nul = b"hel\x00lo";
        unsafe {
            assert!(!rs_valid_spell_word(
                with_nul.as_ptr(),
                with_nul.as_ptr().add(6)
            ));
        }

        // Null pointers
        unsafe {
            assert!(!rs_valid_spell_word(std::ptr::null(), std::ptr::null()));
        }
    }
}
