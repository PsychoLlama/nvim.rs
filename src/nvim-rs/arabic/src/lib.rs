//! Arabic text shaping support functions.
//!
//! This module provides functions for handling Arabic character shaping
//! and combining characters, needed for proper Arabic text rendering.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::similar_names)]

use std::ffi::c_int;

// =============================================================================
// Unicode values for Arabic characters
// =============================================================================

const A_HAMZA: c_int = 0x0621;
const A_ALEF_MADDA: c_int = 0x0622;
const A_ALEF_HAMZA_ABOVE: c_int = 0x0623;
const A_WAW_HAMZA: c_int = 0x0624;
const A_ALEF_HAMZA_BELOW: c_int = 0x0625;
const A_YEH_HAMZA: c_int = 0x0626;
const A_ALEF: c_int = 0x0627;
const A_BEH: c_int = 0x0628;
const A_TEH_MARBUTA: c_int = 0x0629;
const A_TEH: c_int = 0x062a;
const A_THEH: c_int = 0x062b;
const A_JEEM: c_int = 0x062c;
const A_HAH: c_int = 0x062d;
const A_KHAH: c_int = 0x062e;
const A_DAL: c_int = 0x062f;
const A_THAL: c_int = 0x0630;
const A_REH: c_int = 0x0631;
const A_ZAIN: c_int = 0x0632;
const A_SEEN: c_int = 0x0633;
const A_SHEEN: c_int = 0x0634;
const A_SAD: c_int = 0x0635;
const A_DAD: c_int = 0x0636;
const A_TAH: c_int = 0x0637;
const A_ZAH: c_int = 0x0638;
const A_AIN: c_int = 0x0639;
const A_GHAIN: c_int = 0x063a;
const A_TATWEEL: c_int = 0x0640;
const A_FEH: c_int = 0x0641;
const A_QAF: c_int = 0x0642;
const A_KAF: c_int = 0x0643;
const A_LAM: c_int = 0x0644;
const A_MEEM: c_int = 0x0645;
const A_NOON: c_int = 0x0646;
const A_HEH: c_int = 0x0647;
const A_WAW: c_int = 0x0648;
const A_ALEF_MAKSURA: c_int = 0x0649;
const A_YEH: c_int = 0x064a;
const A_FATHATAN: c_int = 0x064b;
const A_DAMMATAN: c_int = 0x064c;
const A_KASRATAN: c_int = 0x064d;
const A_FATHA: c_int = 0x064e;
const A_DAMMA: c_int = 0x064f;
const A_KASRA: c_int = 0x0650;
const A_SHADDA: c_int = 0x0651;
const A_SUKUN: c_int = 0x0652;
const A_MADDA_ABOVE: c_int = 0x0653;
const A_HAMZA_ABOVE: c_int = 0x0654;
const A_HAMZA_BELOW: c_int = 0x0655;

const A_PEH: c_int = 0x067e;
const A_TCHEH: c_int = 0x0686;
const A_JEH: c_int = 0x0698;
const A_FKAF: c_int = 0x06a9;
const A_GAF: c_int = 0x06af;
const A_FYEH: c_int = 0x06cc;

// LAM-ALEF ligature forms
const A_S_LAM_ALEF_MADDA_ABOVE: c_int = 0xfef5;
const A_F_LAM_ALEF_MADDA_ABOVE: c_int = 0xfef6;
const A_S_LAM_ALEF_HAMZA_ABOVE: c_int = 0xfef7;
const A_F_LAM_ALEF_HAMZA_ABOVE: c_int = 0xfef8;
const A_S_LAM_ALEF_HAMZA_BELOW: c_int = 0xfef9;
const A_F_LAM_ALEF_HAMZA_BELOW: c_int = 0xfefa;
const A_S_LAM_ALEF: c_int = 0xfefb;
const A_F_LAM_ALEF: c_int = 0xfefc;

const A_BYTE_ORDER_MARK: c_int = 0xfeff;

// =============================================================================
// Arabic character presentation forms table
// =============================================================================

/// Arabic character entry with presentation forms.
/// Each entry holds: base char, isolated, initial, medial, final forms.
#[derive(Clone, Copy)]
struct AChar {
    c: u32,
    isolated: u32,
    initial: u32,
    medial: u32,
    fin: u32, // 'final' is a reserved keyword
}

impl AChar {
    const fn new(c: u32, isolated: u32, initial: u32, medial: u32, fin: u32) -> Self {
        Self {
            c,
            isolated,
            initial,
            medial,
            fin,
        }
    }
}

/// Sorted list of Arabic characters with their presentation forms.
/// Must be sorted by 'c' field for binary search.
static ACHARS: [AChar; 54] = [
    AChar::new(A_HAMZA as u32, 0xfe80, 0, 0, 0),
    AChar::new(A_ALEF_MADDA as u32, 0xfe81, 0, 0, 0xfe82),
    AChar::new(A_ALEF_HAMZA_ABOVE as u32, 0xfe83, 0, 0, 0xfe84),
    AChar::new(A_WAW_HAMZA as u32, 0xfe85, 0, 0, 0xfe86),
    AChar::new(A_ALEF_HAMZA_BELOW as u32, 0xfe87, 0, 0, 0xfe88),
    AChar::new(A_YEH_HAMZA as u32, 0xfe89, 0xfe8b, 0xfe8c, 0xfe8a),
    AChar::new(A_ALEF as u32, 0xfe8d, 0, 0, 0xfe8e),
    AChar::new(A_BEH as u32, 0xfe8f, 0xfe91, 0xfe92, 0xfe90),
    AChar::new(A_TEH_MARBUTA as u32, 0xfe93, 0, 0, 0xfe94),
    AChar::new(A_TEH as u32, 0xfe95, 0xfe97, 0xfe98, 0xfe96),
    AChar::new(A_THEH as u32, 0xfe99, 0xfe9b, 0xfe9c, 0xfe9a),
    AChar::new(A_JEEM as u32, 0xfe9d, 0xfe9f, 0xfea0, 0xfe9e),
    AChar::new(A_HAH as u32, 0xfea1, 0xfea3, 0xfea4, 0xfea2),
    AChar::new(A_KHAH as u32, 0xfea5, 0xfea7, 0xfea8, 0xfea6),
    AChar::new(A_DAL as u32, 0xfea9, 0, 0, 0xfeaa),
    AChar::new(A_THAL as u32, 0xfeab, 0, 0, 0xfeac),
    AChar::new(A_REH as u32, 0xfead, 0, 0, 0xfeae),
    AChar::new(A_ZAIN as u32, 0xfeaf, 0, 0, 0xfeb0),
    AChar::new(A_SEEN as u32, 0xfeb1, 0xfeb3, 0xfeb4, 0xfeb2),
    AChar::new(A_SHEEN as u32, 0xfeb5, 0xfeb7, 0xfeb8, 0xfeb6),
    AChar::new(A_SAD as u32, 0xfeb9, 0xfebb, 0xfebc, 0xfeba),
    AChar::new(A_DAD as u32, 0xfebd, 0xfebf, 0xfec0, 0xfebe),
    AChar::new(A_TAH as u32, 0xfec1, 0xfec3, 0xfec4, 0xfec2),
    AChar::new(A_ZAH as u32, 0xfec5, 0xfec7, 0xfec8, 0xfec6),
    AChar::new(A_AIN as u32, 0xfec9, 0xfecb, 0xfecc, 0xfeca),
    AChar::new(A_GHAIN as u32, 0xfecd, 0xfecf, 0xfed0, 0xfece),
    AChar::new(A_TATWEEL as u32, 0, 0x0640, 0x0640, 0x0640),
    AChar::new(A_FEH as u32, 0xfed1, 0xfed3, 0xfed4, 0xfed2),
    AChar::new(A_QAF as u32, 0xfed5, 0xfed7, 0xfed8, 0xfed6),
    AChar::new(A_KAF as u32, 0xfed9, 0xfedb, 0xfedc, 0xfeda),
    AChar::new(A_LAM as u32, 0xfedd, 0xfedf, 0xfee0, 0xfede),
    AChar::new(A_MEEM as u32, 0xfee1, 0xfee3, 0xfee4, 0xfee2),
    AChar::new(A_NOON as u32, 0xfee5, 0xfee7, 0xfee8, 0xfee6),
    AChar::new(A_HEH as u32, 0xfee9, 0xfeeb, 0xfeec, 0xfeea),
    AChar::new(A_WAW as u32, 0xfeed, 0, 0, 0xfeee),
    AChar::new(A_ALEF_MAKSURA as u32, 0xfeef, 0, 0, 0xfef0),
    AChar::new(A_YEH as u32, 0xfef1, 0xfef3, 0xfef4, 0xfef2),
    AChar::new(A_FATHATAN as u32, 0xfe70, 0, 0, 0),
    AChar::new(A_DAMMATAN as u32, 0xfe72, 0, 0, 0),
    AChar::new(A_KASRATAN as u32, 0xfe74, 0, 0, 0),
    AChar::new(A_FATHA as u32, 0xfe76, 0, 0xfe77, 0),
    AChar::new(A_DAMMA as u32, 0xfe78, 0, 0xfe79, 0),
    AChar::new(A_KASRA as u32, 0xfe7a, 0, 0xfe7b, 0),
    AChar::new(A_SHADDA as u32, 0xfe7c, 0, 0xfe7c, 0),
    AChar::new(A_SUKUN as u32, 0xfe7e, 0, 0xfe7f, 0),
    AChar::new(A_MADDA_ABOVE as u32, 0, 0, 0, 0),
    AChar::new(A_HAMZA_ABOVE as u32, 0, 0, 0, 0),
    AChar::new(A_HAMZA_BELOW as u32, 0, 0, 0, 0),
    AChar::new(A_PEH as u32, 0xfb56, 0xfb58, 0xfb59, 0xfb57),
    AChar::new(A_TCHEH as u32, 0xfb7a, 0xfb7c, 0xfb7d, 0xfb7b),
    AChar::new(A_JEH as u32, 0xfb8a, 0, 0, 0xfb8b),
    AChar::new(A_FKAF as u32, 0xfb8e, 0xfb90, 0xfb91, 0xfb8f),
    AChar::new(A_GAF as u32, 0xfb92, 0xfb94, 0xfb95, 0xfb93),
    AChar::new(A_FYEH as u32, 0xfbfc, 0xfbfe, 0xfbff, 0xfbfd),
];

// =============================================================================
// External C options
// =============================================================================

extern "C" {
    /// The 'arabicshape' option (`p_arshape`)
    static p_arshape: c_int;
    /// The 'termbidi' option (`p_tbidi`)
    static p_tbidi: c_int;
}

// =============================================================================
// Helper functions
// =============================================================================

/// Find the `AChar` entry for the given Arabic character using binary search.
/// Returns None if not found.
fn find_achar(c: c_int) -> Option<&'static AChar> {
    let c = c as u32;
    let mut low = 0;
    let mut high = ACHARS.len();

    while low < high {
        let mid = usize::midpoint(low, high);
        if ACHARS[mid].c == c {
            return Some(&ACHARS[mid]);
        }
        if c < ACHARS[mid].c {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    None
}

/// Change shape - from Combination (2 char) to Isolated LAM-ALEF form.
const fn chg_c_laa2i(hid_c: c_int) -> c_int {
    match hid_c {
        A_ALEF_MADDA => A_S_LAM_ALEF_MADDA_ABOVE,
        A_ALEF_HAMZA_ABOVE => A_S_LAM_ALEF_HAMZA_ABOVE,
        A_ALEF_HAMZA_BELOW => A_S_LAM_ALEF_HAMZA_BELOW,
        A_ALEF => A_S_LAM_ALEF,
        _ => 0,
    }
}

/// Change shape - from Combination-Isolated to Final LAM-ALEF form.
const fn chg_c_laa2f(hid_c: c_int) -> c_int {
    match hid_c {
        A_ALEF_MADDA => A_F_LAM_ALEF_MADDA_ABOVE,
        A_ALEF_HAMZA_ABOVE => A_F_LAM_ALEF_HAMZA_ABOVE,
        A_ALEF_HAMZA_BELOW => A_F_LAM_ALEF_HAMZA_BELOW,
        A_ALEF => A_F_LAM_ALEF,
        _ => 0,
    }
}

/// Returns whether it is possible to join the given letters.
fn can_join(c1: c_int, c2: c_int) -> bool {
    if let (Some(a1), Some(a2)) = (find_achar(c1), find_achar(c2)) {
        (a1.initial != 0 || a1.medial != 0) && (a2.fin != 0 || a2.medial != 0)
    } else {
        false
    }
}

/// Returns true if 'c' is an Arabic ISO-8859-6 character.
fn a_is_iso(c: c_int) -> bool {
    find_achar(c).is_some()
}

/// Returns true if 'c' is an Arabic 10646 (8859-6 or Form-B).
fn a_is_ok(c: c_int) -> bool {
    a_is_iso(c) || c == A_BYTE_ORDER_MARK
}

/// Returns true if 'c' is a valid Arabic character for shaping.
fn a_is_valid(c: c_int) -> bool {
    a_is_ok(c) && c != A_HAMZA
}

// =============================================================================
// Public API - Combining character checks
// =============================================================================

/// Check whether we are dealing with a character that could be regarded as an
/// Arabic combining character, need to check the character before this.
#[inline]
fn arabic_maycombine_impl(two: c_int) -> bool {
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
#[inline]
fn arabic_combine_impl(one: c_int, two: c_int) -> bool {
    if one == A_LAM {
        return arabic_maycombine_impl(two);
    }
    false
}

/// Check whether we are dealing with Arabic combining characters.
/// For internal Rust use (called by mbyte crate).
#[inline]
#[must_use]
pub fn arabic_combine(one: c_int, two: c_int) -> bool {
    arabic_combine_impl(one, two)
}

/// C-compatible wrapper for `arabic_combine`.
#[export_name = "arabic_combine"]
#[must_use]
pub extern "C" fn rs_arabic_combine(one: c_int, two: c_int) -> bool {
    arabic_combine_impl(one, two)
}

/// Export `arabic_maycombine` for C callers.
#[export_name = "arabic_maycombine"]
#[must_use]
pub extern "C" fn rs_arabic_maycombine(two: c_int) -> bool {
    arabic_maycombine_impl(two)
}

// =============================================================================
// Public API - Arabic shaping
// =============================================================================

/// Do Arabic shaping on character "c". Returns the shaped character.
///
/// # Arguments
/// * `c` - The character to shape
/// * `c1p` - Pointer to the first composing char for "c" (in/out, may be set to 0)
/// * `prev_c` - The previous character (not shaped)
/// * `prev_c1` - The first composing char for the previous char (not shaped)
/// * `next_c` - The next character (not shaped)
///
/// # Safety
/// * `c1p` must be a valid pointer to a `c_int`
#[export_name = "arabic_shape"]
pub unsafe extern "C" fn rs_arabic_shape(
    c: c_int,
    c1p: *mut c_int,
    prev_c: c_int,
    prev_c1: c_int,
    next_c: c_int,
) -> c_int {
    // Deal only with Arabic characters, pass back all others
    if !a_is_ok(c) {
        return c;
    }

    let c1 = *c1p;
    let curr_laa = arabic_combine_impl(c, c1);
    let prev_laa = arabic_combine_impl(prev_c, prev_c1);

    let curr_c = if curr_laa {
        let result = if a_is_valid(prev_c) && can_join(prev_c, A_LAM) && !prev_laa {
            chg_c_laa2f(c1)
        } else {
            chg_c_laa2i(c1)
        };
        // Remove the composing character
        *c1p = 0;
        result
    } else {
        let curr_a = find_achar(c);
        let backward_combine = !prev_laa && can_join(prev_c, c);
        let forward_combine = can_join(c, next_c);

        curr_a.map_or(0, |a| {
            if backward_combine {
                if forward_combine {
                    a.medial as c_int
                } else {
                    a.fin as c_int
                }
            } else if forward_combine {
                a.initial as c_int
            } else {
                a.isolated as c_int
            }
        })
    };

    // Character missing from the table means using original character.
    if curr_c == 0 {
        c
    } else {
        curr_c
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_constants() {
        assert_eq!(A_LAM, 0x0644);
        assert_eq!(A_ALEF, 0x0627);
        assert_eq!(A_ALEF_MADDA, 0x0622);
        assert_eq!(A_ALEF_HAMZA_ABOVE, 0x0623);
        assert_eq!(A_ALEF_HAMZA_BELOW, 0x0625);
    }

    #[test]
    fn test_find_achar() {
        // Test finding existing characters
        let lam = find_achar(A_LAM);
        assert!(lam.is_some());
        let lam = lam.unwrap();
        assert_eq!(lam.c, A_LAM as u32);
        assert_eq!(lam.isolated, 0xfedd);

        // Test non-existent character
        assert!(find_achar(0x0000).is_none());
        assert!(find_achar(0xFFFF).is_none());
    }

    #[test]
    fn test_chg_c_laa2i() {
        assert_eq!(chg_c_laa2i(A_ALEF_MADDA), A_S_LAM_ALEF_MADDA_ABOVE);
        assert_eq!(chg_c_laa2i(A_ALEF_HAMZA_ABOVE), A_S_LAM_ALEF_HAMZA_ABOVE);
        assert_eq!(chg_c_laa2i(A_ALEF), A_S_LAM_ALEF);
        assert_eq!(chg_c_laa2i(0), 0);
    }

    #[test]
    fn test_chg_c_laa2f() {
        assert_eq!(chg_c_laa2f(A_ALEF_MADDA), A_F_LAM_ALEF_MADDA_ABOVE);
        assert_eq!(chg_c_laa2f(A_ALEF_HAMZA_ABOVE), A_F_LAM_ALEF_HAMZA_ABOVE);
        assert_eq!(chg_c_laa2f(A_ALEF), A_F_LAM_ALEF);
        assert_eq!(chg_c_laa2f(0), 0);
    }

    #[test]
    fn test_can_join() {
        // LAM can join with BEH (both have medial forms)
        assert!(can_join(A_LAM, A_BEH));
        // DAL cannot join forward (no initial/medial)
        assert!(!can_join(A_DAL, A_BEH));
    }

    #[test]
    fn test_a_is_iso() {
        assert!(a_is_iso(A_LAM));
        assert!(a_is_iso(A_ALEF));
        assert!(!a_is_iso(0x0000));
    }

    #[test]
    fn test_a_is_ok() {
        assert!(a_is_ok(A_LAM));
        assert!(a_is_ok(A_BYTE_ORDER_MARK));
        assert!(!a_is_ok(0x0000));
    }

    #[test]
    fn test_a_is_valid() {
        assert!(a_is_valid(A_LAM));
        assert!(!a_is_valid(A_HAMZA)); // HAMZA is excluded
    }

    #[test]
    fn test_achars_sorted() {
        // Binary search requires the table to be sorted by 'c'
        for i in 1..ACHARS.len() {
            assert!(
                ACHARS[i - 1].c < ACHARS[i].c,
                "ACHARS not sorted at index {i}: {} >= {}",
                ACHARS[i - 1].c,
                ACHARS[i].c
            );
        }
    }

    #[test]
    fn test_achars_table_size() {
        // Table should have exactly 54 entries
        assert_eq!(ACHARS.len(), 54);
    }

    #[test]
    fn test_find_achar_all_entries() {
        // Verify all entries in ACHARS can be found
        for entry in &ACHARS {
            let found = find_achar(entry.c as c_int);
            assert!(found.is_some(), "Failed to find char 0x{:04x}", entry.c);
            assert_eq!(found.unwrap().c, entry.c);
        }
    }

    #[test]
    fn test_lam_alef_ligatures_isolated() {
        // Test all LAM-ALEF isolated forms
        assert_eq!(chg_c_laa2i(A_ALEF_MADDA), A_S_LAM_ALEF_MADDA_ABOVE);
        assert_eq!(chg_c_laa2i(A_ALEF_HAMZA_ABOVE), A_S_LAM_ALEF_HAMZA_ABOVE);
        assert_eq!(chg_c_laa2i(A_ALEF_HAMZA_BELOW), A_S_LAM_ALEF_HAMZA_BELOW);
        assert_eq!(chg_c_laa2i(A_ALEF), A_S_LAM_ALEF);
        // Non-alef characters should return 0
        assert_eq!(chg_c_laa2i(A_BEH), 0);
        assert_eq!(chg_c_laa2i(A_LAM), 0);
    }

    #[test]
    fn test_lam_alef_ligatures_final() {
        // Test all LAM-ALEF final forms
        assert_eq!(chg_c_laa2f(A_ALEF_MADDA), A_F_LAM_ALEF_MADDA_ABOVE);
        assert_eq!(chg_c_laa2f(A_ALEF_HAMZA_ABOVE), A_F_LAM_ALEF_HAMZA_ABOVE);
        assert_eq!(chg_c_laa2f(A_ALEF_HAMZA_BELOW), A_F_LAM_ALEF_HAMZA_BELOW);
        assert_eq!(chg_c_laa2f(A_ALEF), A_F_LAM_ALEF);
        // Non-alef characters should return 0
        assert_eq!(chg_c_laa2f(A_BEH), 0);
        assert_eq!(chg_c_laa2f(A_LAM), 0);
    }

    #[test]
    fn test_lam_alef_ligature_unicode_range() {
        // Isolated LAM-ALEF ligatures should be in 0xFEF5-0xFEFC range
        let s_lam_alef = A_S_LAM_ALEF;
        let f_lam_alef = A_F_LAM_ALEF;
        assert!(s_lam_alef >= 0xFEF5);
        assert!(s_lam_alef <= 0xFEFC);
        assert!(f_lam_alef >= 0xFEF5);
        assert!(f_lam_alef <= 0xFEFC);
    }

    #[test]
    fn test_can_join_non_joining_chars() {
        // DAL, THAL, REH, ZAIN, WAW only have final forms (no initial/medial)
        assert!(!can_join(A_DAL, A_BEH));
        assert!(!can_join(A_REH, A_BEH));
        assert!(!can_join(A_ZAIN, A_BEH));
        assert!(!can_join(A_WAW, A_BEH));
        // But they can be joined FROM (as second character)
        assert!(can_join(A_BEH, A_DAL));
        assert!(can_join(A_BEH, A_REH));
    }

    #[test]
    fn test_can_join_dual_joining_chars() {
        // Characters with all forms can join in both directions
        assert!(can_join(A_BEH, A_TEH));
        assert!(can_join(A_TEH, A_BEH));
        assert!(can_join(A_SEEN, A_SHEEN));
        assert!(can_join(A_LAM, A_MEEM));
    }

    #[test]
    fn test_can_join_non_arabic() {
        // Non-Arabic characters cannot join
        assert!(!can_join(0x41, A_BEH)); // 'A'
        assert!(!can_join(A_BEH, 0x41));
        assert!(!can_join(0, 0));
    }

    #[test]
    fn test_a_is_iso_arabic_letters() {
        // Test Arabic letter range 0x0621-0x064A
        for c in [A_HAMZA, A_ALEF, A_BEH, A_TEH, A_LAM, A_MEEM, A_NOON, A_YEH] {
            assert!(a_is_iso(c), "0x{c:04x} should be ISO Arabic");
        }
    }

    #[test]
    fn test_a_is_iso_diacritics() {
        // Test Arabic diacritics (tashkeel)
        for c in [
            A_FATHATAN, A_DAMMATAN, A_KASRATAN, A_FATHA, A_DAMMA, A_KASRA, A_SHADDA, A_SUKUN,
        ] {
            assert!(a_is_iso(c), "Diacritic 0x{c:04x} should be ISO Arabic");
        }
    }

    #[test]
    fn test_a_is_ok_bom() {
        // Byte Order Mark should be OK but not ISO
        assert!(a_is_ok(A_BYTE_ORDER_MARK));
        assert!(!a_is_iso(A_BYTE_ORDER_MARK));
    }

    #[test]
    fn test_a_is_valid_excludes_hamza() {
        // HAMZA is OK and ISO but not valid for shaping
        assert!(a_is_ok(A_HAMZA));
        assert!(a_is_iso(A_HAMZA));
        assert!(!a_is_valid(A_HAMZA));
    }

    #[test]
    fn test_achar_presentation_forms() {
        // Test that BEH has all four presentation forms
        let beh = find_achar(A_BEH).unwrap();
        assert_ne!(beh.isolated, 0, "BEH should have isolated form");
        assert_ne!(beh.initial, 0, "BEH should have initial form");
        assert_ne!(beh.medial, 0, "BEH should have medial form");
        assert_ne!(beh.fin, 0, "BEH should have final form");
    }

    #[test]
    fn test_achar_right_joining_only() {
        // DAL only has isolated and final forms (right-joining only)
        let dal = find_achar(A_DAL).unwrap();
        assert_ne!(dal.isolated, 0, "DAL should have isolated form");
        assert_eq!(dal.initial, 0, "DAL should NOT have initial form");
        assert_eq!(dal.medial, 0, "DAL should NOT have medial form");
        assert_ne!(dal.fin, 0, "DAL should have final form");
    }

    #[test]
    fn test_farsi_characters() {
        // Test Farsi/Persian specific characters
        let peh = find_achar(A_PEH);
        assert!(peh.is_some(), "PEH should be in table");
        let tcheh = find_achar(A_TCHEH);
        assert!(tcheh.is_some(), "TCHEH should be in table");
        let gaf = find_achar(A_GAF);
        assert!(gaf.is_some(), "GAF should be in table");
    }
}
