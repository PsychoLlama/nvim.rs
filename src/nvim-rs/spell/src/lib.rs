//! Spell checking utilities for Neovim
//!
//! This crate provides Rust implementations of spell checking functions
//! from `src/nvim/spell.c`.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a spell language (slang_T)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SlangHandle(*mut c_void);

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

/// Opaque handle to spell table (spelltab_T)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SpelltabHandle(*mut c_void);

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
// C Accessor Declarations
// =============================================================================

extern "C" {
    // slang_T word tree accessors
    fn nvim_slang_get_fbyts(slang: SlangHandle) -> *mut u8;
    fn nvim_slang_get_fidxs(slang: SlangHandle) -> *mut IdxT;
    fn nvim_slang_get_kbyts(slang: SlangHandle) -> *mut u8;
    fn nvim_slang_get_kidxs(slang: SlangHandle) -> *mut IdxT;
    fn nvim_slang_get_pbyts(slang: SlangHandle) -> *mut u8;
    fn nvim_slang_get_pidxs(slang: SlangHandle) -> *mut IdxT;

    // slang_T compound word settings
    fn nvim_slang_get_compmax(slang: SlangHandle) -> c_int;
    fn nvim_slang_get_compminlen(slang: SlangHandle) -> c_int;
    fn nvim_slang_get_compsylmax(slang: SlangHandle) -> c_int;
    fn nvim_slang_get_nobreak(slang: SlangHandle) -> bool;

    // slang_T sound folding settings
    fn nvim_slang_get_sofo(slang: SlangHandle) -> bool;
    fn nvim_slang_get_rem_accents(slang: SlangHandle) -> bool;
    fn nvim_slang_get_followup(slang: SlangHandle) -> bool;
    fn nvim_slang_get_collapse(slang: SlangHandle) -> bool;
    fn nvim_slang_get_sal_first(slang: SlangHandle) -> *mut SalfirstT;
    fn nvim_slang_get_regions(slang: SlangHandle) -> *const c_char;

    // spelltab_T accessors
    fn nvim_get_spelltab() -> SpelltabHandle;
    fn nvim_spelltab_get_isw(sp: SpelltabHandle) -> *mut bool;
    fn nvim_spelltab_get_isu(sp: SpelltabHandle) -> *mut bool;
    fn nvim_spelltab_get_fold(sp: SpelltabHandle) -> *mut u8;
    fn nvim_spelltab_get_upper(sp: SpelltabHandle) -> *mut u8;
}

// =============================================================================
// Safe Wrappers for slang_T Accessors
// =============================================================================

impl SlangHandle {
    /// Get case-folded word bytes array
    #[must_use]
    pub fn fbyts(self) -> *mut u8 {
        unsafe { nvim_slang_get_fbyts(self) }
    }

    /// Get case-folded word indexes array
    #[must_use]
    pub fn fidxs(self) -> *mut IdxT {
        unsafe { nvim_slang_get_fidxs(self) }
    }

    /// Get keep-case word bytes array
    #[must_use]
    pub fn kbyts(self) -> *mut u8 {
        unsafe { nvim_slang_get_kbyts(self) }
    }

    /// Get keep-case word indexes array
    #[must_use]
    pub fn kidxs(self) -> *mut IdxT {
        unsafe { nvim_slang_get_kidxs(self) }
    }

    /// Get prefix tree bytes array
    #[must_use]
    pub fn pbyts(self) -> *mut u8 {
        unsafe { nvim_slang_get_pbyts(self) }
    }

    /// Get prefix tree indexes array
    #[must_use]
    pub fn pidxs(self) -> *mut IdxT {
        unsafe { nvim_slang_get_pidxs(self) }
    }

    /// Get maximum compound word count
    #[must_use]
    pub fn compmax(self) -> c_int {
        unsafe { nvim_slang_get_compmax(self) }
    }

    /// Get minimum compound word length
    #[must_use]
    pub fn compminlen(self) -> c_int {
        unsafe { nvim_slang_get_compminlen(self) }
    }

    /// Get maximum compound syllables
    #[must_use]
    pub fn compsylmax(self) -> c_int {
        unsafe { nvim_slang_get_compsylmax(self) }
    }

    /// Get nobreak flag (no spaces between words)
    #[must_use]
    pub fn nobreak(self) -> bool {
        unsafe { nvim_slang_get_nobreak(self) }
    }

    /// Get SOFOFROM/SOFOTO mode flag
    #[must_use]
    pub fn sofo(self) -> bool {
        unsafe { nvim_slang_get_sofo(self) }
    }

    /// Get remove accents flag
    #[must_use]
    pub fn rem_accents(self) -> bool {
        unsafe { nvim_slang_get_rem_accents(self) }
    }

    /// Get SAL followup flag
    #[must_use]
    pub fn followup(self) -> bool {
        unsafe { nvim_slang_get_followup(self) }
    }

    /// Get SAL collapse flag
    #[must_use]
    pub fn collapse(self) -> bool {
        unsafe { nvim_slang_get_collapse(self) }
    }

    /// Get SAL first lookup table
    #[must_use]
    pub fn sal_first(self) -> *mut SalfirstT {
        unsafe { nvim_slang_get_sal_first(self) }
    }

    /// Get regions string
    #[must_use]
    pub fn regions(self) -> *const c_char {
        unsafe { nvim_slang_get_regions(self) }
    }
}

// =============================================================================
// Safe Wrappers for spelltab_T Accessors
// =============================================================================

impl SpelltabHandle {
    /// Get the global spelltab
    #[must_use]
    pub fn global() -> Self {
        unsafe { nvim_get_spelltab() }
    }

    /// Get is-word flags array (256 entries)
    #[must_use]
    pub fn isw(self) -> *mut bool {
        unsafe { nvim_spelltab_get_isw(self) }
    }

    /// Get is-uppercase flags array (256 entries)
    #[must_use]
    pub fn isu(self) -> *mut bool {
        unsafe { nvim_spelltab_get_isu(self) }
    }

    /// Get fold (lowercase) mapping array (256 entries)
    #[must_use]
    pub fn fold(self) -> *mut u8 {
        unsafe { nvim_spelltab_get_fold(self) }
    }

    /// Get uppercase mapping array (256 entries)
    #[must_use]
    pub fn upper(self) -> *mut u8 {
        unsafe { nvim_spelltab_get_upper(self) }
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

/// FFI wrapper for `clear_spell_chartab`.
///
/// Initialize the chartab used for spelling for ASCII.
#[no_mangle]
pub extern "C" fn rs_clear_spell_chartab(sp: SpelltabHandle) {
    clear_spell_chartab_impl(sp);
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

/// FFI wrapper for `spell_valid_case`.
///
/// Check if the word flags match the tree flags for valid case handling.
#[no_mangle]
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

/// FFI wrapper for `byte_in_str`.
///
/// Check if byte `n` appears in string `str`.
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
