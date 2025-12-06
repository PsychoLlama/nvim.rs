//! Spell checking utilities for Neovim
//!
//! This crate provides Rust implementations of spell checking functions
//! from `src/nvim/spell.c`.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

// Word flags from spell_defs.h
const WF_ONECAP: c_int = 0x02;  // word with one capital (or all capitals)
const WF_ALLCAP: c_int = 0x04;  // word must be all capitals
const WF_FIXCAP: c_int = 0x40;  // keep-case word, allcap not allowed
const WF_KEEPCAP: c_int = 0x80; // keep-case word

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
