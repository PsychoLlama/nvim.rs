//! Spell checking utilities for Neovim
//!
//! This crate provides Rust implementations of spell checking functions
//! from `src/nvim/spell.c`.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int};

// Region constant from spell_defs.h
/// Word is valid in all regions.
pub const REGION_ALL: c_int = 0xff;

// Word flags from spell_defs.h
const WF_ONECAP: c_int = 0x02; // word with one capital (or all capitals)
const WF_ALLCAP: c_int = 0x04; // word must be all capitals
const WF_FIXCAP: c_int = 0x40; // keep-case word, allcap not allowed
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
}
