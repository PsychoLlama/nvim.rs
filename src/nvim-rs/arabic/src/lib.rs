//! Arabic text shaping support functions.
//!
//! This module provides functions for handling Arabic combining characters,
//! which is needed for proper grapheme clustering in Arabic text.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// Arabic character constants
const A_LAM: c_int = 0x0644;
const A_ALEF_MADDA: c_int = 0x0622;
const A_ALEF_HAMZA_ABOVE: c_int = 0x0623;
const A_ALEF_HAMZA_BELOW: c_int = 0x0625;
const A_ALEF: c_int = 0x0627;

// External C options (callbacks to C)
extern "C" {
    /// The 'arabicshape' option (`p_arshape`)
    static p_arshape: c_int;
    /// The 'termbidi' option (`p_tbidi`)
    static p_tbidi: c_int;
}

/// Check whether we are dealing with a character that could be regarded as an
/// Arabic combining character, need to check the character before this.
///
/// # Safety
/// Reads global option variables `p_arshape` and `p_tbidi`.
#[inline]
fn arabic_maycombine(two: c_int) -> bool {
    // SAFETY: p_arshape and p_tbidi are initialized during option setup
    let arshape_enabled = unsafe { p_arshape != 0 };
    let tbidi_enabled = unsafe { p_tbidi != 0 };

    if arshape_enabled && !tbidi_enabled {
        return two == A_ALEF_MADDA
            || two == A_ALEF_HAMZA_ABOVE
            || two == A_ALEF_HAMZA_BELOW
            || two == A_ALEF;
    }
    false
}

/// Check whether we are dealing with Arabic combining characters.
/// Returns false for negative values.
/// Note: these are NOT really composing characters!
///
/// # Safety
/// This function reads global option variables through `arabic_maycombine`.
#[no_mangle]
pub extern "C" fn rs_arabic_combine(one: c_int, two: c_int) -> bool {
    if one == A_LAM {
        return arabic_maycombine(two);
    }
    false
}

/// Export `arabic_maycombine` for C callers.
///
/// # Safety
/// Reads global option variables `p_arshape` and `p_tbidi`.
#[no_mangle]
pub extern "C" fn rs_arabic_maycombine(two: c_int) -> bool {
    arabic_maycombine(two)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests can't check actual option values without setting up C state,
    // but we can verify the character constants and basic logic.

    #[test]
    fn test_arabic_constants() {
        // Verify Arabic character constants are correct
        assert_eq!(A_LAM, 0x0644);
        assert_eq!(A_ALEF, 0x0627);
        assert_eq!(A_ALEF_MADDA, 0x0622);
        assert_eq!(A_ALEF_HAMZA_ABOVE, 0x0623);
        assert_eq!(A_ALEF_HAMZA_BELOW, 0x0625);
    }
}
