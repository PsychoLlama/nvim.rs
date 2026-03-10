//! Langmap subsystem for Neovim key mapping.
//!
//! Implements the 'langmap' option which maps keyboard characters from
//! one language to another for Normal mode commands. This allows users
//! with non-Latin keyboards to use Vim commands without switching layouts.
//!
//! Two storage mechanisms:
//! - `langmap_mapchar[256]` (in C global) for ASCII characters < 256
//! - `LANGMAP_ENTRIES` (Rust Vec) for Unicode characters >= 256

#![allow(static_mut_refs)]

use std::ffi::{c_char, c_int};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    /// Read from the C global `langmap_mapchar[index]`.
    fn nvim_langmap_mapchar_get(index: c_int) -> u8;
    /// Write to the C global `langmap_mapchar[index]`.
    fn nvim_langmap_mapchar_set(index: c_int, value: u8);
    /// Read a UTF-8 character from a pointer.
    fn nvim_mapping_utf_ptr2char(p: *const c_char) -> c_int;
    /// Get the byte length of a UTF-8 character at pointer.
    fn nvim_mapping_utfc_ptr2len(p: *const c_char) -> c_int;
    /// Get the display representation of a character (for error messages).
    fn transchar(c: c_int) -> *const c_char;
    /// Format an error message into a buffer using snprintf.
    fn nvim_langmap_format_error(buf: *mut c_char, buflen: usize, msgid: c_int, arg: *const c_char);
}

// =============================================================================
// Langmap entry storage (replaces garray_T langmap_mapga)
// =============================================================================

/// A single langmap entry mapping one character to another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LangmapEntry {
    from: c_int,
    to: c_int,
}

/// Global sorted list of langmap entries for characters >= 256.
///
/// # Safety
/// This is accessed only from the main thread via FFI functions.
static mut LANGMAP_ENTRIES: Vec<LangmapEntry> = Vec::new();

// =============================================================================
// Core langmap functions
// =============================================================================

/// Search for an entry in the langmap entries for `from`. If found, update the
/// `to` field. If not found, insert a new entry at the appropriate sorted position.
///
/// Replaces C function `langmap_set_entry()`.
fn langmap_set_entry(from: c_int, to: c_int) {
    // SAFETY: Single-threaded access from main thread only.
    let entries = unsafe { &mut LANGMAP_ENTRIES };

    // Binary search for existing entry
    match entries.binary_search_by_key(&from, |e| e.from) {
        Ok(idx) => {
            // Found existing entry, update it
            entries[idx].to = to;
        }
        Err(idx) => {
            // Not found, insert at sorted position
            entries.insert(idx, LangmapEntry { from, to });
        }
    }
}

/// Apply langmap to a multi-byte character (>= 256) and return the result.
///
/// Uses binary search in the sorted langmap entries table.
/// Returns the character unmodified if no mapping exists.
///
/// Replaces C function `langmap_adjust_mb()`.
///
/// # Safety
/// Accesses the global `LANGMAP_ENTRIES` vector (single-threaded).
#[export_name = "langmap_adjust_mb"]
pub unsafe extern "C" fn rs_langmap_adjust_mb(c: c_int) -> c_int {
    let entries = &LANGMAP_ENTRIES;

    entries
        .binary_search_by_key(&c, |e| e.from)
        .map_or(c, |idx| entries[idx].to)
}

/// Initialize langmap: set identity mapping for ASCII chars and clear
/// the multi-byte entry table.
///
/// Replaces C function `langmap_init()`.
///
/// # Safety
/// Accesses the C global `langmap_mapchar[]` via accessors and the
/// Rust global `LANGMAP_ENTRIES`.
#[export_name = "langmap_init"]
pub unsafe extern "C" fn rs_langmap_init() {
    // Initialize ASCII identity mapping
    for i in 0..256 {
        nvim_langmap_mapchar_set(i, i as u8);
    }
    // Clear multi-byte entries
    LANGMAP_ENTRIES.clear();
}

/// Error message IDs for langmap parsing.
const LANGMAP_ERR_MATCHING: c_int = 357; // E357: matching character missing
const LANGMAP_ERR_EXTRA: c_int = 358; // E358: extra characters after semicolon

/// Parse the 'langmap' option string and populate the mapping tables.
///
/// Supports two formats:
/// - `"aAbBcCdD"` — alternating from/to pairs
/// - `"abc;ABC"` — semicolon-separated from/to lists
///
/// Returns 0 on success, or an error code (357 or 358) on failure.
/// On error, the error message is written to the provided buffer.
///
/// Replaces the parsing logic of C function `did_set_langmap()`.
///
/// # Safety
/// - `langmap_str` must be a valid NUL-terminated C string (the `p_langmap` global).
/// - `errbuf`/`errbuflen` must be a valid writable buffer.
/// - Accesses C globals via accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_langmap_parse(
    langmap_str: *const c_char,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> c_int {
    // Clear previous map and reinitialize
    LANGMAP_ENTRIES.clear();
    rs_langmap_init();

    let mut p = langmap_str;

    while *p != 0 {
        // Find the separator: scan for ',' or ';' (respecting backslash escapes)
        let mut p2 = p;
        while *p2 != 0 && *p2 != b',' as c_char && *p2 != b';' as c_char {
            if *p2 == b'\\' as c_char && *p2.add(1) != 0 {
                p2 = p2.add(1);
            }
            // MB_PTR_ADV equivalent
            p2 = p2.add(nvim_mapping_utfc_ptr2len(p2) as usize);
        }

        if *p2 == b';' as c_char {
            p2 = p2.add(1); // "abc;ABC" form, p2 now points to 'A'
        } else {
            p2 = std::ptr::null(); // "aAbBcCdD" form
        }

        while *p != 0 {
            if *p == b',' as c_char {
                p = p.add(1);
                break;
            }
            if *p == b'\\' as c_char && *p.add(1) != 0 {
                p = p.add(1);
            }
            let from = nvim_mapping_utf_ptr2char(p);
            let mut to: c_int = 0; // NUL

            if p2.is_null() {
                // "aAbBcCdD" form
                p = p.add(nvim_mapping_utfc_ptr2len(p) as usize);
                if *p != b',' as c_char {
                    if *p == b'\\' as c_char {
                        p = p.add(1);
                    }
                    to = nvim_mapping_utf_ptr2char(p);
                }
            } else {
                // "abc;ABC" form
                if *p2 != b',' as c_char {
                    if *p2 == b'\\' as c_char {
                        p2 = p2.add(1);
                    }
                    to = nvim_mapping_utf_ptr2char(p2);
                }
            }

            if to == 0 {
                // Error: matching character missing
                let tc = transchar(from);
                nvim_langmap_format_error(errbuf, errbuflen, LANGMAP_ERR_MATCHING, tc);
                return LANGMAP_ERR_MATCHING;
            }

            if from >= 256 {
                langmap_set_entry(from, to);
            } else {
                debug_assert!(to <= 255);
                nvim_langmap_mapchar_set(from & 255, to as u8);
            }

            // Advance to next pair
            p = p.add(nvim_mapping_utfc_ptr2len(p) as usize);
            if !p2.is_null() {
                p2 = p2.add(nvim_mapping_utfc_ptr2len(p2) as usize);
                if *p == b';' as c_char {
                    p = p2;
                    if *p != 0 {
                        if *p != b',' as c_char {
                            // Error: extra characters after semicolon
                            nvim_langmap_format_error(errbuf, errbuflen, LANGMAP_ERR_EXTRA, p);
                            return LANGMAP_ERR_EXTRA;
                        }
                        p = p.add(1);
                    }
                    break;
                }
            }
        }
    }

    0 // Success
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_langmap_set_entry_insert() {
        unsafe {
            LANGMAP_ENTRIES.clear();
        }

        // Insert entries out of order
        langmap_set_entry(300, 400);
        langmap_set_entry(100, 200);
        langmap_set_entry(200, 300);

        unsafe {
            // Should be sorted by `from`
            assert_eq!(LANGMAP_ENTRIES.len(), 3);
            assert_eq!(LANGMAP_ENTRIES[0], LangmapEntry { from: 100, to: 200 });
            assert_eq!(LANGMAP_ENTRIES[1], LangmapEntry { from: 200, to: 300 });
            assert_eq!(LANGMAP_ENTRIES[2], LangmapEntry { from: 300, to: 400 });
        }
    }

    #[test]
    fn test_langmap_set_entry_update() {
        unsafe {
            LANGMAP_ENTRIES.clear();
        }

        langmap_set_entry(100, 200);
        langmap_set_entry(100, 999); // Update existing

        unsafe {
            assert_eq!(LANGMAP_ENTRIES.len(), 1);
            assert_eq!(LANGMAP_ENTRIES[0].to, 999);
        }
    }

    #[test]
    fn test_langmap_adjust_mb() {
        unsafe {
            LANGMAP_ENTRIES.clear();
        }

        langmap_set_entry(0x410, 0x430); // Cyrillic А -> а
        langmap_set_entry(0x411, 0x431); // Cyrillic Б -> б

        unsafe {
            // Mapped characters
            assert_eq!(rs_langmap_adjust_mb(0x410), 0x430);
            assert_eq!(rs_langmap_adjust_mb(0x411), 0x431);

            // Unmapped character returns itself
            assert_eq!(rs_langmap_adjust_mb(0x412), 0x412);
            assert_eq!(rs_langmap_adjust_mb(65), 65);
        }
    }

    #[test]
    fn test_langmap_adjust_mb_empty() {
        unsafe {
            LANGMAP_ENTRIES.clear();

            // With empty table, all chars return themselves
            assert_eq!(rs_langmap_adjust_mb(100), 100);
            assert_eq!(rs_langmap_adjust_mb(0x410), 0x410);
        }
    }
}
