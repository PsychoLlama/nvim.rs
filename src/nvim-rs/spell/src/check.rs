//! Core spell checking implementation for Neovim
//!
//! This module provides the main spell checking functions including:
//! - Word lookup in spell dictionary trees
//! - Case folding for spell checking
//! - Compound word validation
//! - Region and flag checking
//!
//! The functions here integrate with the word tree traversal from `wordtree.rs`
//! and the spell file structures from `spellfile.rs`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int, c_void};

use crate::wordtree::{
    get_word_flags, traverse_tree, TreeSearchResult, WF_ALLCAP, WF_BANNED, WF_FIXCAP, WF_KEEPCAP,
    WF_NEEDCOMP, WF_NOCOMPAFT, WF_NOCOMPBEF, WF_ONECAP, WF_RARE, WF_REGION,
};
use crate::{IdxT, SpelltabHandle, REGION_ALL};

// =============================================================================
// Spell Check Result Types
// =============================================================================

/// Result of a spell check operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellResult {
    /// Word is spelled correctly
    Ok = 0,
    /// Word is rare (marked with WF_RARE)
    Rare = 1,
    /// Word is for a different region
    LocalBad = 2,
    /// Word is misspelled
    #[default]
    Bad = 3,
    /// Word is banned (marked as bad in .add file)
    Banned = 4,
}

/// Case type of a word
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaseType {
    /// All lowercase (e.g., "word")
    #[default]
    AllLower = 0,
    /// First letter uppercase (e.g., "Word")
    OneCap = 1,
    /// All uppercase (e.g., "WORD")
    AllCap = 2,
    /// Keep-case (mixed, e.g., "WoRd")
    KeepCap = 3,
}

/// Convert case type to word flags
impl CaseType {
    /// Convert to the corresponding WF_* flag value
    #[must_use]
    pub const fn to_flags(self) -> u32 {
        match self {
            Self::AllLower => 0,
            Self::OneCap => WF_ONECAP,
            Self::AllCap => WF_ALLCAP,
            Self::KeepCap => WF_KEEPCAP,
        }
    }
}

// =============================================================================
// Case Folding Functions
// =============================================================================

/// Fold a character to lowercase using the spell character table.
///
/// For ASCII characters, uses the spelltab fold table.
/// For multibyte characters, returns the character unchanged (to be handled by C).
///
/// # Arguments
/// * `c` - Character (as u32 codepoint)
/// * `spelltab` - Handle to the spell character table
///
/// # Returns
/// The lowercase version of the character
#[inline]
pub fn spell_fold_char(c: u32, spelltab: SpelltabHandle) -> u32 {
    if c < 256 && !spelltab.is_null() {
        let fold = spelltab.fold();
        if fold.is_null() {
            c
        } else {
            // SAFETY: spelltab.fold() returns a valid 256-entry array
            unsafe { u32::from(*fold.add(c as usize)) }
        }
    } else {
        c
    }
}

/// FFI wrapper for spell_fold_char
#[no_mangle]
pub extern "C" fn rs_spell_fold_char(c: u32, spelltab: SpelltabHandle) -> u32 {
    spell_fold_char(c, spelltab)
}

/// Check if a character is an uppercase character according to spell tables.
///
/// # Arguments
/// * `c` - Character (as u32 codepoint)
/// * `spelltab` - Handle to the spell character table
///
/// # Returns
/// True if the character is uppercase
#[inline]
pub fn spell_is_upper(c: u32, spelltab: SpelltabHandle) -> bool {
    if c < 256 && !spelltab.is_null() {
        let isu = spelltab.isu();
        if isu.is_null() {
            false
        } else {
            // SAFETY: spelltab.isu() returns a valid 256-entry array
            unsafe { *isu.add(c as usize) }
        }
    } else {
        false
    }
}

/// FFI wrapper for spell_is_upper
#[no_mangle]
pub extern "C" fn rs_spell_is_upper(c: u32, spelltab: SpelltabHandle) -> bool {
    spell_is_upper(c, spelltab)
}

/// Check if a character is a word character according to spell tables.
///
/// # Arguments
/// * `c` - Character (as u32 codepoint)
/// * `spelltab` - Handle to the spell character table
///
/// # Returns
/// True if the character is a word character
#[inline]
pub fn spell_is_word_char(c: u32, spelltab: SpelltabHandle) -> bool {
    if c < 256 && !spelltab.is_null() {
        let isw = spelltab.isw();
        if isw.is_null() {
            false
        } else {
            // SAFETY: spelltab.isw() returns a valid 256-entry array
            unsafe { *isw.add(c as usize) }
        }
    } else {
        // For multibyte characters, delegate to C's spell_mb_isword_class
        false
    }
}

/// FFI wrapper for spell_is_word_char
#[no_mangle]
pub extern "C" fn rs_spell_is_word_char(c: u32, spelltab: SpelltabHandle) -> bool {
    spell_is_word_char(c, spelltab)
}

// Greek sigma special case constants
const GREEK_CAPITAL_SIGMA: u32 = 0x03A3;
const GREEK_SMALL_SIGMA: u32 = 0x03C3;
const GREEK_SMALL_FINAL_SIGMA: u32 = 0x03C2;

// External C functions for Unicode handling
extern "C" {
    fn spell_tofold(c: c_int) -> c_int;
}

/// Decode a UTF-8 character from a byte slice.
///
/// Returns the codepoint and the number of bytes consumed.
fn utf8_decode(bytes: &[u8]) -> Option<(u32, usize)> {
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0];
    if first == 0 {
        return None;
    }

    if first < 0x80 {
        // ASCII
        return Some((u32::from(first), 1));
    }

    if first < 0xC0 {
        // Invalid start byte (continuation byte)
        return Some((u32::from(first), 1));
    }

    let (len, mask): (usize, u8) = if first < 0xE0 {
        (2, 0x1F)
    } else if first < 0xF0 {
        (3, 0x0F)
    } else if first < 0xF8 {
        (4, 0x07)
    } else {
        // Invalid
        return Some((u32::from(first), 1));
    };

    if bytes.len() < len {
        // Not enough bytes
        return Some((u32::from(first), 1));
    }

    let mut codepoint = u32::from(first & mask);
    for byte in bytes.iter().take(len).skip(1) {
        let cont = *byte;
        if cont & 0xC0 != 0x80 {
            // Invalid continuation byte
            return Some((u32::from(first), 1));
        }
        codepoint = (codepoint << 6) | u32::from(cont & 0x3F);
    }

    Some((codepoint, len))
}

/// Encode a Unicode codepoint as UTF-8 into a buffer.
///
/// Returns the number of bytes written.
fn utf8_encode(codepoint: u32, output: &mut [u8]) -> usize {
    if codepoint < 0x80 {
        if output.is_empty() {
            return 0;
        }
        output[0] = codepoint as u8;
        1
    } else if codepoint < 0x800 {
        if output.len() < 2 {
            return 0;
        }
        output[0] = (0xC0 | (codepoint >> 6)) as u8;
        output[1] = (0x80 | (codepoint & 0x3F)) as u8;
        2
    } else if codepoint < 0x10000 {
        if output.len() < 3 {
            return 0;
        }
        output[0] = (0xE0 | (codepoint >> 12)) as u8;
        output[1] = (0x80 | ((codepoint >> 6) & 0x3F)) as u8;
        output[2] = (0x80 | (codepoint & 0x3F)) as u8;
        3
    } else {
        if output.len() < 4 {
            return 0;
        }
        output[0] = (0xF0 | (codepoint >> 18)) as u8;
        output[1] = (0x80 | ((codepoint >> 12) & 0x3F)) as u8;
        output[2] = (0x80 | ((codepoint >> 6) & 0x3F)) as u8;
        output[3] = (0x80 | (codepoint & 0x3F)) as u8;
        4
    }
}

/// Check if position starts with a word character
fn is_word_char_at(bytes: &[u8], spelltab: SpelltabHandle) -> bool {
    if bytes.is_empty() || bytes[0] == 0 {
        return false;
    }

    let first = bytes[0];
    if first < 128 {
        // ASCII - check spell table
        spell_is_word_char(u32::from(first), spelltab)
    } else {
        // UTF-8 - decode and check if it's a Unicode letter
        if let Some((codepoint, _)) = utf8_decode(bytes) {
            // Check if it's a Unicode letter (alphabetic)
            // This is a simplified check - for full correctness we'd call C
            char::from_u32(codepoint).is_some_and(char::is_alphabetic)
        } else {
            false
        }
    }
}

/// Fold a word to lowercase for spell checking with full UTF-8 support.
///
/// Handles UTF-8 multi-byte characters and the Greek sigma special case:
/// - Greek capital sigma (Σ, 0x03A3) folds to small sigma (σ, 0x03C3)
/// - Except at end of word where it folds to final sigma (ς, 0x03C2)
///
/// # Arguments
/// * `word` - The word to fold (UTF-8 bytes)
/// * `output` - Output buffer for folded word
/// * `spelltab` - Handle to the spell character table
///
/// # Returns
/// Number of bytes written to output, or 0 on error
pub fn spell_casefold(word: &[u8], output: &mut [u8], spelltab: SpelltabHandle) -> usize {
    if output.is_empty() {
        return 0;
    }

    let mut out_idx = 0;
    let mut in_idx = 0;

    while in_idx < word.len() {
        // Decode next UTF-8 character
        let Some((codepoint, char_len)) = utf8_decode(&word[in_idx..]) else {
            break;
        };

        if codepoint == 0 {
            break;
        }

        // Check output space (max 4 bytes for UTF-8 + 1 for NUL)
        if out_idx + 5 > output.len() {
            break;
        }

        // Fold the character
        let folded = if codepoint == GREEK_CAPITAL_SIGMA || codepoint == GREEK_SMALL_FINAL_SIGMA {
            // Greek sigma special case
            // Check if this is the last character or followed by non-word char
            let next_idx = in_idx + char_len;
            let at_end = next_idx >= word.len()
                || word[next_idx] == 0
                || !is_word_char_at(&word[next_idx..], spelltab);
            if at_end {
                GREEK_SMALL_FINAL_SIGMA
            } else {
                GREEK_SMALL_SIGMA
            }
        } else if codepoint < 256 {
            // Use spell fold table for characters < 256
            spell_fold_char(codepoint, spelltab)
        } else {
            // For other Unicode, use Unicode lowercase (via extern C)
            unsafe { spell_tofold(codepoint as c_int) as u32 }
        };

        // Encode the folded character
        let written = utf8_encode(folded, &mut output[out_idx..]);
        if written == 0 {
            break;
        }

        out_idx += written;
        in_idx += char_len;
    }

    // NUL-terminate
    if out_idx < output.len() {
        output[out_idx] = 0;
    }

    out_idx
}

/// FFI wrapper for spell_casefold
///
/// # Safety
/// All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn rs_spell_casefold(
    word: *const u8,
    word_len: usize,
    output: *mut u8,
    output_len: usize,
    spelltab: SpelltabHandle,
) -> usize {
    if word.is_null() || output.is_null() {
        return 0;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    let output_slice = std::slice::from_raw_parts_mut(output, output_len);

    spell_casefold(word_slice, output_slice, spelltab)
}

// =============================================================================
// Case Type Detection
// =============================================================================

/// Determine the case type of a word.
///
/// # Arguments
/// * `word` - The word to analyze (UTF-8 bytes)
/// * `spelltab` - Handle to the spell character table
///
/// # Returns
/// The detected case type
pub fn get_case_type(word: &[u8], spelltab: SpelltabHandle) -> CaseType {
    if word.is_empty() || word[0] == 0 {
        return CaseType::AllLower;
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut first_upper = false;
    let mut first = true;

    for &c in word {
        if c == 0 {
            break;
        }

        // Skip non-ASCII for now (needs UTF-8 decoding)
        if c >= 128 {
            continue;
        }

        let is_upper = spell_is_upper(u32::from(c), spelltab);

        if first {
            first_upper = is_upper;
            first = false;
        }

        if is_upper {
            has_upper = true;
        } else if c.is_ascii_lowercase() {
            has_lower = true;
        }
    }

    if !has_upper {
        CaseType::AllLower
    } else if !has_lower {
        CaseType::AllCap
    } else if first_upper {
        CaseType::OneCap
    } else {
        CaseType::KeepCap
    }
}

/// FFI wrapper for get_case_type
///
/// # Safety
/// `word` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_get_case_type(
    word: *const u8,
    word_len: usize,
    spelltab: SpelltabHandle,
) -> CaseType {
    if word.is_null() {
        return CaseType::AllLower;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    get_case_type(word_slice, spelltab)
}

// =============================================================================
// Bad Word Case Type Detection
// =============================================================================

/// Mixed case flag - word has both upper and lower in middle
pub const WF_MIXCAP: u32 = 0x100;

/// Like get_case_type() but for a KEEPCAP word add ONECAP if the word starts
/// with a capital. So that make_case_word() can turn WOrd into Word.
/// Add ALLCAP for "WOrD".
///
/// This is used by the suggestion system to determine how to adjust the
/// case of suggested words.
///
/// # Arguments
/// * `word` - The word to analyze (UTF-8 bytes)
/// * `spelltab` - Handle to the spell character table
///
/// # Returns
/// Word flags indicating the case pattern (may include WF_KEEPCAP | WF_ONECAP, etc.)
pub fn badword_captype(word: &[u8], spelltab: SpelltabHandle) -> u32 {
    let flags = get_case_type(word, spelltab).to_flags();

    // If not KEEPCAP, return the basic flags
    if (flags & WF_KEEPCAP) == 0 {
        return flags;
    }

    // Count the number of UPPER and lower case letters
    let mut lower_count = 0u32;
    let mut upper_count = 0u32;
    let mut first_upper = false;
    let mut is_first = true;

    for &c in word {
        if c == 0 {
            break;
        }

        // Skip non-ASCII for now (needs UTF-8 decoding)
        if c >= 128 {
            continue;
        }

        let is_upper = spell_is_upper(u32::from(c), spelltab);

        if is_upper {
            upper_count += 1;
            if is_first {
                first_upper = true;
            }
        } else if c.is_ascii_lowercase() {
            lower_count += 1;
        }

        if c.is_ascii_alphabetic() {
            is_first = false;
        }
    }

    let mut result_flags = flags;

    // If there are more UPPER than lower case letters suggest an
    // ALLCAP word. Otherwise, if the first letter is UPPER then
    // suggest ONECAP. Exception: "ALl" most likely should be "All",
    // require three upper case letters.
    if upper_count > lower_count && upper_count > 2 {
        result_flags |= WF_ALLCAP;
    } else if first_upper {
        result_flags |= WF_ONECAP;
    }

    // maCARONI maCAroni - has mixed case in middle
    if upper_count >= 2 && lower_count >= 2 {
        result_flags |= WF_MIXCAP;
    }

    result_flags
}

/// FFI wrapper for badword_captype
///
/// # Safety
/// `word` must be a valid pointer to at least `word_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_badword_captype(
    word: *const u8,
    word_len: usize,
    spelltab: SpelltabHandle,
) -> u32 {
    if word.is_null() {
        return 0;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    badword_captype(word_slice, spelltab)
}

// =============================================================================
// Case Validity Checking
// =============================================================================

/// Check if the word case matches what the dictionary entry requires.
///
/// This implements the spell_valid_case() logic from C:
/// - ALLCAP word matches if tree has no FIXCAP (first branch)
/// - Otherwise, tree must not have ALLCAP/KEEPCAP, and either tree
///   doesn't have ONECAP or word has ONECAP (second branch)
///
/// Note: WF_ALLCAP == 0x04, WF_FIXCAP == 0x40, WF_KEEPCAP == 0x80
///
/// # Arguments
/// * `word_flags` - Case flags of the word being checked
/// * `tree_flags` - Flags from the dictionary tree entry
///
/// # Returns
/// True if the case is valid
#[inline]
#[must_use]
pub const fn spell_valid_case(word_flags: u32, tree_flags: u32) -> bool {
    // First branch: Word is ALLCAP and tree doesn't require FIXCAP
    // (wordflags == WF_ALLCAP && (treeflags & WF_FIXCAP) == 0)
    let first_branch = word_flags == WF_ALLCAP && (tree_flags & WF_FIXCAP) == 0;

    // Second branch: tree doesn't have ALLCAP/KEEPCAP, and either no ONECAP
    // requirement or word satisfies it
    // ((treeflags & (WF_ALLCAP | WF_KEEPCAP)) == 0
    //  && ((treeflags & WF_ONECAP) == 0 || (wordflags & WF_ONECAP) != 0))
    let second_branch = (tree_flags & (WF_ALLCAP | WF_KEEPCAP)) == 0
        && ((tree_flags & WF_ONECAP) == 0 || (word_flags & WF_ONECAP) != 0);

    first_branch || second_branch
}

/// FFI wrapper for spell_valid_case
#[no_mangle]
pub extern "C" fn rs_spell_valid_case_check(word_flags: u32, tree_flags: u32) -> bool {
    spell_valid_case(word_flags, tree_flags)
}

// =============================================================================
// Region Checking
// =============================================================================

/// Check if a word's region matches the required region.
///
/// # Arguments
/// * `word_region` - Region mask from the word flags
/// * `required_region` - Region mask required (REGION_ALL = any region)
///
/// # Returns
/// True if the region is valid
#[inline]
#[must_use]
pub const fn region_matches(word_region: u32, required_region: c_int) -> bool {
    // REGION_ALL matches any region
    if required_region == REGION_ALL {
        return true;
    }
    // Check if the required region bit is set
    ((word_region >> 16) & (required_region as u32)) != 0
}

/// FFI wrapper for region_matches
#[no_mangle]
pub extern "C" fn rs_region_matches(word_region: u32, required_region: c_int) -> bool {
    region_matches(word_region, required_region)
}

// =============================================================================
// Word Lookup Functions
// =============================================================================

/// Result of looking up a word in the dictionary
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WordLookupResult {
    /// The result status
    pub result: SpellResult,
    /// Length of the matched word (bytes)
    pub match_len: usize,
    /// Flags from the dictionary entry
    pub flags: u32,
    /// Index in the tree where the word was found
    pub tree_idx: usize,
}

/// Look up a word in a spell dictionary tree.
///
/// This is the core function for checking if a word exists in the dictionary.
/// It handles:
/// - Tree traversal
/// - Multiple flag/region combinations
/// - Case validation
/// - Region checking
///
/// # Arguments
/// * `byts` - The tree bytes array
/// * `idxs` - The tree indices array
/// * `word` - The word to look up
/// * `case_flags` - Case flags of the word (from get_case_type)
/// * `region` - Required region (REGION_ALL for any)
///
/// # Returns
/// A WordLookupResult indicating whether the word was found and with what flags
pub fn lookup_word(
    byts: &[u8],
    idxs: &[IdxT],
    word: &[u8],
    case_flags: u32,
    region: c_int,
) -> WordLookupResult {
    let mut result = WordLookupResult::default();

    // Traverse the tree
    match traverse_tree(byts, idxs, word, word.len()) {
        TreeSearchResult::Found { word_len, end_idx } => {
            // Check all flag/region combinations at this position
            check_word_endings(
                byts,
                idxs,
                end_idx,
                word_len,
                case_flags,
                region,
                &mut result,
            );
        }
        TreeSearchResult::MultipleEndings { endings } => {
            // Check each possible ending, prefer longest valid match
            for &(word_len, end_idx) in endings.iter().rev() {
                check_word_endings(
                    byts,
                    idxs,
                    end_idx,
                    word_len,
                    case_flags,
                    region,
                    &mut result,
                );
                if result.result == SpellResult::Ok {
                    break;
                }
            }
        }
        TreeSearchResult::NotFound | TreeSearchResult::Empty => {
            result.result = SpellResult::Bad;
        }
    }

    result
}

/// Check all flag/region combinations at a word ending position.
fn check_word_endings(
    byts: &[u8],
    idxs: &[IdxT],
    end_idx: usize,
    word_len: usize,
    case_flags: u32,
    region: c_int,
    result: &mut WordLookupResult,
) {
    let mut idx = end_idx;

    // Iterate through consecutive zeros (multiple flag combinations)
    while idx < byts.len() && byts[idx] == 0 {
        if idx >= idxs.len() {
            break;
        }

        let flags = get_word_flags(idxs, idx);

        // Check if this is a banned word
        if (flags & WF_BANNED) != 0 {
            result.result = SpellResult::Banned;
            result.match_len = word_len;
            result.flags = flags;
            result.tree_idx = idx;
            return;
        }

        // Check case validity
        if !spell_valid_case(case_flags, flags) {
            idx += 1;
            continue;
        }

        // Check region
        let word_region = if (flags & WF_REGION) != 0 {
            flags
        } else {
            REGION_ALL as u32
        };

        if !region_matches(word_region, region) {
            // Region doesn't match, but word exists
            if result.result == SpellResult::Bad {
                result.result = SpellResult::LocalBad;
                result.match_len = word_len;
                result.flags = flags;
                result.tree_idx = idx;
            }
            idx += 1;
            continue;
        }

        // Word matches!
        result.match_len = word_len;
        result.flags = flags;
        result.tree_idx = idx;

        if (flags & WF_RARE) != 0 {
            result.result = SpellResult::Rare;
        } else {
            result.result = SpellResult::Ok;
            return; // Found a good match, done
        }

        idx += 1;
    }
}

/// FFI wrapper for lookup_word
///
/// # Safety
/// All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn rs_lookup_word(
    byts: *const u8,
    idxs: *const IdxT,
    tree_len: usize,
    word: *const u8,
    word_len: usize,
    case_flags: u32,
    region: c_int,
    result_out: *mut WordLookupResult,
) -> c_int {
    if byts.is_null() || idxs.is_null() || word.is_null() || result_out.is_null() {
        return -1;
    }

    let byts_slice = std::slice::from_raw_parts(byts, tree_len);
    let idxs_slice = std::slice::from_raw_parts(idxs, tree_len);
    let word_slice = std::slice::from_raw_parts(word, word_len);

    *result_out = lookup_word(byts_slice, idxs_slice, word_slice, case_flags, region);

    result_out.as_ref().map_or(-1, |r| r.result as c_int)
}

// =============================================================================
// Compound Word Checking
// =============================================================================

/// Flags for compound word checking
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompoundCheckFlags {
    /// Maximum number of compound parts
    pub max_parts: u8,
    /// Minimum length for compound parts
    pub min_len: u8,
    /// Maximum syllables in compound
    pub max_syl: u8,
    /// Allow compounding at start of word
    pub allow_start: bool,
    /// Allow compounding at end of word
    pub allow_end: bool,
}

/// Check if word flags allow it to be part of a compound.
///
/// # Arguments
/// * `flags` - Word flags from dictionary
/// * `at_start` - True if this is the first part of the compound
/// * `at_end` - True if this is the last part of the compound
///
/// # Returns
/// True if the word can be used in this compound position
#[must_use]
pub const fn can_compound(flags: u32, at_start: bool, at_end: bool) -> bool {
    // Check for NEEDCOMP - word can only be used in compounds
    // This is allowed regardless of position

    // Check for NOCOMPBEF - word cannot be followed by another word
    if at_start && (flags & WF_NOCOMPBEF) != 0 {
        return false;
    }

    // Check for NOCOMPAFT - word cannot follow another word
    if !at_start && (flags & WF_NOCOMPAFT) != 0 {
        return false;
    }

    // Check that word has a compound flag (stored in high byte)
    // If no compound flag at all, cannot be in compound
    if (flags >> 24) == 0 {
        return (flags & WF_NEEDCOMP) != 0;
    }

    // Word has compound flag, allow in any position
    // (at_end check is satisfied, at_start/middle positions also OK)
    let _ = at_end; // Acknowledge parameter to avoid unused warning
    true
}

/// FFI wrapper for can_compound
#[no_mangle]
pub extern "C" fn rs_can_compound(flags: u32, at_start: bool, at_end: bool) -> bool {
    can_compound(flags, at_start, at_end)
}

/// Check if a compound flag byte is allowed.
///
/// # Arguments
/// * `allowed_flags` - Array of allowed compound flag bytes (NUL-terminated)
/// * `flag_byte` - The compound flag byte to check
///
/// # Returns
/// True if the flag is allowed
#[must_use]
pub fn compound_flag_in_set(allowed_flags: &[u8], flag_byte: u8) -> bool {
    if flag_byte == 0 {
        return false;
    }

    for &c in allowed_flags {
        if c == 0 {
            break;
        }
        if c == flag_byte {
            return true;
        }
    }

    false
}

/// FFI wrapper for compound_flag_in_set
///
/// # Safety
/// `allowed_flags` must be a valid NUL-terminated array
#[no_mangle]
pub unsafe extern "C" fn rs_compound_flag_in_set(
    allowed_flags: *const u8,
    flags_len: usize,
    flag_byte: u8,
) -> bool {
    if allowed_flags.is_null() {
        return false;
    }

    let flags_slice = std::slice::from_raw_parts(allowed_flags, flags_len);
    compound_flag_in_set(flags_slice, flag_byte)
}

// =============================================================================
// Phase 150: Additional Compound Word FFI Exports
// =============================================================================

/// Create a new CompoundCheckFlags with defaults.
#[no_mangle]
pub extern "C" fn rs_compound_check_flags_default() -> CompoundCheckFlags {
    CompoundCheckFlags::default()
}

/// Create CompoundCheckFlags with specified values.
#[no_mangle]
pub extern "C" fn rs_compound_check_flags_new(
    max_parts: u8,
    min_len: u8,
    max_syl: u8,
    allow_start: bool,
    allow_end: bool,
) -> CompoundCheckFlags {
    CompoundCheckFlags {
        max_parts,
        min_len,
        max_syl,
        allow_start,
        allow_end,
    }
}

/// Get max_parts from CompoundCheckFlags.
///
/// # Safety
/// `flags` must be a valid pointer to a CompoundCheckFlags.
#[no_mangle]
pub unsafe extern "C" fn rs_compound_check_flags_max_parts(flags: *const CompoundCheckFlags) -> u8 {
    if flags.is_null() {
        return 0;
    }
    (*flags).max_parts
}

/// Get min_len from CompoundCheckFlags.
///
/// # Safety
/// `flags` must be a valid pointer to a CompoundCheckFlags.
#[no_mangle]
pub unsafe extern "C" fn rs_compound_check_flags_min_len(flags: *const CompoundCheckFlags) -> u8 {
    if flags.is_null() {
        return 0;
    }
    (*flags).min_len
}

/// Get max_syl from CompoundCheckFlags.
///
/// # Safety
/// `flags` must be a valid pointer to a CompoundCheckFlags.
#[no_mangle]
pub unsafe extern "C" fn rs_compound_check_flags_max_syl(flags: *const CompoundCheckFlags) -> u8 {
    if flags.is_null() {
        return 0;
    }
    (*flags).max_syl
}

/// Check if compound is allowed at start.
///
/// # Safety
/// `flags` must be a valid pointer to a CompoundCheckFlags.
#[no_mangle]
pub unsafe extern "C" fn rs_compound_check_flags_allow_start(
    flags: *const CompoundCheckFlags,
) -> bool {
    if flags.is_null() {
        return false;
    }
    (*flags).allow_start
}

/// Check if compound is allowed at end.
///
/// # Safety
/// `flags` must be a valid pointer to a CompoundCheckFlags.
#[no_mangle]
pub unsafe extern "C" fn rs_compound_check_flags_allow_end(
    flags: *const CompoundCheckFlags,
) -> bool {
    if flags.is_null() {
        return false;
    }
    (*flags).allow_end
}

// Note: rs_word_needs_compound, rs_word_no_compound_before, rs_word_no_compound_after,
// rs_word_compound_flag, rs_word_has_compound_flag are exported from wordtree.rs

/// Check if word can be the first part of a compound.
#[no_mangle]
pub extern "C" fn rs_can_compound_start(flags: u32) -> bool {
    can_compound(flags, true, false)
}

/// Check if word can be a middle part of a compound.
#[no_mangle]
pub extern "C" fn rs_can_compound_middle(flags: u32) -> bool {
    can_compound(flags, false, false)
}

/// Check if word can be the last part of a compound.
#[no_mangle]
pub extern "C" fn rs_can_compound_end(flags: u32) -> bool {
    can_compound(flags, false, true)
}

/// Validate compound word part count.
#[no_mangle]
pub extern "C" fn rs_compound_valid_parts(part_count: u8, max_parts: u8) -> bool {
    max_parts == 0 || part_count <= max_parts
}

/// Validate compound word part length.
#[no_mangle]
pub extern "C" fn rs_compound_valid_len(word_len: usize, min_len: u8) -> bool {
    min_len == 0 || word_len >= min_len as usize
}

// Note: WF_* flag constants are exported from wordtree.rs

// =============================================================================
// Phase 324: Compound Rule Matching
// =============================================================================

/// Match compound flags against compound rules.
///
/// This checks if the compound flags collected so far could possibly match
/// any compound rule. It's used to stop trying compounds early when
/// there's no matching rule.
///
/// Rules are in the format: "abc/def/[gh]ij/..." where:
/// - Each letter matches a flag byte directly
/// - [...] matches any of the enclosed flags
/// - Rules are separated by /
///
/// # Arguments
/// * `comprules` - The compound rules string (NUL-terminated, rules separated by /)
/// * `compflags` - The compound flags collected so far (NUL-terminated)
///
/// # Returns
/// true if the flags match the start of any rule
pub fn match_compoundrule(comprules: &[u8], compflags: &[u8]) -> bool {
    if comprules.is_empty() || comprules[0] == 0 {
        return false;
    }

    let mut rule_idx = 0;

    // Loop over all the COMPOUNDRULE entries
    while rule_idx < comprules.len() && comprules[rule_idx] != 0 {
        // Loop over the flags in the compound word we have made,
        // match them against the current rule entry
        let mut flag_idx = 0;
        let mut p = rule_idx;

        'flags: loop {
            let c = if flag_idx < compflags.len() {
                compflags[flag_idx]
            } else {
                0
            };

            if c == 0 {
                // Found a rule that matches for the flags we have so far
                return true;
            }

            if p >= comprules.len() || comprules[p] == 0 || comprules[p] == b'/' {
                // End of rule, it's too short
                break 'flags;
            }

            if comprules[p] == b'[' {
                // Compare against all the flags in []
                p += 1;
                let mut bracket_match = false;

                while p < comprules.len() && comprules[p] != b']' && comprules[p] != 0 {
                    if comprules[p] == c {
                        bracket_match = true;
                    }
                    p += 1;
                }

                if !bracket_match {
                    break 'flags;
                }

                // Skip past the ]
                if p < comprules.len() && comprules[p] == b']' {
                    p += 1;
                }
            } else if comprules[p] != c {
                // Flag of word doesn't match flag in pattern
                break 'flags;
            } else {
                p += 1;
            }

            flag_idx += 1;
        }

        // Skip to the next "/", where the next pattern starts
        while rule_idx < comprules.len() && comprules[rule_idx] != 0 && comprules[rule_idx] != b'/'
        {
            rule_idx += 1;
        }

        // Skip the "/" itself
        if rule_idx < comprules.len() && comprules[rule_idx] == b'/' {
            rule_idx += 1;
        }
    }

    // Checked all the rules and none of them match the flags
    false
}

/// FFI wrapper for match_compoundrule
///
/// # Safety
/// `comprules` and `compflags` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_match_compoundrule(
    comprules: *const u8,
    comprules_len: usize,
    compflags: *const u8,
    compflags_len: usize,
) -> bool {
    if comprules.is_null() || compflags.is_null() {
        return false;
    }

    let comprules_slice = std::slice::from_raw_parts(comprules, comprules_len);
    let compflags_slice = std::slice::from_raw_parts(compflags, compflags_len);

    match_compoundrule(comprules_slice, compflags_slice)
}

// =============================================================================
// Phase 321: Find Word - Core Spell Checking
// =============================================================================

/// Find mode for word checking.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindWordMode {
    /// Check case-folded word in fold-case tree
    FoldWord = 0,
    /// Check word in keep-case tree
    KeepWord = 1,
    /// Check for word after a prefix
    Prefix = 2,
    /// Check for compound word continuation (fold-case)
    Compound = 3,
    /// Check for compound word continuation (keep-case)
    KeepCompound = 4,
}

/// Result of find_word operation
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FindWordResult {
    /// Result code (SP_OK, SP_BAD, etc)
    pub result: c_int,
    /// Length of matched word in bytes (in original word)
    pub word_len: c_int,
    /// Flags from the dictionary entry
    pub flags: u32,
}

/// Constants matching C SP_* values
pub const SP_BANNED: c_int = -1;
pub const SP_RARE: c_int = 0;
pub const SP_OK: c_int = 1;
pub const SP_LOCAL: c_int = 2;
pub const SP_BAD: c_int = 3;

/// FFI export for SP_BANNED constant
#[no_mangle]
pub extern "C" fn rs_sp_banned() -> c_int {
    SP_BANNED
}

/// FFI export for SP_RARE constant
#[no_mangle]
pub extern "C" fn rs_sp_rare() -> c_int {
    SP_RARE
}

/// FFI export for SP_OK constant
#[no_mangle]
pub extern "C" fn rs_sp_ok() -> c_int {
    SP_OK
}

/// FFI export for SP_LOCAL constant
#[no_mangle]
pub extern "C" fn rs_sp_local() -> c_int {
    SP_LOCAL
}

/// FFI export for SP_BAD constant
#[no_mangle]
pub extern "C" fn rs_sp_bad() -> c_int {
    SP_BAD
}

/// Core word finding function implementing the tree traversal from find_word().
///
/// This function traverses the spell dictionary tree to find if a word matches.
/// It handles:
/// - Binary search through sorted sibling bytes
/// - Multiple possible word endings
/// - Flag/region validation
/// - Case validation
/// - Space folding (one space can match multiple spaces)
///
/// # Arguments
/// * `byts` - The tree bytes array
/// * `idxs` - The tree indices array
/// * `word` - The word to look up (case-folded for FOLDWORD mode, original otherwise)
/// * `word_len` - Available bytes in word
/// * `mode` - The find mode (FOLDWORD, KEEPWORD, PREFIX, COMPOUND, KEEPCOMPOUND)
/// * `start_offset` - Starting offset in word (for PREFIX/COMPOUND modes)
/// * `capflags` - Capitalization flags of original word (WF_ONECAP, WF_ALLCAP, etc)
/// * `region` - Required region bitmask (REGION_ALL for any)
///
/// # Returns
/// A FindWordResult with the best match found
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn find_word_in_tree(
    byts: &[u8],
    idxs: &[crate::IdxT],
    word: &[u8],
    word_len: usize,
    mode: FindWordMode,
    start_offset: usize,
    capflags: u32,
    region: c_int,
) -> FindWordResult {
    let mut result = FindWordResult {
        result: SP_BAD,
        word_len: 0,
        flags: 0,
    };

    if byts.is_empty() || idxs.is_empty() {
        return result;
    }

    // For keep-case modes, we have unlimited folded length
    let flen_available = if mode == FindWordMode::KeepWord || mode == FindWordMode::KeepCompound {
        9999usize
    } else {
        word_len.saturating_sub(start_offset)
    };

    let mut arridx: usize = 0;
    let mut wlen: usize = start_offset;
    let mut flen = flen_available;

    // Arrays to store possible word endings (max MAXWLEN entries)
    let mut endlen = [0usize; 254];
    let mut endidx = [0usize; 254];
    let mut endidxcnt: usize = 0;

    // Traverse the tree
    loop {
        if arridx >= byts.len() {
            break;
        }

        let len = byts[arridx] as usize;
        arridx += 1;

        if arridx >= byts.len() {
            break;
        }

        // If the first possible byte is a zero, the word could end here
        if byts[arridx] == 0 {
            if endidxcnt >= 254 {
                // Corrupted spell file protection
                break;
            }
            endlen[endidxcnt] = wlen;
            endidx[endidxcnt] = arridx;
            endidxcnt += 1;
            arridx += 1;

            let mut remaining = len.saturating_sub(1);

            // Skip over consecutive zeros (multiple flag/region combinations)
            while remaining > 0 && arridx < byts.len() && byts[arridx] == 0 {
                arridx += 1;
                remaining -= 1;
            }

            if remaining == 0 {
                break; // No children, word must end here
            }
        }

        // Stop at end of word
        if wlen >= word.len() {
            break;
        }

        // Get the byte to search for
        let mut c = word[wlen];
        if c == b'\t' {
            c = b' ';
        }

        // Calculate search range
        let first_sibling = arridx;
        let mut sibling_count = len;

        // Account for zeros we skipped
        if endidxcnt > 0 && endidx[endidxcnt - 1] == arridx - 1 {
            // We just recorded an ending, adjust count
            let zeros_at_end = 1; // At least one zero
            sibling_count = sibling_count.saturating_sub(zeros_at_end);
        }

        if sibling_count == 0 {
            break;
        }

        let last_sibling = first_sibling + sibling_count - 1;
        if last_sibling >= byts.len() || last_sibling >= idxs.len() {
            break;
        }

        // Binary search for the byte
        let found = crate::wordtree::tree_binary_search(byts, first_sibling, last_sibling, c);

        match found {
            Some(found_idx) => {
                if found_idx >= idxs.len() {
                    break;
                }
                arridx = idxs[found_idx] as usize;
                wlen += 1;
                flen = flen.saturating_sub(1);

                // Handle space folding: one space matches multiple spaces
                if c == b' ' {
                    while wlen < word.len() && flen > 0 {
                        let next_c = word[wlen];
                        if next_c != b' ' && next_c != b'\t' {
                            break;
                        }
                        wlen += 1;
                        flen = flen.saturating_sub(1);
                    }
                }
            }
            None => break,
        }
    }

    // Check all possible endings, starting from longest
    while endidxcnt > 0 {
        endidxcnt -= 1;
        let end_arridx = endidx[endidxcnt];
        let end_wlen = endlen[endidxcnt];

        // Verify we're at a valid ending position
        if end_arridx == 0 || end_arridx >= byts.len() {
            continue;
        }

        // Check the flags at this ending
        let prev_idx = end_arridx.saturating_sub(1);
        if prev_idx >= byts.len() {
            continue;
        }

        let mut flags_len = byts[prev_idx] as usize;
        let mut idx = end_arridx;

        // Iterate through all flag combinations at this ending
        while flags_len > 0 && idx < byts.len() && byts[idx] == 0 {
            if idx >= idxs.len() {
                break;
            }

            let flags = idxs[idx] as u32;
            idx += 1;
            flags_len -= 1;

            // For fold-case mode, validate case
            if mode == FindWordMode::FoldWord {
                // KEEPCAP words must be in keep-case tree
                if capflags == WF_KEEPCAP {
                    continue;
                }
                // Validate case flags
                if !spell_valid_case(capflags, flags) {
                    continue;
                }
            }

            // Check if word is banned
            if (flags & WF_BANNED) != 0 {
                if result.result > SP_BANNED {
                    result.result = SP_BANNED;
                    result.word_len = end_wlen as c_int;
                    result.flags = flags;
                }
                continue;
            }

            // Check region
            let res = if (flags & WF_REGION) != 0 {
                let word_region = (flags >> 16) & 0xff;
                if region == REGION_ALL || (word_region & region as u32) != 0 {
                    if (flags & WF_RARE) != 0 {
                        SP_RARE
                    } else {
                        SP_OK
                    }
                } else {
                    SP_LOCAL
                }
            } else if (flags & WF_RARE) != 0 {
                SP_RARE
            } else {
                SP_OK
            };

            // Update result if better than current
            if res < result.result || (res == result.result && end_wlen as c_int > result.word_len)
            {
                result.result = res;
                result.word_len = end_wlen as c_int;
                result.flags = flags;

                if res == SP_OK {
                    return result;
                }
            }
        }
    }

    result
}

/// FFI wrapper for find_word_in_tree
///
/// # Safety
/// All pointers must be valid, arrays must have at least `tree_len` elements.
#[no_mangle]
pub unsafe extern "C" fn rs_find_word_in_tree(
    byts: *const u8,
    idxs: *const crate::IdxT,
    tree_len: usize,
    word: *const u8,
    word_len: usize,
    mode: FindWordMode,
    start_offset: usize,
    capflags: u32,
    region: c_int,
    result_out: *mut FindWordResult,
) -> c_int {
    if byts.is_null() || idxs.is_null() || word.is_null() || result_out.is_null() {
        return SP_BAD;
    }

    let byts_slice = std::slice::from_raw_parts(byts, tree_len);
    let idxs_slice = std::slice::from_raw_parts(idxs, tree_len);
    let word_slice = std::slice::from_raw_parts(word, word_len);

    let result = find_word_in_tree(
        byts_slice,
        idxs_slice,
        word_slice,
        word_len,
        mode,
        start_offset,
        capflags,
        region,
    );

    *result_out = result;
    result.result
}

/// Check word in both fold-case and keep-case trees, returning the best result.
///
/// This is a higher-level function that combines checking both trees.
///
/// # Arguments
/// * `fbyts` - Fold-case tree bytes
/// * `fidxs` - Fold-case tree indices
/// * `flen` - Fold-case tree length
/// * `kbyts` - Keep-case tree bytes (may be empty)
/// * `kidxs` - Keep-case tree indices (may be empty)
/// * `klen` - Keep-case tree length
/// * `fword` - Case-folded word
/// * `fword_len` - Length of case-folded word
/// * `word` - Original word (for keep-case checking)
/// * `word_len` - Length of original word
/// * `capflags` - Capitalization flags
/// * `region` - Required region
///
/// # Returns
/// The best FindWordResult from either tree
#[allow(clippy::too_many_arguments)]
pub fn check_word_both_trees(
    fbyts: &[u8],
    fidxs: &[crate::IdxT],
    kbyts: &[u8],
    kidxs: &[crate::IdxT],
    fword: &[u8],
    word: &[u8],
    capflags: u32,
    region: c_int,
) -> FindWordResult {
    let mut best = FindWordResult {
        result: SP_BAD,
        word_len: 0,
        flags: 0,
    };

    // Check fold-case tree
    if !fbyts.is_empty() && !fidxs.is_empty() {
        let fold_result = find_word_in_tree(
            fbyts,
            fidxs,
            fword,
            fword.len(),
            FindWordMode::FoldWord,
            0,
            capflags,
            region,
        );
        if fold_result.result < best.result
            || (fold_result.result == best.result && fold_result.word_len > best.word_len)
        {
            best = fold_result;
        }
    }

    // Check keep-case tree
    if !kbyts.is_empty() && !kidxs.is_empty() {
        let keep_result = find_word_in_tree(
            kbyts,
            kidxs,
            word,
            word.len(),
            FindWordMode::KeepWord,
            0,
            0, // capflags not used for keep-case
            region,
        );
        if keep_result.result < best.result
            || (keep_result.result == best.result && keep_result.word_len > best.word_len)
        {
            best = keep_result;
        }
    }

    best
}

/// FFI wrapper for check_word_both_trees
///
/// # Safety
/// All pointers must be valid
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_check_word_both_trees(
    fbyts: *const u8,
    fidxs: *const crate::IdxT,
    flen: usize,
    kbyts: *const u8,
    kidxs: *const crate::IdxT,
    klen: usize,
    fword: *const u8,
    fword_len: usize,
    word: *const u8,
    word_len: usize,
    capflags: u32,
    region: c_int,
    result_out: *mut FindWordResult,
) -> c_int {
    if result_out.is_null() {
        return SP_BAD;
    }

    // Handle null tree pointers gracefully
    let fbyts_slice = if fbyts.is_null() || flen == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(fbyts, flen)
    };
    let fidxs_slice = if fidxs.is_null() || flen == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(fidxs, flen)
    };
    let kbyts_slice = if kbyts.is_null() || klen == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(kbyts, klen)
    };
    let kidxs_slice = if kidxs.is_null() || klen == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(kidxs, klen)
    };

    let fword_slice = if fword.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(fword, fword_len)
    };
    let word_slice = if word.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(word, word_len)
    };

    let result = check_word_both_trees(
        fbyts_slice,
        fidxs_slice,
        kbyts_slice,
        kidxs_slice,
        fword_slice,
        word_slice,
        capflags,
        region,
    );

    *result_out = result;
    result.result
}

// =============================================================================
// Phase 1: Simple utility functions migrated from spell.c
// =============================================================================

extern "C" {
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn check_need_cap(wp: *mut c_void, lnum: i32, col: c_int) -> bool;
    fn spell_suggest_list(
        ga: *mut GArrayRaw,
        word: *const c_char,
        maxcount: c_int,
        need_cap: bool,
        interactive: bool,
    );
    fn nvim_for_all_windows_in_curtab(
        callback: unsafe extern "C" fn(*mut c_void, *mut c_void),
        ud: *mut c_void,
    );
    fn nvim_parse_spelllang(win: *mut c_void) -> *const c_char;
    fn nvim_win_get_buffer(wp: *const c_void) -> *mut c_void;
    fn nvim_win_get_p_spell(wp: *const c_void) -> c_int;
    fn nvim_win_get_cursor_lnum(wp: *const c_void) -> i32;
    fn get_cursor_line_ptr() -> *const c_char;
    #[link_name = "curwin"]
    static curwin_p1: *mut c_void;
    #[link_name = "curbuf"]
    static curbuf_p1: *mut c_void;
    // For synblock-based accessors
    fn nvim_synblock_get_b_p_spc(block: *const c_void) -> *const c_char;
    fn nvim_synblock_get_b_cap_prog(block: *const c_void) -> *mut c_void;
    fn nvim_synblock_set_b_cap_prog(block: *mut c_void, prog: *mut c_void);
    fn vim_regcomp(s: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regfree(prog: *mut c_void);
    fn concat_str(a: *const c_char, b: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    /// e_invarg is a const char[] in C; take its address to get *const c_char
    #[link_name = "e_invarg"]
    static e_invarg_arr: [c_char; 1];
}

use crate::GArrayRaw;

/// Case-fold "str[len]" into "buf[buflen]". The result is NUL terminated.
///
/// Uses the character definitions from the .spl file.
/// When using a multi-byte 'encoding' the length may change.
/// Returns FAIL (non-zero) when something went wrong, OK (0) on success.
/// This matches the C spell_casefold() signature exactly.
///
/// # Safety
/// All pointers must be valid. wp must be a valid win_T pointer.
#[export_name = "spell_casefold"]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_spell_casefold_c_compat(
    wp: *const c_void,
    str_: *const c_char,
    len: c_int,
    buf: *mut c_char,
    buflen: c_int,
) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = -1;
    const NUL: u8 = 0;

    if len >= buflen {
        if buflen > 0 {
            *(buf as *mut u8) = NUL;
        }
        return FAIL;
    }

    let mut outi: c_int = 0;
    let mut p = str_;
    let end = str_.add(len as usize);

    while p < end {
        if outi + 4 > buflen {
            // MB_MAXBYTES == 4
            *(buf.add(outi as usize) as *mut u8) = NUL;
            return FAIL;
        }
        let mut c = mb_cptr2char_adv(std::ptr::addr_of_mut!(p));

        // Exception: Greek capital sigma 0x03A3 folds to 0x03C3, except
        // when it is the last character in a word, then it folds to 0x03C2.
        if c == 0x03a3 || c == 0x03c2 {
            if p >= end || !crate::rs_spell_iswordp(p, wp) {
                c = 0x03c2;
            } else {
                c = 0x03c3;
            }
        } else {
            c = crate::spell_tofold(c);
        }

        outi += utf_char2bytes(c, buf.add(outi as usize));
    }
    *(buf.add(outi as usize) as *mut u8) = NUL;

    OK
}

/// Move "p" to the end of word "start".
/// Uses the spell-checking word characters.
///
/// # Safety
/// `start` and `win` must be valid pointers.
#[export_name = "spell_to_word_end"]
pub unsafe extern "C" fn rs_spell_to_word_end(start: *mut c_char, win: *mut c_void) -> *mut c_char {
    let mut p = start;
    while *(p as *const u8) != 0 && crate::rs_spell_iswordp(p, win) {
        let step = utfc_ptr2len(p).max(1) as usize;
        p = p.add(step);
    }
    p
}

/// For Insert mode completion CTRL-X s:
/// Find start of the word in front of column "startcol".
/// Returns the column number of the word.
///
/// # Safety
/// Accesses curwin global and cursor position.
#[export_name = "spell_word_start"]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_spell_word_start(startcol: c_int) -> c_int {
    if crate::rs_no_spell_checking(curwin_p1) {
        return startcol;
    }

    let line = get_cursor_line_ptr();
    let mut p = line.add(startcol as usize);

    // Find a word character before "startcol".
    while p > line {
        // MB_PTR_BACK: p -= utf_head_off(line, p - 1) + 1
        let back = utf_head_off(line, p.sub(1)) + 1;
        p = p.sub(back as usize);
        if crate::rs_spell_iswordp_nmw(p, curwin_p1) {
            break;
        }
    }

    let mut col: c_int = 0;

    // Go back to start of the word.
    while p > line {
        col = p.offset_from(line) as c_int;
        // MB_PTR_BACK
        let back = utf_head_off(line, p.sub(1)) + 1;
        p = p.sub(back as usize);
        if !crate::rs_spell_iswordp(p, curwin_p1) {
            break;
        }
        col = 0;
    }

    col
}

// Global variable for spell_expand_check_cap / expand_spelling.
// This is a static variable that mirrors the C `spell_expand_need_cap`.
static mut SPELL_EXPAND_NEED_CAP: bool = false;

/// Check if the word at the given column needs to start with a capital.
/// Must be called before expand_spelling().
///
/// # Safety
/// Accesses curwin global.
#[export_name = "spell_expand_check_cap"]
pub unsafe extern "C" fn rs_spell_expand_check_cap(col: c_int) {
    SPELL_EXPAND_NEED_CAP = check_need_cap(curwin_p1, nvim_win_get_cursor_lnum(curwin_p1), col);
}

/// Get list of spelling suggestions for Insert mode completion CTRL-X s.
/// Returns the number of matches. The matches are in matchp[], array of
/// allocated strings.
///
/// # Safety
/// All pointers must be valid.
#[export_name = "expand_spelling"]
pub unsafe extern "C" fn rs_expand_spelling(
    _lnum: i32,
    pat: *const c_char,
    matchp: *mut *mut *mut c_char,
) -> c_int {
    let mut ga = GArrayRaw {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    spell_suggest_list(
        std::ptr::addr_of_mut!(ga),
        pat,
        100,
        SPELL_EXPAND_NEED_CAP,
        true,
    );
    *matchp = ga.ga_data as *mut *mut c_char;
    ga.ga_len
}

/// Callback for did_set_spell_option window iteration.
struct DidSetSpellState {
    errmsg: *const c_char,
    done: bool,
}

unsafe extern "C" fn did_set_spell_option_cb(wp: *mut c_void, ud: *mut c_void) {
    let state = ud as *mut DidSetSpellState;
    if (*state).done {
        return; // already processed a matching window
    }
    if nvim_win_get_buffer(wp) == curbuf_p1 && nvim_win_get_p_spell(wp) != 0 {
        (*state).errmsg = nvim_parse_spelllang(wp);
        (*state).done = true;
    }
}

/// Called when 'spell' or 'spelllang' is set.
/// Parse the new 'spelllang' and return an error message if failed.
///
/// # Safety
/// Accesses window globals.
#[export_name = "did_set_spell_option"]
pub unsafe extern "C" fn rs_did_set_spell_option() -> *const c_char {
    let mut state = DidSetSpellState {
        errmsg: std::ptr::null(),
        done: false,
    };
    nvim_for_all_windows_in_curtab(
        did_set_spell_option_cb,
        std::ptr::addr_of_mut!(state).cast(),
    );
    state.errmsg
}

/// Set `synblock->b_cap_prog` to the regexp program for 'spellcapcheck'.
/// Returns error message when failed, NULL when OK.
///
/// # Safety
/// synblock must be a valid synblock_T pointer.
#[export_name = "compile_cap_prog"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_compile_cap_prog(synblock: *mut c_void) -> *const c_char {
    const RE_MAGIC: c_int = 1; // matches C RE_MAGIC

    let rp = nvim_synblock_get_b_cap_prog(synblock);
    let spc = nvim_synblock_get_b_p_spc(synblock);

    if spc.is_null() || *(spc as *const u8) == 0 {
        nvim_synblock_set_b_cap_prog(synblock, std::ptr::null_mut());
    } else {
        // Prepend "^" so we only match at one column
        let re = concat_str(c"^".as_ptr(), spc);
        let new_prog = vim_regcomp(re, RE_MAGIC);
        xfree(re as *mut c_void);
        if new_prog.is_null() {
            nvim_synblock_set_b_cap_prog(synblock, rp);
            return e_invarg_arr.as_ptr();
        }
        nvim_synblock_set_b_cap_prog(synblock, new_prog);
    }

    vim_regfree(rp);
    std::ptr::null()
}

// =============================================================================
// Phase 2: Compound word functions migrated from spell.c
// =============================================================================

extern "C" {
    fn vim_regexec_prog(
        prog: *mut *mut c_void,
        ignore_case: bool,
        line: *const c_char,
        col: i32,
    ) -> bool;
    fn strlen(s: *const c_char) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

/// Check if "ptr" matches a compound pattern.
///
/// Returns true if the word boundary at ptr[wlen] matches one of the compound
/// patterns in gap (sl_comppat). Checks both first and second halves of each
/// pattern pair.
///
/// # Safety
/// ptr must be valid, gap must be a valid garray_T with char** data.
#[export_name = "match_checkcompoundpattern"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_match_checkcompoundpattern(
    ptr: *const c_char,
    wlen: c_int,
    gap: *const GArrayRaw,
) -> bool {
    let ga_len = (*gap).ga_len;
    let mut i = 0;
    while i + 1 < ga_len {
        let data = (*gap).ga_data as *const *const c_char;
        // Second part: must match at start of following word
        let p2 = *data.add(i as usize + 1);
        let p2_len = strlen(p2);
        if strncmp(ptr.add(wlen as usize), p2, p2_len) == 0 {
            // First part: must match at end of previous word
            let p1 = *data.add(i as usize);
            let len = strlen(p1) as c_int;
            if len <= wlen && strncmp(ptr.add((wlen - len) as usize), p1, len as usize) == 0 {
                return true;
            }
        }
        i += 2;
    }
    false
}

/// Return true if "flags" is a valid sequence of compound flags and "word"
/// does not have too many syllables.
///
/// # Safety
/// slang, word, flags must be valid pointers.
#[export_name = "can_compound"]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_can_compound_c_compat(
    slang: *mut crate::SlangRaw,
    word: *const c_char,
    flags: *const u8,
) -> bool {
    if (*slang).sl_compprog.is_null() {
        return false;
    }

    // Convert single-byte flags to UTF-8 characters
    let mut uflags = [0u8; crate::MAXWLEN * 2 + 1];
    let mut p = uflags.as_mut_ptr().cast::<c_char>();
    let mut i = 0usize;
    loop {
        let c = *flags.add(i);
        if c == 0 {
            break;
        }
        p = p.add(utf_char2bytes(c_int::from(c), p) as usize);
        i += 1;
    }
    *p = 0;

    if !vim_regexec_prog(
        std::ptr::addr_of_mut!((*slang).sl_compprog),
        false,
        uflags.as_ptr().cast::<c_char>(),
        0,
    ) {
        return false;
    }

    // Count syllables - if too many AND too many compound words, reject
    if (*slang).sl_compsylmax < crate::MAXWLEN as c_int {
        let syllables = crate::count_syllables(slang, word);
        if syllables > (*slang).sl_compsylmax {
            return (strlen(flags.cast::<c_char>()) as c_int) < (*slang).sl_compmax;
        }
    }
    true
}

/// Return true if "compflags" can be the start of a valid compound rule in slang.
///
/// This is the canonical C-ABI export of match_compoundrule using the C signature.
///
/// # Safety
/// slang and compflags must be valid pointers.
#[export_name = "match_compoundrule"]
pub unsafe extern "C" fn rs_match_compoundrule_c_compat(
    slang: *mut crate::SlangRaw,
    compflags: *const u8,
) -> bool {
    if (*slang).sl_comprules.is_null() {
        return false;
    }
    // Build NUL-terminated slices for the Rust implementation
    let rules = (*slang).sl_comprules;
    // Find length of comprules (NUL-terminated)
    let mut rules_len = 0usize;
    while *rules.add(rules_len) != 0 {
        rules_len += 1;
    }
    rules_len += 1; // include NUL

    let mut flags_len = 0usize;
    while *compflags.add(flags_len) != 0 {
        flags_len += 1;
    }
    flags_len += 1; // include NUL

    let comprules_slice = std::slice::from_raw_parts(rules, rules_len);
    let compflags_slice = std::slice::from_raw_parts(compflags, flags_len);
    match_compoundrule(comprules_slice, compflags_slice)
}

/// Return non-zero if the prefix at arridx in sl_pidxs matches "flags" for "word".
/// Returns the WF_* flags for the matching prefix, or 0 if no match.
///
/// # Safety
/// slang and word must be valid pointers.
#[export_name = "valid_word_prefix"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_valid_word_prefix(
    totprefcnt: c_int,
    arridx: c_int,
    flags: c_int,
    word: *mut c_char,
    slang: *mut crate::SlangRaw,
    cond_req: bool,
) -> c_int {
    const WF_HAS_AFF: c_int = 0x0200;
    const WF_PFX_NC: c_int = 0x0040_0000;
    const WF_RAREPFX: c_int = 0x0020_0000;
    let _ = WF_RAREPFX; // used indirectly via return value

    let prefid = ((flags as u32) >> 24) as c_int;
    let pidxs = (*slang).sl_pidxs;

    let mut prefcnt = totprefcnt - 1;
    while prefcnt >= 0 {
        let pidx = *pidxs.add((arridx + prefcnt) as usize);

        // Check prefix ID (low byte of pidx)
        if prefid != (pidx & 0xff) {
            prefcnt -= 1;
            continue;
        }

        // Check if prefix doesn't combine and word has suffix
        if (flags & WF_HAS_AFF) != 0 && (pidx & WF_PFX_NC) != 0 {
            prefcnt -= 1;
            continue;
        }

        // Check the condition regexp
        let cond_idx = ((pidx as u32 >> 8) & 0xffff) as usize;
        let rp_ptr = (*slang).sl_prefprog.add(cond_idx);
        if !(*rp_ptr).is_null() {
            if !vim_regexec_prog(rp_ptr, false, word, 0) {
                prefcnt -= 1;
                continue;
            }
        } else if cond_req {
            prefcnt -= 1;
            continue;
        }

        // Match found!
        return pidx;
    }
    0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_result_default() {
        assert_eq!(SpellResult::default(), SpellResult::Bad);
    }

    #[test]
    fn test_case_type_to_flags() {
        assert_eq!(CaseType::AllLower.to_flags(), 0);
        assert_eq!(CaseType::OneCap.to_flags(), WF_ONECAP);
        assert_eq!(CaseType::AllCap.to_flags(), WF_ALLCAP);
        assert_eq!(CaseType::KeepCap.to_flags(), WF_KEEPCAP);
    }

    #[test]
    fn test_spell_valid_case() {
        // ALLCAP word, tree has no FIXCAP -> valid via first branch
        assert!(spell_valid_case(WF_ALLCAP, 0));
        assert!(spell_valid_case(WF_ALLCAP, WF_ONECAP));
        assert!(spell_valid_case(WF_ALLCAP, WF_ALLCAP)); // ALLCAP tree, no FIXCAP

        // ALLCAP word, tree has FIXCAP only -> valid via second branch
        // First branch: (WF_FIXCAP & WF_FIXCAP) != 0 -> FALSE
        // Second branch: (WF_FIXCAP & (WF_ALLCAP|WF_KEEPCAP)) == 0 -> TRUE
        assert!(spell_valid_case(WF_ALLCAP, WF_FIXCAP));

        // ALLCAP word, tree has FIXCAP|ALLCAP -> both branches fail
        assert!(!spell_valid_case(WF_ALLCAP, WF_FIXCAP | WF_ALLCAP));

        // Lower case word matches if tree doesn't require caps
        assert!(spell_valid_case(0, 0));
        assert!(!spell_valid_case(0, WF_ONECAP));
        assert!(!spell_valid_case(0, WF_ALLCAP));
        assert!(!spell_valid_case(0, WF_KEEPCAP));

        // OneCap word matches if tree requires OneCap or no caps
        assert!(spell_valid_case(WF_ONECAP, 0));
        assert!(spell_valid_case(WF_ONECAP, WF_ONECAP));
        assert!(!spell_valid_case(WF_ONECAP, WF_ALLCAP));
    }

    #[test]
    fn test_region_matches() {
        // REGION_ALL matches anything
        assert!(region_matches(0, REGION_ALL));
        assert!(region_matches(0x00FF_0000, REGION_ALL));

        // Specific region check
        assert!(region_matches(0x0001_0000, 0x01));
        assert!(region_matches(0x0002_0000, 0x02));
        assert!(!region_matches(0x0001_0000, 0x02));
    }

    #[test]
    fn test_can_compound() {
        // Word with no compound flags can't compound (unless NEEDCOMP)
        assert!(!can_compound(0, true, false));
        assert!(can_compound(WF_NEEDCOMP, true, false));

        // NOCOMPBEF prevents being first part
        assert!(!can_compound(WF_NOCOMPBEF | (1 << 24), true, false));
        assert!(can_compound(WF_NOCOMPBEF | (1 << 24), false, true));

        // NOCOMPAFT prevents being after another word
        assert!(can_compound(WF_NOCOMPAFT | (1 << 24), true, false));
        assert!(!can_compound(WF_NOCOMPAFT | (1 << 24), false, false));
    }

    #[test]
    fn test_compound_flag_in_set() {
        let flags = b"abc\0";

        assert!(compound_flag_in_set(flags, b'a'));
        assert!(compound_flag_in_set(flags, b'b'));
        assert!(compound_flag_in_set(flags, b'c'));
        assert!(!compound_flag_in_set(flags, b'd'));
        assert!(!compound_flag_in_set(flags, 0));
    }

    #[test]
    fn test_word_lookup_result_default() {
        let result = WordLookupResult::default();
        assert_eq!(result.result, SpellResult::Bad);
        assert_eq!(result.match_len, 0);
        assert_eq!(result.flags, 0);
    }

    #[test]
    fn test_lookup_word_found() {
        // Tree for word "a"
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, 0]; // flags = 0

        let result = lookup_word(&byts, &idxs, b"a", 0, REGION_ALL);
        assert_eq!(result.result, SpellResult::Ok);
        assert_eq!(result.match_len, 1);
    }

    #[test]
    fn test_lookup_word_not_found() {
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, 0];

        let result = lookup_word(&byts, &idxs, b"b", 0, REGION_ALL);
        assert_eq!(result.result, SpellResult::Bad);
    }

    #[test]
    fn test_lookup_word_rare() {
        // Tree for word "a" with RARE flag
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, WF_RARE as IdxT];

        let result = lookup_word(&byts, &idxs, b"a", 0, REGION_ALL);
        assert_eq!(result.result, SpellResult::Rare);
    }

    #[test]
    fn test_lookup_word_banned() {
        // Tree for word "a" with BANNED flag
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, WF_BANNED as IdxT];

        let result = lookup_word(&byts, &idxs, b"a", 0, REGION_ALL);
        assert_eq!(result.result, SpellResult::Banned);
    }

    // =============================================================================
    // Phase 321 Tests: find_word_in_tree
    // =============================================================================

    #[test]
    fn test_sp_constants() {
        assert_eq!(SP_BANNED, -1);
        assert_eq!(SP_RARE, 0);
        assert_eq!(SP_OK, 1);
        assert_eq!(SP_LOCAL, 2);
        assert_eq!(SP_BAD, 3);
    }

    #[test]
    fn test_find_word_in_tree_simple() {
        // Tree for word "cat": c -> a -> t -> END
        // Structure: [count, bytes...][count, bytes...] etc.
        // Root: 1 child 'c' at idx 2
        // 'c' node: 1 child 'a' at idx 4
        // 'a' node: 1 child 't' at idx 6
        // 't' node: 1 ending (0) with flags 0
        let byts: [u8; 8] = [
            1, b'c', // Root: 1 sibling 'c'
            1, b'a', // After 'c': 1 sibling 'a'
            1, b't', // After 'a': 1 sibling 't'
            1, 0, // After 't': 1 sibling (end marker)
        ];
        let idxs: [IdxT; 8] = [
            0, 2, // 'c' -> idx 2
            0, 4, // 'a' -> idx 4
            0, 6, // 't' -> idx 6
            0, 0, // flags = 0 (word is OK)
        ];

        let result = find_word_in_tree(
            &byts,
            &idxs,
            b"cat",
            3,
            FindWordMode::FoldWord,
            0,
            0,
            REGION_ALL,
        );
        assert_eq!(result.result, SP_OK);
        assert_eq!(result.word_len, 3);
    }

    #[test]
    fn test_find_word_in_tree_not_found() {
        let byts: [u8; 8] = [1, b'c', 1, b'a', 1, b't', 1, 0];
        let idxs: [IdxT; 8] = [0, 2, 0, 4, 0, 6, 0, 0];

        let result = find_word_in_tree(
            &byts,
            &idxs,
            b"dog",
            3,
            FindWordMode::FoldWord,
            0,
            0,
            REGION_ALL,
        );
        assert_eq!(result.result, SP_BAD);
    }

    #[test]
    fn test_find_word_in_tree_rare() {
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, WF_RARE as IdxT];

        let result = find_word_in_tree(
            &byts,
            &idxs,
            b"a",
            1,
            FindWordMode::FoldWord,
            0,
            0,
            REGION_ALL,
        );
        assert_eq!(result.result, SP_RARE);
        assert_eq!(result.word_len, 1);
    }

    #[test]
    fn test_find_word_in_tree_banned() {
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, WF_BANNED as IdxT];

        let result = find_word_in_tree(
            &byts,
            &idxs,
            b"a",
            1,
            FindWordMode::FoldWord,
            0,
            0,
            REGION_ALL,
        );
        assert_eq!(result.result, SP_BANNED);
    }

    #[test]
    fn test_find_word_in_tree_region() {
        // Tree with region flag
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let region_1 = 0x0001_0000u32; // Region 1 in bits 16-23
        let idxs: [IdxT; 4] = [0, 2, 0, (WF_REGION | region_1) as IdxT];

        // Matching region
        let result = find_word_in_tree(
            &byts,
            &idxs,
            b"a",
            1,
            FindWordMode::FoldWord,
            0,
            0,
            0x01, // Region 1
        );
        assert_eq!(result.result, SP_OK);

        // Non-matching region
        let result = find_word_in_tree(
            &byts,
            &idxs,
            b"a",
            1,
            FindWordMode::FoldWord,
            0,
            0,
            0x02, // Region 2
        );
        assert_eq!(result.result, SP_LOCAL);
    }

    #[test]
    fn test_check_word_both_trees() {
        // Simple tree for "a"
        let fbyts: [u8; 4] = [1, b'a', 1, 0];
        let fidxs: [IdxT; 4] = [0, 2, 0, 0];

        let result = check_word_both_trees(&fbyts, &fidxs, &[], &[], b"a", b"a", 0, REGION_ALL);
        assert_eq!(result.result, SP_OK);
    }

    #[test]
    fn test_find_word_default_result() {
        let result = FindWordResult::default();
        assert_eq!(result.result, 0); // c_int default is 0
        assert_eq!(result.word_len, 0);
        assert_eq!(result.flags, 0);
    }

    // =========================================================================
    // Phase 323 Tests: UTF-8 Case Folding
    // =========================================================================

    #[test]
    fn test_utf8_decode_ascii() {
        let (cp, len) = utf8_decode(b"a").unwrap();
        assert_eq!(cp, 0x61);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_utf8_decode_two_byte() {
        // é is U+00E9, encoded as 0xC3 0xA9
        let bytes = [0xC3, 0xA9];
        let (cp, len) = utf8_decode(&bytes).unwrap();
        assert_eq!(cp, 0xE9);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_utf8_decode_three_byte() {
        // Greek capital sigma Σ is U+03A3, encoded as 0xCE 0xA3
        let bytes = [0xCE, 0xA3];
        let (cp, len) = utf8_decode(&bytes).unwrap();
        assert_eq!(cp, 0x03A3);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_utf8_encode_ascii() {
        let mut buf = [0u8; 4];
        let len = utf8_encode(0x61, &mut buf);
        assert_eq!(len, 1);
        assert_eq!(buf[0], b'a');
    }

    #[test]
    fn test_utf8_encode_two_byte() {
        let mut buf = [0u8; 4];
        let len = utf8_encode(0xE9, &mut buf);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], &[0xC3, 0xA9]);
    }

    #[test]
    fn test_utf8_encode_three_byte() {
        let mut buf = [0u8; 4];
        let len = utf8_encode(0x03A3, &mut buf);
        assert_eq!(len, 2); // Greek sigma fits in 2 bytes
        assert_eq!(&buf[..2], &[0xCE, 0xA3]);
    }

    #[test]
    fn test_greek_sigma_constants() {
        assert_eq!(GREEK_CAPITAL_SIGMA, 0x03A3);
        assert_eq!(GREEK_SMALL_SIGMA, 0x03C3);
        assert_eq!(GREEK_SMALL_FINAL_SIGMA, 0x03C2);
    }

    // =========================================================================
    // Phase 324 Tests: Compound Rule Matching
    // =========================================================================

    #[test]
    fn test_match_compoundrule_simple() {
        // Rule "ab" matches flags "a", "ab", but not "c", "abc"
        assert!(match_compoundrule(b"ab\0", b"a\0"));
        assert!(match_compoundrule(b"ab\0", b"ab\0"));
        assert!(!match_compoundrule(b"ab\0", b"c\0"));
        assert!(!match_compoundrule(b"ab\0", b"abc\0"));
    }

    #[test]
    fn test_match_compoundrule_multiple_rules() {
        // Multiple rules separated by /
        // "ab/cd" should match "a", "ab", "c", "cd"
        assert!(match_compoundrule(b"ab/cd\0", b"a\0"));
        assert!(match_compoundrule(b"ab/cd\0", b"ab\0"));
        assert!(match_compoundrule(b"ab/cd\0", b"c\0"));
        assert!(match_compoundrule(b"ab/cd\0", b"cd\0"));
        assert!(!match_compoundrule(b"ab/cd\0", b"e\0"));
    }

    #[test]
    fn test_match_compoundrule_brackets() {
        // "[ab]c" should match flags starting with a or b
        assert!(match_compoundrule(b"[ab]c\0", b"a\0"));
        assert!(match_compoundrule(b"[ab]c\0", b"b\0"));
        assert!(match_compoundrule(b"[ab]c\0", b"ac\0"));
        assert!(match_compoundrule(b"[ab]c\0", b"bc\0"));
        assert!(!match_compoundrule(b"[ab]c\0", b"c\0"));
        assert!(!match_compoundrule(b"[ab]c\0", b"dc\0"));
    }

    #[test]
    fn test_match_compoundrule_empty() {
        // Empty rules or flags
        assert!(!match_compoundrule(b"\0", b"a\0"));
        assert!(!match_compoundrule(b"", b"a\0"));
        // Empty flags means nothing collected yet - should match any rule
        assert!(match_compoundrule(b"ab\0", b"\0"));
    }

    // =========================================================================
    // Bad Word Case Type Tests
    // =========================================================================

    #[test]
    fn test_badword_captype_all_lower() {
        let spelltab = SpelltabHandle::null();
        let flags = badword_captype(b"hello", spelltab);
        // All lowercase returns 0 (AllLower has no flags)
        assert_eq!(flags & (WF_ONECAP | WF_ALLCAP | WF_KEEPCAP), 0);
    }

    #[test]
    fn test_badword_captype_first_cap() {
        let spelltab = SpelltabHandle::null();
        let flags = badword_captype(b"Hello", spelltab);
        // First letter cap returns OneCap
        assert_eq!(flags & WF_ONECAP, WF_ONECAP);
    }

    #[test]
    fn test_badword_captype_all_caps() {
        let spelltab = SpelltabHandle::null();
        let flags = badword_captype(b"HELLO", spelltab);
        // All caps returns AllCap
        assert_eq!(flags & WF_ALLCAP, WF_ALLCAP);
    }

    #[test]
    fn test_badword_captype_keepcap_with_first() {
        let spelltab = SpelltabHandle::null();
        // "WOrd" is KEEPCAP but starts with capital, should suggest ONECAP
        let flags = badword_captype(b"WOrd", spelltab);
        // Should have KEEPCAP and ONECAP
        assert!((flags & WF_KEEPCAP) != 0);
        assert!((flags & WF_ONECAP) != 0);
    }

    #[test]
    fn test_badword_captype_mixcap() {
        let spelltab = SpelltabHandle::null();
        // "maCARONI" has mixed case in middle
        let flags = badword_captype(b"maCARONI", spelltab);
        // Should have MIXCAP flag
        assert!((flags & WF_MIXCAP) != 0);
    }
}
