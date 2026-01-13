//! Soundfold (phonetic folding) utilities for spell checking
//!
//! This module provides helpers for soundfolding - converting words
//! to their phonetic representation for sound-alike matching.
//!
//! Soundfolding is used to find words that sound similar even if they
//! are spelled differently (e.g., "phone" and "fone").

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::option_if_let_else)]

use std::ffi::c_int;

// =============================================================================
// Soundfold Constants
// =============================================================================

/// Maximum length of a soundfolded word
pub const MAXSOFO_LEN: usize = 256;

/// Soundalike flags
pub mod sal_flags {
    use std::ffi::c_int;

    /// Follow-up rules (keep processing after match)
    pub const SAL_F0LLOWUP: c_int = 1;
    /// Collapse adjacent sounds
    pub const SAL_COLLAPSE: c_int = 2;
    /// Remove accents before conversion
    pub const SAL_REM_ACCENTS: c_int = 4;
    /// SOFO table (simple fold table, not SAL rules)
    pub const SAL_SOFO: c_int = 8;
}

// =============================================================================
// Soundfold State
// =============================================================================

/// State for soundfold conversion.
#[derive(Debug, Clone, Default)]
pub struct SoundfoldState {
    /// Flags for the conversion
    pub flags: i32,
    /// Position in source word
    pub src_pos: usize,
    /// Position in output buffer
    pub out_pos: usize,
    /// Previous output character (for collapse)
    pub prev_out: i32,
    /// Whether we're at word start
    pub at_start: bool,
}

impl SoundfoldState {
    /// Create a new soundfold state.
    #[must_use]
    pub const fn new(flags: i32) -> Self {
        Self {
            flags,
            src_pos: 0,
            out_pos: 0,
            prev_out: 0,
            at_start: true,
        }
    }

    /// Reset the state for a new word.
    pub fn reset(&mut self) {
        self.src_pos = 0;
        self.out_pos = 0;
        self.prev_out = 0;
        self.at_start = true;
    }

    /// Check if followup is enabled.
    #[must_use]
    pub const fn has_followup(&self) -> bool {
        (self.flags & sal_flags::SAL_F0LLOWUP) != 0
    }

    /// Check if collapse is enabled.
    #[must_use]
    pub const fn has_collapse(&self) -> bool {
        (self.flags & sal_flags::SAL_COLLAPSE) != 0
    }

    /// Check if accent removal is enabled.
    #[must_use]
    pub const fn has_rem_accents(&self) -> bool {
        (self.flags & sal_flags::SAL_REM_ACCENTS) != 0
    }

    /// Check if using SOFO table (not SAL rules).
    #[must_use]
    pub const fn is_sofo(&self) -> bool {
        (self.flags & sal_flags::SAL_SOFO) != 0
    }

    /// Add a character to the output, handling collapse.
    pub fn add_output(&mut self, c: i32) -> bool {
        if self.has_collapse() && c == self.prev_out {
            // Skip duplicate
            return false;
        }
        self.prev_out = c;
        self.out_pos += 1;
        self.at_start = false;
        true
    }
}

// =============================================================================
// SOFO Table Conversion
// =============================================================================

/// Result of a SOFO table lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SofoResult {
    /// Character was mapped to another character
    Mapped(i32),
    /// Character should be removed (no mapping)
    Remove,
    /// Character not in table, keep as-is
    Keep,
}

/// Look up a character in a SOFO table.
///
/// SOFO tables are parallel arrays: `from` contains source characters,
/// `to` contains the corresponding target characters.
///
/// # Arguments
///
/// * `from` - Source characters array
/// * `to` - Target characters array
/// * `c` - Character to look up
///
/// # Returns
///
/// `SofoResult` indicating how to handle the character
#[must_use]
pub fn sofo_lookup(from: &[i32], to: &[i32], c: i32) -> SofoResult {
    // Binary search in the from array
    let result = from.binary_search(&c);

    match result {
        Ok(idx) => {
            if idx < to.len() {
                let to_char = to[idx];
                if to_char == 0 {
                    SofoResult::Remove
                } else {
                    SofoResult::Mapped(to_char)
                }
            } else {
                SofoResult::Keep
            }
        }
        Err(_) => SofoResult::Keep,
    }
}

/// Perform simple soundfold using SOFO table.
///
/// This converts a word by looking up each character in the SOFO table
/// and replacing it with the target character.
///
/// # Arguments
///
/// * `word` - Input word as codepoints
/// * `from` - SOFO source characters (sorted)
/// * `to` - SOFO target characters
/// * `output` - Output buffer for folded codepoints
///
/// # Returns
///
/// Number of codepoints written to output
#[must_use]
pub fn sofo_fold(word: &[i32], from: &[i32], to: &[i32], output: &mut [i32]) -> usize {
    let mut out_pos = 0;
    let mut prev_out = 0i32;

    for &c in word {
        if c == 0 {
            break;
        }

        let folded = match sofo_lookup(from, to, c) {
            SofoResult::Mapped(tc) => tc,
            SofoResult::Remove => continue,
            SofoResult::Keep => c,
        };

        // Collapse duplicates
        if folded != prev_out && out_pos < output.len() {
            output[out_pos] = folded;
            out_pos += 1;
            prev_out = folded;
        }
    }

    // Null-terminate if space
    if out_pos < output.len() {
        output[out_pos] = 0;
    }

    out_pos
}

// =============================================================================
// SAL Rule Matching
// =============================================================================

/// A SAL (soundalike) rule.
#[derive(Debug, Clone)]
pub struct SalRule {
    /// Pattern to match (from)
    pub from: Vec<i32>,
    /// Replacement (to)
    pub to: Vec<i32>,
    /// Minimum match length
    pub min_match: usize,
}

impl SalRule {
    /// Check if this rule matches at the given position.
    ///
    /// # Arguments
    ///
    /// * `word` - The word being processed
    /// * `pos` - Current position in word
    /// * `at_start` - Whether this is the start of the word
    ///
    /// # Returns
    ///
    /// Number of characters consumed if matched, 0 if no match
    #[must_use]
    pub fn matches(&self, word: &[i32], pos: usize, at_start: bool) -> usize {
        if self.from.is_empty() {
            return 0;
        }

        // Check for start-of-word anchor
        let (pattern, check_start) = if self.from[0] == b'^' as i32 {
            if !at_start {
                return 0;
            }
            (&self.from[1..], true)
        } else {
            (&self.from[..], false)
        };

        if pattern.is_empty() {
            return 0;
        }

        // Match the pattern
        let mut matched = 0;
        for (i, &pat_char) in pattern.iter().enumerate() {
            let word_pos = pos + i;
            if word_pos >= word.len() || word[word_pos] == 0 {
                // End of word
                if i >= self.min_match {
                    break;
                }
                return 0;
            }

            // Check for character class or literal match
            if pat_char == b'(' as i32 {
                // Character class - find matching )
                // Simplified: just check if current char is in class
                // Full implementation would parse the class
                return 0;
            } else if pat_char != word[word_pos] {
                return 0;
            }

            matched += 1;
        }

        // Return matched count if conditions are met
        if (check_start && matched > 0) || matched >= self.min_match {
            matched
        } else {
            0
        }
    }
}

// =============================================================================
// Soundfold Similarity
// =============================================================================

/// Score for soundfold similarity.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundfoldScore {
    /// Number of matching characters
    pub matches: i32,
    /// Number of differing characters
    pub diffs: i32,
    /// Penalty score (lower is better)
    pub penalty: i32,
}

impl SoundfoldScore {
    /// Check if two soundfolded words are similar enough.
    #[must_use]
    pub const fn is_similar(&self, threshold: i32) -> bool {
        self.penalty <= threshold
    }
}

/// Compare two soundfolded words for similarity.
///
/// # Arguments
///
/// * `word1` - First soundfolded word (codepoints)
/// * `word2` - Second soundfolded word (codepoints)
///
/// # Returns
///
/// Similarity score
#[must_use]
pub fn soundfold_compare(word1: &[i32], word2: &[i32]) -> SoundfoldScore {
    let len1 = word1.iter().take_while(|&&c| c != 0).count();
    let len2 = word2.iter().take_while(|&&c| c != 0).count();

    let mut matches = 0;
    let mut i = 0;
    let mut j = 0;

    while i < len1 && j < len2 {
        if word1[i] == word2[j] {
            matches += 1;
            i += 1;
            j += 1;
        } else if len1 - i > len2 - j {
            // word1 is longer, skip a char in word1
            i += 1;
        } else {
            // word2 is longer or equal, skip a char in word2
            j += 1;
        }
    }

    let total = len1.max(len2) as i32;
    let diffs = total - matches;

    SoundfoldScore {
        matches,
        diffs,
        // Simple penalty: more diffs = higher penalty
        penalty: diffs * 10,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if soundfold flags have followup.
#[unsafe(no_mangle)]
pub extern "C" fn rs_sal_has_followup(flags: c_int) -> c_int {
    c_int::from((flags & sal_flags::SAL_F0LLOWUP) != 0)
}

/// Check if soundfold flags have collapse.
#[unsafe(no_mangle)]
pub extern "C" fn rs_sal_has_collapse(flags: c_int) -> c_int {
    c_int::from((flags & sal_flags::SAL_COLLAPSE) != 0)
}

/// Check if soundfold flags have accent removal.
#[unsafe(no_mangle)]
pub extern "C" fn rs_sal_has_rem_accents(flags: c_int) -> c_int {
    c_int::from((flags & sal_flags::SAL_REM_ACCENTS) != 0)
}

/// Check if soundfold uses SOFO table.
#[unsafe(no_mangle)]
pub extern "C" fn rs_sal_is_sofo(flags: c_int) -> c_int {
    c_int::from((flags & sal_flags::SAL_SOFO) != 0)
}

/// Compare two soundfolded words (FFI).
///
/// # Safety
///
/// Both pointers must be valid null-terminated arrays.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_soundfold_compare(
    word1: *const c_int,
    len1: usize,
    word2: *const c_int,
    len2: usize,
) -> c_int {
    if word1.is_null() || word2.is_null() {
        return i32::MAX;
    }

    let slice1 = std::slice::from_raw_parts(word1, len1);
    let slice2 = std::slice::from_raw_parts(word2, len2);

    soundfold_compare(slice1, slice2).penalty
}

// =============================================================================
// Phase 150: Additional FFI Exports for Soundalike & Compound
// =============================================================================

/// Get SAL_F0LLOWUP flag constant.
#[no_mangle]
pub extern "C" fn rs_sal_flag_followup() -> c_int {
    sal_flags::SAL_F0LLOWUP
}

/// Get SAL_COLLAPSE flag constant.
#[no_mangle]
pub extern "C" fn rs_sal_flag_collapse() -> c_int {
    sal_flags::SAL_COLLAPSE
}

/// Get SAL_REM_ACCENTS flag constant.
#[no_mangle]
pub extern "C" fn rs_sal_flag_rem_accents() -> c_int {
    sal_flags::SAL_REM_ACCENTS
}

/// Get SAL_SOFO flag constant.
#[no_mangle]
pub extern "C" fn rs_sal_flag_sofo() -> c_int {
    sal_flags::SAL_SOFO
}

/// Get maximum soundfold length constant.
#[no_mangle]
pub extern "C" fn rs_maxsofo_len() -> usize {
    MAXSOFO_LEN
}

/// Create a new SoundfoldState.
///
/// # Safety
/// `state_out` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_new(flags: c_int, state_out: *mut SoundfoldState) {
    if state_out.is_null() {
        return;
    }
    *state_out = SoundfoldState::new(flags);
}

/// Reset a SoundfoldState.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_reset(state: *mut SoundfoldState) {
    if state.is_null() {
        return;
    }
    (*state).reset();
}

/// Check if SoundfoldState has followup.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_has_followup(state: *const SoundfoldState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).has_followup()
}

/// Check if SoundfoldState has collapse.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_has_collapse(state: *const SoundfoldState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).has_collapse()
}

/// Check if SoundfoldState has accent removal.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_has_rem_accents(state: *const SoundfoldState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).has_rem_accents()
}

/// Check if SoundfoldState uses SOFO table.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_is_sofo(state: *const SoundfoldState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).is_sofo()
}

/// Add output character to SoundfoldState (handles collapse).
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_add_output(
    state: *mut SoundfoldState,
    c: c_int,
) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).add_output(c)
}

/// Get SoundfoldState output position.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_out_pos(state: *const SoundfoldState) -> usize {
    if state.is_null() {
        return 0;
    }
    (*state).out_pos
}

/// Get SoundfoldState source position.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_src_pos(state: *const SoundfoldState) -> usize {
    if state.is_null() {
        return 0;
    }
    (*state).src_pos
}

/// Set SoundfoldState source position.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_set_src_pos(state: *mut SoundfoldState, pos: usize) {
    if state.is_null() {
        return;
    }
    (*state).src_pos = pos;
}

/// Set SoundfoldState output position.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_set_out_pos(state: *mut SoundfoldState, pos: usize) {
    if state.is_null() {
        return;
    }
    (*state).out_pos = pos;
}

/// Check if SoundfoldState is at word start.
///
/// # Safety
/// `state` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_state_at_start(state: *const SoundfoldState) -> bool {
    if state.is_null() {
        return true;
    }
    (*state).at_start
}

/// Perform SOFO table lookup.
///
/// Returns: 0 if character should be removed, the mapped character if mapped,
/// or the original character if not in table.
///
/// # Safety
/// `from` and `to` must be valid arrays of length `table_len`.
#[no_mangle]
pub unsafe extern "C" fn rs_sofo_lookup(
    from: *const c_int,
    to: *const c_int,
    table_len: usize,
    c: c_int,
) -> c_int {
    if from.is_null() || to.is_null() || table_len == 0 {
        return c; // Keep character if no table
    }

    let from_slice = std::slice::from_raw_parts(from, table_len);
    let to_slice = std::slice::from_raw_parts(to, table_len);

    match sofo_lookup(from_slice, to_slice, c) {
        SofoResult::Mapped(tc) => tc,
        SofoResult::Remove => 0,
        SofoResult::Keep => c,
    }
}

/// Perform SOFO fold on a word.
///
/// # Safety
/// All pointers must be valid with the given lengths.
#[no_mangle]
pub unsafe extern "C" fn rs_sofo_fold(
    word: *const c_int,
    word_len: usize,
    from: *const c_int,
    to: *const c_int,
    table_len: usize,
    output: *mut c_int,
    output_len: usize,
) -> usize {
    if word.is_null() || from.is_null() || to.is_null() || output.is_null() {
        return 0;
    }

    let word_slice = std::slice::from_raw_parts(word, word_len);
    let from_slice = std::slice::from_raw_parts(from, table_len);
    let to_slice = std::slice::from_raw_parts(to, table_len);
    let output_slice = std::slice::from_raw_parts_mut(output, output_len);

    sofo_fold(word_slice, from_slice, to_slice, output_slice)
}

/// Get number of matches from a soundfold comparison.
///
/// # Safety
/// Pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_compare_matches(
    word1: *const c_int,
    len1: usize,
    word2: *const c_int,
    len2: usize,
) -> c_int {
    if word1.is_null() || word2.is_null() {
        return 0;
    }

    let slice1 = std::slice::from_raw_parts(word1, len1);
    let slice2 = std::slice::from_raw_parts(word2, len2);

    soundfold_compare(slice1, slice2).matches
}

/// Get number of diffs from a soundfold comparison.
///
/// # Safety
/// Pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_compare_diffs(
    word1: *const c_int,
    len1: usize,
    word2: *const c_int,
    len2: usize,
) -> c_int {
    if word1.is_null() || word2.is_null() {
        return i32::MAX;
    }

    let slice1 = std::slice::from_raw_parts(word1, len1);
    let slice2 = std::slice::from_raw_parts(word2, len2);

    soundfold_compare(slice1, slice2).diffs
}

/// Check if two soundfolded words are similar within a threshold.
///
/// # Safety
/// Pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_is_similar(
    word1: *const c_int,
    len1: usize,
    word2: *const c_int,
    len2: usize,
    threshold: c_int,
) -> bool {
    if word1.is_null() || word2.is_null() {
        return false;
    }

    let slice1 = std::slice::from_raw_parts(word1, len1);
    let slice2 = std::slice::from_raw_parts(word2, len2);

    soundfold_compare(slice1, slice2).is_similar(threshold)
}

/// Calculate soundfold similarity score (percentage-based).
///
/// Returns a score from 0-100 where 100 is identical.
///
/// # Safety
/// Pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_soundfold_similarity_score(
    word1: *const c_int,
    len1: usize,
    word2: *const c_int,
    len2: usize,
) -> c_int {
    if word1.is_null() || word2.is_null() {
        return 0;
    }

    let slice1 = std::slice::from_raw_parts(word1, len1);
    let slice2 = std::slice::from_raw_parts(word2, len2);

    let score = soundfold_compare(slice1, slice2);
    let total = score.matches + score.diffs;
    if total == 0 {
        return 100;
    }

    (score.matches * 100) / total
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundfold_state() {
        let state = SoundfoldState::new(sal_flags::SAL_F0LLOWUP | sal_flags::SAL_COLLAPSE);
        assert!(state.has_followup());
        assert!(state.has_collapse());
        assert!(!state.has_rem_accents());
        assert!(!state.is_sofo());
    }

    #[test]
    fn test_sofo_lookup() {
        let from = [b'a' as i32, b'b' as i32, b'c' as i32];
        let to = [b'A' as i32, 0, b'C' as i32];

        assert_eq!(
            sofo_lookup(&from, &to, b'a' as i32),
            SofoResult::Mapped(b'A' as i32)
        );
        assert_eq!(sofo_lookup(&from, &to, b'b' as i32), SofoResult::Remove);
        assert_eq!(
            sofo_lookup(&from, &to, b'c' as i32),
            SofoResult::Mapped(b'C' as i32)
        );
        assert_eq!(sofo_lookup(&from, &to, b'd' as i32), SofoResult::Keep);
    }

    #[test]
    fn test_sofo_fold() {
        let word = [b'a' as i32, b'b' as i32, b'c' as i32, 0];
        let from = [b'a' as i32, b'b' as i32, b'c' as i32];
        let to = [b'A' as i32, 0, b'A' as i32]; // b maps to nothing, c maps to A

        let mut output = [0i32; 10];
        let len = sofo_fold(&word, &from, &to, &mut output);

        // Should be just "A" (a->A, b removed, c->A collapsed with first A)
        assert_eq!(len, 1);
        assert_eq!(output[0], b'A' as i32);
    }

    #[test]
    fn test_soundfold_compare_identical() {
        let word1 = [b'f' as i32, b'o' as i32, b'n' as i32, 0];
        let word2 = [b'f' as i32, b'o' as i32, b'n' as i32, 0];

        let score = soundfold_compare(&word1, &word2);
        assert_eq!(score.matches, 3);
        assert_eq!(score.diffs, 0);
        assert_eq!(score.penalty, 0);
    }

    #[test]
    fn test_soundfold_compare_different() {
        let word1 = [b'f' as i32, b'o' as i32, b'n' as i32, 0];
        let word2 = [b'f' as i32, b'a' as i32, b'n' as i32, 0];

        let score = soundfold_compare(&word1, &word2);
        assert_eq!(score.matches, 2); // f and n match
        assert_eq!(score.diffs, 1);
    }

    #[test]
    fn test_soundfold_compare_length_diff() {
        let word1 = [b'f' as i32, b'o' as i32, b'n' as i32, b'e' as i32, 0];
        let word2 = [b'f' as i32, b'o' as i32, b'n' as i32, 0];

        let score = soundfold_compare(&word1, &word2);
        assert_eq!(score.diffs, 1); // One extra char in word1
    }
}
