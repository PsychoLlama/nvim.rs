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

use std::ffi::c_int;

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

/// Fold a word to lowercase for spell checking.
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
    for &c in word {
        if c == 0 {
            break;
        }

        // For ASCII, use the spell fold table
        let folded = if c < 128 {
            spell_fold_char(u32::from(c), spelltab) as u8
        } else {
            // UTF-8 continuation bytes or high bytes - copy as-is
            // Full UTF-8 folding should be done by C code
            c
        };

        if out_idx >= output.len() - 1 {
            break;
        }
        output[out_idx] = folded;
        out_idx += 1;
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
pub unsafe extern "C" fn rs_compound_check_flags_max_parts(
    flags: *const CompoundCheckFlags,
) -> u8 {
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
}
