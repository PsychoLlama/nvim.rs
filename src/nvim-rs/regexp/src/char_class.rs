//! POSIX character class recognition for regular expressions.
//!
//! Handles `[:name:]` patterns like `[:alnum:]`, `[:alpha:]`, etc.

use std::ffi::{c_char, c_int};

// =============================================================================
// Character Class Constants
// =============================================================================

// These values must match the C enum in regexp.c
pub const CLASS_ALNUM: c_int = 0;
pub const CLASS_ALPHA: c_int = 1;
pub const CLASS_BLANK: c_int = 2;
pub const CLASS_CNTRL: c_int = 3;
pub const CLASS_DIGIT: c_int = 4;
pub const CLASS_GRAPH: c_int = 5;
pub const CLASS_LOWER: c_int = 6;
pub const CLASS_PRINT: c_int = 7;
pub const CLASS_PUNCT: c_int = 8;
pub const CLASS_SPACE: c_int = 9;
pub const CLASS_UPPER: c_int = 10;
pub const CLASS_XDIGIT: c_int = 11;
pub const CLASS_TAB: c_int = 12;
pub const CLASS_RETURN: c_int = 13;
pub const CLASS_BACKSPACE: c_int = 14;
pub const CLASS_ESCAPE: c_int = 15;
pub const CLASS_IDENT: c_int = 16;
pub const CLASS_KEYWORD: c_int = 17;
pub const CLASS_FNAME: c_int = 18;
pub const CLASS_NONE: c_int = 99;

// =============================================================================
// Lookup Table
// =============================================================================

/// Entry in the character class lookup table.
/// Sorted by `name` for binary search.
struct CharClassEntry {
    /// The class constant (CLASS_ALNUM, etc.)
    class: c_int,
    /// The name suffix after "[:". e.g. "alnum:]" for [:alnum:]
    name: &'static [u8],
}

/// Character class table, sorted by name for binary search.
/// The name includes the trailing ":]" to match the C implementation.
static CHAR_CLASS_TAB: &[CharClassEntry] = &[
    CharClassEntry {
        class: CLASS_ALNUM,
        name: b"alnum:]",
    },
    CharClassEntry {
        class: CLASS_ALPHA,
        name: b"alpha:]",
    },
    CharClassEntry {
        class: CLASS_BACKSPACE,
        name: b"backspace:]",
    },
    CharClassEntry {
        class: CLASS_BLANK,
        name: b"blank:]",
    },
    CharClassEntry {
        class: CLASS_CNTRL,
        name: b"cntrl:]",
    },
    CharClassEntry {
        class: CLASS_DIGIT,
        name: b"digit:]",
    },
    CharClassEntry {
        class: CLASS_ESCAPE,
        name: b"escape:]",
    },
    CharClassEntry {
        class: CLASS_FNAME,
        name: b"fname:]",
    },
    CharClassEntry {
        class: CLASS_GRAPH,
        name: b"graph:]",
    },
    CharClassEntry {
        class: CLASS_IDENT,
        name: b"ident:]",
    },
    CharClassEntry {
        class: CLASS_KEYWORD,
        name: b"keyword:]",
    },
    CharClassEntry {
        class: CLASS_LOWER,
        name: b"lower:]",
    },
    CharClassEntry {
        class: CLASS_PRINT,
        name: b"print:]",
    },
    CharClassEntry {
        class: CLASS_PUNCT,
        name: b"punct:]",
    },
    CharClassEntry {
        class: CLASS_RETURN,
        name: b"return:]",
    },
    CharClassEntry {
        class: CLASS_SPACE,
        name: b"space:]",
    },
    CharClassEntry {
        class: CLASS_TAB,
        name: b"tab:]",
    },
    CharClassEntry {
        class: CLASS_UPPER,
        name: b"upper:]",
    },
    CharClassEntry {
        class: CLASS_XDIGIT,
        name: b"xdigit:]",
    },
];

// =============================================================================
// Implementation
// =============================================================================

/// Check if a byte is an ASCII lowercase letter.
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Compare input string with a table entry name.
/// Returns Ordering based on comparison of input prefix with entry name.
#[inline]
fn cmp_entry(input: &[u8], entry: &CharClassEntry) -> std::cmp::Ordering {
    // Compare up to the length of the entry name
    let len = entry.name.len();
    for i in 0..len {
        if i >= input.len() {
            // input is shorter - comes before
            return std::cmp::Ordering::Less;
        }
        match input[i].cmp(&entry.name[i]) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// Check for a character class name "[:name:]". `pp` points to the '['.
/// Returns one of the CLASS_* items. CLASS_NONE means that no item was
/// recognized. Otherwise `pp` is advanced to after the item.
///
/// # Safety
/// `pp` must point to a valid pointer to a null-terminated string.
pub unsafe fn get_char_class_impl(pp: *mut *mut c_char) -> c_int {
    let ptr = *pp;

    // Check for the pattern: [: followed by at least 3 lowercase letters
    // ptr[0] = '[', ptr[1] = ':', ptr[2..4] = lowercase letters
    let c1 = *ptr.add(1) as u8;
    let c2 = *ptr.add(2) as u8;
    let c3 = *ptr.add(3) as u8;
    let c4 = *ptr.add(4) as u8;

    if c1 != b':' || !ascii_islower(c2) || !ascii_islower(c3) || !ascii_islower(c4) {
        return CLASS_NONE;
    }

    // Build a slice from the input starting at ptr+2 (after "[:")
    // We need to find how long the string is, but we'll limit it to a reasonable max
    let input_start = ptr.add(2);
    let mut input_len = 0usize;
    while input_len < 32 {
        let c = *input_start.add(input_len) as u8;
        if c == 0 {
            break;
        }
        input_len += 1;
        // Stop after we see "]" since that's the end of the pattern
        if c == b']' {
            break;
        }
    }

    // Create a slice for comparison
    let input = std::slice::from_raw_parts(input_start as *const u8, input_len);

    // Binary search in the sorted table
    let result = CHAR_CLASS_TAB.binary_search_by(|entry| {
        // We want to find if input starts with entry.name
        cmp_entry(input, entry).reverse()
    });

    match result {
        Ok(idx) => {
            let entry = &CHAR_CLASS_TAB[idx];
            // Advance pp past the matched class: "[:" + name
            *pp = ptr.add(2 + entry.name.len());
            entry.class
        }
        Err(_) => CLASS_NONE,
    }
}

// =============================================================================
// FFI Export
// =============================================================================

/// Check for a character class name "[:name:]".
///
/// # Safety
/// `pp` must point to a valid pointer to a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_char_class(pp: *mut *mut c_char) -> c_int {
    get_char_class_impl(pp)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Helper to test get_char_class with a string
    unsafe fn test_get_class(s: &str) -> (c_int, usize) {
        let cstr = CString::new(s).unwrap();
        let mut ptr = cstr.as_ptr() as *mut c_char;
        let original = ptr;
        let result = get_char_class_impl(&mut ptr);
        let advance = ptr.offset_from(original) as usize;
        (result, advance)
    }

    #[test]
    fn test_alnum() {
        unsafe {
            let (class, advance) = test_get_class("[:alnum:]rest");
            assert_eq!(class, CLASS_ALNUM);
            assert_eq!(advance, 9); // "[:alnum:]".len()
        }
    }

    #[test]
    fn test_alpha() {
        unsafe {
            let (class, advance) = test_get_class("[:alpha:]");
            assert_eq!(class, CLASS_ALPHA);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_digit() {
        unsafe {
            let (class, advance) = test_get_class("[:digit:]");
            assert_eq!(class, CLASS_DIGIT);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_space() {
        unsafe {
            let (class, advance) = test_get_class("[:space:]");
            assert_eq!(class, CLASS_SPACE);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_upper() {
        unsafe {
            let (class, advance) = test_get_class("[:upper:]");
            assert_eq!(class, CLASS_UPPER);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_lower() {
        unsafe {
            let (class, advance) = test_get_class("[:lower:]");
            assert_eq!(class, CLASS_LOWER);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_xdigit() {
        unsafe {
            let (class, advance) = test_get_class("[:xdigit:]");
            assert_eq!(class, CLASS_XDIGIT);
            assert_eq!(advance, 10); // "[:xdigit:]".len()
        }
    }

    #[test]
    fn test_blank() {
        unsafe {
            let (class, advance) = test_get_class("[:blank:]");
            assert_eq!(class, CLASS_BLANK);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_cntrl() {
        unsafe {
            let (class, advance) = test_get_class("[:cntrl:]");
            assert_eq!(class, CLASS_CNTRL);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_graph() {
        unsafe {
            let (class, advance) = test_get_class("[:graph:]");
            assert_eq!(class, CLASS_GRAPH);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_print() {
        unsafe {
            let (class, advance) = test_get_class("[:print:]");
            assert_eq!(class, CLASS_PRINT);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_punct() {
        unsafe {
            let (class, advance) = test_get_class("[:punct:]");
            assert_eq!(class, CLASS_PUNCT);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_tab() {
        unsafe {
            let (class, advance) = test_get_class("[:tab:]");
            assert_eq!(class, CLASS_TAB);
            assert_eq!(advance, 7);
        }
    }

    #[test]
    fn test_return() {
        unsafe {
            let (class, advance) = test_get_class("[:return:]");
            assert_eq!(class, CLASS_RETURN);
            assert_eq!(advance, 10);
        }
    }

    #[test]
    fn test_backspace() {
        unsafe {
            let (class, advance) = test_get_class("[:backspace:]");
            assert_eq!(class, CLASS_BACKSPACE);
            assert_eq!(advance, 13);
        }
    }

    #[test]
    fn test_escape() {
        unsafe {
            let (class, advance) = test_get_class("[:escape:]");
            assert_eq!(class, CLASS_ESCAPE);
            assert_eq!(advance, 10);
        }
    }

    #[test]
    fn test_ident() {
        unsafe {
            let (class, advance) = test_get_class("[:ident:]");
            assert_eq!(class, CLASS_IDENT);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_keyword() {
        unsafe {
            let (class, advance) = test_get_class("[:keyword:]");
            assert_eq!(class, CLASS_KEYWORD);
            assert_eq!(advance, 11);
        }
    }

    #[test]
    fn test_fname() {
        unsafe {
            let (class, advance) = test_get_class("[:fname:]");
            assert_eq!(class, CLASS_FNAME);
            assert_eq!(advance, 9);
        }
    }

    #[test]
    fn test_not_a_class() {
        unsafe {
            // Just a bracket
            let (class, advance) = test_get_class("[abc]");
            assert_eq!(class, CLASS_NONE);
            assert_eq!(advance, 0);

            // Missing colon
            let (class, advance) = test_get_class("[alnum:]");
            assert_eq!(class, CLASS_NONE);
            assert_eq!(advance, 0);

            // Invalid name
            let (class, advance) = test_get_class("[:unknown:]");
            assert_eq!(class, CLASS_NONE);
            assert_eq!(advance, 0);

            // Too few lowercase letters at start
            let (class, advance) = test_get_class("[:AB:]");
            assert_eq!(class, CLASS_NONE);
            assert_eq!(advance, 0);

            // Uppercase not allowed
            let (class, advance) = test_get_class("[:ALPHA:]");
            assert_eq!(class, CLASS_NONE);
            assert_eq!(advance, 0);
        }
    }

    #[test]
    fn test_partial_match_not_accepted() {
        unsafe {
            // "[:alph:]" should not match "[:alpha:]"
            let (class, advance) = test_get_class("[:alph:]");
            assert_eq!(class, CLASS_NONE);
            assert_eq!(advance, 0);
        }
    }

    #[test]
    fn test_class_constants_values() {
        // Verify constants match C enum order
        assert_eq!(CLASS_ALNUM, 0);
        assert_eq!(CLASS_ALPHA, 1);
        assert_eq!(CLASS_BLANK, 2);
        assert_eq!(CLASS_CNTRL, 3);
        assert_eq!(CLASS_DIGIT, 4);
        assert_eq!(CLASS_GRAPH, 5);
        assert_eq!(CLASS_LOWER, 6);
        assert_eq!(CLASS_PRINT, 7);
        assert_eq!(CLASS_PUNCT, 8);
        assert_eq!(CLASS_SPACE, 9);
        assert_eq!(CLASS_UPPER, 10);
        assert_eq!(CLASS_XDIGIT, 11);
        assert_eq!(CLASS_TAB, 12);
        assert_eq!(CLASS_RETURN, 13);
        assert_eq!(CLASS_BACKSPACE, 14);
        assert_eq!(CLASS_ESCAPE, 15);
        assert_eq!(CLASS_IDENT, 16);
        assert_eq!(CLASS_KEYWORD, 17);
        assert_eq!(CLASS_FNAME, 18);
        assert_eq!(CLASS_NONE, 99);
    }

    #[test]
    fn test_table_is_sorted() {
        // Verify the table is correctly sorted for binary search
        for i in 1..CHAR_CLASS_TAB.len() {
            let prev = CHAR_CLASS_TAB[i - 1].name;
            let curr = CHAR_CLASS_TAB[i].name;
            assert!(
                prev < curr,
                "Table not sorted: {:?} should come before {:?}",
                std::str::from_utf8(prev),
                std::str::from_utf8(curr)
            );
        }
    }
}
